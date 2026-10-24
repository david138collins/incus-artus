//! Panini projection — an optional full-screen pass that re-maps the camera image
//! (shader: `assets/shaders/panini.wesl`).
//!
//! The camera keeps rendering a normal image at the chosen field of view. Panini then squeezes
//! the stretched edges back in and enlarges the middle, so a wide FOV ("Quake Pro") can keep the
//! natural-looking centre of a narrow one.
//!
//! - **distance**: 0 = no effect (a normal camera), 1 = full Panini.
//! - **crop to fit**: the squeeze leaves empty corners; 1 zooms in until they're gone,
//!   0 leaves them black.
//!
//! The centre of the screen never moves, so the crosshair and block targeting stay exact.

use super::FlyCamera;
use crate::debug::SettingsAppExt;
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

pub(super) struct PaniniPlugin;

impl Plugin for PaniniPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FullscreenMaterialPlugin::<PaniniPass>::default())
            .init_resource::<PaniniSettings>()
            .add_systems(PostUpdate, apply_panini)
            .add_settings::<PaniniSettings>("panini");
    }
}

/// Tunables (shown on the "Camera" debug page, and saved in "everything" presets).
#[derive(Resource, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PaniniSettings {
    pub enabled: bool,
    /// 0 = no effect, 1 = full Panini.
    pub distance: f32,
    /// 0 = leave the empty corners black, 1 = zoom in until they're gone.
    pub crop_to_fit: f32,
}

impl Default for PaniniSettings {
    fn default() -> Self {
        Self {
            // David's choice, 22 Oct 2026, with a 100° field of view.
            enabled: true,
            distance: 0.3,
            crop_to_fit: 1.0,
        }
    }
}

/// The values the shader gets, attached to the camera while Panini is on.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
#[extract_app(RenderApp)]
pub struct PaniniPass {
    /// x, y = tan of half the FOV (sideways, up), z = distance, w = scale.
    params: Vec4,
}

impl FullscreenMaterial for PaniniPass {
    fn fragment_shader() -> ShaderRef {
        "shaders/panini.wesl".into()
    }
}

/// Half-size of the view at distance 1, from the vertical FOV (radians) and width ÷ height.
fn view_extents(fov: f32, aspect: f32) -> Vec2 {
    let up = (fov * 0.5).tan();
    Vec2::new(up * aspect, up)
}

/// How far to zoom in so the squeezed image's corners are covered (1 = not at all).
/// The same sums as the shader, run backwards from the corner of the view.
fn crop_scale(extents: Vec2, distance: f32) -> f32 {
    let view_dist = 1.0 + distance;
    let hyp = (extents.x * extents.x + 1.0).sqrt();
    let cyl_dist_minus_d = 1.0 / hyp;
    let cyl_dist = cyl_dist_minus_d + distance;
    let crop = extents * cyl_dist_minus_d * (view_dist / cyl_dist);
    (crop.x / extents.x).min(crop.y / extents.y).clamp(0.0, 1.0)
}

/// The scale the shader uses for these settings.
fn scale_for(settings: &PaniniSettings, extents: Vec2) -> f32 {
    1.0 + (crop_scale(extents, settings.distance) - 1.0) * settings.crop_to_fit
}

/// Attach, update or remove the pass on the camera to match the settings and the current FOV.
fn apply_panini(
    mut commands: Commands,
    settings: Res<PaniniSettings>,
    cameras: Query<(Entity, &Projection, Option<&PaniniPass>), With<FlyCamera>>,
) {
    for (entity, projection, pass) in &cameras {
        let Projection::Perspective(perspective) = projection else {
            continue;
        };
        if !settings.enabled || settings.distance <= 0.0 {
            if pass.is_some() {
                commands.entity(entity).remove::<PaniniPass>();
            }
            continue;
        }
        let extents = view_extents(perspective.fov, perspective.aspect_ratio);
        let params = extents.extend(settings.distance).extend(scale_for(&settings, extents));
        if pass.is_none_or(|pass| pass.params != params) {
            commands.entity(entity).insert(PaniniPass { params });
        }
    }
}

/// The Panini part of the Camera debug page.
pub fn panini_debug_section(ui: &mut egui::Ui, world: &mut World) {
    let view = world
        .query_filtered::<&Projection, With<FlyCamera>>()
        .iter(world)
        .next()
        .and_then(|projection| match projection {
            Projection::Perspective(perspective) => Some((perspective.fov, perspective.aspect_ratio)),
            _ => None,
        });
    let mut settings = world.resource_mut::<PaniniSettings>();
    ui.strong("Panini projection");
    ui.checkbox(&mut settings.enabled, "on");
    ui.add(crate::debug::Slider::new(&mut settings.distance, 0.0..=1.0).text("distance (0 = normal, 1 = full)"));
    ui.add(crate::debug::Slider::new(&mut settings.crop_to_fit, 0.0..=1.0).text("crop to fit (hide empty corners)"));
    if let Some((fov, aspect)) = view {
        let extents = view_extents(fov, aspect);
        let scale = if settings.enabled { scale_for(&settings, extents) } else { 1.0 };
        // In the very middle the image is only scaled, so it looks like a normal camera with this FOV.
        let centre = 2.0 * (extents.y * scale).atan();
        ui.label(format!(
            "Field of view now: {:.0}° up-down, {:.0}° side to side.",
            fov.to_degrees(),
            (2.0 * extents.x.atan()).to_degrees()
        ));
        ui.label(format!("The centre of the screen looks like {:.0}° up-down.", centre.to_degrees()));
    }
    if ui.small_button("Reset Panini settings").clicked() {
        *settings = PaniniSettings::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_distance_means_no_crop() {
        let extents = view_extents(100f32.to_radians(), 16.0 / 9.0);
        assert!((crop_scale(extents, 0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn full_panini_zooms_in() {
        let extents = view_extents(100f32.to_radians(), 16.0 / 9.0);
        let scale = crop_scale(extents, 1.0);
        assert!(scale > 0.2 && scale < 1.0, "{scale}");
    }
}
