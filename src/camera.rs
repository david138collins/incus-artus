//! The flying camera, in the game's axes.
//!
//! The game's axes, as in old [David: facing north, east is on your right]:
//!   +x north, −x south;  +y east, −y west;  +z up (z = 0 is bedrock).
//! Facing is measured from north toward east, in degrees (east is 90°); pitch is up from level.
//!
//! Its controls are old's, so the hands already know them:
//!   Tab            capture / release the mouse (released, the panel can be used)
//!   mouse          look around (while captured)
//!   W A S D        fly forward / left / back / right, level whatever the tilt
//!   Space / Ctrl   fly up / down
//!   Shift          fly faster (five times, old's)
//! Its speed and Shift's multiplier are set on the panel's Camera page (old's 20 blocks a second and 5 by default).
//! It runs on real time: a tool for looking at the world, not part of it.

use glam::Vec3;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Degrees turned for each pixel the mouse moves (old's).
const SENSITIVITY: f32 = 0.1;

/// How fast the camera flies: blocks a second, and how many times that with Shift held. Set on the panel's Camera page
/// [David, 2026-11-12, old's 20 a second too slow to see against the far cells: "i should have a speed and multiplier
/// slider in the camera tab please"].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pace {
    pub speed: f32,
    pub multiplier: f32,
}

impl Pace {
    /// Old's.
    pub const DEFAULT: Pace = Pace { speed: 20.0, multiplier: 5.0 };
}

#[derive(Clone, Copy)]
pub struct Camera {
    /// Where the eye is: north, east, up, in blocks.
    pub position: Vec3,
    /// Degrees from north toward east.
    pub facing: f32,
    /// Degrees up from level.
    pub pitch: f32,
    pub pace: Pace,
}

impl Camera {
    pub fn new() -> Self {
        Self { position: Vec3::ZERO, facing: 0.0, pitch: 0.0, pace: Pace::DEFAULT }
    }

    /// The mouse moved (while captured): the camera turns.
    pub fn look(&mut self, right: f32, down: f32) {
        self.facing = (self.facing + right * SENSITIVITY).rem_euclid(360.0);
        self.pitch = (self.pitch - down * SENSITIVITY).clamp(-89.0, 89.0);
    }

    /// The keys held for `seconds`: the camera flies.
    pub fn fly(&mut self, held: &HashSet<KeyCode>, seconds: f32) {
        let (sin, cos) = self.facing.to_radians().sin_cos();
        let forward = Vec3::new(cos, sin, 0.0);
        let right = Vec3::new(-sin, cos, 0.0);
        let mut way = Vec3::ZERO;
        for (key, step) in [(KeyCode::KeyW, forward), (KeyCode::KeyS, -forward), (KeyCode::KeyD, right), (KeyCode::KeyA, -right), (KeyCode::Space, Vec3::Z), (KeyCode::ControlLeft, -Vec3::Z)] {
            if held.contains(&key) {
                way += step;
            }
        }
        let fast = held.contains(&KeyCode::ShiftLeft) || held.contains(&KeyCode::ShiftRight);
        self.position += way.normalize_or_zero() * self.pace.speed * if fast { self.pace.multiplier } else { 1.0 } * seconds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// It flies at the speed and multiplier set: a second forward, a second forward with Shift, a second up.
    #[test]
    fn it_flies_at_the_speed_and_multiplier_set() {
        let mut camera = Camera::new();
        camera.pace = Pace { speed: 100.0, multiplier: 3.0 };
        camera.fly(&HashSet::from([KeyCode::KeyW]), 1.0);
        assert!((camera.position - Vec3::new(100.0, 0.0, 0.0)).length() < 1e-3, "{}", camera.position);
        camera.fly(&HashSet::from([KeyCode::KeyW, KeyCode::ShiftLeft]), 1.0);
        assert!((camera.position - Vec3::new(400.0, 0.0, 0.0)).length() < 1e-3, "{}", camera.position);
        camera.pace = Pace::DEFAULT;
        camera.fly(&HashSet::from([KeyCode::Space]), 1.0);
        assert!((camera.position - Vec3::new(400.0, 0.0, 20.0)).length() < 1e-3, "{}", camera.position);
    }
}
