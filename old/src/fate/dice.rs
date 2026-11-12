//! Dice — the random number generator underneath everything.
//!
//! Two ways to get randomness:
//! - **A stream** (`Dice`): roll after roll, each different. Its whole state is one number, so it can
//!   be saved with the world and carry on exactly where it left off.
//! - **A position hash** (`planet_hash`): a random-looking number worked out from the world seed and a
//!   position. The same seed and position always give the same number, in any order. World generation
//!   ("planet forming") uses this, so the mountains are always in the same place.
//!
//! The method is SplitMix64: tiny, fast, and plenty random for a game.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dice {
    #[serde(with = "crate::save::hex_u64")]
    state: u64,
}

impl Dice {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next random 64-bit number.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.state)
    }

    /// A random number from 0 up to (not including) 1.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A whole number from `low` to `high`, both included.
    pub fn range(&mut self, low: u32, high: u32) -> u32 {
        if high <= low {
            return low;
        }
        low + (self.next_u64() % (high - low + 1) as u64) as u32
    }
}

/// Scrambles a number thoroughly (the SplitMix64 finaliser).
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A random-looking number fixed by the seed, a purpose ("salt", so rivers and ores don't share
/// numbers) and a position. For world generation.
#[allow(dead_code)] // first used by the world-gen suite
pub fn planet_hash(seed: u64, salt: u64, x: i64, y: i64, z: i64) -> u64 {
    let mut h = mix(seed ^ mix(salt));
    for value in [x, y, z] {
        h = mix(h ^ (value as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    }
    h
}

/// `planet_hash`'s start, its seed and salt mixed: the part of it a graphics card is handed, worked out
/// on the processor (the card finishes it at each spot).
pub fn roll_start(seed: u64, salt: u64) -> u64 {
    mix(seed ^ mix(salt))
}

/// A stream for one purpose, so separate systems don't disturb each other's dice.
pub fn derive_seed(seed: u64, purpose: &str) -> u64 {
    purpose.bytes().fold(mix(seed), |h, byte| mix(h ^ byte as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_rolls() {
        let mut a = Dice::new(42);
        let mut b = Dice::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn rolls_are_spread_evenly() {
        let mut dice = Dice::new(7);
        let mut counts = [0u32; 20];
        for _ in 0..20_000 {
            counts[dice.range(1, 20) as usize - 1] += 1;
        }
        // Each face should come up about 1,000 times.
        assert!(counts.iter().all(|&count| (850..1150).contains(&count)), "{counts:?}");
    }

    #[test]
    fn planet_hash_depends_only_on_its_inputs() {
        assert_eq!(planet_hash(1, 2, 3, 4, 5), planet_hash(1, 2, 3, 4, 5));
        assert_ne!(planet_hash(1, 2, 3, 4, 5), planet_hash(1, 2, 3, 4, 6));
        assert_ne!(planet_hash(1, 2, 3, 4, 5), planet_hash(1, 3, 3, 4, 5));
    }
}
