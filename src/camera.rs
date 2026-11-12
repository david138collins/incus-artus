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
//!   Shift          fly five times faster
//! It runs on real time: a tool for looking at the world, not part of it.

use glam::Vec3;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

/// Flying speed, blocks a second (old's).
const SPEED: f32 = 20.0;
/// Shift's multiplier (old's).
const FAST: f32 = 5.0;
/// Degrees turned for each pixel the mouse moves (old's).
const SENSITIVITY: f32 = 0.1;

pub struct Camera {
    /// Where the eye is: north, east, up, in blocks.
    pub position: Vec3,
    /// Degrees from north toward east.
    pub facing: f32,
    /// Degrees up from level.
    pub pitch: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self { position: Vec3::ZERO, facing: 0.0, pitch: 0.0 }
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
        self.position += way.normalize_or_zero() * SPEED * if fast { FAST } else { 1.0 } * seconds;
    }
}
