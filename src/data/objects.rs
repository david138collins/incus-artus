//! Objects — real things in the world that aren't blocks [`docs/objects.md`; David, 2026-11-01:
//! "blocks are blocks, objects are objects, and blueprints are made of both"]: a brazier, a torch, a
//! cup, a loaf, a sword, a bed; and, later, plants ("objects can be 2D, like grass, or 3d, like corn or
//! brazieres or beds").
//!
//! Each object is a data file, `assets/data/<pack>/objects/<name>.toml`, its ID `<pack>:<name>`:
//! ```toml
//! name = "Brazier"
//! model = "braziere.gltf"   # a glTF (Blockbench's export) in assets/models/
//! height = 1.5              # how tall it stands, in blocks: the model is scaled to fit. Left out,
//!                           # it's as made (Blockbench's 16 pixels to a block, the blocks' own)
//! stick_over = 2            # how many pixels it may stick out of the room it holds, and just clip
//! solid = true              # bodies bump into it: into its model's own shape
//! sits = "floor"            # how it's placed: "floor" (walls and ceilings later)
//! tags = ["light"]
//! ```
//!
//! **Its shape is its model's** [David, 2026-11-01: "basically we just go by the models size";
//! "the collision still needs to be from the models geometry, not its volume"]: read from the glTF
//! when the object is loaded (`read_shape`), its size in pixels and the boxes its parts fill, pixel
//! by pixel. What bodies bump into is those boxes; the room it holds in the world (`world/objects.rs`)
//! goes by its size.
//! **Room kept, not built yet:** a 2D form (a sprite, or crossed sprites, with no model: grass, and in
//! meadow numbers drawn into the chunk's mesh); attachments at the model's named points (a flame
//! sprite, a light: stops 2 and 3); objects placed by Ecology from its plan, never saved.

use super::data_folder;
use super::loading::find_files;
use crate::ecs::math::{IVec3, Mat4, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// `assets/models`: the objects' models.
pub fn models_folder() -> PathBuf {
    super::assets_folder().join("models")
}

/// An object's number in the running game (its place in the registry). Saves use its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u16);

/// How an object is placed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sits {
    /// Standing on a floor, in the middle of its cell.
    #[default]
    Floor,
    /// On a wall (torches first, Counsel's lean), or hanging from a ceiling: not built yet.
    Wall,
    Ceiling,
}

/// One object.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectDef {
    /// Its ID (`incus:braziere`), and what it's called.
    pub name: String,
    pub display_name: String,
    /// Its model's file in `assets/models/`.
    pub model: String,
    /// How tall it stands, in blocks, the model scaled to fit; `None`: as made (a glTF's unit is a
    /// block, Blockbench's 16 pixels).
    pub height: Option<f32>,
    /// How many pixels it may stick out of the room it holds (and just clip) and still count as that
    /// size: a cup 10 pixels across, with 2, still sits on an octant.
    pub stick_over: u8,
    pub solid: bool,
    pub sits: Sits,
    pub tags: Vec<String>,
    /// Its model's shape, read when it's loaded (`read_shape`); `None` until then, or if the model
    /// couldn't be read (it then counts as a block across, its height tall).
    pub shape: Option<Shape>,
    /// An animated sprite hung on it (a brazier's flame), and the light it gives (stop 2).
    pub sprite: Option<SpriteAttach>,
    pub light: Option<LightAttach>,
}

/// Which point of a sprite's frame goes on its spot [David: "a drop down for bottom, bottom center,
/// top center"].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    /// Its base on the spot, growing up from it: a flame in a cup.
    #[default]
    BottomCentre,
    /// Its middle on the spot: something floating.
    Centre,
    /// Its top on the spot, hanging down: a drip under a ceiling.
    TopCentre,
}

/// An animated sprite hung on an object: a strip of frames side by side (`read_strip`), drawn full
/// bright and turning to face you, as Doom's flames do [`docs/objects.md` section 10].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteAttach {
    /// The strip, in `assets/textures/sprites/`.
    pub file: String,
    /// How long each frame shows, in milliseconds ("probably just set by ms for now").
    #[serde(default = "frame_ms")]
    pub frame_ms: u32,
    #[serde(default)]
    pub anchor: Anchor,
    /// The model's named point it goes on (a Blockbench locator); none: on top of the model, in the
    /// middle.
    #[serde(default)]
    pub at: Option<String>,
    /// Moved from there, in pixels: across, along, up.
    #[serde(default)]
    pub offset: [i32; 3],
    /// Its size: 1, one strip pixel to one block pixel.
    #[serde(default = "one")]
    pub scale: f32,
}

fn frame_ms() -> u32 {
    110
}

fn one() -> f32 {
    1.0
}

/// The light an object gives, from its sprite's spot (or its middle): spread through the air into
/// the faces round it, a step dimmer each block, stopping at walls (`render/placed_light.rs`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LightAttach {
    /// Its colour (0 to 255 each: red, green, blue).
    pub colour: [u8; 3],
    /// How far it reaches, in blocks (15 at most).
    #[serde(default = "reach")]
    pub reach: f32,
    /// How bright it is at its middle, 0 to 1.
    #[serde(default = "one")]
    pub brightness: f32,
}

fn reach() -> f32 {
    8.0
}

/// `assets/textures/sprites`: the objects' sprite strips.
pub fn sprites_folder() -> PathBuf {
    super::assets_folder().join("textures").join("sprites")
}

/// A sprite strip, read: its pixels (RGBA, the markers taken out), its size, and its frames (each
/// one's left edge and width, in pixels).
#[derive(Clone, Debug, PartialEq)]
pub struct Strip {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub frames: Vec<(u32, u32)>,
}

/// A strip of frames side by side, from a PNG's bytes. **The strip marks its own frames** (David: "i
/// put a magenta dot at the corner of each 16 pixel cell"): the colour of its bottom-left pixel is the
/// marker, and each pixel of that colour along the bottom row starts a frame, running to the next (or
/// to the edge). The markers are taken out. With no marker (the bottom-left pixel clear), the frames
/// are square, as tall as the strip.
pub fn read_strip(bytes: &[u8]) -> Result<Strip, String> {
    let (width, height, rgba) = png_rgba(bytes)?;
    if width == 0 || height == 0 {
        return Err("it's empty".to_string());
    }
    strip_frames(width, height, rgba)
}

/// A PNG's size and pixels as RGBA, 8 bits a channel, from its bytes.
pub fn png_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|error| error.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size().ok_or("it's too big")?];
    let info = reader.next_frame(&mut buffer).map_err(|error| error.to_string())?;
    let (width, height) = (info.width, info.height);
    let channels = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::Rgb => 3,
        png::ColorType::GrayscaleAlpha => 2,
        _ => 1,
    };
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height as usize {
        for x in 0..width as usize {
            let pixel = &buffer[y * info.line_size + x * channels..][..channels];
            rgba.extend_from_slice(&match channels {
                4 => [pixel[0], pixel[1], pixel[2], pixel[3]],
                3 => [pixel[0], pixel[1], pixel[2], 255],
                2 => [pixel[0], pixel[0], pixel[0], pixel[1]],
                _ => [pixel[0], pixel[0], pixel[0], 255],
            });
        }
    }
    Ok((width, height, rgba))
}

/// The strip's frames, found by its markers (see `read_strip`), the markers taken out.
fn strip_frames(width: u32, height: u32, mut rgba: Vec<u8>) -> Result<Strip, String> {
    let at = |x: u32| ((height - 1) * width + x) as usize * 4;
    let marker: [u8; 4] = rgba[at(0)..at(0) + 4].try_into().unwrap_or_default();
    let mut starts = Vec::new();
    if marker[3] > 0 {
        for x in 0..width {
            if rgba[at(x)..at(x) + 4] == marker {
                starts.push(x);
                rgba[at(x)..at(x) + 4].copy_from_slice(&[0; 4]);
            }
        }
    } else {
        starts = (0..width).step_by(height as usize).collect();
    }
    let frames = starts.iter().enumerate().map(|(i, &start)| (start, starts.get(i + 1).copied().unwrap_or(width) - start)).collect();
    Ok(Strip { width, height, rgba, frames })
}

impl ObjectDef {
    /// Its shape, or, without one, a block across and its height (or a block) tall.
    pub fn shape_or_block(&self) -> Shape {
        self.shape.clone().unwrap_or_else(|| Shape::block(self.height.unwrap_or(1.0)))
    }
}

/// An object's shape, read from its model, standing unturned with its feet's middle at the origin.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    /// Model units to blocks.
    pub scale: f32,
    /// The middle of the bottom of the model's box, in the model's own units and axes (glTF's: Y up).
    pub feet: Vec3,
    /// Its size in pixels (16 to a block), in the game's axes: across (x), along (y), up (z).
    pub size: IVec3,
    /// The boxes its parts fill, pixel by pixel (blocks, from its feet, the game's axes): what bodies
    /// bump into.
    pub boxes: Vec<(Vec3, Vec3)>,
    /// Its named points (Blockbench's locators, and its groups), where things hang on it (blocks,
    /// from its feet, the game's axes).
    pub points: Vec<(String, Vec3)>,
}

impl Shape {
    /// A block across, `height` blocks tall, one box: for a model that couldn't be read.
    pub fn block(height: f32) -> Self {
        let size = IVec3::new(16, 16, (height * 16.0).round().max(1.0) as i32);
        Self { scale: 1.0, feet: Vec3::ZERO, size, boxes: vec![(Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.5, 0.5, size.z as f32 / 16.0))], points: Vec::new() }
    }

    /// Where a sprite goes (blocks, from the feet, unturned): its named point, or on top in the
    /// middle; then moved by its offset (pixels).
    // Read by the old renderer, which the regroup cut (docs/regroup.md); the new one reads it again.
    #[allow(dead_code)]
    pub fn spot(&self, sprite: &SpriteAttach) -> Vec3 {
        let point = sprite.at.as_ref().and_then(|name| self.points.iter().find(|(point, _)| point == name)).map(|(_, at)| *at);
        point.unwrap_or(Vec3::new(0.0, 0.0, self.size.z as f32 / 16.0)) + IVec3::from_array(sprite.offset).as_vec3() / 16.0
    }
}

/// The most pixels a shape is worked out in, each way (32 blocks).
const MOST_PIXELS: u32 = 512;

/// A model's shape, from its glTF (the file's bytes): every mesh in the default scene, placed as its
/// nodes place it, scaled so it stands `height` blocks tall (or as made), and filled in pixel by pixel.
///
/// - **Each part** (a mesh's primitive: one of Blockbench's cubes) is the box round its own points,
///   turned and moved as its node is. A pixel is filled where its middle is inside a part; a part
///   thinner than a pixel (Blockbench's flat planes) still fills one.
/// - **The pixels filled** are joined into as few boxes as easily found (along x, then y, then z).
pub fn read_shape(bytes: &[u8], height: Option<f32>) -> Result<Shape, String> {
    let gltf = gltf::Gltf::from_slice(bytes).map_err(|error| error.to_string())?;
    let scene = gltf.default_scene().or_else(|| gltf.scenes().next()).ok_or("it has no scene")?;
    // Each part: the corners of its own box, and how it's placed (the model's axes).
    let corner = |value: Option<gltf::json::Value>| -> Option<Vec3> {
        let numbers: Vec<f32> = value?.as_array()?.iter().filter_map(|number| number.as_f64()).map(|number| number as f32).collect();
        (numbers.len() == 3).then(|| Vec3::new(numbers[0], numbers[1], numbers[2]))
    };
    let mut parts: Vec<(Vec3, Vec3, Mat4)> = Vec::new();
    // Its named points: the nodes with no mesh of their own (Blockbench's locators, and its groups).
    let mut points: Vec<(String, Vec3)> = Vec::new();
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|node| (node, Mat4::IDENTITY)).collect();
    while let Some((node, above)) = stack.pop() {
        let place = above * Mat4::from_cols_array_2d(&node.transform().matrix());
        stack.extend(node.children().map(|child| (child, place)));
        if node.mesh().is_none()
            && let Some(name) = node.name()
        {
            points.push((name.to_string(), place.transform_point3(Vec3::ZERO)));
        }
        for primitive in node.mesh().iter().flat_map(|mesh| mesh.primitives()) {
            if let Some(points) = primitive.get(&gltf::Semantic::Positions)
                && let (Some(low), Some(high)) = (corner(points.min()), corner(points.max()))
            {
                parts.push((low, high, place));
            }
        }
    }
    // The model's own axes (glTF: Y up, as Bevy's) to the game's (Z up), and back.
    let to_game = |at: Vec3| Vec3::new(-at.z, at.x, at.y);
    let to_model = |at: Vec3| Vec3::new(at.y, at.z, -at.x);
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for (from, to, place) in &parts {
        for i in 0..8 {
            let local = Vec3::new(if i & 1 == 0 { from.x } else { to.x }, if i & 2 == 0 { from.y } else { to.y }, if i & 4 == 0 { from.z } else { to.z });
            let at = to_game(place.transform_point3(local));
            (low, high) = (low.min(at), high.max(at));
        }
    }
    if parts.is_empty() || low.z > high.z {
        return Err("it has no meshes".to_string());
    }
    let extent = high - low;
    let scale = height.map_or(1.0, |height| height / extent.z.max(1e-3));
    // A pixel, in the model's units; how many pixels it is each way.
    let pixel = 1.0 / (16.0 * scale);
    let count = |length: f32| ((length / pixel - 1e-3).ceil().max(1.0) as u32).min(MOST_PIXELS);
    let (nx, ny, nz) = (count(extent.x), count(extent.y), count(extent.z));
    let feet = Vec3::new(low.x + extent.x / 2.0, low.y + extent.y / 2.0, low.z);
    let size = IVec3::new(nx as i32, ny as i32, nz as i32);
    let in_blocks = |cells: IVec3| cells.as_vec3() / 16.0 - Vec3::new(extent.x * scale / 2.0, extent.y * scale / 2.0, 0.0);

    // Filled pixel by pixel: a pixel's middle inside a part, its box widened to a pixel if it's thinner.
    let parts: Vec<(Vec3, Vec3, Mat4)> = parts
        .iter()
        .map(|(from, to, place)| {
            let widen = (*to - *from).map(|thickness| if thickness < pixel { pixel / 2.0 } else { pixel * 1e-3 });
            (*from - widen, *to + widen, place.inverse())
        })
        .collect();
    let index = |x: u32, y: u32, z: u32| (x + nx * (y + ny * z)) as usize;
    let mut filled = vec![false; (nx * ny * nz) as usize];
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                let middle = low + (Vec3::new(x as f32, y as f32, z as f32) + 0.5) * pixel;
                let model = to_model(middle);
                // Half-open, so a plane on the line between two pixels fills one, not both.
                filled[index(x, y, z)] = parts.iter().any(|(from, to, unplace)| {
                    let local = unplace.transform_point3(model);
                    local.cmpge(*from).all() && local.cmplt(*to).all()
                });
            }
        }
    }
    // Joined into boxes: a run along x, as far along y as the same run is filled, as far up z as
    // the same rectangle is.
    let mut taken = vec![false; filled.len()];
    let free = |taken: &[bool], x: u32, y: u32, z: u32| filled[index(x, y, z)] && !taken[index(x, y, z)];
    let mut boxes = Vec::new();
    for z in 0..nz {
        for y in 0..ny {
            for x in 0..nx {
                if !free(&taken, x, y, z) {
                    continue;
                }
                let mut x1 = x + 1;
                while x1 < nx && free(&taken, x1, y, z) {
                    x1 += 1;
                }
                let mut y1 = y + 1;
                while y1 < ny && (x..x1).all(|i| free(&taken, i, y1, z)) {
                    y1 += 1;
                }
                let mut z1 = z + 1;
                while z1 < nz && (y..y1).all(|j| (x..x1).all(|i| free(&taken, i, j, z1))) {
                    z1 += 1;
                }
                for k in z..z1 {
                    for j in y..y1 {
                        for i in x..x1 {
                            taken[index(i, j, k)] = true;
                        }
                    }
                }
                let corner = |x: u32, y: u32, z: u32| in_blocks(IVec3::new(x as i32, y as i32, z as i32));
                boxes.push((corner(x, y, z), corner(x1, y1, z1)));
            }
        }
    }
    let points = points.into_iter().map(|(name, at)| (name, (to_game(at) - feet) * scale)).collect();
    Ok(Shape { scale, feet: to_model(feet), size, boxes, points })
}

/// What an object's data file holds.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectFile {
    #[serde(default)]
    name: Option<String>,
    model: String,
    #[serde(default)]
    height: Option<f32>,
    #[serde(default = "two")]
    stick_over: u8,
    #[serde(default)]
    solid: bool,
    #[serde(default)]
    sits: Sits,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    sprite: Option<SpriteAttach>,
    #[serde(default)]
    light: Option<LightAttach>,
}

fn two() -> u8 {
    2
}

/// An object from its data file's text (its shape is read from its model afterwards).
pub fn parse_object(id: &str, text: &str) -> Result<ObjectDef, String> {
    let file: ObjectFile = toml::from_str(text).map_err(|error| error.to_string())?;
    if let Some(height) = file.height.filter(|height| !(0.05..=16.0).contains(height)) {
        return Err(format!("height {height} is out of range (0.05 to 16 blocks)"));
    }
    let display_name = file.name.unwrap_or_else(|| id.split(':').nth(1).unwrap_or(id).replace('_', " "));
    Ok(ObjectDef { name: id.to_string(), display_name, model: file.model, height: file.height, stick_over: file.stick_over, solid: file.solid, sits: file.sits, tags: file.tags, shape: None, sprite: file.sprite, light: file.light })
}

/// An object's data file, as text (kept to the settings it has, with the file's own header).
pub fn object_file_text(def: &ObjectDef, header: &str) -> String {
    let mut text = header.to_string();
    text += &format!("name = {:?}\nmodel = {:?}\n", def.display_name, def.model);
    if let Some(height) = def.height {
        text += &format!("height = {height}\n");
    }
    if def.stick_over != 2 {
        text += &format!("stick_over = {}\n", def.stick_over);
    }
    if def.solid {
        text += "solid = true\n";
    }
    if def.sits != Sits::Floor {
        text += &format!("sits = {:?}\n", format!("{:?}", def.sits).to_lowercase());
    }
    if !def.tags.is_empty() {
        text += &format!("tags = {:?}\n", def.tags);
    }
    if let Some(sprite) = &def.sprite {
        text += &format!("\n[sprite]\nfile = {:?}\nframe_ms = {}\n", sprite.file, sprite.frame_ms);
        if sprite.anchor != Anchor::BottomCentre {
            text += &format!("anchor = {:?}\n", if sprite.anchor == Anchor::Centre { "centre" } else { "top_centre" });
        }
        if let Some(at) = &sprite.at {
            text += &format!("at = {at:?}\n");
        }
        if sprite.offset != [0; 3] {
            text += &format!("offset = {:?}\n", sprite.offset);
        }
        if sprite.scale != 1.0 {
            text += &format!("scale = {}\n", sprite.scale);
        }
    }
    if let Some(light) = &def.light {
        text += &format!("\n[light]\ncolour = {:?}\nreach = {}\nbrightness = {}\n", light.colour, light.reach, light.brightness);
    }
    text
}

/// Every object, from the data folder.
#[derive(crate::ecs::prelude::Resource, Default)]
pub struct ObjectRegistry {
    defs: Vec<ObjectDef>,
    ids: HashMap<String, ObjectId>,
    files: HashMap<PathBuf, Option<SystemTime>>,
    /// Each object file's model and sprite strip, and when each was last changed: one changed is read
    /// again.
    models: HashMap<PathBuf, Vec<(PathBuf, Option<SystemTime>)>>,
    sources: HashMap<String, PathBuf>,
    /// What's wrong, per file.
    pub problems: BTreeMap<String, String>,
    /// Goes up whenever a definition changes.
    revision: u64,
}

impl ObjectRegistry {
    pub fn load() -> Self {
        let mut registry = Self::default();
        registry.reload_changed_files();
        registry
    }

    /// Read every object file that's new or changed. True if any definition changed. Objects keep
    /// their numbers; one whose file has gone stays until the game restarts.
    pub fn reload_changed_files(&mut self) -> bool {
        let mut problems = Vec::new();
        let found = find_files(&data_folder(), "objects", &mut problems);
        let mut changed = false;
        let modified = |path: &Path| std::fs::metadata(path).and_then(|metadata| metadata.modified()).ok();
        for file in found {
            let model_same = self.models.get(&file.path).is_none_or(|watched| watched.iter().all(|(path, when)| modified(path) == *when));
            if self.files.get(&file.path) == Some(&file.modified) && model_same {
                continue;
            }
            self.files.insert(file.path.clone(), file.modified);
            let key = file.path.display().to_string();
            match std::fs::read_to_string(&file.path).map_err(|error| error.to_string()).and_then(|text| parse_object(&file.id, &text)) {
                Ok(mut def) => {
                    self.problems.remove(&key);
                    self.sources.insert(file.id.clone(), file.path.clone());
                    // Its shape, from its model.
                    let model = models_folder().join(&def.model);
                    let mut watched = vec![(model.clone(), modified(&model))];
                    if let Some(sprite) = &def.sprite {
                        let strip = sprites_folder().join(&sprite.file);
                        watched.push((strip.clone(), modified(&strip)));
                    }
                    self.models.insert(file.path.clone(), watched);
                    match std::fs::read(&model).map_err(|error| error.to_string()).and_then(|bytes| read_shape(&bytes, def.height)) {
                        Ok(shape) => def.shape = Some(shape),
                        Err(error) => {
                            self.problems.insert(key, format!("its model {} couldn't be read ({error}): it counts as a block across", def.model));
                        }
                    }
                    // A model or strip changed on its own changes what's drawn, if not the settings.
                    changed |= self.insert_or_replace(def) || !model_same;
                }
                Err(error) => {
                    self.problems.insert(key, format!("couldn't be used: {error}"));
                }
            }
        }
        for problem in problems.into_iter().filter(|problem| !problem.starts_with("no data folder")) {
            self.problems.insert("data folder".to_string(), problem);
        }
        if changed {
            self.revision += 1;
        }
        changed
    }

    fn insert_or_replace(&mut self, def: ObjectDef) -> bool {
        match self.ids.get(&def.name) {
            Some(&id) => {
                let slot = &mut self.defs[id.0 as usize];
                let changed = *slot != def;
                *slot = def;
                changed
            }
            None => {
                self.ids.insert(def.name.clone(), ObjectId(self.defs.len() as u16));
                self.defs.push(def);
                true
            }
        }
    }

    pub fn id(&self, name: &str) -> Option<ObjectId> {
        self.ids.get(name).copied()
    }

    pub fn def(&self, id: ObjectId) -> Option<&ObjectDef> {
        self.defs.get(id.0 as usize)
    }

    pub fn by_name(&self, name: &str) -> Option<&ObjectDef> {
        self.id(name).and_then(|id| self.def(id))
    }

    pub fn defs(&self) -> &[ObjectDef] {
        &self.defs
    }

    /// The data file an object came from.
    pub fn source(&self, name: &str) -> Option<&Path> {
        self.sources.get(name).map(PathBuf::as_path)
    }

    // Read by the old renderer, which the regroup cut (docs/regroup.md); the new one reads it again.
    #[allow(dead_code)]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// For tests: one object, as if from a file.
    #[cfg(test)]
    pub fn with(defs: Vec<ObjectDef>) -> Self {
        let mut registry = Self::default();
        for def in defs {
            registry.insert_or_replace(def);
        }
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_object_file_reads_and_writes_back_the_same() {
        let text = "name = \"Brazier\"\nmodel = \"braziere.gltf\"\nheight = 1.5\nsolid = true\n";
        let def = parse_object("incus:braziere", text).unwrap();
        assert_eq!((def.display_name.as_str(), def.model.as_str(), def.height, def.solid, def.sits), ("Brazier", "braziere.gltf", Some(1.5), true, Sits::Floor));
        assert_eq!(object_file_text(&def, ""), text);
        // Defaults: named from its file, as made, sticking over 2 pixels, not solid.
        let cup = parse_object("incus:tin_cup", "model = \"cup.gltf\"").unwrap();
        assert_eq!((cup.display_name.as_str(), cup.height, cup.stick_over, cup.solid), ("tin cup", None, 2, false));
        assert_eq!(object_file_text(&cup, ""), "name = \"tin cup\"\nmodel = \"cup.gltf\"\n");
        assert!(parse_object("incus:x", "model = \"x.gltf\"\nheight = 0").is_err());
        assert!(parse_object("incus:x", "model = \"x.gltf\"\ncolour = 3").is_err(), "unknown settings are refused");
    }

    /// A glTF of boxes, as Blockbench writes them: each a mesh whose points span `low` to `high`, its
    /// node moved by `moved` (the model's axes: Y up).
    fn boxes_gltf(boxes: &[([f32; 3], [f32; 3], [f32; 3])]) -> Vec<u8> {
        let n = boxes.len();
        let views: Vec<String> = (0..n).map(|i| format!("{{\"buffer\":0,\"byteOffset\":{},\"byteLength\":12}}", i * 12)).collect();
        let accessors: Vec<String> = boxes.iter().enumerate().map(|(i, (low, high, _))| format!("{{\"bufferView\":{i},\"componentType\":5126,\"count\":1,\"type\":\"VEC3\",\"min\":{low:?},\"max\":{high:?}}}")).collect();
        let meshes: Vec<String> = (0..n).map(|i| format!("{{\"primitives\":[{{\"attributes\":{{\"POSITION\":{i}}}}}]}}")).collect();
        let nodes: Vec<String> = boxes.iter().enumerate().map(|(i, (_, _, moved))| format!("{{\"mesh\":{i},\"translation\":{moved:?}}}")).collect();
        let scene: Vec<String> = (0..n).map(|i| i.to_string()).collect();
        format!(
            "{{\"asset\":{{\"version\":\"2.0\"}},\"scene\":0,\"scenes\":[{{\"nodes\":[{}]}}],\"nodes\":[{}],\"meshes\":[{}],\"accessors\":[{}],\"bufferViews\":[{}],\"buffers\":[{{\"byteLength\":{}}}]}}",
            scene.join(","),
            nodes.join(","),
            meshes.join(","),
            accessors.join(","),
            views.join(","),
            n * 12
        )
        .into_bytes()
    }

    /// Is the point (blocks, from the feet) inside one of the shape's boxes?
    fn inside(shape: &Shape, at: Vec3) -> bool {
        shape.boxes.iter().any(|(low, high)| at.cmpgt(*low).all() && at.cmplt(*high).all())
    }

    /// A PNG, `width` × `height`, every pixel `fill`, with `marks` (x along the bottom row) in pink.
    fn strip_png(width: u32, height: u32, fill: [u8; 4], marks: &[u32]) -> Vec<u8> {
        let mut pixels: Vec<u8> = (0..width * height).flat_map(|_| fill).collect();
        for &x in marks {
            let at = (((height - 1) * width + x) * 4) as usize;
            pixels[at..at + 4].copy_from_slice(&[254, 0, 191, 255]);
        }
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().unwrap().write_image_data(&pixels).unwrap();
        bytes
    }

    #[test]
    fn a_strip_marks_its_own_frames_and_the_marks_are_taken_out() {
        // David's: frames 16 wide and 24 tall, a pink dot at each one's bottom-left corner.
        let strip = read_strip(&strip_png(112, 24, [0, 0, 0, 0], &[0, 16, 32, 48, 64, 80, 96])).unwrap();
        assert_eq!(strip.frames, (0..7).map(|i| (i * 16, 16)).collect::<Vec<_>>());
        assert!(strip.rgba.chunks(4).all(|pixel| pixel == [0, 0, 0, 0]), "the dots are gone");
        // Frames needn't be the same width.
        assert_eq!(read_strip(&strip_png(20, 8, [0, 0, 0, 0], &[0, 12])).unwrap().frames, vec![(0, 12), (12, 8)]);
        // No dots (the corner clear): square frames, as tall as the strip.
        assert_eq!(read_strip(&strip_png(48, 16, [0, 0, 0, 0], &[])).unwrap().frames, vec![(0, 16), (16, 16), (32, 16)]);
    }

    #[test]
    fn a_sprite_and_a_light_read_and_write_back_the_same() {
        let text = "name = \"Brazier\"\nmodel = \"braziere.gltf\"\nheight = 1.5\nsolid = true\n\n[sprite]\nfile = \"green_flame_animation.png\"\nframe_ms = 110\nat = \"flame\"\noffset = [0, 0, -2]\nscale = 0.5\n\n[light]\ncolour = [80, 255, 60]\nreach = 8\nbrightness = 0.9\n";
        let def = parse_object("incus:braziere", text).unwrap();
        let sprite = def.sprite.as_ref().unwrap();
        assert_eq!((sprite.frame_ms, sprite.anchor, sprite.at.as_deref(), sprite.offset, sprite.scale), (110, Anchor::BottomCentre, Some("flame"), [0, 0, -2], 0.5));
        assert_eq!(def.light.as_ref().map(|light| light.colour), Some([80, 255, 60]));
        assert_eq!(object_file_text(&def, ""), text);
        // A sprite goes on its named point, or on top in the middle; moved by its offset.
        let mut shape = Shape::block(1.5);
        assert_eq!(shape.spot(sprite), Vec3::new(0.0, 0.0, 1.5 - 2.0 / 16.0));
        shape.points.push(("flame".into(), Vec3::new(0.0, 0.25, 1.0)));
        assert_eq!(shape.spot(sprite), Vec3::new(0.0, 0.25, 1.0 - 2.0 / 16.0));
    }

    #[test]
    fn a_models_shape_is_its_own_size_and_its_parts_pixel_by_pixel() {
        // A block, as made: 16 pixels each way, one box, standing on its feet.
        let block = read_shape(&boxes_gltf(&[([0.0; 3], [1.0; 3], [0.0; 3])]), None).unwrap();
        assert_eq!((block.size, block.scale), (IVec3::splat(16), 1.0));
        assert_eq!(block.boxes, vec![(Vec3::new(-0.5, -0.5, 0.0), Vec3::new(0.5, 0.5, 1.0))]);
        // Scaled to stand 2 blocks tall: 32 pixels each way.
        assert_eq!(read_shape(&boxes_gltf(&[([0.0; 3], [1.0; 3], [0.0; 3])]), Some(2.0)).unwrap().size, IVec3::splat(32));
        // A pole 2 pixels thick, a block tall, under a plate a block across and a pixel thick: what's
        // filled is the pole and the plate, not the air beside the pole.
        let table = read_shape(&boxes_gltf(&[([-0.0625, 0.0, -0.0625], [0.0625, 1.0, 0.0625], [0.0; 3]), ([-0.5, 0.0, -0.5], [0.5, 0.0625, 0.5], [0.0, 1.0, 0.0])]), None).unwrap();
        assert_eq!(table.size, IVec3::new(16, 16, 17));
        assert!(inside(&table, Vec3::new(0.0, 0.0, 0.5)), "the pole");
        assert!(inside(&table, Vec3::new(0.4, -0.4, 1.03)), "the plate");
        assert!(!inside(&table, Vec3::new(0.4, -0.4, 0.5)), "beside the pole is air");
        // A flat plane (no thickness, as Blockbench's planes are) still fills a pixel.
        let plane = read_shape(&boxes_gltf(&[([0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0; 3])]), None).unwrap();
        assert_eq!(plane.size, IVec3::new(1, 16, 16));
        assert!(plane.boxes.len() == 1 && inside(&plane, Vec3::new(0.03, 0.2, 0.5)));
    }
}
