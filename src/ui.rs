//! The debug panel's egui, as the game's systems see it: the same names `bevy_egui` gave them
//! (`EguiContexts`, `ctx_mut`, the `EguiPrimaryContextPass` schedule), so the panel's pages didn't
//! change when bevy_egui went (the regroup, docs/regroup.md). The runner (`runner/`) starts egui's
//! frame before the game's update, runs `EguiPrimaryContextPass` after it, and draws what it made.

use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// egui's context (cheap to clone: it's shared).
#[derive(Resource, Clone, Default)]
pub struct EguiCtx(pub egui::Context);

/// The panel's egui, for a system: `let ctx = contexts.ctx_mut()?;`.
#[derive(SystemParam)]
pub struct EguiContexts<'w> {
    ctx: ResMut<'w, EguiCtx>,
}

impl EguiContexts<'_> {
    pub fn ctx_mut(&mut self) -> Result<&mut egui::Context, BevyError> {
        Ok(&mut self.ctx.0)
    }
}

/// Where the panel's systems run: after the game's update, inside egui's frame.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EguiPrimaryContextPass;

/// Whether the panel wants the keyboard (a text box being typed in), as of the last frame: the game
/// leaves it alone then.
#[derive(Resource, Clone, Copy, Default)]
pub struct EguiWantsInput {
    pub keyboard: bool,
}

impl EguiWantsInput {
    pub fn wants_any_keyboard_input(&self) -> bool {
        self.keyboard
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EguiCtx>().init_resource::<EguiWantsInput>().init_schedule(EguiPrimaryContextPass);
        let schedule = app.get_schedule_mut(EguiPrimaryContextPass).expect("just made");
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    }
}
