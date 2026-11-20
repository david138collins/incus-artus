//! Presets: a page's settings saved to a named file and loaded back [David, 2026-11-18: "we do need a preset system now,
//! i think, in order for me to better check out the features as they start to grow. the same way we did it before,
//! where there's a 'default' preset, and if you save a preset named default, it becomes the default, and there's a
//! little dot that turns green when you are on the default preset. this can replace the defaulter buttons inside the
//! panel"].
//!
//! Old's way (`old/src/debug/presets.rs`) and its words, as he knows them. A page of the panel has a bar (`Bar`): the
//! dot, a name box, Save, a Load list and a button for the folder [its place, his choice of three: "On each page"].
//! Presets are plain text, `presets/<category>/<name>.toml` in the project's folder, so they can be read, compared,
//! copied or changed by hand. One saved before a setting existed still loads (the setting keeps its built-in value),
//! and a setting that no longer exists is passed over.
//!
//! The preset named "default" is what the game starts with (`start`). Till one is saved the built-in settings are the
//! default, and "default" is always in the Load list: the way back to it [his choice of two: "From the Load list
//! only"; the dot only shows].
//!
//! Since build 228 (egui out) the bar isn't drawn: it comes back, drawn by us, on the debug panel's Camera and Render
//! tabs. Till then the game still starts on each page's "default" preset, and `Bar` keeps what the bar does.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use crate::camera::Pace;
use crate::far_eye::FarLook;
use crate::near_eye::NearLook;

/// The preset's name that is a page's default.
pub const DEFAULT: &str = "default";

/// Where every category's presets are kept: `presets/` in the project's folder.
pub fn folder() -> PathBuf {
    crate::session::project_folder().join("presets")
}

/// What the Render page's presets hold: the near eye's settings, the lights' with them, and the far eye's. The lantern
/// lit or out is the L key's, not a setting: a preset doesn't keep it (`NearLook::lantern` is passed over in the file)
/// and it's out in every preset compared.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, serde::Deserialize)]
#[serde(default)]
pub struct RenderPreset {
    pub near: NearLook,
    pub far: FarLook,
}

impl RenderPreset {
    /// The settings as they are, as a preset (the Render tab's bar saves it, when it's drawn again).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn of(near: &NearLook, far: &FarLook) -> Self {
        Self { near: NearLook { lantern: false, ..*near }, far: *far }
    }
}

/// What the Camera page's presets hold: how fast the camera flies, Shift's multiplier, and the field of view.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, serde::Deserialize)]
#[serde(default)]
pub struct CameraPreset {
    pub speed: f32,
    pub multiplier: f32,
    pub field_of_view: f32,
}

impl Default for CameraPreset {
    fn default() -> Self {
        Self { speed: Pace::DEFAULT.speed, multiplier: Pace::DEFAULT.multiplier, field_of_view: crate::camera::FIELD_OF_VIEW }
    }
}

/// Letters, digits, `-` and `_` only (spaces become `_`), so any name makes a safe file name.
pub fn clean_name(name: &str) -> String {
    name.trim().chars().map(|c| if c == ' ' { '_' } else { c }).filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect()
}

/// The names of the presets saved in a folder, in alphabetical order (the bar's Load list, when it's drawn again).
#[allow(dead_code)]
fn saved(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .filter_map(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
}

/// A setting as the file shows it: a number kept in single precision is written as the shortest number that reads back
/// to it (0.05, not 0.05000000074505806).
#[cfg_attr(not(test), allow(dead_code))]
fn tidy(value: &mut toml::Value) {
    match value {
        toml::Value::Float(number) => {
            let single = *number as f32;
            if single as f64 == *number
                && let Ok(short) = single.to_string().parse::<f64>()
                && short as f32 == single
            {
                *number = short;
            }
        }
        toml::Value::Array(values) => values.iter_mut().for_each(tidy),
        toml::Value::Table(table) => table.iter_mut().for_each(|(_, value)| tidy(value)),
        _ => {}
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn save<T: Serialize>(folder: &Path, name: &str, value: &T) -> Result<PathBuf, String> {
    std::fs::create_dir_all(folder).map_err(|error| error.to_string())?;
    let mut settings = toml::Value::try_from(value).map_err(|error| error.to_string())?;
    tidy(&mut settings);
    let text = toml::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    let path = folder.join(format!("{name}.toml"));
    std::fs::write(&path, text).map_err(|error| error.to_string())?;
    Ok(path)
}

fn load<T: DeserializeOwned>(folder: &Path, name: &str) -> Result<T, String> {
    let text = std::fs::read_to_string(folder.join(format!("{name}.toml"))).map_err(|error| error.to_string())?;
    toml::from_str(&text).map_err(|error| error.to_string())
}

/// What a page starts with, and how to say so in the session's log: the preset `INCUS_START_PRESET` names for its
/// category if it names one (`render/<name>`, several with commas between: `render/dyno,camera/dyno`); else, on a
/// flight, the built-in settings (so flights stay like for like whatever David saves); else its "default" preset; else
/// the built-in settings.
pub fn start<T: DeserializeOwned + Default>(root: &Path, category: &str, flight: bool) -> (T, String) {
    let asked = std::env::var("INCUS_START_PRESET").ok();
    let named = asked.as_deref().and_then(|asked| asked.split(',').filter_map(|one| one.trim().split_once('/')).find(|(of, _)| *of == category).map(|(_, name)| clean_name(name)));
    let folder = root.join(category);
    let read = |name: &str| match load::<T>(&folder, name) {
        Ok(preset) => (preset, format!("the preset \"{name}\"")),
        Err(error) => {
            log::warn!("The {category} preset \"{name}\" couldn't be read ({error}): the built-in settings instead");
            (T::default(), format!("the built-in settings (the preset \"{name}\" couldn't be read)"))
        }
    };
    match named {
        Some(name) => read(&name),
        None if flight => (T::default(), "the built-in settings (a flight)".to_string()),
        None if folder.join(format!("{DEFAULT}.toml")).exists() => read(DEFAULT),
        None => (T::default(), "the built-in settings (no \"default\" preset saved)".to_string()),
    }
}

/// A page's preset bar, and what it remembers between frames (not drawn since build 228: see above).
#[cfg_attr(not(test), allow(dead_code))]
pub struct Bar<T> {
    folder: PathBuf,
    /// What's in the name box: "default" to begin with where there is one (so Ctrl+S saves over it straight away, as
    /// old's did), else empty.
    pub name: String,
    /// What the last save or load came to, shown small under the bar.
    status: Option<String>,
    /// The default: the "default" preset as last read (the built-in settings where there's none, or it can't be
    /// read), its file's time then, and when the file was last looked at (once a second).
    default: T,
    default_file: Option<SystemTime>,
    looked: Option<Instant>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl<T: Serialize + DeserializeOwned + Default + PartialEq + Clone> Bar<T> {
    /// The bar for `category`, its presets in `root/<category>/`.
    pub fn new(root: &Path, category: &str) -> Self {
        let folder = root.join(category);
        let name = if folder.join(format!("{DEFAULT}.toml")).exists() { DEFAULT.to_string() } else { String::new() };
        let mut bar = Self { folder, name, status: None, default: T::default(), default_file: None, looked: None };
        bar.look(true);
        bar
    }

    /// The default read again if its file has changed (looked at once a second, or `now`).
    fn look(&mut self, now: bool) {
        if !now && self.looked.is_some_and(|at| at.elapsed().as_secs_f32() < 1.0) {
            return;
        }
        self.looked = Some(Instant::now());
        let file = std::fs::metadata(self.folder.join(format!("{DEFAULT}.toml"))).and_then(|about| about.modified()).ok();
        if file != self.default_file || now {
            self.default_file = file;
            self.default = file.and_then(|_| load::<T>(&self.folder, DEFAULT).ok()).unwrap_or_default();
        }
    }

    /// Whether `value` is exactly the default.
    pub fn on_default(&mut self, value: &T) -> bool {
        self.look(false);
        *value == self.default
    }

    /// `value` saved under the name in the box (Save, and Ctrl+S): what to say of it.
    pub fn save_named(&mut self, value: &T) -> String {
        let clean = clean_name(&self.name);
        let said = if clean.is_empty() {
            "Type a name in the preset box first.".to_string()
        } else {
            match save(&self.folder, &clean, value) {
                Ok(_) if clean == DEFAULT => "Saved \"default\": the game starts with it now.".to_string(),
                Ok(_) => format!("Saved \"{clean}\"."),
                Err(error) => format!("Couldn't save: {error}"),
            }
        };
        self.look(true);
        self.status = Some(said.clone());
        said
    }

    /// The preset `name` loaded into `value` ("default" with none saved: the built-in settings): whether it was.
    fn load_named(&mut self, name: &str, value: &mut T) -> bool {
        let built_in = name == DEFAULT && !self.folder.join(format!("{DEFAULT}.toml")).exists();
        let loaded = if built_in { Ok(T::default()) } else { load::<T>(&self.folder, name) };
        match loaded {
            Ok(loaded) => {
                *value = loaded;
                self.name = name.to_string();
                self.status = Some(if built_in { "Loaded \"default\" (the built-in settings).".to_string() } else { format!("Loaded \"{name}\".") });
                true
            }
            Err(error) => {
                self.status = Some(format!("Couldn't load \"{name}\": {error}"));
                false
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder of its own for a test's presets, empty to begin with.
    pub fn test_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join("incus artus preset tests").join(name);
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn a_preset_saved_and_loaded_comes_back_the_same() {
        let root = test_root("round trip");
        let mut near = NearLook::DEFAULT;
        (near.radius, near.pixel, near.detail, near.flicker, near.lantern_colour, near.lantern) = (48.0, 4, 0.015625, 0.05, [0.2, 0.9, 0.31], true);
        let far = FarLook { fog: true, fog_reach: 3_333.3, detail: -2, fade_4: [1.7, 0.33], ..FarLook::DEFAULT };
        let preset = RenderPreset::of(&near, &far);
        let path = save(&root.join("render"), "try", &preset).unwrap();
        assert_eq!(load::<RenderPreset>(&root.join("render"), "try").unwrap(), preset);
        // The lantern lit isn't kept, and the file's numbers are the short ones.
        let text = std::fs::read_to_string(path).unwrap();
        assert!(!preset.near.lantern && !text.contains("lantern ="), "the lantern lit or out is the L key's:\n{text}");
        assert!(text.contains("flicker = 0.05\n") && text.contains("radius = 48.0\n"), "numbers as he'd write them:\n{text}");
        let camera = CameraPreset { speed: 333.0, multiplier: 2.5, field_of_view: 30.0 };
        save(&root.join("camera"), "try", &camera).unwrap();
        assert_eq!(load::<CameraPreset>(&root.join("camera"), "try").unwrap(), camera);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn an_older_or_newer_file_still_loads() {
        let root = test_root("older and newer");
        std::fs::create_dir_all(root.join("render")).unwrap();
        // A setting missing keeps its built-in value; one that no longer exists is passed over; a whole number will do
        // for a setting kept as a fraction.
        std::fs::write(root.join("render").join("old.toml"), "[near]\nradius = 16\ngone_since = 3\n\n[far]\nfog = true\n\n[no_such_part]\na = 1\n").unwrap();
        let loaded: RenderPreset = load(&root.join("render"), "old").unwrap();
        assert_eq!(loaded, RenderPreset { near: NearLook { radius: 16.0, ..NearLook::DEFAULT }, far: FarLook { fog: true, ..FarLook::DEFAULT } });
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_dot_is_on_for_the_default_and_the_game_starts_with_it() {
        let root = test_root("the dot");
        let mut bar: Bar<CameraPreset> = Bar::new(&root, "camera");
        let mut value = CameraPreset::default();
        // No file: the built-in settings are the default, the box is empty, and the game starts on them.
        assert!(bar.on_default(&value) && bar.name.is_empty());
        assert_eq!(start::<CameraPreset>(&root, "camera", false).0, CameraPreset::default());
        value.speed = 250.0;
        assert!(!bar.on_default(&value));
        // Saved as "default": the dot's on again, and the game starts with it (a flight doesn't).
        bar.name = "default".to_string();
        assert_eq!(bar.save_named(&value), "Saved \"default\": the game starts with it now.");
        assert!(bar.on_default(&value));
        assert_eq!(start::<CameraPreset>(&root, "camera", false).0, value);
        assert_eq!(start::<CameraPreset>(&root, "camera", true).0, CameraPreset::default());
        assert_eq!(Bar::<CameraPreset>::new(&root, "camera").name, "default");
        // Another saved and loaded back; then "default" from the list.
        value.speed = 5.0;
        bar.name = "slow walk!".to_string();
        assert_eq!(bar.save_named(&value), "Saved \"slow_walk\".");
        let mut other = CameraPreset::default();
        assert!(bar.load_named("slow_walk", &mut other) && other == value && !bar.on_default(&other));
        assert!(bar.load_named("default", &mut other) && other.speed == 250.0 && bar.on_default(&other));
        assert!(!bar.load_named("never saved", &mut other));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_name_is_cleaned_for_a_file() {
        assert_eq!(clean_name("  night cave 2 "), "night_cave_2");
        assert_eq!(clean_name("../up\\and:away?"), "upandaway");
        assert_eq!(clean_name("  "), "");
    }
}
