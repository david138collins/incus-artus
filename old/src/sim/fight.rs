//! Fights on the books — the generic fight resolution for every fight that isn't played out.
//!
//! Compare the two sides' strengths, work out the d20 number the attacker needs, roll it
//! through Fate, then work out the losses. David's example: "Village A needs to roll a 16 to win
//! this raid. They rolled a 10, so they lost 4 of their 10 fighters."
//!
//! The same routine (or a specific subset of it) finishes fights the local simulation had to
//! let go of, such as the player falling while followers fight on.

use crate::fate::{D20, Fate, Wanted};

/// One side of a fight.
#[derive(Clone, Debug)]
pub struct Side {
    pub fighters: u32,
    /// How strong each fighter is (1 = an ordinary militia fighter).
    pub strength_each: f32,
}

impl Side {
    fn strength(&self) -> f32 {
        self.fighters as f32 * self.strength_each
    }
}

/// Defenders fight on their own ground: their strength counts this much more.
pub const HOME_GROUND: f32 = 1.2;

#[derive(Clone, Copy, Debug)]
pub struct FightResult {
    pub roll: D20,
    pub attacker_lost: u32,
    pub defender_lost: u32,
    /// A natural 20: the stuff of news.
    pub remarkable: bool,
}

impl FightResult {
    pub fn attacker_won(&self) -> bool {
        self.roll.success
    }
}

/// The d20 number the attacker needs: 11 when evenly matched, lower when stronger, never
/// below 2 or above 20 (there's always a chance either way).
pub fn needed_to_win(attacker: &Side, defender: &Side) -> u8 {
    let a = attacker.strength();
    let d = defender.strength() * HOME_GROUND;
    if a + d <= 0.0 {
        return 11;
    }
    (11.0 + 9.0 * (d - a) / (d + a)).round().clamp(2.0, 20.0) as u8
}

/// Resolve a fight. `category` is the Fate category (`world.raid`); `wanted` is whether the
/// player wants the attacker to win.
pub fn resolve(fate: &mut Fate, category: &str, attacker: &Side, defender: &Side, wanted: Wanted) -> FightResult {
    let needed = needed_to_win(attacker, defender);
    let roll = fate.d20(category, needed, wanted);
    let (attacker_lost, defender_lost) = losses(attacker, defender, roll);
    FightResult { roll, attacker_lost, defender_lost, remarkable: roll.rolled == 20 }
}

/// Who loses how many. Losing badly costs the attacker more: missing a 16 by 6 loses 6/16 of them.
fn losses(attacker: &Side, defender: &Side, roll: D20) -> (u32, u32) {
    let needed = roll.needed.max(1) as f32;
    let rolled = roll.rolled as f32;
    let (a, d) = (attacker.fighters as f32, defender.fighters as f32);
    let (attacker_lost, defender_lost) = if roll.success {
        let margin = rolled - needed;
        (a * 0.1 * needed / 20.0, d * (0.3 + 0.05 * margin))
    } else {
        (a * (needed - rolled) / needed, d * 0.1 * rolled / needed)
    };
    (
        (attacker_lost.round() as u32).min(attacker.fighters),
        (defender_lost.round() as u32).min(defender.fighters),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn davids_example() {
        // Needed a 16, rolled a 10, with 10 fighters: 4 lost.
        let attacker = Side { fighters: 10, strength_each: 1.0 };
        let defender = Side { fighters: 10, strength_each: 1.0 };
        let roll = D20 { needed: 16, rolled: 10, success: false };
        assert_eq!(losses(&attacker, &defender, roll).0, 4);
    }

    #[test]
    fn stronger_sides_need_less() {
        let weak = Side { fighters: 4, strength_each: 1.0 };
        let strong = Side { fighters: 20, strength_each: 1.0 };
        assert!(needed_to_win(&strong, &weak) < 11);
        assert!(needed_to_win(&weak, &strong) > 11);
        assert!((2..=20).contains(&needed_to_win(&Side { fighters: 1000, strength_each: 1.0 }, &weak)));
    }

    #[test]
    fn nobody_loses_more_than_they_have() {
        let mut fate = Fate::new(3);
        for _ in 0..200 {
            let a = Side { fighters: 3, strength_each: 1.0 };
            let d = Side { fighters: 2, strength_each: 2.0 };
            let result = resolve(&mut fate, "test", &a, &d, Wanted::Either);
            assert!(result.attacker_lost <= 3 && result.defender_lost <= 2);
        }
    }
}
