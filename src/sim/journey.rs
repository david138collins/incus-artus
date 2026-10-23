//! Journeys on the move — where a caravan *would* be, worked out from speed and distance.
//!
//! A journey in the background is just "which leg of the route, how far along it, how long it's
//! been resting". Moving it forward any length of time is a few sums, however long that time is:
//! a dot on a loop that jumps ahead once a day.
//!
//! Roads come later; for now each leg is a straight line between two stops.

use super::records::{Journey, Site, SiteId};
use bevy::math::Vec2;

/// Where a stop is (`None` if the site has gone).
fn stop_position(sites: &[Site], id: SiteId) -> Option<Vec2> {
    sites.iter().find(|site| site.id == id).map(|site| site.at.vec())
}

/// The two ends of the journey's current leg.
pub fn leg_ends(journey: &Journey, sites: &[Site]) -> Option<(Vec2, Vec2)> {
    if journey.route.len() < 2 {
        return None;
    }
    let from = stop_position(sites, journey.route[journey.leg % journey.route.len()])?;
    let to = stop_position(sites, journey.route[(journey.leg + 1) % journey.route.len()])?;
    Some((from, to))
}

/// Where it is right now (as of its last update).
pub fn position(journey: &Journey, sites: &[Site]) -> Option<Vec2> {
    let (from, to) = leg_ends(journey, sites)?;
    let length = from.distance(to);
    if length <= 0.0 {
        return Some(from);
    }
    Some(from.lerp(to, (journey.along / length).clamp(0.0, 1.0)))
}

/// Move it on by `seconds`. Returns the stops it arrived at on the way, in order.
pub fn advance(journey: &mut Journey, sites: &[Site], seconds: f64) -> Vec<SiteId> {
    let mut arrivals = Vec::new();
    let mut left = seconds as f32;
    // A guard against a broken route (or a huge skip) looping forever.
    let mut steps = 0;
    while left > 0.0 && steps < 10_000 {
        steps += 1;
        if journey.waiting > 0.0 {
            let rest = journey.waiting.min(left);
            journey.waiting -= rest;
            left -= rest;
            continue;
        }
        let Some((from, to)) = leg_ends(journey, sites) else {
            break;
        };
        let remaining = (from.distance(to) - journey.along).max(0.0);
        let speed = journey.speed.max(0.01);
        let time_to_arrive = remaining / speed;
        if left >= time_to_arrive {
            left -= time_to_arrive;
            journey.leg = (journey.leg + 1) % journey.route.len();
            journey.along = 0.0;
            journey.waiting = journey.dwell;
            arrivals.push(journey.route[journey.leg]);
        } else {
            journey.along += speed * left;
            left = 0.0;
        }
    }
    arrivals
}

/// Where it *would* be after `seconds` more, without changing it.
pub fn estimate(journey: &Journey, sites: &[Site], seconds: f64) -> Option<Vec2> {
    let mut copy = journey.clone();
    advance(&mut copy, sites, seconds);
    position(&copy, sites)
}

/// Say where it is in words: "resting at Test Village" or "40% of the way from A to B".
pub fn describe(journey: &Journey, sites: &[Site]) -> String {
    let name = |id: SiteId| {
        sites.iter().find(|site| site.id == id).map_or("(gone)".to_string(), |site| site.name.clone())
    };
    if journey.route.len() < 2 {
        return "has nowhere to go".to_string();
    }
    let from = journey.route[journey.leg % journey.route.len()];
    let to = journey.route[(journey.leg + 1) % journey.route.len()];
    if journey.waiting > 0.0 {
        return format!("resting at {}", name(from));
    }
    let percent = leg_ends(journey, sites).map_or(0.0, |(a, b)| journey.along / a.distance(b).max(1.0) * 100.0);
    format!("{percent:.0}% of the way from {} to {}", name(from), name(to))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::records::{Condition, JourneyId, SimLevel, Spot};

    fn site(id: u32, x: f32) -> Site {
        Site {
            id: SiteId(id),
            name: format!("site {id}"),
            kind: "village".to_string(),
            at: Spot::new(x, 0.0),
            radius: 10.0,
            level: SimLevel::Thin,
            hooks: Vec::new(),
            condition: Condition::default(),
            problems: Vec::new(),
            last_update: 0.0,
            next_update: 0.0,
        }
    }

    fn caravan() -> Journey {
        Journey {
            id: JourneyId(1),
            name: "caravan".to_string(),
            kind: "caravan".to_string(),
            route: vec![SiteId(1), SiteId(2)],
            leg: 0,
            along: 0.0,
            waiting: 0.0,
            speed: 2.0,
            dwell: 10.0,
            cargo: 0.0,
            level: SimLevel::Thin,
            last_update: 0.0,
            next_update: 0.0,
            next_encounter_check: 0.0,
        }
    }

    #[test]
    fn travels_rests_and_turns_back() {
        let sites = vec![site(1, 0.0), site(2, 100.0)];
        let mut journey = caravan();
        // 100 blocks at 2 per second = 50 seconds to the far end.
        assert!(advance(&mut journey, &sites, 25.0).is_empty());
        assert_eq!(position(&journey, &sites), Some(Vec2::new(50.0, 0.0)));
        assert_eq!(advance(&mut journey, &sites, 30.0), vec![SiteId(2)]);
        // Arrived after 25 more seconds, then rested 5 of its 10.
        assert!((journey.waiting - 5.0).abs() < 1e-4);
        // 5 more resting, then 10 seconds back towards the start: at 80.
        advance(&mut journey, &sites, 15.0);
        let at = position(&journey, &sites).unwrap();
        assert!((at.x - 80.0).abs() < 1e-3, "{at}");
    }

    #[test]
    fn one_big_step_lands_where_many_small_ones_do() {
        let sites = vec![site(1, 0.0), site(2, 100.0)];
        let mut big = caravan();
        let mut small = caravan();
        advance(&mut big, &sites, 1000.0);
        for _ in 0..1000 {
            advance(&mut small, &sites, 1.0);
        }
        let (a, b) = (position(&big, &sites).unwrap(), position(&small, &sites).unwrap());
        assert!(a.distance(b) < 0.1, "{a} vs {b}");
        assert_eq!(big.leg, small.leg);
    }
}
