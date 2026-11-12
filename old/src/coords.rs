//! Coordinates — the ONE place where game axes meet Bevy's axes.
//!
//! [The regroup, docs/regroup.md: Bevy's renderer was cut, so nothing reads this for now. The new
//! renderer (phase 1) chooses its own axes and keeps the floating origin below; this file is
//! rewritten or goes then.]
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

//! **Drawn from a point near you (the floating origin)** [`docs/far-view.md` R1; David, 2026-11-02:
//! "it seems insignificant but also like it could cause problems later"]: a 32-bit position is kept
//! only to about 1/500 of a block 18,000 blocks out, so a world placed from its middle shimmered
//! (blocks' corners rounding as the camera moved). What's drawn is placed from `RenderOrigin`
//! instead, a point on a 1,024-block grid near the camera, moved (and everything drawn shifted with
//! it) when the camera has gone further than that. Game positions stay the world's; only Bevy's are
//! measured from the origin: `RenderOrigin::to_bevy` and `to_game` turn one into the other.

#![allow(dead_code)]

use crate::ecs::math::{IVec3, Vec3};
use crate::ecs::prelude::Resource;

/// Game position → Bevy position (for placing things on screen).
pub fn game_to_bevy(game: Vec3) -> Vec3 {
    Vec3::new(game.y, game.z, -game.x)
}

/// Bevy position → game position (e.g. turning a camera position back into game terms).
pub fn bevy_to_game(bevy: Vec3) -> Vec3 {
    Vec3::new(-bevy.z, bevy.x, bevy.y)
}

/// The point (game axes, blocks; its height always 0) that what's drawn is placed from.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderOrigin(pub IVec3);

/// How far apart the origin's places are (blocks): it moves when the camera is further than this
/// from it, either way, to the place nearest the camera.
pub const ORIGIN_GRID: i32 = 1_024;

impl RenderOrigin {
    /// A game position, placed for drawing (measured from the origin, in Bevy's axes).
    pub fn to_bevy(self, game: Vec3) -> Vec3 {
        game_to_bevy(game - self.0.as_vec3())
    }

    /// A drawn position back in the game's terms.
    pub fn to_game(self, bevy: Vec3) -> Vec3 {
        bevy_to_game(bevy) + self.0.as_vec3()
    }

    /// Where the origin should be for a camera at `at` (game axes): unchanged while it's within the
    /// grid's spacing, else the grid's place nearest the camera.
    pub fn for_camera(self, at: Vec3) -> RenderOrigin {
        let offset = at - self.0.as_vec3();
        if offset.x.abs() <= ORIGIN_GRID as f32 && offset.y.abs() <= ORIGIN_GRID as f32 {
            return self;
        }
        let snap = |v: f32| (v / ORIGIN_GRID as f32).round() as i32 * ORIGIN_GRID;
        RenderOrigin(IVec3::new(snap(at.x), snap(at.y), 0))
    }
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

    #[test]
    fn the_origin_moves_only_past_its_grid_and_places_round_trip() {
        let origin = RenderOrigin(IVec3::new(17_408, -4_096, 0));
        // Within the grid's spacing it stays; past it, it moves to the nearest place.
        assert_eq!(origin.for_camera(Vec3::new(18_000.0, -3_500.0, 900.0)), origin);
        assert_eq!(origin.for_camera(Vec3::new(18_500.0, -4_096.0, 900.0)), RenderOrigin(IVec3::new(18_432, -4_096, 0)));
        // Far from the world's middle, a place drawn near the origin is small, so kept finely, and
        // comes back as it went.
        let far = Vec3::new(17_719.25, -4_000.5, 610.375);
        let drawn = origin.to_bevy(far);
        assert!(drawn.abs().max_element() < 1_024.0);
        assert_eq!(origin.to_game(drawn), far);
    }
}
