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
//!
//! Screenshots: press F12 to save one in the `screenshots/` folder (handy for showing Claude
//! what you see). For automatic testing, starting the game with the environment variable
//! `INCUS_SELFTEST_SECONDS=N` takes `screenshots/selftest.png` after N seconds, then quits.
//! With `INCUS_SELFTEST_MINIMIZE=1` as well, it minimizes the window 8 seconds before the
//! screenshot and restores it a second later (minimizing once closed the game).
//!
//! Presets: pages can save and load their settings as named files (see `presets.rs`), and
//! the Overview page saves and loads all of them at once (see `settings.rs`).

mod presets;
mod settings;
mod slider;

pub use presets::{default_preset, preset_bar, save_shortcut};
pub use slider::Slider;
pub use settings::SettingsAppExt;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext, egui};
use std::path::PathBuf;

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
            .init_resource::<settings::SettingsGroups>()
            .init_resource::<SelfTest>()
            .add_debug_page("Overview", overview_page);
        // The panel, screenshots and self-tests are developer tools: a player's copy has the
        // pages registered (harmless) but never shows them (see `dev` in Cargo.toml).
        #[cfg(feature = "dev")]
        app.add_systems(Update, (toggle_panel, screenshot_key, run_self_test)).add_systems(EguiPrimaryContextPass, draw_panel);
    }
}

fn toggle_panel(keys: Res<ButtonInput<KeyCode>>, mut panel: ResMut<DebugPanel>) {
    if keys.just_pressed(KeyCode::F3) {
        panel.visible = !panel.visible;
    }
}

/// The project folder (the one holding `assets/`), or the game's own folder once it's shipped.
pub fn project_folder() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::current_exe().map(|exe| exe.parent().map(PathBuf::from).unwrap_or_default())
        })
        .unwrap_or_default()
}

/// The `screenshots/` folder (next to `assets/`).
fn screenshot_folder() -> PathBuf {
    project_folder().join("screenshots")
}

fn take_screenshot(commands: &mut Commands, file_name: &str) {
    let folder = screenshot_folder();
    if let Err(error) = std::fs::create_dir_all(&folder) {
        warn!("Couldn't create {}: {error}", folder.display());
        return;
    }
    let path = folder.join(file_name);
    info!("Saving screenshot to {}", path.display());
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

fn screenshot_key(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F12) {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        take_screenshot(&mut commands, &format!("screenshot-{seconds}.png"));
    }
}

/// Self-test mode (see the module notes): screenshot after N seconds, quit a moment later.
#[derive(Resource)]
struct SelfTest {
    screenshot_after: Option<f32>,
    screenshot_taken: bool,
    /// Minimize and restore the window before the screenshot.
    minimize: bool,
}

impl Default for SelfTest {
    fn default() -> Self {
        Self {
            screenshot_after: std::env::var("INCUS_SELFTEST_SECONDS")
                .ok()
                .and_then(|value| value.parse().ok()),
            screenshot_taken: false,
            minimize: std::env::var("INCUS_SELFTEST_MINIMIZE").is_ok(),
        }
    }
}

fn run_self_test(
    mut commands: Commands,
    mut test: ResMut<SelfTest>,
    time: Res<Time<Real>>,
    mut exit: MessageWriter<AppExit>,
    mut windows: Query<&mut Window>,
) {
    let Some(after) = test.screenshot_after else {
        return;
    };
    let elapsed = time.elapsed_secs();
    if test.minimize
        && let Ok(mut window) = windows.single_mut()
    {
        let previous = elapsed - time.delta_secs();
        if previous < after - 8.0 && elapsed >= after - 8.0 {
            info!("Self-test: minimizing the window");
            window.set_minimized(true);
        }
        if previous < after - 7.0 && elapsed >= after - 7.0 {
            info!("Self-test: restoring the window");
            window.set_minimized(false);
        }
    }
    if !test.screenshot_taken && elapsed >= after {
        take_screenshot(&mut commands, "selftest.png");
        test.screenshot_taken = true;
    }
    if elapsed >= after + 2.0 {
        info!("Self-test finished, quitting");
        exit.write(AppExit::Success);
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
                    // Long pages scroll instead of running off the bottom of the screen.
                    egui::ScrollArea::vertical().show(ui, |ui| (page.draw)(ui, world));
                }
            });
    });
}

/// Overview page: general health of the program.
fn overview_page(ui: &mut egui::Ui, world: &mut World) {
    settings::everything_presets(ui, world);
    ui.separator();

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
