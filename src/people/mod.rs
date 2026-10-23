//! People — named people living their days: a matrix of urges that fill up, a field of pressures
//! that lower the fill line, and chance opportunities like running into a friend.
//! See docs/jobs.md for the design.
//!
//! This is a SKETCH on the test world: about eight people in a test settlement, shown as coloured
//! cubes, going between their homes, workplaces, a kitchen and a tavern. "Work" here only means
//! going to their workplace and working the urge off; Jobs (looking for something that needs doing)
//! plug in exactly there later.
//!
//! The settlement is a site in the world simulation. While it's **local** (you're near), people are
//! advanced every tick and meet each other by walking past. Otherwise they're advanced once per
//! update of the settlement's level (hourly when rich, daily when thin), and meetings are rolled on
//! the books. Same rules either way.

mod behaviour;
mod debug;
mod meetings;
mod records;
mod stand_ins;
mod test_town;

#[allow(unused_imports)] // for the systems to come (jobs, the manager)
pub use records::{Doing, Person, PersonId, Place, PlaceId, PlaceKind, Pressure, Urge};

use crate::data::BlockRegistry;
use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::events::{EventLogAppExt, LoggedEvent};
use crate::fate::Fate;
use crate::save::{WorldFolder, WorldStateAppExt};
use crate::sim::{SimLevel, SimSettings, SimWorld, SiteId};
use crate::time::{DAY_SECONDS, HOUR_SECONDS, TimeControl, WorldClock};
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How one urge behaves.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UrgeRule {
    /// Charge gained per in-game hour while not being satisfied.
    pub rate_per_hour: f32,
    /// Charge needed to make it the goal (before pressures).
    pub threshold: f32,
    /// Charge lost per in-game hour while being satisfied.
    pub drain_per_hour: f32,
}

/// Tunables (on the "People" debug page).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PeopleSettings {
    pub work: UrgeRule,
    pub eat: UrgeRule,
    pub sleep: UrgeRule,
    pub socialise: UrgeRule,
    pub wander: UrgeRule,
    /// Someone busy switches only when another urge is this many times fuller than what they're doing.
    pub interrupt_ratio: f32,
    /// How much night presses the sleep threshold down.
    pub night_sleep_pressure: f32,
    pub night_from_hour: f32,
    pub night_to_hour: f32,
    /// While asleep, other urges charge at this fraction of their rate.
    pub asleep_charge_factor: f32,
    /// How much every urge's charging wobbles at random (0.2 = ±20%).
    pub randomness: f32,
    /// Blocks per (world) second.
    pub walk_speed: f32,
    /// People this close notice each other...
    pub chat_distance: f32,
    /// ...with this chance (more for friends)...
    pub chat_chance: f64,
    /// ...which presses their socialise threshold down this much for a few minutes.
    pub chat_pressure: f32,
    /// A chat lasts this many in-game minutes...
    pub chat_minutes: f32,
    /// ...takes this much off the socialise charge...
    pub chat_satisfaction: f32,
    /// ...and makes them this much friendlier (friendship goes 0 to 1).
    pub chat_friendship: f32,
    /// Chance per in-game hour of a wave of weirdness.
    pub weirdness_per_hour: f64,
    /// How much a wave presses its urge down.
    pub weirdness_pressure: f32,
    /// Names and activities floating over the people.
    pub show_labels: bool,
}

impl Default for PeopleSettings {
    fn default() -> Self {
        Self {
            // Work builds over ~4 hours and takes ~8 hours of work to satisfy.
            work: UrgeRule { rate_per_hour: 250.0, threshold: 1000.0, drain_per_hour: 125.0 },
            // Hungry every ~6 hours; a meal takes half an hour.
            eat: UrgeRule { rate_per_hour: 160.0, threshold: 1000.0, drain_per_hour: 2000.0 },
            // Sleepy after ~16 hours awake (sooner at night); sleeps ~8 hours.
            sleep: UrgeRule { rate_per_hour: 60.0, threshold: 1000.0, drain_per_hour: 125.0 },
            // Wants company every ~10 hours; an evening at the tavern takes ~2 hours.
            socialise: UrgeRule { rate_per_hour: 100.0, threshold: 1000.0, drain_per_hour: 500.0 },
            // A wander now and then.
            wander: UrgeRule { rate_per_hour: 40.0, threshold: 1000.0, drain_per_hour: 1500.0 },
            interrupt_ratio: 1.5,
            night_sleep_pressure: 450.0,
            night_from_hour: 21.0,
            night_to_hour: 6.0,
            asleep_charge_factor: 0.3,
            randomness: 0.2,
            walk_speed: 1.4,
            chat_distance: 3.0,
            chat_chance: 0.5,
            chat_pressure: 500.0,
            chat_minutes: 6.0,
            chat_satisfaction: 200.0,
            chat_friendship: 0.08,
            weirdness_per_hour: 0.05,
            weirdness_pressure: 600.0,
            show_labels: true,
        }
    }
}

impl PeopleSettings {
    pub fn rule(&self, urge: Urge) -> UrgeRule {
        match urge {
            Urge::Work => self.work,
            Urge::Eat => self.eat,
            Urge::Sleep => self.sleep,
            Urge::Socialise => self.socialise,
            Urge::Wander => self.wander,
        }
    }

    pub fn rule_mut(&mut self, urge: Urge) -> &mut UrgeRule {
        match urge {
            Urge::Work => &mut self.work,
            Urge::Eat => &mut self.eat,
            Urge::Sleep => &mut self.sleep,
            Urge::Socialise => &mut self.socialise,
            Urge::Wander => &mut self.wander,
        }
    }
}

/// Everyone, and the places they go. Saved with the world (`people.toml`).
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct People {
    pub people: Vec<Person>,
    pub places: Vec<Place>,
    /// When each settlement's people were last advanced (by site number).
    last_update: Vec<(SiteId, f64)>,
    /// Who has noticed whom lately (so they don't roll again every tick). Not saved.
    #[serde(skip)]
    recently_met: HashMap<(PersonId, PersonId), f64>,
}

impl People {
    fn last_update(&self, site: SiteId) -> Option<f64> {
        self.last_update.iter().find(|(s, _)| *s == site).map(|(_, t)| *t)
    }

    fn set_last_update(&mut self, site: SiteId, now: f64) {
        match self.last_update.iter_mut().find(|(s, _)| *s == site) {
            Some(entry) => entry.1 = now,
            None => self.last_update.push((site, now)),
        }
    }
}

// ── Announcements ───────────────────────────────────────────────────────────

/// Someone set off to satisfy an urge.
#[derive(Message, Debug, Clone)]
pub struct ActivityStarted {
    pub person: String,
    pub urge: Urge,
    pub place: String,
    pub tick: u64,
}

impl LoggedEvent for ActivityStarted {
    const KIND: &'static str = "activity";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        let what = match self.urge {
            Urge::Work => "goes to work",
            Urge::Eat => "goes to eat",
            Urge::Sleep => "goes to bed",
            Urge::Socialise => "goes for company",
            Urge::Wander => "goes for a wander",
        };
        format!("{} {what} ({})", self.person, self.place)
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// Two people ran into each other.
#[derive(Message, Debug, Clone)]
pub struct PeopleMet {
    pub a: String,
    pub b: String,
    /// Their friendship afterwards, or `None` if they just nodded and walked on.
    pub chatted: Option<f32>,
    /// Rolled on the books (the settlement wasn't local).
    pub bookkeeping: bool,
    pub tick: u64,
}

impl LoggedEvent for PeopleMet {
    const KIND: &'static str = "meeting";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        match self.chatted {
            Some(friendship) => format!(
                "{} and {} stopped to chat{} (friendship now {:.2})",
                self.a,
                self.b,
                if self.bookkeeping { ", on the books" } else { "" },
                friendship
            ),
            None => format!("{} and {} nodded to each other and walked on", self.a, self.b),
        }
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// A wave of weirdness came by.
#[derive(Message, Debug, Clone)]
pub struct MoodWave {
    pub what: String,
    pub people: usize,
    pub tick: u64,
}

impl LoggedEvent for MoodWave {
    const KIND: &'static str = "mood wave";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        format!("{} came over {} people", self.what, self.people)
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

// ── Plugin ──────────────────────────────────────────────────────────────────

pub struct PeoplePlugin;

impl Plugin for PeoplePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<People>()
            .init_resource::<PeopleSettings>()
            .add_settings::<PeopleSettings>("people")
            .add_world_state_with("people", save_people, load_people, fresh_people)
            .add_message::<ActivityStarted>()
            .add_message::<PeopleMet>()
            .add_message::<MoodWave>()
            .log_event::<ActivityStarted>()
            .log_event::<PeopleMet>()
            .log_event::<MoodWave>()
            .add_systems(FixedUpdate, live.after(crate::sim::WorldSimSet))
            .add_plugins(stand_ins::PeopleStandInPlugin)
            .add_systems(EguiPrimaryContextPass, debug::draw_labels)
            .add_debug_page("People", debug::people_debug_page);
    }
}

/// Collected announcements, sent after an update.
#[derive(Default)]
pub(crate) struct PeopleNews {
    activities: Vec<ActivityStarted>,
    meetings: Vec<PeopleMet>,
    waves: Vec<MoodWave>,
}

impl PeopleNews {
    fn send(self, world: &mut World) {
        for message in self.activities {
            world.write_message(message);
        }
        for message in self.meetings {
            world.write_message(message);
        }
        for message in self.waves {
            world.write_message(message);
        }
    }
}

/// Once per tick: advance each settlement's people if it's local, or if its update is due.
fn live(world: &mut World) {
    let now = world.resource::<WorldClock>().seconds;
    let tick = world.resource::<TimeControl>().tick();
    let settings = world.resource::<PeopleSettings>().clone();
    let sim_settings = world.resource::<SimSettings>().clone();
    // The settlements people live in, with their level and size.
    let sites: Vec<(SiteId, SimLevel, Vec2, f32)> = world
        .resource::<SimWorld>()
        .sites
        .iter()
        .map(|site| (site.id, site.level, site.at.vec(), site.radius))
        .collect();

    let mut news = PeopleNews::default();
    world.resource_scope(|world, mut people: Mut<People>| {
        let mut fate = world.resource_mut::<Fate>();
        let settled: Vec<SiteId> = {
            let mut ids: Vec<SiteId> = people.people.iter().map(|p| p.site).collect();
            ids.sort();
            ids.dedup();
            ids
        };
        for site in settled {
            let Some(&(_, level, centre, radius)) = sites.iter().find(|(id, ..)| *id == site) else {
                continue;
            };
            let last = people.last_update(site).unwrap_or(now);
            let interval = match level {
                SimLevel::Local => 0.0,
                SimLevel::Rich => sim_settings.rich_every_hours * HOUR_SECONDS,
                SimLevel::Thin => sim_settings.thin_every_days * DAY_SECONDS,
            };
            if now - last < interval || now <= last {
                if people.last_update(site).is_none() {
                    people.set_last_update(site, now);
                }
                continue;
            }
            let seconds = now - last;
            people.set_last_update(site, now);
            advance_settlement(&mut people, site, level, centre, radius, last, seconds, &settings, &mut fate, tick, &mut news);
        }
    });
    news.send(world);
}

/// Advance one settlement's people by `seconds` from `start`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn advance_settlement(
    people: &mut People,
    site: SiteId,
    level: SimLevel,
    centre: Vec2,
    radius: f32,
    start: f64,
    seconds: f64,
    settings: &PeopleSettings,
    fate: &mut Fate,
    tick: u64,
    news: &mut PeopleNews,
) {
    let People { people: everyone, places, recently_met, .. } = people;
    let around = behaviour::Surroundings { settings, places, centre, radius };
    for person in everyone.iter_mut().filter(|p| p.site == site) {
        for happening in behaviour::advance(person, &around, start, seconds, fate) {
            let behaviour::Happening::Started { urge, place } = happening;
            news.activities.push(ActivityStarted { person: person.name.clone(), urge, place, tick });
        }
    }

    // Meetings: in person if they're really walking about (short steps), on the books otherwise.
    let mut residents: Vec<Person> = everyone.iter().filter(|p| p.site == site).cloned().collect();
    let met = if level == SimLevel::Local && seconds <= 5.0 {
        meetings::meetings_in_person(&mut residents, settings, start + seconds, recently_met, fate)
    } else {
        meetings::meetings_on_the_books(&mut residents, settings, seconds, fate)
    };
    if let Some((what, count)) = meetings::maybe_a_wave(&mut residents, settings, start + seconds, seconds, fate) {
        news.waves.push(MoodWave { what, people: count, tick });
    }
    for resident in residents {
        if let Some(person) = everyone.iter_mut().find(|p| p.id == resident.id) {
            *person = resident;
        }
    }
    for meeting in met {
        news.meetings.push(match meeting {
            meetings::MeetingNews::Chatted { a, b, friendship, bookkeeping } => {
                PeopleMet { a, b, chatted: Some(friendship), bookkeeping, tick }
            }
            meetings::MeetingNews::WalkedOn { a, b } => PeopleMet { a, b, chatted: None, bookkeeping: false, tick },
        });
    }
}

// ── Saving ──────────────────────────────────────────────────────────────────

fn save_people(world: &World, folder: &WorldFolder) -> Result<String, String> {
    let people = world.resource::<People>();
    folder.write_toml(people)?;
    Ok(format!("{} people, {} places", people.people.len(), people.places.len()))
}

fn load_people(world: &mut World, folder: &WorldFolder) -> Result<String, String> {
    match folder.read_toml::<People>()? {
        Some(people) => {
            let note = format!("{} people", people.people.len());
            world.insert_resource(people);
            Ok(note)
        }
        None => {
            fresh_people(world, folder.seed);
            Ok("not in this world, started fresh".to_string())
        }
    }
}

fn fresh_people(world: &mut World, _seed: u64) {
    let settlement = world
        .resource::<SimWorld>()
        .sites
        .iter()
        .find(|site| site.kind == "settlement")
        .map(|site| (site.id, site.at.vec()));
    let people = match settlement {
        Some((id, centre)) => {
            let mut fate = world.resource_mut::<Fate>();
            test_town::test_town(id, centre, &mut fate)
        }
        None => People::default(),
    };
    world.insert_resource(people);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn people_survive_saving() {
        let mut fate = Fate::new(1);
        let mut town = test_town::test_town(SiteId(1), Vec2::ZERO, &mut fate);
        town.people[0].doing = Doing::Chatting { with: PersonId(2), seconds_left: 3.0 };
        town.people[0].befriend(PersonId(2), 0.3);
        behaviour::press(&mut town.people[1], Urge::Eat, 100.0, "test".to_string(), Some(50.0));
        let text = toml::to_string_pretty(&town).expect("saves");
        let back: People = toml::from_str(&text).expect("loads");
        assert_eq!(back.people.len(), town.people.len());
        assert_eq!(back.people[0].doing, town.people[0].doing);
        assert_eq!(back.people[0].friendship(PersonId(2)), 0.3);
        assert_eq!(back.people[1].pressures, town.people[1].pressures);
    }

    #[test]
    fn a_week_on_the_books_makes_friends_and_stays_sane() {
        let mut fate = Fate::new(5);
        let mut town = test_town::test_town(SiteId(1), Vec2::ZERO, &mut fate);
        let settings = PeopleSettings::default();
        let mut news = PeopleNews::default();
        let mut now = 0.0;
        for _ in 0..(7 * 24) {
            advance_settlement(&mut town, SiteId(1), SimLevel::Rich, Vec2::ZERO, 50.0, now, HOUR_SECONDS, &settings, &mut fate, 0, &mut news);
            now += HOUR_SECONDS;
        }
        assert!(town.people.iter().any(|p| !p.friends.is_empty()), "nobody made a friend in a week");
        assert!(!news.activities.is_empty());
        for person in &town.people {
            assert!(person.at.vec().length() < 60.0, "{} wandered off to {:?}", person.name, person.at);
        }
    }
}
