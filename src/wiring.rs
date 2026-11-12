//! The wiring: every lane, pass and store in the game, declared in one place (plan 2.1).
//!
//! The game runs from this list: the frame's passes run in the order they're listed here, and the card's clock gives each
//! pass on the card its slot. The software map (the editor's page, F2) is drawn from it live, and `master/map.md` is
//! drawn from it by the test below on every build, which fails if the code and the list ever disagree. So the map can't
//! fall behind the code, or keep what's gone.
//!
//! The words, one meaning each [David, 2026-11-12: "the words you're using are fine"]:
//! - a **lane** is a line of work with its own pace (the frame runs once a frame; the window's events as they come);
//! - a **pass** is one piece of a lane's work, on the processor or the card, timed by its own clock;
//! - a **stage** is one step of the world's recipe (none yet: they join when the recipe does);
//! - a **store** is something held: where it lives, its fixed limits, who writes it and who reads it.
//!
//! The ids below are listed in the same order as their declarations, so each declaration is found by its id; the test
//! checks they line up.

/// Where something runs or is held.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Where {
    Processor,
    Card,
    Disk,
}

impl Where {
    pub fn name(self) -> &'static str {
        match self {
            Where::Processor => "the processor",
            Where::Card => "the card",
            Where::Disk => "the disk",
        }
    }
}

/// A switch: its name, and what it does.
pub type Switch = (&'static str, &'static str);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LaneId {
    Window,
    Frame,
    Loading,
}

pub struct Lane {
    pub id: LaneId,
    pub name: &'static str,
    /// When it runs.
    pub runs: &'static str,
    pub what: &'static str,
    pub switches: &'static [Switch],
    pub files: &'static [&'static str],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PassId {
    HearingTheWindow,
    GoingToAMark,
    Flying,
    PuttingTheFarCellsOnTheCard,
    LayingOutThePanel,
    GettingThePicture,
    TheFarStart,
    TheFar8,
    TheFar4,
    TheFar2,
    BringingItUp,
    ThePanel,
    HandingTheCardItsWork,
    ShowingIt,
    Measuring,
    TheDoorway,
    SummingTheFarCells,
    MakingTheFarTextures,
}

pub struct Pass {
    pub id: PassId,
    pub name: &'static str,
    pub lane: LaneId,
    pub on: Where,
    pub what: &'static str,
    pub reads: &'static [StoreId],
    pub writes: &'static [StoreId],
    pub switches: &'static [Switch],
    pub files: &'static [&'static str],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StoreId {
    KeysHeld,
    Camera,
    Clocks,
    Wiring,
    PanelShapes,
    ScreenPicture,
    FlightReport,
    SessionLog,
    Export,
    TileColumns,
    Marks,
    Changes,
    FarCells,
    FarBricks,
    FarLayers,
    FarLook,
    FarTextures,
}

pub struct Store {
    pub id: StoreId,
    pub name: &'static str,
    pub on: Where,
    pub what: &'static str,
    /// The fixed sizes it lives within, each with its value from the code itself.
    pub limits: &'static [(&'static str, u64)],
    pub files: &'static [&'static str],
}

pub const LANES: [Lane; 3] = [
    Lane {
        id: LaneId::Window,
        name: "the window",
        runs: "as the window's events come",
        what: "What the window hears: keys pressed and let go, the mouse moving, the window resized, closed or left.",
        switches: &[],
        files: &["src/runner.rs"],
    },
    Lane {
        id: LaneId::Frame,
        name: "the frame",
        runs: "once a frame",
        what: "Each frame, its passes in order: the camera goes to a mark if it's time, flies, the panels are laid out, the card draws the picture, and it's handed to the screen.",
        switches: &[
            ("INCUS_FLIGHT", "a measured flight: every mark in data/marks.toml in turn, 5 s settling and 20 s measured at each, uncapped with the panel hidden; a report, then quits"),
            ("WGPU_ADAPTER_NAME", "the card, by part of its name (intel: the laptop's Intel card)"),
            ("WGPU_POWER_PREF", "the card, by power (low or high)"),
            ("WGPU_BACKEND", "the card's language (DirectX 12 unless set)"),
            ("WGPU_DX12_COMPILER", "the shader compiler (the Windows SDK's DXC unless set)"),
        ],
        files: &["src/main.rs", "src/runner.rs", "src/card.rs"],
    },
    Lane {
        id: LaneId::Loading,
        name: "loading",
        runs: "as tiles are asked for",
        what: "The world, read a tile at a time through the doorway. Going to a mark asks it for the ground under the eye; the far cells' lane will ask for tiles (2.2).",
        switches: &[],
        files: &["src/doorway.rs"],
    },
];

pub const PASSES: [Pass; 18] = [
    Pass {
        id: PassId::HearingTheWindow,
        name: "hearing the window",
        lane: LaneId::Window,
        on: Where::Processor,
        what: "Each event: the keys held kept, the mouse turning the camera while it's captured, and F2 (the editor), F3 (the panel), F11 (window or full screen) and Tab (the mouse).",
        reads: &[],
        writes: &[StoreId::KeysHeld, StoreId::Camera],
        switches: &[],
        files: &["src/runner.rs", "src/camera.rs"],
    },
    Pass {
        id: PassId::GoingToAMark,
        name: "going to a mark",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "At the start, and on a flight once a mark is measured: the camera taken to the next mark, a player's eye over the ground the doorway answers, and the mark's changes made through the doorway (the build's placings, the scatter).",
        reads: &[StoreId::Marks, StoreId::TileColumns],
        writes: &[StoreId::Camera, StoreId::Changes],
        switches: &[("INCUS_MARK", "the mark the game starts at, by its name (the first in data/marks.toml unless set)")],
        files: &["src/marks.rs", "src/runner.rs"],
    },
    Pass {
        id: PassId::Flying,
        name: "flying",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "The camera flies by the keys held (WASD, Space, Ctrl, Shift), on real time. Not while the editor is open.",
        reads: &[StoreId::KeysHeld],
        writes: &[StoreId::Camera],
        switches: &[],
        files: &["src/camera.rs"],
    },
    Pass {
        id: PassId::PuttingTheFarCellsOnTheCard,
        name: "putting the far cells on the card",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "The regions summed since the last frame put on the card as bricks, at most 3 ms a frame, so the far view sharpens in live without holding the frame up; the levels over 1,024 made as their regions come in.",
        reads: &[StoreId::FarCells],
        writes: &[StoreId::FarBricks],
        switches: &[],
        files: &["src/bricks.rs", "src/far_eye.rs"],
    },
    Pass {
        id: PassId::LayingOutThePanel,
        name: "laying out the panel",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "The developer panel (F3) and the editor (F2) laid out, and cut into shapes for the card. The panel's Render page changes the far eye's look.",
        reads: &[StoreId::Camera, StoreId::Clocks, StoreId::Wiring],
        writes: &[StoreId::PanelShapes, StoreId::FarLook],
        switches: &[],
        files: &["src/panel.rs", "src/editor/mod.rs", "src/editor/software_map.rs"],
    },
    Pass {
        id: PassId::GettingThePicture,
        name: "getting the picture",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "The screen's next picture to draw on, waited for if none is free (capped, this is where the frame waits for the screen).",
        reads: &[],
        writes: &[],
        switches: &[],
        files: &["src/runner.rs"],
    },
    Pass {
        id: PassId::TheFarStart,
        name: "the far start",
        lane: LaneId::Frame,
        on: Where::Card,
        what: "One cone down each tile of 8 x 8 screen pixels, skipping over the far cells' tops, to where any of the tile's rays could first touch: they all start there. The layers' numbers for the frame written first, on the processor.",
        reads: &[StoreId::Camera, StoreId::FarBricks, StoreId::FarLook],
        writes: &[StoreId::FarLayers],
        switches: &[
            ("INCUS_SIGHT_LEVELS", "the levels' view: each pixel tinted by the level its surface was read from"),
            ("INCUS_SIGHT_STEPS", "the steps' view: dark blue few steps, through green, to red 256 or more"),
            ("INCUS_SIGHT_NO_SHADOWS", "the sun's shadows left out (for measuring)"),
            ("INCUS_SIGHT_NO_SKY", "the sky light left out (for measuring)"),
        ],
        files: &["src/far_eye.rs", "src/far.wgsl"],
    },
    Pass {
        id: PassId::TheFar8,
        name: "the far 8 x 8",
        lane: LaneId::Frame,
        on: Where::Card,
        what: "The pyramid's base: a ray each 8 x 8 screen pixels through the far cells (old's far march: the level by distance, skipping empty space, the eight cells round each step blended, gathered front to back, faces lit by the sun, fog), its shadow ray towards the sun and its sky cones where it shows, and where the 4 x 4 is worth it.",
        reads: &[StoreId::FarBricks, StoreId::FarTextures, StoreId::FarLayers],
        writes: &[StoreId::FarLayers],
        switches: &[],
        files: &["src/far_eye.rs", "src/far.wgsl"],
    },
    Pass {
        id: PassId::TheFar4,
        name: "the far 4 x 4",
        lane: LaneId::Frame,
        on: Where::Card,
        what: "A ray each 4 x 4 screen pixels, only where the 8 x 8 above said it's worth it (never for sky), starting a little short of the nearest touch above it; and where the 2 x 2 is worth it.",
        reads: &[StoreId::FarBricks, StoreId::FarTextures, StoreId::FarLayers],
        writes: &[StoreId::FarLayers],
        switches: &[],
        files: &["src/far_eye.rs", "src/far.wgsl"],
    },
    Pass {
        id: PassId::TheFar2,
        name: "the far 2 x 2",
        lane: LaneId::Frame,
        on: Where::Card,
        what: "A ray each 2 x 2 screen pixels, only where the 4 x 4 above said it's worth it: the finest layer.",
        reads: &[StoreId::FarBricks, StoreId::FarTextures, StoreId::FarLayers],
        writes: &[StoreId::FarLayers],
        switches: &[],
        files: &["src/far_eye.rs", "src/far.wgsl"],
    },
    Pass {
        id: PassId::BringingItUp,
        name: "bringing it up",
        lane: LaneId::Frame,
        on: Where::Card,
        what: "The pyramid onto the screen's picture: the 8 x 8 everywhere, each finer layer over it as much as it's worth there, crisp (a layer's pixel a square of screen pixels).",
        reads: &[StoreId::FarLayers],
        writes: &[StoreId::ScreenPicture],
        switches: &[],
        files: &["src/far_eye.rs", "src/far_up.wgsl"],
    },
    Pass {
        id: PassId::ThePanel,
        name: "the panel",
        lane: LaneId::Frame,
        on: Where::Card,
        what: "The panels drawn over the far eye's picture (the picture cleared to the sky's colour first, where there's no far eye). Recorded on the processor, run on the card.",
        reads: &[StoreId::PanelShapes],
        writes: &[StoreId::ScreenPicture],
        switches: &[],
        files: &["src/runner.rs"],
    },
    // Handing over, in its two parts (old's names), so a wait in either shows which it was in [David, 2026-11-12: "go ahead
    // and split it, so later on we can see it and tune it if it ever becomes something that can be improved"].
    Pass {
        id: PassId::HandingTheCardItsWork,
        name: "handing the card its work",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "The frame's work for the card finished and handed to it, with the card's clock asked to copy its times out.",
        reads: &[],
        writes: &[],
        switches: &[],
        files: &["src/runner.rs"],
    },
    Pass {
        id: PassId::ShowingIt,
        name: "showing it",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "The picture handed to the screen, waited for if the screen's queue is full (Windows lets the frame run only so far ahead of what the card and the screen have taken).",
        reads: &[StoreId::ScreenPicture],
        writes: &[],
        switches: &[],
        files: &["src/runner.rs"],
    },
    Pass {
        id: PassId::Measuring,
        name: "measuring",
        lane: LaneId::Frame,
        on: Where::Processor,
        what: "The frame's length, and the card's times read back from frames it has finished; on a flight, kept for its report.",
        reads: &[],
        writes: &[StoreId::Clocks, StoreId::FlightReport],
        switches: &[],
        files: &["src/runner.rs", "src/clocks.rs", "src/flight.rs"],
    },
    Pass {
        id: PassId::TheDoorway,
        name: "the doorway",
        lane: LaneId::Loading,
        on: Where::Processor,
        what: "Answers \"what's here?\" a tile at a time: each column's ground, its river's or lake's water, and its blocks from bedrock up as runs; and what changed in it. The engine never opens a world file itself. The export is read-only.",
        reads: &[StoreId::Export, StoreId::Changes],
        writes: &[StoreId::TileColumns],
        switches: &[],
        files: &["src/doorway.rs"],
    },
    Pass {
        id: PassId::SummingTheFarCells,
        name: "summing the far cells",
        lane: LaneId::Loading,
        on: Where::Processor,
        what: "The far cells summed from the doorway's columns, a region (1,024 blocks a side) at a time, nearest the spawn first: cubes of 32 blocks and every level above, each exactly its children added. On the processor, not the card [David, 2026-11-12: \"its a 1 time event, and it's quick\"]: it's the loading lane, not the frame; the doorway's columns are already on the processor; and the card stays free for drawing. Nothing in the game asks yet: its test does, and the far eye will (2.2, step 2).",
        reads: &[StoreId::TileColumns],
        writes: &[StoreId::FarCells],
        switches: &[],
        files: &["src/far.rs"],
    },
    Pass {
        id: PassId::MakingTheFarTextures,
        name: "making the far textures",
        lane: LaneId::Loading,
        on: Where::Processor,
        what: "Once every region is in, on a thread of its own: each far brick made again as a 9 x 9 x 9 block of texels (its border the next cells over) in the atlas, the two ways of change 1 (exact: colour and faces times surface, 16-bit; spread: spread into empty neighbours, 8-bit), and sent to the card an atlas layer at a time.",
        reads: &[StoreId::FarCells],
        writes: &[StoreId::FarTextures],
        switches: &[("INCUS_FAR_BLEND", "which ways are made: code (none), exact, spread, or both (unset: both, the code's blend till one is chosen)")],
        files: &["src/atlas.rs"],
    },
];

pub const STORES: [Store; 17] = [
    Store {
        id: StoreId::KeysHeld,
        name: "the keys held",
        on: Where::Processor,
        what: "The keys held down now (let go of all when the window is left).",
        limits: &[],
        files: &["src/runner.rs"],
    },
    Store {
        id: StoreId::Camera,
        name: "the camera",
        on: Where::Processor,
        what: "Where the eye is (north, east, up, in blocks) and where it looks (facing from north toward east, pitch up from level).",
        limits: &[],
        files: &["src/camera.rs"],
    },
    Store {
        id: StoreId::Clocks,
        name: "the clocks",
        on: Where::Processor,
        what: "The frame's recent lengths, each pass's recent times on the processor, and the card's on the card.",
        limits: &[("recent frames kept, a pass", crate::clocks::RECENT as u64), ("frames on their way back from the card at once", crate::clocks::READBACKS as u64)],
        files: &["src/clocks.rs"],
    },
    Store {
        id: StoreId::Wiring,
        name: "the wiring",
        on: Where::Processor,
        what: "This list: every lane, pass and store, fixed in the game. The frame runs in its order; the software map is drawn from it.",
        limits: &[],
        files: &["src/wiring.rs"],
    },
    Store {
        id: StoreId::PanelShapes,
        name: "the panel's shapes",
        on: Where::Processor,
        what: "The panels cut into shapes for the card (made afresh each frame).",
        limits: &[],
        files: &["src/panel.rs"],
    },
    Store {
        id: StoreId::ScreenPicture,
        name: "the screen's picture",
        on: Where::Card,
        what: "The picture the card draws on and the screen shows, the window's size.",
        limits: &[],
        files: &["src/card.rs"],
    },
    Store {
        id: StoreId::FlightReport,
        name: "the flight's report",
        on: Where::Disk,
        what: "A measured flight's report, logs/flight-<seconds since 1970>.txt (only on a flight).",
        limits: &[],
        files: &["src/flight.rs"],
    },
    Store {
        id: StoreId::SessionLog,
        name: "the session's log",
        on: Where::Disk,
        what: "The session's messages and any crash, logs/session-<seconds since 1970>.log. Every part writes to it, so it has no lines on the map.",
        limits: &[],
        files: &["src/session.rs"],
    },
    Store {
        id: StoreId::Export,
        name: "the export",
        on: Where::Disk,
        what: "Old's test world written out as columns, exports/nice25 test: where it lies, the sea's level, every profile, and a file a tile (512 x 512 columns, raw deflate). The tablet takes its place at 3.1.",
        limits: &[],
        files: &["src/doorway.rs"],
    },
    Store {
        id: StoreId::TileColumns,
        name: "a tile's columns",
        on: Where::Processor,
        what: "A tile's answer: each column's ground, water and profile as the export holds them, its runs made when asked.",
        limits: &[],
        files: &["src/doorway.rs"],
    },
    Store {
        id: StoreId::Marks,
        name: "the marks",
        on: Where::Disk,
        what: "The places every flight stands at, the same each time, data/marks.toml: where, how high, which way, fog, whose marks (near or far), and what's changed there first. The game starts at the first.",
        limits: &[],
        files: &["src/marks.rs"],
    },
    Store {
        id: StoreId::Changes,
        name: "what changed",
        on: Where::Processor,
        what: "The changes made through the doorway, by tile: whole blocks and bars, in the order they were made. A mark's are taken back when the flight moves on.",
        limits: &[],
        files: &["src/doorway.rs"],
    },
    Store {
        id: StoreId::FarCells,
        name: "the far cells",
        on: Where::Processor,
        what: "The world as the far eye sees it: cubes of 32 blocks and every level above, 16 bytes a cell (old's form: how solid, how wet, which ways its faces look, their colour, how their light and height spread, how much surface shows), only each column's surface band kept. Colours from data/far colours.toml, old's, until the Registrar.",
        limits: &[("blocks a side of the first level's cells", 1 << crate::far::FIRST), ("blocks a side of a region, made at once", 1 << crate::far::REGION)],
        files: &["src/far.rs"],
    },
    Store {
        id: StoreId::FarBricks,
        name: "the far bricks",
        on: Where::Card,
        what: "The far cells on the card, the whole world as old's bake kept it: each level's columns' bands (the lowest cell and how many: solid below, empty above), and the bands' cells in bricks of 8 x 8 x 8 found by arithmetic. The pool grows as it fills.",
        limits: &[("levels, cells of 32 to 65,536", crate::bricks::LEVELS as u64), ("cells a brick at most", 512), ("the largest buffer asked of the card (MB)", 2047)],
        files: &["src/bricks.rs"],
    },
    Store {
        id: StoreId::FarLayers,
        name: "the far layers",
        on: Where::Card,
        what: "The screen pyramid's pictures (8 x 8, 4 x 4 and 2 x 2 screen pixels a pixel), each with how far its rays first touched anything and how much the layer below is worth; and where each tile's rays start.",
        limits: &[("layers", crate::far_eye::LAYERS.len() as u64)],
        files: &["src/far_eye.rs"],
    },
    Store {
        id: StoreId::FarLook,
        name: "the far eye's look",
        on: Where::Processor,
        what: "What David tunes on the Render page (F3): where the levels hand over, where the 4 x 4 and the 2 x 2 fade in, and the sky's weight at a skyline; each with its default (Default puts them all back).",
        limits: &[],
        files: &["src/far_eye.rs", "src/panel.rs"],
    },
    Store {
        id: StoreId::FarTextures,
        name: "the far textures",
        on: Where::Card,
        what: "The far bricks again as 3D textures for the card's texture hardware to blend (change 1): how solid and how watery (16-bit), and each way's colour and faces; each brick's slot beside its place in the bricks' index.",
        limits: &[("texels a side of a brick, its border included", crate::atlas::SIDE as u64), ("bricks a side of an atlas layer", crate::atlas::ACROSS as u64)],
        files: &["src/atlas.rs"],
    },
];

pub fn lane(id: LaneId) -> &'static Lane {
    &LANES[id as usize]
}

pub fn pass(id: PassId) -> &'static Pass {
    &PASSES[id as usize]
}

pub fn store(id: StoreId) -> &'static Store {
    &STORES[id as usize]
}

/// A lane's passes, in the order they run.
pub fn passes_in(lane: LaneId) -> impl Iterator<Item = &'static Pass> {
    PASSES.iter().filter(move |pass| pass.lane == lane)
}

/// The passes on the card, in the order they run.
pub fn card_passes() -> impl Iterator<Item = &'static Pass> {
    PASSES.iter().filter(|pass| pass.on == Where::Card)
}

/// A pass's slot in the card's clock: its place among the passes on the card (none if it isn't on the card).
pub fn card_slot(id: PassId) -> Option<usize> {
    card_passes().position(|pass| pass.id == id)
}

/// The passes that write a store, and those that read it.
pub fn writers(store: StoreId) -> impl Iterator<Item = &'static Pass> {
    PASSES.iter().filter(move |pass| pass.writes.contains(&store))
}

pub fn readers(store: StoreId) -> impl Iterator<Item = &'static Pass> {
    PASSES.iter().filter(move |pass| pass.reads.contains(&store))
}

/// Names, joined: "a, b and c" ("none" for none).
pub fn joined<'a>(names: impl Iterator<Item = &'a str>) -> String {
    let names: Vec<&str> = names.collect();
    match names.len() {
        0 => "none".to_string(),
        1 => names[0].to_string(),
        n => format!("{} and {}", names[..n - 1].join(", "), names[n - 1]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn top() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// Every `.rs` file under `src/`, named from the project's top ("src/...").
    fn code_files() -> Vec<String> {
        fn walk(folder: &Path, found: &mut Vec<PathBuf>) {
            for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, found);
                } else if path.extension().is_some_and(|extension| extension == "rs") {
                    found.push(path);
                }
            }
        }
        let mut found = Vec::new();
        walk(&top().join("src"), &mut found);
        let mut names: Vec<String> = found.iter().map(|path| path.strip_prefix(top()).unwrap().to_string_lossy().replace('\\', "/")).collect();
        names.sort();
        names
    }

    fn all_files() -> Vec<&'static str> {
        LANES.iter().flat_map(|lane| lane.files.iter()).chain(PASSES.iter().flat_map(|pass| pass.files.iter())).chain(STORES.iter().flat_map(|store| store.files.iter())).copied().collect()
    }

    fn all_switches() -> Vec<&'static str> {
        LANES.iter().flat_map(|lane| lane.switches.iter()).chain(PASSES.iter().flat_map(|pass| pass.switches.iter())).map(|(name, _)| *name).collect()
    }

    /// The switches the code reads: every "INCUS_..." or "WGPU_..." written in it, outside this list (and not the
    /// build's own stamp, `INCUS_BUILD...`, which `build.rs` makes).
    fn switches_in_code() -> Vec<String> {
        let mut found = Vec::new();
        for file in code_files().iter().filter(|file| file.as_str() != "src/wiring.rs") {
            let text = std::fs::read_to_string(top().join(file)).unwrap();
            for start in ["\"INCUS_", "\"WGPU_"] {
                for (at, _) in text.match_indices(start) {
                    let name: String = text[at + 1..].chars().take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_').collect();
                    if name.ends_with('_') || name.starts_with("INCUS_BUILD") {
                        continue;
                    }
                    if !found.contains(&name) {
                        found.push(name);
                    }
                }
            }
        }
        found.sort();
        found
    }

    #[test]
    fn the_ids_line_up_with_their_declarations() {
        for (k, lane) in LANES.iter().enumerate() {
            assert_eq!(lane.id as usize, k, "{} is listed out of its id's order", lane.name);
        }
        for (k, pass) in PASSES.iter().enumerate() {
            assert_eq!(pass.id as usize, k, "{} is listed out of its id's order", pass.name);
        }
        for (k, store) in STORES.iter().enumerate() {
            assert_eq!(store.id as usize, k, "{} is listed out of its id's order", store.name);
        }
        // A lane's passes are listed together, in the order they run.
        let lanes: Vec<LaneId> = PASSES.iter().map(|pass| pass.lane).collect();
        let mut seen = Vec::new();
        for lane in lanes.windows(2).filter(|pair| pair[0] != pair[1]).map(|pair| pair[0]).chain(lanes.last().copied()) {
            assert!(!seen.contains(&lane), "{}'s passes aren't listed together", super::lane(lane).name);
            seen.push(lane);
        }
    }

    #[test]
    fn every_file_is_on_the_map_and_every_file_on_it_exists() {
        let named = all_files();
        for file in &named {
            assert!(top().join(file).is_file(), "the wiring names {file}, which doesn't exist");
        }
        for file in code_files() {
            assert!(named.contains(&file.as_str()), "{file} is on no lane, pass or store in the wiring (src/wiring.rs)");
        }
    }

    #[test]
    fn every_switch_is_on_the_map_and_every_switch_on_it_is_read() {
        let declared = all_switches();
        let read = switches_in_code();
        for name in &read {
            assert!(declared.contains(&name.as_str()), "the code reads {name}, which is on no lane or pass in the wiring (src/wiring.rs)");
        }
        for name in &declared {
            assert!(read.iter().any(|read| read == name), "the wiring names {name}, which the code never reads");
        }
    }

    #[test]
    fn every_store_is_written_or_read_by_a_pass() {
        for store in &STORES {
            if store.id == StoreId::SessionLog {
                continue;
            }
            assert!(writers(store.id).count() + readers(store.id).count() > 0, "{} has no lines: nothing writes or reads it", store.name);
        }
    }

    /// The newest kept flight (`kept logs/flight-*.txt`): its name, and its text.
    /// The newest kept flight flown as the game runs (no switch but `INCUS_FLIGHT`: a flight with something left out for
    /// measuring isn't the game's cost).
    fn newest_flight() -> Option<(String, String)> {
        let folder = top().join("kept logs");
        let mut flights: Vec<String> = std::fs::read_dir(&folder).ok()?.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).filter(|name| name.starts_with("flight-") && name.ends_with(".txt")).collect();
        flights.sort();
        flights.into_iter().rev().find_map(|name| {
            let text = std::fs::read_to_string(folder.join(&name)).ok()?;
            text.lines().any(|line| line == "Switches: INCUS_FLIGHT=1").then_some((name, text))
        })
    }

    /// A pass's cost from a flight's report: the rest of its line ("    <name>: ...").
    fn cost_in(report: &str, name: &str) -> Option<String> {
        let start = format!("    {name}: ");
        report.lines().find_map(|line| line.strip_prefix(start.as_str())).map(|rest| rest.trim().to_string())
    }

    /// `master/map.md`, drawn from the wiring.
    fn draw_map() -> String {
        let flight = newest_flight();
        let mut text = String::new();
        text += "# The software map\n\n";
        text += "What exists and how it's wired, drawn from the code's own list of its parts (`src/wiring.rs`) by its test on\n";
        text += "every build: change the list, never this file. In the game, the same map is live: the editor (F2), \"Software map\".\n\n";
        text += "The words: a **lane** is a line of work with its own pace; a **pass** is one piece of a lane's work, on the\n";
        text += "processor or the card; a **stage** is one step of the world's recipe (none yet); a **store** is something held.\n\n";
        let flight_line = match &flight {
            Some((name, report)) => format!("The costs are the newest kept flight's flown as the game runs, at its first standing, `kept logs/{name}`: {}.\n\n", report.lines().next().unwrap_or("").trim_start_matches("The flight: ")),
            None => "No flight is kept yet, so no costs.\n\n".to_string(),
        };
        text += &flight_line;
        // The diagram (Mermaid: VS Code and GitHub draw it). Arrows: a store into what reads it, a pass into what it writes.
        text += "```mermaid\nflowchart TB\n";
        let key = |name: &str| name.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
        for lane in &LANES {
            text += &format!("  subgraph lane_{}[\"{} ({})\"]\n    direction LR\n", key(lane.name), lane.name, lane.runs);
            let passes: Vec<&Pass> = passes_in(lane.id).collect();
            for pass in &passes {
                text += &format!("    pass_{}[\"{}\"]\n", key(pass.name), pass.name);
            }
            text += "  end\n";
        }
        for store in &STORES {
            text += &format!("  store_{}[(\"{}\")]\n", key(store.name), store.name);
        }
        for pass in &PASSES {
            for store in pass.reads {
                text += &format!("  store_{} --> pass_{}\n", key(super::store(*store).name), key(pass.name));
            }
            for store in pass.writes {
                text += &format!("  pass_{} --> store_{}\n", key(pass.name), key(super::store(*store).name));
            }
        }
        text += "```\n\n";
        text += "## The lanes\n\n";
        for lane in &LANES {
            text += &format!("### {}\n\n", lane.name);
            text += &format!("- **Runs:** {}.\n- **What it is:** {}\n", lane.runs, lane.what);
            text += &format!("- **Its passes, in order:** {}.\n", joined(passes_in(lane.id).map(|pass| pass.name)));
            for (name, what) in lane.switches {
                text += &format!("- **Switch** `{name}`: {what}.\n");
            }
            text += &format!("- **Files:** {}.\n\n", joined(lane.files.iter().copied()));
        }
        text += "## The passes\n\n";
        for pass in &PASSES {
            text += &format!("### {}\n\n", pass.name);
            text += &format!("- **A pass** in {}, on {}.\n- **What it is:** {}\n", lane(pass.lane).name, pass.on.name(), pass.what);
            text += &format!("- **Reads:** {}. **Writes:** {}.\n", joined(pass.reads.iter().map(|store| super::store(*store).name)), joined(pass.writes.iter().map(|store| super::store(*store).name)));
            for (name, what) in pass.switches {
                text += &format!("- **Switch** `{name}`: {what}.\n");
            }
            if let Some((name, report)) = &flight {
                let cost = cost_in(report, pass.name).unwrap_or_else(|| "not measured".to_string());
                text += &format!("- **Cost, last measured** (`kept logs/{name}`): {cost}.\n");
            }
            text += &format!("- **Files:** {}.\n\n", joined(pass.files.iter().copied()));
        }
        text += "## The stores\n\n";
        for store in &STORES {
            text += &format!("### {}\n\n", store.name);
            text += &format!("- **A store** on {}.\n- **What it is:** {}\n", store.on.name(), store.what);
            text += &format!("- **Written by:** {}. **Read by:** {}.\n", joined(writers(store.id).map(|pass| pass.name)), joined(readers(store.id).map(|pass| pass.name)));
            for (name, value) in store.limits {
                text += &format!("- **Limit:** {name}, {value}.\n");
            }
            text += &format!("- **Files:** {}.\n\n", joined(store.files.iter().copied()));
        }
        text
    }

    /// `master/map.md` is what the wiring draws. If it isn't, it's drawn again and the test fails, so the new map goes
    /// into the same commit as the change.
    #[test]
    fn the_map_is_drawn_from_the_wiring() {
        let path = top().join("master").join("map.md");
        let drawn = draw_map();
        let kept = std::fs::read_to_string(&path).unwrap_or_default().replace("\r\n", "\n");
        if kept != drawn {
            std::fs::write(&path, &drawn).unwrap();
            panic!("master/map.md was out of date with the wiring; it's been drawn again: run the tests again, and commit it");
        }
    }
}
