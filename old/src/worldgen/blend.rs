//! Mixing — how a layer combines with the layers below it, like the blend modes of an image editor.
//!
//! Layers mix their **numbers**, not their colours: each pixel's value (0 to 1) from the layers
//! below meets this layer's value, and the mode decides what comes out. The finished numbers are
//! coloured afterwards, with the result's palette. This is what the world needs: a mountain field
//! that really is "continents × peaks", not a picture of one.
//!
//! **Opacity** fades between the layers below (0%) and the full mix (100%). Where nothing is
//! below, a layer's own value is used as it is, whatever its mode.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mix {
    #[default]
    Normal,
    Add,
    Subtract,
    Multiply,
    Screen,
    Overlay,
    Dodge,
    Burn,
    Lighten,
    Darken,
    Difference,
}

impl Mix {
    pub const ALL: [Mix; 11] = [
        Mix::Normal,
        Mix::Add,
        Mix::Subtract,
        Mix::Multiply,
        Mix::Screen,
        Mix::Overlay,
        Mix::Dodge,
        Mix::Burn,
        Mix::Lighten,
        Mix::Darken,
        Mix::Difference,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Mix::Normal => "normal",
            Mix::Add => "add",
            Mix::Subtract => "subtract",
            Mix::Multiply => "multiply",
            Mix::Screen => "screen",
            Mix::Overlay => "overlay",
            Mix::Dodge => "dodge",
            Mix::Burn => "burn",
            Mix::Lighten => "lighten",
            Mix::Darken => "darken",
            Mix::Difference => "difference",
        }
    }

    /// What the mode does, and what it's good for in terrain.
    pub fn about(self) -> &'static str {
        match self {
            Mix::Normal => "Covers what's below.",
            Mix::Add => "Below + this. Piles hills on. Bright layers soon fill to white.",
            Mix::Subtract => "Below − this. Carves valleys out. Bright layers soon empty to black.",
            Mix::Multiply => "Below × this. Dark wins, white leaves it alone.\nA mask: continents × mountains = mountains only on land.",
            Mix::Screen => "The opposite of multiply. Light wins, black leaves it alone.",
            Mix::Overlay => "Pushes the darks below darker and the lights lighter, by this layer.\nAdds detail without changing the big shapes much.",
            Mix::Dodge => "Brightens what's below, harder the brighter this layer is.",
            Mix::Burn => "Darkens what's below, harder the darker this layer is.",
            Mix::Lighten => "Keeps the higher of the two. Merges two mountain ranges.",
            Mix::Darken => "Keeps the lower of the two.",
            Mix::Difference => "How far apart the two are. Ridges and creases where they cross.",
        }
    }

    /// The mode on its own: `below` meets `this`, both 0 to 1.
    pub fn apply(self, below: f32, this: f32) -> f32 {
        let (a, b) = (below, this);
        let mixed = match self {
            Mix::Normal => b,
            Mix::Add => a + b,
            Mix::Subtract => a - b,
            Mix::Multiply => a * b,
            Mix::Screen => 1.0 - (1.0 - a) * (1.0 - b),
            Mix::Overlay => {
                if a < 0.5 {
                    2.0 * a * b
                } else {
                    1.0 - 2.0 * (1.0 - a) * (1.0 - b)
                }
            }
            Mix::Dodge => {
                if b >= 1.0 {
                    1.0
                } else {
                    a / (1.0 - b)
                }
            }
            Mix::Burn => {
                if b <= 0.0 {
                    0.0
                } else {
                    1.0 - (1.0 - a) / b
                }
            }
            Mix::Lighten => a.max(b),
            Mix::Darken => a.min(b),
            Mix::Difference => (a - b).abs(),
        };
        mixed.clamp(0.0, 1.0)
    }

    /// The mode faded by `opacity` (0 = just what's below, 1 = the full mix).
    pub fn over(self, below: f32, this: f32, opacity: f32) -> f32 {
        (below + (self.apply(below, this) - below) * opacity.clamp(0.0, 1.0)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_and_black_are_the_neutral_values_they_should_be() {
        for a in [0.0, 0.3, 0.8, 1.0] {
            assert!((Mix::Multiply.apply(a, 1.0) - a).abs() < 1e-6, "multiply by white keeps it");
            assert!((Mix::Screen.apply(a, 0.0) - a).abs() < 1e-6, "screen with black keeps it");
            assert!((Mix::Add.apply(a, 0.0) - a).abs() < 1e-6);
            assert!((Mix::Subtract.apply(a, 0.0) - a).abs() < 1e-6);
            assert!((Mix::Dodge.apply(a, 0.0) - a).abs() < 1e-6);
            assert!((Mix::Burn.apply(a, 1.0) - a).abs() < 1e-6);
        }
    }

    #[test]
    fn opacity_fades_between_below_and_the_mix() {
        assert_eq!(Mix::Normal.over(0.2, 0.8, 0.0), 0.2);
        assert_eq!(Mix::Normal.over(0.2, 0.8, 1.0), 0.8);
        assert!((Mix::Normal.over(0.2, 0.8, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn every_mode_stays_between_black_and_white() {
        for mix in Mix::ALL {
            for a in [0.0, 0.25, 0.5, 0.75, 1.0] {
                for b in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    let v = mix.apply(a, b);
                    assert!((0.0..=1.0).contains(&v), "{} gave {v} for {a}, {b}", mix.name());
                }
            }
        }
    }
}
