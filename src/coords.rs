//! Coordinates — the ONE place where game axes meet Bevy's axes.
//!
//! Game axes (used by every game system):
//!   +X = north   −X = south
//!   +Y = east    −Y = west
//!   +Z = up      Z = 0 is bedrock
//!
//! Bevy axes (used only for drawing):
//!   +X = right   +Y = up   −Z = forward (into the screen)
//!
//! We map: game north → Bevy forward (−Z), game east → Bevy right (+X), game up → Bevy up (+Y).
//! So someone facing north has east on their right, as David specified.
//!
//! The game's axes are a mirror image of Bevy's (left-handed vs right-handed);
//! this mapping includes the mirror, so nothing else in the code ever has to care.

use bevy::math::Vec3;

/// Game position → Bevy position (for placing things on screen).
pub fn game_to_bevy(game: Vec3) -> Vec3 {
    Vec3::new(game.y, game.z, -game.x)
}

/// Bevy position → game position (e.g. turning a camera position back into game terms).
pub fn bevy_to_game(bevy: Vec3) -> Vec3 {
    Vec3::new(-bevy.z, bevy.x, bevy.y)
}

// `cargo test` runs these checks automatically, so a mistake here gets caught.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn north_is_forward_east_is_right_up_is_up() {
        assert_eq!(game_to_bevy(Vec3::X), Vec3::NEG_Z); // north → forward
        assert_eq!(game_to_bevy(Vec3::Y), Vec3::X); // east → right
        assert_eq!(game_to_bevy(Vec3::Z), Vec3::Y); // up → up
    }

    #[test]
    fn round_trip() {
        let p = Vec3::new(12.5, -300.0, 144.0);
        assert_eq!(bevy_to_game(game_to_bevy(p)), p);
    }
}
