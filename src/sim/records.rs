//! Records — what the world simulation knows: sites (places), journeys (things on the move),
//! and hooks (why the player is involved with a place).
//!
//! Everything here is plain data, saved with the world as readable text (`sim.toml`).

use crate::events::PlayerId;
use crate::ecs::math::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt;

/// How much detail something is simulated in. See docs/simulation.md.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SimLevel {
    /// Bookkeeping: a few numbers, updated once in a while (default: daily).
    #[default]
    Thin,
    /// Granular, economy-focused records, updated more often (default: hourly).
    Rich,
    /// The full simulation: real blocks, real people. The world sim keeps its hands off.
    Local,
}

impl fmt::Display for SimLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SimLevel::Thin => "thin",
            SimLevel::Rich => "rich",
            SimLevel::Local => "local",
        })
    }
}

/// A spot on the map (game axes: x north, y east), in blocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Spot {
    pub x: f32,
    pub y: f32,
}

impl Spot {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn vec(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SiteId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct JourneyId(pub u32);

/// Either kind of thing the simulation tracks (for announcements and stand-ins).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimThing {
    Site(SiteId),
    Journey(JourneyId),
}

/// Why the player is involved with a place. While a place has any hook, it's simulated Rich.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookKind {
    /// Took a quest here.
    Quest,
    /// Lives here (took residence).
    Residence,
    /// Owns it (the player's own settlement).
    Owner,
    /// Has followers or inhabitants here.
    Followers,
    /// Added from the debug panel.
    Test,
}

impl fmt::Display for HookKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HookKind::Quest => "quest",
            HookKind::Residence => "residence",
            HookKind::Owner => "owner",
            HookKind::Followers => "followers",
            HookKind::Test => "test",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hook {
    pub id: u32,
    pub kind: HookKind,
    /// Who it belongs to (`None` = nobody in particular).
    pub holder: Option<PlayerId>,
    /// What it is, in words ("find the missing miller").
    pub label: String,
    /// World clock second it expires at (`None` = until removed).
    pub expires_at: Option<f64>,
}

/// The bookkeeping numbers for a place. Fractions are allowed (a village can be 0.4 of a guard
/// closer to hiring one); they're shown rounded.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Condition {
    pub population: f32,
    pub fighters: f32,
    pub wealth: f32,
    pub food: f32,
}

/// A place: village, town, hostile site, the player's settlement...
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    pub id: SiteId,
    pub name: String,
    /// What sort of place (`village`, `town`, `goblin_den`...). Picks the rules it follows.
    pub kind: String,
    pub at: Spot,
    /// The footprint: its area of direct control, in blocks from the centre.
    pub radius: f32,
    pub level: SimLevel,
    pub hooks: Vec<Hook>,
    pub condition: Condition,
    /// Each a possible procedural quest ("goblins are raiding nearby").
    pub problems: Vec<String>,
    /// World clock second its rules last ran, and when they're due again.
    pub last_update: f64,
    pub next_update: f64,
}

/// Something on the move along a route: a caravan, courier, traveller, raiding party...
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Journey {
    pub id: JourneyId,
    pub name: String,
    /// `caravan`, `courier`, `wanderer`...
    pub kind: String,
    /// The stops, visited in a loop.
    pub route: Vec<SiteId>,
    /// Travelling from `route[leg]` to the next stop.
    pub leg: usize,
    /// Blocks travelled along this leg.
    pub along: f32,
    /// Seconds left resting in a town before setting off.
    pub waiting: f32,
    /// Blocks per second (a wagon ~1.5).
    pub speed: f32,
    /// Seconds it rests at each stop.
    pub dwell: f32,
    /// Goods carried, as wealth.
    pub cargo: f32,
    pub level: SimLevel,
    /// World clock second its position was last worked out.
    pub last_update: f64,
    pub next_update: f64,
    /// When it next rolls to see if it's near the player (while Rich).
    pub next_encounter_check: f64,
}
