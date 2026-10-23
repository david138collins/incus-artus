//! Sim — the world simulation: everything that happens away from the player, in as little detail
//! as it can get away with. See docs/simulation.md for the design.
//!
//! Three levels (`SimLevel`):
//! - **Local**: where a `Presence` is (the player, later followers and settlements). The full
//!   simulation owns it; the world sim keeps its hands off.
//! - **Rich**: places the player is involved with (they have **hooks**: a quest, residence...).
//!   Updated often (default: every in-game hour).
//! - **Thin**: everything else. Bookkeeping, updated rarely (default: every in-game day).
//!
//! Rich and Thin run the *same rules*, given however much time has passed, so they come out about
//! the same. Only a few places are updated per tick, so the work is spread out.
//!
//! Every level change is announced (`SimLevelChanged`), which is the hook for anything that needs
//! to expand a place into real things or collapse it back (see `stand_ins.rs`).
//!
//! The places in the world now are TEST stand-ins with toy rules (`toy.rs`), until world
//! generation makes real ones.

mod debug;
mod fight;
mod journey;
mod records;
mod stand_ins;
mod toy;

#[allow(unused_imports)] // for systems that don't exist yet
pub use records::{Hook, HookKind, Journey, JourneyId, SimLevel, SimThing, Site, SiteId, Spot};

use crate::data::BlockRegistry;
use crate::debug::{DebugAppExt, SettingsAppExt};
use crate::events::{EventLogAppExt, LoggedEvent};
use crate::fate::{Fate, Wanted};
use crate::save::{WorldFolder, WorldStateAppExt};
use crate::time::{DAY_SECONDS, HOUR_SECONDS, TimeControl, WorldClock};
use crate::world::WorldEdits;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Anything with a `Presence` keeps the world around it in the local simulation. The player
/// has one; followers and settlements will too.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Presence(pub Vec3);

/// Tunables (on the "Sim" debug page).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimSettings {
    /// In-game hours between updates of a Rich place.
    pub rich_every_hours: f64,
    /// In-game days between updates of a Thin place.
    pub thin_every_days: f64,
    /// A place becomes local when a presence is within its footprint plus this many blocks...
    pub local_margin: f32,
    /// ...and stops being local only once they're this much further away (no flicker at the edge).
    pub leave_extra: f32,
    /// A journey can be met when a presence is within this many blocks of where it would be.
    pub encounter_range: f32,
    /// In-game minutes between "is it here yet?" rolls for a journey that's near.
    pub encounter_check_minutes: f64,
    /// At most this many places and journeys are updated per tick.
    pub updates_per_tick: usize,
    /// Show the magenta stand-ins for local places and journeys.
    pub show_stand_ins: bool,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            rich_every_hours: 1.0,
            thin_every_days: 1.0,
            local_margin: 48.0,
            leave_extra: 16.0,
            encounter_range: 200.0,
            encounter_check_minutes: 10.0,
            updates_per_tick: 8,
            show_stand_ins: true,
        }
    }
}

impl SimSettings {
    fn interval(&self, level: SimLevel) -> f64 {
        match level {
            SimLevel::Thin => self.thin_every_days * DAY_SECONDS,
            SimLevel::Rich => self.rich_every_hours * HOUR_SECONDS,
            SimLevel::Local => 0.0,
        }
    }
}

/// Everything the world simulation knows. Saved with the world (`sim.toml`).
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SimWorld {
    pub sites: Vec<Site>,
    pub journeys: Vec<Journey>,
    next_site: u32,
    next_journey: u32,
    next_hook: u32,
    /// For the debug page: how much the last tick did.
    #[serde(skip)]
    pub last_updates: usize,
    #[serde(skip)]
    pub last_millis: f64,
}

impl SimWorld {
    pub fn add_site(&mut self, mut site: Site) -> SiteId {
        self.next_site += 1;
        site.id = SiteId(self.next_site);
        let id = site.id;
        self.sites.push(site);
        id
    }

    pub fn add_journey(&mut self, mut journey: Journey) -> JourneyId {
        self.next_journey += 1;
        journey.id = JourneyId(self.next_journey);
        let id = journey.id;
        self.journeys.push(journey);
        id
    }

    fn new_hook_id(&mut self) -> u32 {
        self.next_hook += 1;
        self.next_hook
    }
}

// ── Requests and announcements ──────────────────────────────────────────────

/// Please add or remove a hook on a place (taking a quest, taking residence, ...).
#[derive(Message, Debug, Clone)]
pub enum SiteHookRequest {
    Add { site: SiteId, kind: HookKind, label: String, lasts: Option<f64> },
    Remove { site: SiteId, hook: u32 },
}

/// Something changed simulation level (a place expanded into the local sim, a caravan was met...).
#[derive(Message, Debug, Clone)]
pub struct SimLevelChanged {
    #[allow(dead_code)] // for the systems that will expand and collapse places for real
    pub thing: SimThing,
    pub name: String,
    pub from: SimLevel,
    pub to: SimLevel,
    pub why: String,
    pub tick: u64,
}

impl LoggedEvent for SimLevelChanged {
    const KIND: &'static str = "sim level";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        format!("{}: {} → {} ({})", self.name, self.from, self.to, self.why)
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// A fight was resolved on the books. Later this travels as news.
#[derive(Message, Debug, Clone)]
pub struct RaidResolved {
    pub attacker: String,
    pub defender: String,
    pub attackers: u32,
    pub defenders: u32,
    pub needed: u8,
    pub rolled: u8,
    pub attacker_won: bool,
    pub attacker_lost: u32,
    pub defender_lost: u32,
    pub remarkable: bool,
    pub tick: u64,
}

impl LoggedEvent for RaidResolved {
    const KIND: &'static str = "raid";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        format!(
            "{} raided {}: needed {}, rolled {}, {}. Attackers lost {} of {}, defenders lost {} of {}.{}",
            self.attacker,
            self.defender,
            self.needed,
            self.rolled,
            if self.attacker_won { "won" } else { "lost" },
            self.attacker_lost,
            self.attackers,
            self.defender_lost,
            self.defenders,
            if self.remarkable { " A remarkable victory!" } else { "" }
        )
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// A raid reached a place that's in the local simulation. It isn't calculated: the local sim
/// should play it out for real (nothing local can fight yet, so for now it's only announced).
#[derive(Message, Debug, Clone)]
pub struct RaidReachedLocal {
    pub attacker: String,
    #[allow(dead_code)] // for the local sim, to play the raid out where it lands
    pub defender: SiteId,
    pub defender_name: String,
    pub fighters: u32,
    pub tick: u64,
}

impl LoggedEvent for RaidReachedLocal {
    const KIND: &'static str = "raid (local)";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        format!(
            "{} fighters from {} attack {}, where you are! (to be played out locally later)",
            self.fighters, self.attacker, self.defender_name
        )
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// A journey reached a stop.
#[derive(Message, Debug, Clone)]
pub struct JourneyArrived {
    pub journey: String,
    pub site: String,
    /// Goods delivered and picked up, as wealth.
    pub delivered: f32,
    pub picked_up: f32,
    pub tick: u64,
}

impl LoggedEvent for JourneyArrived {
    const KIND: &'static str = "caravan";
    fn describe(&self, _blocks: &BlockRegistry) -> String {
        format!(
            "{} arrived at {}: delivered {:.0} wealth of goods, picked up {:.0}",
            self.journey, self.site, self.delivered, self.picked_up
        )
    }
    fn tick(&self) -> Option<u64> {
        Some(self.tick)
    }
}

/// Announcements collected during an update, sent afterwards.
#[derive(Default)]
pub(crate) struct Outbox {
    levels: Vec<SimLevelChanged>,
    raids: Vec<RaidResolved>,
    local_raids: Vec<RaidReachedLocal>,
    arrivals: Vec<JourneyArrived>,
    tick: u64,
}

impl Outbox {
    fn send(self, world: &mut World) {
        for message in self.levels {
            world.write_message(message);
        }
        for message in self.raids {
            world.write_message(message);
        }
        for message in self.local_raids {
            world.write_message(message);
        }
        for message in self.arrivals {
            world.write_message(message);
        }
    }
}

// ── Plugin ──────────────────────────────────────────────────────────────────

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimWorld>()
            .init_resource::<SimSettings>()
            .add_settings::<SimSettings>("sim")
            .add_world_state_with("sim", save_sim, load_sim, fresh_sim)
            .add_message::<SiteHookRequest>()
            .add_message::<SimLevelChanged>()
            .add_message::<RaidResolved>()
            .add_message::<RaidReachedLocal>()
            .add_message::<JourneyArrived>()
            .log_event::<SimLevelChanged>()
            .log_event::<RaidResolved>()
            .log_event::<RaidReachedLocal>()
            .log_event::<JourneyArrived>()
            .add_systems(FixedUpdate, run_world_sim.after(WorldEdits))
            .add_plugins(stand_ins::StandInPlugin)
            .init_resource::<debug::SimPageState>()
            .add_systems(EguiPrimaryContextPass, debug::draw_local_notice)
            .add_debug_page("Sim", debug::sim_debug_page);
    }
}

/// Once per tick: hooks, levels, and the places and journeys that are due.
fn run_world_sim(world: &mut World) {
    let started = Instant::now();
    let presences: Vec<Vec2> = world
        .query::<&Presence>()
        .iter(world)
        .map(|presence| presence.0.truncate())
        .collect();
    let now = world.resource::<WorldClock>().seconds;
    let tick = world.resource::<TimeControl>().tick();
    let settings = world.resource::<SimSettings>().clone();
    let requests: Vec<SiteHookRequest> = world.resource_mut::<Messages<SiteHookRequest>>().drain().collect();

    let mut out = Outbox { tick, ..default() };
    world.resource_scope(|world, mut sim: Mut<SimWorld>| {
        let mut fate = world.resource_mut::<Fate>();
        for request in requests {
            apply_hook_request(&mut sim, request, now);
        }
        let updates = advance_world(&mut sim, &mut fate, &settings, now, &presences, Some(settings.updates_per_tick), &mut out);
        sim.last_updates = updates;
        sim.last_millis = started.elapsed().as_secs_f64() * 1000.0;
    });
    out.send(world);
}

fn apply_hook_request(sim: &mut SimWorld, request: SiteHookRequest, now: f64) {
    match request {
        SiteHookRequest::Add { site, kind, label, lasts } => {
            let id = sim.new_hook_id();
            if let Some(site) = sim.sites.iter_mut().find(|s| s.id == site) {
                site.hooks.push(Hook { id, kind, holder: None, label, expires_at: lasts.map(|span| now + span) });
            }
        }
        SiteHookRequest::Remove { site, hook } => {
            if let Some(site) = sim.sites.iter_mut().find(|s| s.id == site) {
                site.hooks.retain(|h| h.id != hook);
            }
        }
    }
}

/// The world simulation's heartbeat. Decides every level, then runs the rules for whatever is
/// due (at most `budget` of them; `None` = everything due). Returns how many it updated.
fn advance_world(
    sim: &mut SimWorld,
    fate: &mut Fate,
    settings: &SimSettings,
    now: f64,
    presences: &[Vec2],
    budget: Option<usize>,
    out: &mut Outbox,
) -> usize {
    let nearest = |at: Vec2| presences.iter().map(|p| p.distance(at)).fold(f32::INFINITY, f32::min);

    // ── Sites: hooks expire, levels are decided ──
    for index in 0..sim.sites.len() {
        let site = &mut sim.sites[index];
        let expired: Vec<String> = site
            .hooks
            .iter()
            .filter(|hook| hook.expires_at.is_some_and(|end| end <= now))
            .map(|hook| format!("{} hook ended", hook.kind))
            .collect();
        site.hooks.retain(|hook| hook.expires_at.is_none_or(|end| end > now));

        let distance = nearest(site.at.vec());
        let enter = site.radius + settings.local_margin;
        let (level, why) = if distance <= enter || (site.level == SimLevel::Local && distance <= enter + settings.leave_extra) {
            (SimLevel::Local, "the player is here".to_string())
        } else if !site.hooks.is_empty() {
            let kinds: Vec<String> = site.hooks.iter().map(|hook| hook.kind.to_string()).collect();
            (SimLevel::Rich, format!("hooks: {}", kinds.join(", ")))
        } else if !expired.is_empty() {
            (SimLevel::Thin, expired.join(", "))
        } else {
            (SimLevel::Thin, "nothing involves the player here".to_string())
        };
        if level != site.level {
            let from = site.level;
            if from == SimLevel::Local {
                // The local sim owned the time it was here; the rules carry on from now.
                site.last_update = now;
            }
            if level == SimLevel::Local {
                // Catch up to now before the local sim takes over.
                toy::run_site_rules(sim, index, now, fate, out);
            }
            let site = &mut sim.sites[index];
            site.level = level;
            site.next_update = match level {
                SimLevel::Local => f64::INFINITY,
                _ if from == SimLevel::Local => now + settings.interval(level),
                _ => site.next_update.min(now + settings.interval(level)),
            };
            out.levels.push(SimLevelChanged {
                thing: SimThing::Site(site.id),
                name: site.name.clone(),
                from,
                to: level,
                why,
                tick: out.tick,
            });
        }
    }

    // ── Journeys: levels (and the "are they here yet?" roll) ──
    for index in 0..sim.journeys.len() {
        let journey = &sim.journeys[index];
        let elapsed = now - journey.last_update;
        let Some(at) = journey::estimate(journey, &sim.sites, elapsed) else {
            continue;
        };
        let distance = nearest(at);
        let range = settings.encounter_range;
        let old = journey.level;
        let (level, why) = match old {
            SimLevel::Local if distance <= range + settings.leave_extra => (SimLevel::Local, String::new()),
            _ if distance <= range => {
                if old != SimLevel::Local && now >= journey.next_encounter_check {
                    sim.journeys[index].next_encounter_check = now + settings.encounter_check_minutes / 60.0 * HOUR_SECONDS;
                    // The nearer it would be, the likelier it's really here.
                    let closeness = (1.0 - distance / range).clamp(0.0, 1.0) as f64;
                    if fate.roll("world.encounter", closeness * closeness, Wanted::Either) {
                        (SimLevel::Local, format!("met, {distance:.0} blocks away"))
                    } else {
                        (SimLevel::Rich, format!("nearby, {distance:.0} blocks away"))
                    }
                } else if old == SimLevel::Local {
                    (SimLevel::Local, String::new())
                } else {
                    (SimLevel::Rich, format!("nearby, {distance:.0} blocks away"))
                }
            }
            _ => (SimLevel::Thin, "far from the player".to_string()),
        };
        if level != old {
            // Bring it up to now, then switch.
            update_journey(sim, index, now, out);
            let journey = &mut sim.journeys[index];
            journey.level = level;
            journey.next_update = now + settings.interval(level);
            out.levels.push(SimLevelChanged {
                thing: SimThing::Journey(journey.id),
                name: journey.name.clone(),
                from: old,
                to: level,
                why,
                tick: out.tick,
            });
        }
    }

    // ── Updates: local journeys every tick; the rest when due, a few per tick ──
    let mut updates = 0;
    let mut budget_left = budget.unwrap_or(usize::MAX);
    for index in 0..sim.journeys.len() {
        let journey = &sim.journeys[index];
        let local = journey.level == SimLevel::Local;
        if local || (journey.next_update <= now && budget_left > 0) {
            update_journey(sim, index, now, out);
            let journey = &mut sim.journeys[index];
            journey.next_update = now + settings.interval(journey.level);
            if !local {
                budget_left -= 1;
                updates += 1;
            }
        }
    }
    for index in 0..sim.sites.len() {
        if budget_left == 0 {
            break;
        }
        let site = &sim.sites[index];
        if site.level != SimLevel::Local && site.next_update <= now {
            toy::run_site_rules(sim, index, now, fate, out);
            let site = &mut sim.sites[index];
            site.next_update = now + settings.interval(site.level);
            budget_left -= 1;
            updates += 1;
        }
    }
    updates
}

/// Move a journey up to `now`, handling its arrivals.
fn update_journey(sim: &mut SimWorld, index: usize, now: f64, out: &mut Outbox) {
    let elapsed = now - sim.journeys[index].last_update;
    let arrivals = journey::advance(&mut sim.journeys[index], &sim.sites, elapsed);
    sim.journeys[index].last_update = now;
    for site in arrivals {
        toy::journey_arrived(sim, index, site, out);
    }
}

/// Debug: run the world simulation forward `seconds` in one go (the clock moves too).
pub(crate) fn skip_ahead(world: &mut World, seconds: f64) {
    let settings = world.resource::<SimSettings>().clone();
    let presences: Vec<Vec2> = world.query::<&Presence>().iter(world).map(|p| p.0.truncate()).collect();
    let tick = world.resource::<TimeControl>().tick();
    let start = world.resource::<WorldClock>().seconds;
    let end = start + seconds;
    let step = settings.interval(SimLevel::Rich).max(60.0);
    let mut out = Outbox { tick, ..default() };
    world.resource_scope(|world, mut sim: Mut<SimWorld>| {
        let mut fate = world.resource_mut::<Fate>();
        let mut now = start;
        while now < end {
            now = (now + step).min(end);
            advance_world(&mut sim, &mut fate, &settings, now, &presences, None, &mut out);
        }
    });
    world.resource_mut::<WorldClock>().seconds = end;
    out.send(world);
}

// ── Saving ──────────────────────────────────────────────────────────────────

fn save_sim(world: &World, folder: &WorldFolder) -> Result<String, String> {
    let sim = world.resource::<SimWorld>();
    folder.write_toml(sim)?;
    Ok(format!("{} places, {} journeys", sim.sites.len(), sim.journeys.len()))
}

fn load_sim(world: &mut World, folder: &WorldFolder) -> Result<String, String> {
    match folder.read_toml::<SimWorld>()? {
        Some(sim) => {
            let note = format!("{} places, {} journeys", sim.sites.len(), sim.journeys.len());
            world.insert_resource(sim);
            Ok(note)
        }
        None => {
            fresh_sim(world, folder.seed);
            Ok("not in this world, started fresh".to_string())
        }
    }
}

fn fresh_sim(world: &mut World, _seed: u64) {
    world.insert_resource(toy::test_world_places());
}
