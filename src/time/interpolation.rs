//! Interpolation — smooth display of things that move on fixed ticks.
//!
//! The simulation moves things 30 times a second, but the screen draws 60+ times a second.
//! Drawing an object exactly where the last tick left it makes motion look jerky.
//! Instead, we remember where it was on the previous tick and where it is now, and draw it
//! part-way between, according to how far we are into the next tick.
//!
//! Usage: give an entity a `SimPosition` (in game coordinates) and a `Transform`.
//! Simulation systems change `SimPosition::current` on ticks; this module sets the
//! `Transform` every frame. Simulation code never touches `Transform`.

use bevy::prelude::*;
use bevy::transform::TransformSystems;
use serde::{Deserialize, Serialize};

/// A position owned by the simulation, in game coordinates (X north, Y east, Z up).
#[derive(Component, Debug, Clone, Copy)]
pub struct SimPosition {
    /// Where the most recent tick put it.
    pub current: Vec3,
    /// Where it was one tick earlier.
    pub previous: Vec3,
}

impl SimPosition {
    pub fn new(position: Vec3) -> Self {
        Self {
            current: position,
            previous: position,
        }
    }

    /// Jump straight to a position, without sliding there on screen.
    pub fn teleport(&mut self, position: Vec3) {
        self.current = position;
        self.previous = position;
    }

    /// Where to draw it this frame (part-way between the last two ticks).
    pub fn display(&self, fixed_time: &Time<Fixed>, settings: &InterpolationSettings) -> Vec3 {
        // From the last tick's place by the share of the way to this one's: glam's `lerp` sums
        // `previous × (1 − f) + current × f`, which far from the world's middle rounds a step off the
        // place itself, differently each frame, even when the two are the same, and a standing
        // pawn's camera quivered by that step (0.001 blocks at 9,300 out) [David, build 32:
        // "unfortunately, it is jittering. i can see it in the gabbro blocks"].
        if self.previous == self.current {
            return self.current;
        }
        self.previous + (self.current - self.previous) * blend_fraction(fixed_time, settings)
    }
}

/// 0.0 = just after a tick, 1.0 = the next tick is due. Always 1.0 with smoothing off.
/// Use it to draw any value that changes on ticks smoothly: `previous.lerp(current, fraction)`.
pub fn blend_fraction(fixed_time: &Time<Fixed>, settings: &InterpolationSettings) -> f32 {
    if settings.enabled {
        fixed_time.overstep_fraction()
    } else {
        1.0
    }
}

/// Lets the debug panel switch smoothing off, to see the raw ticks.
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InterpolationSettings {
    pub enabled: bool,
}

impl Default for InterpolationSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

pub(super) struct InterpolationPlugin;

impl Plugin for InterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedFirst, remember_previous_positions)
            .add_systems(
                PostUpdate,
                draw_interpolated_positions.before(TransformSystems::Propagate),
            );
    }
}

/// At the start of each tick, "current" becomes "previous".
fn remember_previous_positions(mut query: Query<&mut SimPosition>) {
    for mut position in &mut query {
        position.previous = position.current;
    }
}

/// Every frame, place the visual between the previous and current tick positions.
fn draw_interpolated_positions(
    fixed_time: Res<Time<Fixed>>,
    settings: Res<InterpolationSettings>,
    mut query: Query<(&SimPosition, &mut Transform)>,
    origin: Res<crate::coords::RenderOrigin>,
) {
    for (position, mut transform) in &mut query {
        transform.translation = origin.to_bevy(position.display(&fixed_time, &settings));
    }
}
