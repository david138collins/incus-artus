//! Screen — which "program inside the program" is showing: the world, or the World Gen suite.
//!
//! Press F2 to switch. While the suite is showing, the physical world is switched off:
//! - simulation time is paused (no ticks: no physics, no world simulation, no people);
//! - the mouse is released, and the camera, the walking body and the on-screen notices stop
//!   reading input or drawing;
//! - the suite covers the whole window.
//!
//! Coming back puts time back the way it was (paused stays paused).
//!
//! For automatic testing, starting the game with `INCUS_START_SCREEN=worldgen` opens the suite
//! straight away.

use crate::camera::MouseCapture;
use crate::time::TimeControl;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Screen {
    /// The world: walking, flying, building.
    #[default]
    Play,
    /// The World Gen suite (see the `worldgen` module).
    WorldGen,
}

/// Was time already paused when the suite opened? (So closing it doesn't un-pause by surprise.)
#[derive(Resource, Default)]
struct PausedBeforeSuite(bool);

pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        let start = match std::env::var("INCUS_START_SCREEN").as_deref() {
            Ok("worldgen") => Screen::WorldGen,
            _ => Screen::Play,
        };
        app.insert_state(start)
            .init_resource::<PausedBeforeSuite>()
            .add_systems(Update, switch_screen)
            .add_systems(OnEnter(Screen::WorldGen), switch_world_off)
            .add_systems(OnExit(Screen::WorldGen), switch_world_on);
    }
}

fn switch_screen(keys: Res<ButtonInput<KeyCode>>, screen: Res<State<Screen>>, mut next: ResMut<NextState<Screen>>) {
    if keys.just_pressed(KeyCode::F2) {
        next.set(match screen.get() {
            Screen::Play => Screen::WorldGen,
            Screen::WorldGen => Screen::Play,
        });
    }
}

fn switch_world_off(
    mut control: ResMut<TimeControl>,
    mut paused_before: ResMut<PausedBeforeSuite>,
    mut capture: ResMut<MouseCapture>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    paused_before.0 = control.paused;
    control.paused = true;
    capture.captured = false;
    for mut cursor in &mut cursors {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

fn switch_world_on(mut control: ResMut<TimeControl>, paused_before: Res<PausedBeforeSuite>) {
    control.paused = paused_before.0;
}
