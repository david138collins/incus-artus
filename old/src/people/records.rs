//! Records — people, the places they go, the urges that drive them and the pressures on those urges.
//! Plain data, saved with the world.

use crate::sim::{SiteId, Spot};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The impulses. Each charges up over time; the one past its threshold becomes the active goal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Urge {
    /// The generic will to go out and do things. Later: looking for a Job.
    Work,
    Eat,
    Sleep,
    Socialise,
    /// Wandering about, for no reason in particular.
    Wander,
}

impl Urge {
    pub const ALL: [Urge; 5] = [Urge::Work, Urge::Eat, Urge::Sleep, Urge::Socialise, Urge::Wander];

    pub fn index(self) -> usize {
        self as usize
    }
}

impl fmt::Display for Urge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Urge::Work => "work",
            Urge::Eat => "eat",
            Urge::Sleep => "sleep",
            Urge::Socialise => "socialise",
            Urge::Wander => "wander",
        })
    }
}

/// Something pressing an urge's threshold down, so less charge is enough to act on it.
/// An opportunity, a mood, a wave of weirdness... (night is worked out from the clock, not stored).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pressure {
    pub urge: Urge,
    /// How far it lowers the threshold, in units of charge.
    pub amount: f32,
    /// Why, for the debug page ("ran into Villager 3").
    pub why: String,
    /// World clock second it ends at (`None` = until removed).
    pub ends_at: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PersonId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaceKind {
    Home,
    /// Where someone works. Later: workshops with stations, mines, fields...
    Workplace,
    /// Where food is.
    Kitchen,
    /// Where people go to be together.
    Tavern,
}

/// A place people go. Later these are real regions and buildings; for now, a spot with a name.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Place {
    pub id: PlaceId,
    pub name: String,
    pub kind: PlaceKind,
    pub site: SiteId,
    pub at: Spot,
}

/// What someone is doing right now.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Doing {
    /// Nothing is pressing: standing about.
    #[default]
    Idle,
    /// On the way to satisfy an urge.
    Going { urge: Urge, place: Option<PlaceId>, to: Spot },
    /// Satisfying an urge (working, eating, sleeping...). Ends when the charge runs out.
    Satisfying { urge: Urge, place: Option<PlaceId> },
    /// Stopped to talk with someone they ran into.
    Chatting { with: PersonId, seconds_left: f32 },
}

impl Doing {
    /// The urge being worked on, if any.
    // Read by the old renderer, which the regroup cut (docs/regroup.md); the new one reads it again.
    #[allow(dead_code)]
    pub fn urge(&self) -> Option<Urge> {
        match self {
            Doing::Going { urge, .. } | Doing::Satisfying { urge, .. } => Some(*urge),
            Doing::Chatting { .. } => Some(Urge::Socialise),
            Doing::Idle => None,
        }
    }
}

/// A person. Everyone has a name from the very beginning.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    /// The settlement they live in.
    pub site: SiteId,
    pub home: PlaceId,
    /// Where they work (`None` = no assigned workplace; they work about the settlement).
    pub work: Option<PlaceId>,
    /// Where they are now.
    pub at: Spot,
    /// How charged each urge is (indexed by `Urge::index`).
    pub charges: [f32; 5],
    pub pressures: Vec<Pressure>,
    pub doing: Doing,
    /// People they know, with how friendly they are.
    pub friends: Vec<Friend>,
}

/// Friendship with someone: 0 (strangers) to 1 (close friends).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Friend {
    pub with: PersonId,
    pub friendship: f32,
}

impl Person {
    pub fn charge(&self, urge: Urge) -> f32 {
        self.charges[urge.index()]
    }

    pub fn charge_mut(&mut self, urge: Urge) -> &mut f32 {
        &mut self.charges[urge.index()]
    }

    pub fn friendship(&self, other: PersonId) -> f32 {
        self.friends.iter().find(|friend| friend.with == other).map_or(0.0, |friend| friend.friendship)
    }

    /// Become friendlier with someone (up to 1).
    pub fn befriend(&mut self, other: PersonId, amount: f32) -> f32 {
        match self.friends.iter_mut().find(|friend| friend.with == other) {
            Some(friend) => {
                friend.friendship = (friend.friendship + amount).min(1.0);
                friend.friendship
            }
            None => {
                self.friends.push(Friend { with: other, friendship: amount.min(1.0) });
                amount.min(1.0)
            }
        }
    }
}
