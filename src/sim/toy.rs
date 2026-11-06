//! Toy rules — TEMPORARY. Three test places and a caravan, following made-up rules, only to prove
//! the simulation machinery works (levels, hooks, spreading the work, fights on the books,
//! caravans, saving). Real site types and their rules come from data files later.
//!
//! - **Villages and towns** grow, earn wealth, hire guards, and raid goblin dens that threaten them.
//! - **Goblin dens** breed goblins and raid the nearest village.
//! - **Caravans** carry goods: deliver what they carry, pick up a share of the stop's wealth.
//!
//! Every rule is given *how much time has passed* and works it out in one go, so a place updated
//! once a day ends up about where one updated every hour would.

use super::fight::{self, Side};
use super::records::{Condition, Journey, JourneyId, SimLevel, Site, SiteId, Spot};
use super::{JourneyArrived, Outbox, RaidReachedLocal, RaidResolved, SimWorld};
use crate::fate::{Fate, Wanted};
use crate::time::DAY_SECONDS;

/// How far a place looks for trouble (or targets), in blocks.
const THREAT_RANGE: f32 = 1500.0;

/// The test world's places.
pub fn test_world_places() -> SimWorld {
    let mut sim = SimWorld::default();
    let site = |name: &str, kind: &str, x: f32, y: f32, radius: f32, condition: Condition| Site {
        id: SiteId(0),
        name: name.to_string(),
        kind: kind.to_string(),
        at: Spot::new(x, y),
        radius,
        level: SimLevel::Thin,
        hooks: Vec::new(),
        condition,
        problems: Vec::new(),
        last_update: 0.0,
        next_update: 0.0,
    };
    // The player's own test settlement: the test world itself. Its people are real records
    // (see the people module), so its condition numbers stay at zero and it follows no toy rules.
    let settlement = sim.add_site(site("Test Settlement", "settlement", 0.0, 0.0, 50.0, Condition::default()));
    let hook = sim.new_hook_id();
    if let Some(home) = sim.sites.iter_mut().find(|s| s.id == settlement) {
        // Owned by the player, so it stays Rich while you're away.
        home.hooks.push(super::records::Hook {
            id: hook,
            kind: super::records::HookKind::Owner,
            holder: Some(crate::events::PlayerId(0)),
            label: "your settlement".to_string(),
            expires_at: None,
        });
    }
    let village = sim.add_site(site(
        "Test Village",
        "village",
        150.0,
        0.0,
        16.0,
        Condition { population: 60.0, fighters: 4.0, wealth: 40.0, food: 50.0 },
    ));
    sim.add_site(site(
        "Test Goblin Den",
        "goblin_den",
        0.0,
        600.0,
        12.0,
        Condition { population: 0.0, fighters: 6.0, wealth: 5.0, food: 10.0 },
    ));
    let town = sim.add_site(site(
        "Test Market Town",
        "town",
        -1500.0,
        400.0,
        40.0,
        Condition { population: 400.0, fighters: 30.0, wealth: 300.0, food: 300.0 },
    ));
    sim.add_journey(Journey {
        id: JourneyId(0),
        name: "Test Caravan".to_string(),
        kind: "caravan".to_string(),
        route: vec![village, town],
        leg: 0,
        // Starts part-way out of the village, heading past the test world.
        along: 60.0,
        waiting: 0.0,
        speed: 1.5,
        dwell: 2.0 * crate::time::HOUR_SECONDS as f32,
        cargo: 10.0,
        level: SimLevel::Thin,
        last_update: 0.0,
        next_update: 0.0,
        next_encounter_check: 0.0,
    });
    sim
}

/// Run a place's rules for the time since it last ran.
pub fn run_site_rules(sim: &mut SimWorld, index: usize, now: f64, fate: &mut Fate, out: &mut Outbox) {
    let days = ((now - sim.sites[index].last_update) / DAY_SECONDS).max(0.0) as f32;
    sim.sites[index].last_update = now;
    if days <= 0.0 {
        return;
    }
    // A raid's chance per day becomes a chance for however many days passed.
    let chance_over = |per_day: f64| 1.0 - (1.0 - per_day).powf(days as f64);

    match sim.sites[index].kind.as_str() {
        "village" | "town" => {
            let threat = nearest(sim, index, "goblin_den").filter(|&den| sim.sites[den].condition.fighters >= 5.0);
            let site = &mut sim.sites[index];
            let c = &mut site.condition;
            c.food = (c.food + c.population * 0.02 * days - c.population * 0.015 * days).max(0.0);
            let growth = if c.food > c.population * 0.5 { 0.01 } else { -0.02 };
            c.population = (c.population * (1.0 + growth * days)).max(0.0);
            c.wealth += c.population * 0.05 * days;
            // Hire guards up to a tenth of the people, while they can pay for them.
            let wanted_guards = c.population * 0.1;
            let hire = (wanted_guards - c.fighters).clamp(0.0, days).min(c.wealth / 10.0);
            c.fighters += hire;
            c.wealth -= hire * 10.0;

            site.problems.retain(|problem| !problem.starts_with("Goblins"));
            if let Some(den) = threat {
                let den_name = sim.sites[den].name.clone();
                sim.sites[index].problems.push(format!("Goblins from {den_name} threaten the roads"));
                if sim.sites[index].condition.fighters >= 6.0 && fate.roll("world.decision.raid", chance_over(0.3), Wanted::Either) {
                    raid(sim, index, den, 1.0, 1.0, fate, out);
                }
            }
        }
        "goblin_den" => {
            let c = &mut sim.sites[index].condition;
            c.fighters = (c.fighters + 0.5 * days).min(40.0);
            if c.fighters >= 8.0 {
                if let Some(target) = nearest(sim, index, "village") {
                    if fate.roll("world.decision.raid", chance_over(0.25), Wanted::Either) {
                        raid(sim, index, target, 0.7, 1.0, fate, out);
                    }
                }
            }
        }
        _ => {}
    }
}

/// The nearest place of a kind within reach.
fn nearest(sim: &SimWorld, from: usize, kind: &str) -> Option<usize> {
    let here = sim.sites[from].at.vec();
    sim.sites
        .iter()
        .enumerate()
        .filter(|(index, site)| *index != from && site.kind == kind && site.at.vec().distance(here) <= THREAT_RANGE)
        .min_by(|a, b| a.1.at.vec().distance(here).total_cmp(&b.1.at.vec().distance(here)))
        .map(|(index, _)| index)
}

/// One place raids another, on the books, unless the target is in the local simulation.
fn raid(
    sim: &mut SimWorld,
    attacker: usize,
    defender: usize,
    attacker_strength: f32,
    defender_strength: f32,
    fate: &mut Fate,
    out: &mut Outbox,
) {
    let fighters = sim.sites[attacker].condition.fighters.floor() as u32;
    if sim.sites[defender].level == SimLevel::Local {
        // The player is there: this is a fight to play out for real, not to calculate.
        out.local_raids.push(RaidReachedLocal {
            attacker: sim.sites[attacker].name.clone(),
            defender: sim.sites[defender].id,
            defender_name: sim.sites[defender].name.clone(),
            fighters,
            tick: out.tick,
        });
        return;
    }
    let attackers = Side { fighters, strength_each: attacker_strength };
    let defenders = Side { fighters: sim.sites[defender].condition.fighters.floor() as u32, strength_each: defender_strength };
    let result = fight::resolve(fate, "world.raid", &attackers, &defenders, Wanted::Either);

    let (a, d) = (sim.sites[attacker].condition.clone(), sim.sites[defender].condition.clone());
    sim.sites[attacker].condition.fighters = (a.fighters - result.attacker_lost as f32).max(0.0);
    sim.sites[defender].condition.fighters = (d.fighters - result.defender_lost as f32).max(0.0);
    if result.attacker_won() {
        // Spoils: a third of their wealth and food; raiders kill some of the people too.
        let wealth = d.wealth * 0.33;
        let food = d.food * 0.33;
        sim.sites[attacker].condition.wealth += wealth;
        sim.sites[attacker].condition.food += food;
        let target = &mut sim.sites[defender].condition;
        target.wealth -= wealth;
        target.food -= food;
        target.population = (target.population - result.defender_lost as f32).max(0.0);
    }
    out.raids.push(RaidResolved {
        attacker: sim.sites[attacker].name.clone(),
        defender: sim.sites[defender].name.clone(),
        attackers: attackers.fighters,
        defenders: defenders.fighters,
        needed: result.roll.needed,
        rolled: result.roll.rolled,
        attacker_won: result.attacker_won(),
        attacker_lost: result.attacker_lost,
        defender_lost: result.defender_lost,
        remarkable: result.remarkable,
        tick: out.tick,
    });
}

/// A journey reached a stop: deliver its goods, pick up a share of the stop's wealth.
pub fn journey_arrived(sim: &mut SimWorld, journey: usize, site: SiteId, out: &mut Outbox) {
    let Some(stop) = sim.sites.iter_mut().find(|s| s.id == site) else {
        return;
    };
    let delivered = sim.journeys[journey].cargo;
    stop.condition.wealth += delivered;
    let picked_up = (stop.condition.wealth * 0.1).max(0.0);
    stop.condition.wealth -= picked_up;
    sim.journeys[journey].cargo = picked_up;
    out.arrivals.push(JourneyArrived {
        journey: sim.journeys[journey].name.clone(),
        site: stop.name.clone(),
        delivered,
        picked_up,
        tick: out.tick,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{SimSettings, advance_world};
    use crate::ecs::math::Vec2;

    #[test]
    fn a_year_in_the_background_runs_and_stays_sane() {
        let mut sim = test_world_places();
        let mut fate = Fate::new(11);
        let settings = SimSettings::default();
        let mut out = Outbox::default();
        let far_away = [Vec2::new(30_000.0, 30_000.0)];
        let mut now = 0.0;
        for _ in 0..(28 * 24) {
            now += crate::time::HOUR_SECONDS;
            advance_world(&mut sim, &mut fate, &settings, now, &far_away, None, crate::time::GiveWay::Nothing, &mut out);
        }
        for site in &sim.sites {
            let c = &site.condition;
            assert!(c.population.is_finite() && c.population >= 0.0, "{}: {c:?}", site.name);
            assert!(c.fighters >= 0.0 && c.wealth.is_finite(), "{}: {c:?}", site.name);
            // Only the player's own settlement (owner hook) is rich.
            let expected = if site.kind == "settlement" { SimLevel::Rich } else { SimLevel::Thin };
            assert_eq!(site.level, expected, "{}", site.name);
        }
        // The caravan made trips.
        assert!(!out.arrivals.is_empty());
    }

    /// [David 2026-11-05: "its just a pause... as long as they recall what they are doing once they
    /// 'snap out of it'"] While the World sim gives way, the thin places wait (nothing of theirs runs
    /// or changes); once it's back, they catch up on all the time that passed, as if they'd never
    /// waited.
    #[test]
    fn a_place_that_gives_way_waits_and_then_catches_up() {
        use crate::time::GiveWay;
        let mut sim = test_world_places();
        let mut fate = Fate::new(5);
        let settings = SimSettings::default();
        let mut out = Outbox::default();
        let far_away = [Vec2::new(30_000.0, 30_000.0)];
        advance_world(&mut sim, &mut fate, &settings, crate::time::HOUR_SECONDS, &far_away, None, GiveWay::Nothing, &mut out);
        let thin: Vec<usize> = (0..sim.sites.len()).filter(|&k| sim.sites[k].level == SimLevel::Thin).collect();
        assert!(!thin.is_empty());
        let before: Vec<(f64, String)> = thin.iter().map(|&k| (sim.sites[k].last_update, format!("{:?}", sim.sites[k].condition))).collect();
        // Two days pass while the World sim gives way: the thin places are left as they were.
        let later = 2.0 * crate::time::DAY_SECONDS;
        advance_world(&mut sim, &mut fate, &settings, later, &far_away, None, GiveWay::World, &mut out);
        let waiting: Vec<(f64, String)> = thin.iter().map(|&k| (sim.sites[k].last_update, format!("{:?}", sim.sites[k].condition))).collect();
        assert_eq!(waiting, before, "a thin place ran while the World sim gave way");
        // Back: each runs, caught up to now.
        advance_world(&mut sim, &mut fate, &settings, later, &far_away, None, GiveWay::Nothing, &mut out);
        for &k in &thin {
            assert_eq!(sim.sites[k].last_update, later, "{} didn't catch up", sim.sites[k].name);
        }
    }

    #[test]
    fn standing_in_a_village_makes_it_local_and_walking_away_lets_it_go() {
        let mut sim = test_world_places();
        let mut fate = Fate::new(2);
        let settings = SimSettings::default();
        let mut out = Outbox::default();
        let v = sim.sites.iter().position(|s| s.name == "Test Village").unwrap();
        let village = sim.sites[v].at.vec();
        advance_world(&mut sim, &mut fate, &settings, 1.0, &[village], None, crate::time::GiveWay::Nothing, &mut out);
        assert_eq!(sim.sites[v].level, SimLevel::Local);
        // Just past the entry distance, but inside the "leave" distance: still local.
        let edge = village + Vec2::new(sim.sites[v].radius + settings.local_margin + 5.0, 0.0);
        advance_world(&mut sim, &mut fate, &settings, 2.0, &[edge], None, crate::time::GiveWay::Nothing, &mut out);
        assert_eq!(sim.sites[v].level, SimLevel::Local);
        advance_world(&mut sim, &mut fate, &settings, 3.0, &[village + Vec2::new(500.0, 0.0)], None, crate::time::GiveWay::Nothing, &mut out);
        assert_eq!(sim.sites[v].level, SimLevel::Thin);
    }

    #[test]
    fn a_hook_keeps_a_place_rich_until_it_expires() {
        let mut sim = test_world_places();
        let mut fate = Fate::new(4);
        let settings = SimSettings::default();
        let mut out = Outbox::default();
        let far = [Vec2::new(30_000.0, 0.0)];
        let t = sim.sites.iter().position(|s| s.name == "Test Market Town").unwrap();
        let town = sim.sites[t].id;
        crate::sim::apply_hook_request(
            &mut sim,
            crate::sim::SiteHookRequest::Add {
                site: town,
                kind: crate::sim::HookKind::Quest,
                label: "test".to_string(),
                lasts: Some(100.0),
            },
            0.0,
        );
        advance_world(&mut sim, &mut fate, &settings, 50.0, &far, None, crate::time::GiveWay::Nothing, &mut out);
        assert_eq!(sim.sites[t].level, SimLevel::Rich);
        advance_world(&mut sim, &mut fate, &settings, 150.0, &far, None, crate::time::GiveWay::Nothing, &mut out);
        assert_eq!(sim.sites[t].level, SimLevel::Thin);
        assert!(sim.sites[t].hooks.is_empty());
    }

    #[test]
    fn the_world_sim_survives_saving() {
        let sim = test_world_places();
        let text = toml::to_string_pretty(&sim).unwrap();
        let back: SimWorld = toml::from_str(&text).unwrap();
        assert_eq!(back.sites.len(), 4);
        assert_eq!(back.journeys[0].route, sim.journeys[0].route);
    }
}
