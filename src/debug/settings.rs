//! "Everything" presets — save every setting on the debug panel in one file, and load it back.
//!
//! Each module registers its settings with one line in its plugin:
//!
//! ```ignore
//! app.add_settings::<PlayerSettings>("player");
//! ```
//!
//! The preset bar for all of them sits at the top of the Overview page. A preset is one TOML
//! file in `presets/everything/`, with one section per group (`[player]`, `[camera]`, …).
//!
//! Loading a preset that's missing a group (saved before that group existed) leaves that
//! group as it is now, and says so.

use super::preset_bar;
use bevy::prelude::*;
use bevy_egui::egui;
use serde::Serialize;
use serde::de::DeserializeOwned;

type SaveFn = fn(&World) -> Result<toml::Value, String>;
type LoadFn = fn(&mut World, toml::Value) -> Result<(), String>;

struct SettingsGroup {
    name: &'static str,
    save: SaveFn,
    load: LoadFn,
}

/// Every registered settings group, plus the result of the last load.
#[derive(Resource, Default)]
pub struct SettingsGroups {
    groups: Vec<SettingsGroup>,
    last_report: Option<String>,
}

/// Adds `app.add_settings(...)` to Bevy's `App`.
pub trait SettingsAppExt {
    /// Register a settings resource that can be saved and loaded as it is.
    fn add_settings<T: Resource + Serialize + DeserializeOwned>(&mut self, name: &'static str) -> &mut Self;
    /// Register a group with its own save and load (for settings spread over many things).
    fn add_settings_with(&mut self, name: &'static str, save: SaveFn, load: LoadFn) -> &mut Self;
}

impl SettingsAppExt for App {
    fn add_settings<T: Resource + Serialize + DeserializeOwned>(&mut self, name: &'static str) -> &mut Self {
        self.add_settings_with(name, save_resource::<T>, load_resource::<T>)
    }

    fn add_settings_with(&mut self, name: &'static str, save: SaveFn, load: LoadFn) -> &mut Self {
        self.world_mut()
            .get_resource_or_init::<SettingsGroups>()
            .groups
            .push(SettingsGroup { name, save, load });
        self
    }
}

fn save_resource<T: Resource + Serialize>(world: &World) -> Result<toml::Value, String> {
    let settings = world.get_resource::<T>().ok_or("not set up yet")?;
    toml::Value::try_from(settings).map_err(|error| error.to_string())
}

fn load_resource<T: Resource + DeserializeOwned>(world: &mut World, value: toml::Value) -> Result<(), String> {
    let settings: T = value.try_into().map_err(|error| error.to_string())?;
    world.insert_resource(settings);
    Ok(())
}

/// Everything, as one TOML table (what a preset file holds).
fn save_all(world: &World, groups: &SettingsGroups) -> toml::Table {
    let mut table = toml::Table::new();
    for group in &groups.groups {
        if let Ok(value) = (group.save)(world) {
            table.insert(group.name.to_string(), value);
        }
    }
    table
}

/// The preset bar for everything, drawn at the top of the Overview page.
pub fn everything_presets(ui: &mut egui::Ui, world: &mut World) {
    world.resource_scope(|world, mut groups: Mut<SettingsGroups>| {
        ui.strong("Everything preset");
        ui.small("Saves and loads every setting on this panel at once.");

        // What everything is set to right now.
        let mut current = save_all(world, &groups);
        let before = current.clone();
        preset_bar(ui, "everything", &mut current);

        // A preset was loaded: hand each group its part.
        if current != before {
            let mut loaded = Vec::new();
            let mut problems = Vec::new();
            for group in &groups.groups {
                match current.get(group.name) {
                    Some(value) => match (group.load)(world, value.clone()) {
                        Ok(()) => loaded.push(group.name),
                        Err(error) => problems.push(format!("{}: {error}", group.name)),
                    },
                    None => problems.push(format!("{}: not in this preset, left as it was", group.name)),
                }
            }
            let mut report = format!("Loaded: {}", loaded.join(", "));
            for problem in problems {
                report.push_str(&format!("\n{problem}"));
            }
            groups.last_report = Some(report);
        }
        if let Some(report) = &groups.last_report {
            ui.small(report);
        }
        let names: Vec<&str> = groups.groups.iter().map(|group| group.name).collect();
        ui.small(format!("Groups: {}", names.join(", ")));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::CameraSettings;
    use crate::data::BlockRegistry;
    use crate::player::PlayerSettings;

    #[test]
    fn everything_survives_a_round_trip_through_a_file() {
        let mut app = App::new();
        app.init_resource::<PlayerSettings>()
            .init_resource::<CameraSettings>()
            .init_resource::<BlockRegistry>()
            .add_settings::<PlayerSettings>("player")
            .add_settings::<CameraSettings>("camera")
            .add_settings_with("blocks", crate::data::save_block_tweaks, crate::data::load_block_tweaks);
        let world = app.world_mut();

        // Change a few things, save to text.
        world.resource_mut::<PlayerSettings>().walk_speed = 2.5;
        world.resource_mut::<CameraSettings>().field_of_view = 88.0;
        let stone = world.resource::<BlockRegistry>().id("incus:stone").unwrap();
        world.resource_mut::<BlockRegistry>().def_mut(stone).unwrap().friction = 0.1;
        let groups = world.remove_resource::<SettingsGroups>().unwrap();
        let text = toml::to_string_pretty(&save_all(world, &groups)).expect("saves");

        // Put everything back to defaults, then load the text.
        world.insert_resource(PlayerSettings::default());
        world.insert_resource(CameraSettings::default());
        world.insert_resource(BlockRegistry::default());
        let table: toml::Table = toml::from_str(&text).expect("parses");
        for group in &groups.groups {
            (group.load)(world, table[group.name].clone()).expect("loads");
        }
        assert_eq!(world.resource::<PlayerSettings>().walk_speed, 2.5);
        assert_eq!(world.resource::<CameraSettings>().field_of_view, 88.0);
        assert_eq!(world.resource::<BlockRegistry>().def(stone).unwrap().friction, 0.1);
    }
}
