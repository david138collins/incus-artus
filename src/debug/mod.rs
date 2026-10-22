//! Debug — the developer panel.
//!
//! The panel is a window of tabbed *pages*. Each module adds its own page with
//! one line in its plugin:
//!
//! ```ignore
//! app.add_debug_page("Time", time_debug_page);
//! ```
//!
//! A page is a plain function that receives the panel area (`ui`) to draw into, and
//! the whole game `World` so it can show and change anything: resources, entities, settings.
//! The rule that "systems don't reach into each other" deliberately has one exception:
//! the debug panel. Its whole job is to look at, and poke, everything.
//!
//! Press F3 to show or hide the panel.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext, egui};

/// What a debug page is: a function that draws into the panel.
pub type DebugPageFn = fn(&mut egui::Ui, &mut World);

struct DebugPage {
    name: &'static str,
    draw: DebugPageFn,
}

/// The panel's state: which pages exist, which one is open, and whether it's shown.
#[derive(Resource)]
pub struct DebugPanel {
    pages: Vec<DebugPage>,
    active: usize,
    pub visible: bool,
}

impl Default for DebugPanel {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            active: 0,
            visible: true,
        }
    }
}

/// Adds `app.add_debug_page(...)` to Bevy's `App`, so any plugin can register a page.
pub trait DebugAppExt {
    fn add_debug_page(&mut self, name: &'static str, draw: DebugPageFn) -> &mut Self;
}

impl DebugAppExt for App {
    fn add_debug_page(&mut self, name: &'static str, draw: DebugPageFn) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<DebugPanel>()
            .pages
            .push(DebugPage { name, draw });
        self
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<DebugPanel>()
            .add_systems(Update, toggle_panel)
            .add_systems(EguiPrimaryContextPass, draw_panel)
            .add_debug_page("Overview", overview_page);
    }
}

fn toggle_panel(keys: Res<ButtonInput<KeyCode>>, mut panel: ResMut<DebugPanel>) {
    if keys.just_pressed(KeyCode::F3) {
        panel.visible = !panel.visible;
    }
}

/// Draws the panel window: a row of page tabs, then the open page.
///
/// This is an *exclusive* system: it takes the whole `World`, which is what lets
/// each page function reach anything it wants to display.
fn draw_panel(world: &mut World) {
    if !world.resource::<DebugPanel>().visible {
        return;
    }

    // Grab a handle to egui's drawing context. It's cheap to copy and doesn't keep the World borrowed.
    let ctx = {
        let mut query = world.query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>();
        match query.single_mut(world) {
            Ok(mut context) => context.get_mut().clone(),
            Err(_) => return, // no camera yet, so nothing to draw on
        }
    };

    // Temporarily take the panel out of the World so pages can use the World freely.
    world.resource_scope(|world, mut panel: Mut<DebugPanel>| {
        egui::Window::new("Debug  [F3]")
            .default_pos([10.0, 10.0])
            .default_width(320.0)
            .show(&ctx, |ui| {
                let mut clicked = None;
                ui.horizontal_wrapped(|ui| {
                    for (index, page) in panel.pages.iter().enumerate() {
                        if ui.selectable_label(panel.active == index, page.name).clicked() {
                            clicked = Some(index);
                        }
                    }
                });
                if let Some(index) = clicked {
                    panel.active = index;
                }
                ui.separator();
                if let Some(page) = panel.pages.get(panel.active) {
                    (page.draw)(ui, world);
                }
            });
    });
}

/// Overview page: general health of the program.
fn overview_page(ui: &mut egui::Ui, world: &mut World) {
    let diagnostics = world.resource::<DiagnosticsStore>();
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    ui.label(format!("FPS: {fps:.0}"));
    ui.label(format!("Frame time: {frame_ms:.2} ms"));

    let window_size = world
        .query::<&Window>()
        .iter(world)
        .next()
        .map(|w| format!("{} × {}", w.width() as u32, w.height() as u32));
    if let Some(size) = window_size {
        ui.label(format!("Window: {size}"));
    }

    ui.separator();
    ui.small("F3 hides this panel.");
}
