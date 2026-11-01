//! Distributors (`docs/placer.md` sections 3 and 4, stop 3): what's placed where.
//!
//! - **A distributor** has a list of **candidates**, in order. Mineralogy's are blocks from the
//!   library ("the surface": snowy ground, gabbro, stone, sand, dry grass, grass).
//! - **Each candidate's suitability** at a spot is a stack of layers, as a made field is: fields
//!   through curves, mixed by modes, so "multiply" is "and". A candidate with no layers fits
//!   everywhere (1).
//! - **The chooser** picks one at each spot: the first that fits (its suitability at least `at`, in
//!   order), the best fit (the most suitable), or by chance (weighted by suitability, the same at a
//!   spot every time).
//! - **What a candidate puts down** is a column profile, the first kind: its block on top, then the
//!   blocks under it, each to a depth, then the rock (stone); or, with `top_only`, its block on top
//!   of what the distributors before it put down (a river's bed of gravel over the soil). Inclusions
//!   (ores in their rock), points (trees, boulders, towns) and pieces joined at hooks come later as
//!   more kinds; a kind this game doesn't know is greyed, named and skipped.
//! - **Later distributors go over earlier ones** wherever one of their candidates is chosen, and read
//!   what the earlier ones chose ("the rock is limestone or marble"): the linear rule, within one user.
//!
//! A block that isn't registered is skipped, so a new block lands in the world as soon as it's made
//! in the Registrar, and until then the ground is as it was.

use super::Layer;
use serde::{Deserialize, Serialize};

/// One distributor: its candidates, and how one is chosen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Distributor {
    pub name: String,
    /// Off: left out, to see what it does.
    pub on: bool,
    pub chooser: Chooser,
    pub candidates: Vec<Candidate>,
}

impl Default for Distributor {
    fn default() -> Self {
        Self { name: "a new distributor".into(), on: true, chooser: Chooser::default(), candidates: Vec::new() }
    }
}

/// How a distributor picks among its candidates at a spot.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Chooser {
    /// The first, in order, whose suitability is at least `at`.
    FirstFit { at: f32 },
    /// The most suitable (the first of equals); none where all are 0.
    Best,
    /// At random, weighted by suitability, the same at a spot every time; none where all are 0.
    Chance,
    /// A kind this game doesn't know: nothing is chosen.
    #[serde(other)]
    Unknown,
}

impl Default for Chooser {
    fn default() -> Self {
        Chooser::FirstFit { at: 0.5 }
    }
}

impl Chooser {
    pub fn name(self) -> &'static str {
        match self {
            Chooser::FirstFit { .. } => "the first that fits",
            Chooser::Best => "the best fit",
            Chooser::Chance => "by chance",
            Chooser::Unknown => "? (unknown)",
        }
    }
}

/// One candidate: a block, what it puts down, and where it suits.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Candidate {
    /// Off: skipped, to see what it does.
    pub on: bool,
    /// The block's name in the library ("incus:sand").
    pub block: String,
    pub puts: Puts,
    /// Its suitability, as a made field's layers (none: everywhere).
    pub layers: Vec<Layer>,
}

impl Default for Candidate {
    fn default() -> Self {
        Self { on: true, block: "incus:stone".into(), puts: Puts::default(), layers: Vec::new() }
    }
}

/// What a candidate puts down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Puts {
    /// A column profile: its block on top, then `under`, each so many blocks deep, then stone. With
    /// `top_only`, only its block on top, over what's already there.
    Column {
        #[serde(default)]
        under: Vec<Under>,
        #[serde(default)]
        top_only: bool,
    },
    /// A kind this game doesn't know: skipped.
    #[serde(other)]
    Unknown,
}

impl Default for Puts {
    fn default() -> Self {
        Puts::Column { under: Vec::new(), top_only: false }
    }
}

/// A block under a column profile's top, so many blocks deep.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Under {
    pub block: String,
    pub depth: i32,
}

impl Default for Under {
    fn default() -> Self {
        Self { block: "incus:dirt".into(), depth: 3 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distributors_keep_through_a_preset_and_unknown_kinds_are_kept_out() {
        let distributor = Distributor {
            name: "the surface".into(),
            on: true,
            chooser: Chooser::FirstFit { at: 0.5 },
            candidates: vec![Candidate { block: "incus:grass".into(), puts: Puts::Column { under: vec![Under { block: "incus:dirt".into(), depth: 2 }], top_only: false }, ..Candidate::default() }],
        };
        let back: Distributor = toml::from_str(&toml::to_string(&distributor).unwrap()).unwrap();
        assert_eq!(back, distributor);
        // A newer game's chooser and kind of placing.
        let newer = "name = \"trees\"\nchooser = { kind = \"spaced\", gap = 8 }\n[[candidates]]\nblock = \"incus:oak\"\nputs = { kind = \"points\", spacing = 6 }\n";
        let read: Distributor = toml::from_str(newer).unwrap();
        assert_eq!((read.chooser, &read.candidates[0].puts), (Chooser::Unknown, &Puts::Unknown));
    }
}
