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
    /// Pages with their own presets: the category (`presets/<category>/`) and the groups it holds.
    pages: Vec<(&'static str, &'static [&'static str])>,
}

/// Adds `app.add_settings(...)` to Bevy's `App`.
pub trait SettingsAppExt {
    /// Register a settings resource that can be saved and loaded as it is.
    fn add_settings<T: Resource + Serialize + DeserializeOwned>(&mut self, name: &'static str) -> &mut Self;
    /// Register a group with its own save and load (for settings spread over many things).
    fn add_settings_with(&mut self, name: &'static str, save: SaveFn, load: LoadFn) -> &mut Self;
    /// Give a debug page presets of its own, holding these groups (see `page_presets`).
    fn add_page_preset(&mut self, category: &'static str, groups: &'static [&'static str]) -> &mut Self;
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

    fn add_page_preset(&mut self, category: &'static str, groups: &'static [&'static str]) -> &mut Self {
        self.world_mut().get_resource_or_init::<SettingsGroups>().pages.push((category, groups));
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

// ── Page presets ────────────────────────────────────────────────────────────
//
// A page (Render, Camera) with presets of its own: the same as an "everything" preset, but only the
// groups on that page, in `presets/<category>/`. As in the Editor Suite, a preset named "default" is
// what the page starts with when the game opens, what its reset buttons go back to, and lights the
// "default" light while the page matches it. [David, 2026-10-27]

/// The groups a page's presets hold (`[]` if it has none).
fn page_groups(groups: &SettingsGroups, category: &str) -> &'static [&'static str] {
    groups.pages.iter().find(|(name, _)| *name == category).map_or(&[], |&(_, groups)| groups)
}

/// The page's groups as one TOML table (what its preset file holds).
fn save_page(world: &World, groups: &SettingsGroups, category: &str) -> toml::Table {
    let names = page_groups(groups, category);
    let mut table = toml::Table::new();
    for group in groups.groups.iter().filter(|group| names.contains(&group.name)) {
        if let Ok(value) = (group.save)(world) {
            table.insert(group.name.to_string(), value);
        }
    }
    table
}

/// Hands each of the page's groups its part of `table` (groups it doesn't hold are left as they are).
fn load_page(world: &mut World, groups: &SettingsGroups, category: &str, table: &toml::Table) {
    let names = page_groups(groups, category);
    for group in groups.groups.iter().filter(|group| names.contains(&group.name)) {
        if let Some(value) = table.get(group.name) {
            let _ = (group.load)(world, value.clone());
        }
    }
}

/// A page's preset bar, with its "default" light, at the top of the page.
pub fn page_presets(ui: &mut egui::Ui, world: &mut World, category: &str) {
    world.resource_scope(|world, groups: Mut<SettingsGroups>| {
        let mut current = save_page(world, &groups, category);
        let before = current.clone();
        ui.horizontal(|ui| {
            super::default_light(ui, category, &before);
            ui.small("Save this page's settings as a preset; one named \"default\" is what the game starts with.");
        });
        preset_bar(ui, category, &mut current);
        if current != before {
            load_page(world, &groups, category, &current);
        }
    });
}

/// One group's settings from a page's "default" preset, or the built-in ones (for the page's reset
/// buttons).
pub fn page_default<T: DeserializeOwned + Default>(category: &str, group: &str) -> T {
    super::default_preset::<toml::Table>(category).and_then(|table| table.get(group)?.clone().try_into().ok()).unwrap_or_default()
}

/// At start-up, every page with a "default" preset gets it.
pub fn use_default_page_presets(world: &mut World) {
    world.resource_scope(|world, groups: Mut<SettingsGroups>| {
        for &(category, _) in &groups.pages {
            if let Some(table) = super::default_preset::<toml::Table>(category) {
                load_page(world, &groups, category, &table);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::CameraSettings;
    use crate::data::BlockRegistry;
    use crate::player::PlayerSettings;

    #[test]
    fn a_page_preset_holds_only_its_own_groups() {
        let mut app = App::new();
        app.init_resource::<PlayerSettings>()
            .init_resource::<CameraSettings>()
            .add_settings::<PlayerSettings>("player")
            .add_settings::<CameraSettings>("camera")
            .add_page_preset("camera page", &["camera"]);
        let world = app.world_mut();
        let groups = world.remove_resource::<SettingsGroups>().unwrap();
        world.resource_mut::<CameraSettings>().field_of_view = 88.0;
        let text = toml::to_string_pretty(&save_page(world, &groups, "camera page")).unwrap();
        assert!(!text.contains("[player]"), "{text}");
        // Read back over changed settings: the page's group comes back, the others are left alone.
        world.resource_mut::<CameraSettings>().field_of_view = 60.0;
        world.resource_mut::<PlayerSettings>().walk_speed = 2.5;
        load_page(world, &groups, "camera page", &toml::from_str(&text).unwrap());
        assert_eq!(world.resource::<CameraSettings>().field_of_view, 88.0);
        assert_eq!(world.resource::<PlayerSettings>().walk_speed, 2.5);
        // And it reads back exactly as saved (for the "default" light).
        let table: toml::Table = toml::from_str(&text).unwrap();
        assert!(table == save_page(world, &groups, "camera page"));
    }

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
