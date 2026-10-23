//! Behaviour — how a person decides what to do: a matrix of urges that fill up, and a field of
//! pressures that lower the fill line.
//!
//! Every moment:
//! 1. Every urge charges up (with a little random wobble), except the one being satisfied, which
//!    drains. Asleep, the others charge more slowly.
//! 2. An urge whose charge reaches its **threshold** (lowered by pressures, and by night for sleep)
//!    can become the goal. The fullest one, measured as charge ÷ threshold, wins.
//! 3. Someone busy only switches when another urge is *much* fuller (the interrupt ratio), so
//!    people finish their meal before going to work, but a starving worker downs tools.
//!
//! It's self-balancing: whatever is neglected keeps charging until it wins.
//!
//! The same code runs a few seconds at a time when the settlement is local and an hour at a time
//! when it isn't; long spans are cut into short steps so the decisions still happen in order.

use super::PeopleSettings;
use super::records::{Doing, Person, Place, PlaceKind, Pressure, Urge};
use crate::fate::Fate;
use crate::sim::Spot;
use crate::time::{DAY_SECONDS, HOUR_SECONDS};
use bevy::math::Vec2;

/// The longest step people are advanced by at once, in world seconds.
pub const STEP: f64 = 30.0;

/// What a person's world looks like while they're being advanced.
pub struct Surroundings<'a> {
    pub settings: &'a PeopleSettings,
    pub places: &'a [Place],
    /// The settlement's centre and size (for wandering).
    pub centre: Vec2,
    pub radius: f32,
}

/// Something worth announcing.
pub enum Happening {
    Started { urge: Urge, place: String },
}

/// Is it night at this moment?
pub fn is_night(settings: &PeopleSettings, seconds: f64) -> bool {
    let hour = (seconds.rem_euclid(DAY_SECONDS) / HOUR_SECONDS) as f32;
    let (from, to) = (settings.night_from_hour, settings.night_to_hour);
    if from > to { hour >= from || hour < to } else { hour >= from && hour < to }
}

/// The threshold an urge has to reach right now, after every pressure on it.
pub fn threshold(person: &Person, urge: Urge, settings: &PeopleSettings, now: f64) -> f32 {
    let base = settings.rule(urge).threshold;
    let mut pressed: f32 = person.pressures.iter().filter(|p| p.urge == urge).map(|p| p.amount).sum();
    if urge == Urge::Sleep && is_night(settings, now) {
        pressed += settings.night_sleep_pressure;
    }
    // Never below a tenth of the base: even a strong push needs *some* charge.
    (base - pressed).max(base * 0.1)
}

/// How full an urge is: 1.0 = at its threshold.
pub fn fullness(person: &Person, urge: Urge, settings: &PeopleSettings, now: f64) -> f32 {
    person.charge(urge) / threshold(person, urge, settings, now)
}

/// The fullest urge at or past its threshold, leaving one out if asked.
fn fullest(person: &Person, settings: &PeopleSettings, now: f64, except: Option<Urge>) -> Option<(Urge, f32)> {
    Urge::ALL
        .into_iter()
        .filter(|&urge| Some(urge) != except)
        .map(|urge| (urge, fullness(person, urge, settings, now)))
        .filter(|&(_, full)| full >= 1.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Where an urge sends someone, and the place's name.
fn destination(person: &Person, urge: Urge, around: &Surroundings, fate: &mut Fate) -> (Option<super::records::PlaceId>, Spot, String) {
    let nearest = |kind: PlaceKind| {
        around
            .places
            .iter()
            .filter(|place| place.kind == kind && place.site == person.site)
            .min_by(|a, b| a.at.vec().distance(person.at.vec()).total_cmp(&b.at.vec().distance(person.at.vec())))
    };
    let place = match urge {
        Urge::Work => person.work.and_then(|id| around.places.iter().find(|p| p.id == id)),
        Urge::Sleep => around.places.iter().find(|p| p.id == person.home),
        Urge::Eat => nearest(PlaceKind::Kitchen),
        Urge::Socialise => nearest(PlaceKind::Tavern),
        Urge::Wander => None,
    };
    match place {
        Some(place) => (Some(place.id), standing_spot(place.at, person.id), place.name.clone()),
        None => {
            // Somewhere at random within the settlement.
            let angle = fate.noise() as f32 * std::f32::consts::TAU;
            let distance = around.radius * 0.8 * (fate.noise() as f32).sqrt();
            let spot = around.centre + Vec2::new(angle.cos(), angle.sin()) * distance;
            (None, Spot::new(spot.x, spot.y), "somewhere about".to_string())
        }
    }
}

/// Each person's own spot at a place, a step or two from its centre, so people sharing a place
/// don't stand inside each other.
pub fn standing_spot(centre: Spot, person: super::records::PersonId) -> Spot {
    let angle = person.0 as f32 * 2.399_963;
    let distance = 1.2 + 0.4 * (person.0 % 3) as f32;
    Spot::new(centre.x + angle.cos() * distance, centre.y + angle.sin() * distance)
}

fn start(person: &mut Person, urge: Urge, around: &Surroundings, fate: &mut Fate, news: &mut Vec<Happening>) {
    let (place, to, name) = destination(person, urge, around, fate);
    person.doing = Doing::Going { urge, place, to };
    news.push(Happening::Started { urge, place: name });
}

/// Pick the next thing to do (or stand about if nothing is pressing).
fn decide(person: &mut Person, around: &Surroundings, now: f64, fate: &mut Fate, news: &mut Vec<Happening>) {
    match fullest(person, around.settings, now, None) {
        Some((urge, _)) => start(person, urge, around, fate, news),
        None => person.doing = Doing::Idle,
    }
}

/// Advance one person by `seconds`, starting at world time `now`.
pub fn advance(person: &mut Person, around: &Surroundings, now: f64, seconds: f64, fate: &mut Fate) -> Vec<Happening> {
    let settings = around.settings;
    let mut news = Vec::new();
    let mut t = now;
    let mut left = seconds;
    while left > 1e-6 {
        let step = left.min(STEP);
        t += step;
        left -= step;
        let step = step as f32;

        // Pressures whose time is up go away.
        person.pressures.retain(|p| p.ends_at.is_none_or(|end| end > t));

        // 1. Charge every urge; drain the one being satisfied.
        let satisfying = match &person.doing {
            Doing::Satisfying { urge, .. } => Some(*urge),
            Doing::Chatting { .. } => Some(Urge::Socialise),
            _ => None,
        };
        let asleep = satisfying == Some(Urge::Sleep);
        let chatting = matches!(person.doing, Doing::Chatting { .. });
        for urge in Urge::ALL {
            let rule = settings.rule(urge);
            let charge = person.charge_mut(urge);
            if satisfying == Some(urge) {
                // A chat satisfies its whole amount over the length of the chat.
                let drain = if chatting {
                    settings.chat_satisfaction / (settings.chat_minutes / 60.0).max(0.01)
                } else {
                    rule.drain_per_hour
                };
                *charge = (*charge - drain * step / HOUR_SECONDS as f32).max(0.0);
            } else {
                let wobble = 1.0 + settings.randomness * (fate.noise() as f32 * 2.0 - 1.0);
                let slow = if asleep { settings.asleep_charge_factor } else { 1.0 };
                *charge = (*charge + rule.rate_per_hour * wobble * slow * step / HOUR_SECONDS as f32).min(rule.threshold * 3.0);
            }
        }

        // 2. Carry on with what they're doing.
        match person.doing.clone() {
            Doing::Idle => decide(person, around, t, fate, &mut news),
            Doing::Going { urge, place, to } => {
                let here = person.at.vec();
                let towards = to.vec() - here;
                let walk = settings.walk_speed * step;
                if towards.length() <= walk {
                    person.at = to;
                    person.doing = Doing::Satisfying { urge, place };
                } else {
                    let next = here + towards.normalize() * walk;
                    person.at = Spot::new(next.x, next.y);
                }
            }
            Doing::Satisfying { urge, .. } => {
                if person.charge(urge) <= 0.0 {
                    decide(person, around, t, fate, &mut news);
                }
            }
            Doing::Chatting { with, seconds_left } => {
                let remaining = seconds_left - step;
                if remaining <= 0.0 {
                    decide(person, around, t, fate, &mut news);
                } else {
                    person.doing = Doing::Chatting { with, seconds_left: remaining };
                }
            }
        }

        // 3. A much fuller urge interrupts (not mid-conversation).
        if let Doing::Going { urge, .. } | Doing::Satisfying { urge, .. } = person.doing.clone() {
            let current = fullness(person, urge, settings, t).max(1.0);
            if let Some((other, full)) = fullest(person, settings, t, Some(urge)) {
                if full >= current * settings.interrupt_ratio {
                    start(person, other, around, fate, &mut news);
                }
            }
        }
    }
    news
}

/// Press an urge's threshold down for a while.
pub fn press(person: &mut Person, urge: Urge, amount: f32, why: String, ends_at: Option<f64>) {
    person.pressures.push(Pressure { urge, amount, why, ends_at });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::people::records::{PersonId, PlaceId};
    use crate::sim::SiteId;

    fn person() -> Person {
        Person {
            id: PersonId(1),
            name: "Test".to_string(),
            site: SiteId(1),
            home: PlaceId(1),
            work: Some(PlaceId(2)),
            at: Spot::new(0.0, 0.0),
            charges: [0.0; 5],
            pressures: Vec::new(),
            doing: Doing::Idle,
            friends: Vec::new(),
        }
    }

    fn places() -> Vec<Place> {
        let place = |id, name: &str, kind, x| Place {
            id: PlaceId(id),
            name: name.to_string(),
            kind,
            site: SiteId(1),
            at: Spot::new(x, 0.0),
        };
        vec![
            place(1, "home", PlaceKind::Home, 0.0),
            place(2, "workshop", PlaceKind::Workplace, 20.0),
            place(3, "kitchen", PlaceKind::Kitchen, -20.0),
            place(4, "tavern", PlaceKind::Tavern, 10.0),
        ]
    }

    #[test]
    fn a_pressure_lowers_the_line_so_less_charge_is_enough() {
        let settings = PeopleSettings::default();
        let mut someone = person();
        let base = threshold(&someone, Urge::Socialise, &settings, 12.0 * HOUR_SECONDS);
        press(&mut someone, Urge::Socialise, 200.0, "test".to_string(), None);
        assert_eq!(threshold(&someone, Urge::Socialise, &settings, 12.0 * HOUR_SECONDS), base - 200.0);
    }

    #[test]
    fn the_fullest_urge_wins_and_they_go_to_its_place() {
        let settings = PeopleSettings::default();
        let places = places();
        let around = Surroundings { settings: &settings, places: &places, centre: Vec2::ZERO, radius: 40.0 };
        let mut fate = Fate::new(1);
        let mut someone = person();
        *someone.charge_mut(Urge::Eat) = 2000.0;
        *someone.charge_mut(Urge::Work) = 1200.0;
        advance(&mut someone, &around, 12.0 * HOUR_SECONDS, 1.0, &mut fate);
        assert!(matches!(someone.doing, Doing::Going { urge: Urge::Eat, .. }), "{:?}", someone.doing);
    }

    #[test]
    fn nobody_is_stuck_forever_a_day_goes_round() {
        // Over a few days, everyone works, eats and sleeps: neglected urges always win in the end.
        let settings = PeopleSettings::default();
        let places = places();
        let around = Surroundings { settings: &settings, places: &places, centre: Vec2::ZERO, radius: 40.0 };
        let mut fate = Fate::new(7);
        let mut someone = person();
        let mut seen = std::collections::BTreeSet::new();
        let mut now = 0.0;
        for _ in 0..(3 * 24 * 60) {
            advance(&mut someone, &around, now, 60.0, &mut fate);
            now += 60.0;
            if let Doing::Satisfying { urge, .. } = someone.doing {
                seen.insert(urge);
            }
            for urge in Urge::ALL {
                assert!(someone.charge(urge) <= settings.rule(urge).threshold * 3.0);
            }
        }
        for urge in [Urge::Work, Urge::Eat, Urge::Sleep] {
            assert!(seen.contains(&urge), "never did {urge}: {seen:?}");
        }
    }

    /// The details drift between the levels (that's fine), but a day should look about the same:
    /// advanced an hour at a time (rich) or a tick at a time (local), people spend about as many
    /// hours asleep and at work.
    #[test]
    fn an_hour_at_a_time_lives_about_the_same_day_as_tick_by_tick() {
        let settings = PeopleSettings::default();
        let places = places();
        let around = Surroundings { settings: &settings, places: &places, centre: Vec2::ZERO, radius: 40.0 };
        let hours_doing = |step: f64, seed: u64| {
            let mut someone = person();
            let mut fate = Fate::new(seed);
            let mut counts = [0u32; 5];
            let steps_per_hour = (HOUR_SECONDS / step).round() as usize;
            for hour in 0..(6 * 24) {
                for k in 0..steps_per_hour {
                    advance(&mut someone, &around, hour as f64 * HOUR_SECONDS + k as f64 * step, step, &mut fate);
                }
                if let Doing::Satisfying { urge, .. } = someone.doing {
                    counts[urge.index()] += 1;
                }
            }
            counts
        };
        let rich = hours_doing(HOUR_SECONDS, 3);
        let local = hours_doing(1.0 / 30.0 * 75.0, 3); // 2.5 s steps, to keep the test quick
        for urge in [Urge::Sleep, Urge::Work] {
            let (a, b) = (rich[urge.index()] as i32, local[urge.index()] as i32);
            assert!((a - b).abs() <= 12, "{urge}: {a} hours vs {b} hours over 6 days");
            assert!(a > 20, "{urge}: only {a} hours over 6 days");
        }
    }
}
