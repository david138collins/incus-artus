//! The window and the frame.
//!
//! The frame runs its passes in the order the wiring lists them (`wiring.rs`, the frame's lane), each timed on the
//! processor; the card's clock times the passes on the card. The window's events are the other lane: each one is heard
//! as it comes, and timed too.
//!
//! Keys (old's): F2 the editor, F3 the debug panel (in the game), F11 the editor full screen, Tab the mouse (`camera.rs`
//! has the flying keys), Esc a menu shut. The game is borderless and full screen [David, 2026-11-05: the same direct
//! path to the screen as exclusive full screen, and alt-tab is instant]. The window never wears Windows' own frame
//! [2026-11-20: "there should be no wrapper at all, so the entire thing is 95gui"]: the editor is shown filling the
//! screen above the taskbar, as a smaller window, or full screen (F11, the editor only), in its own 95 frame, and it
//! comes back across F2 the way it was [2026-11-19: "when you hit f2 itll go back to the editor, windowed or
//! fullscreen"]. Windows does the moving and the sizing, from the editor's title bar and edges.
//!
//! The editor and the debug panel are the 95 drawing's sheets (`gui/`), laid out on the processor and drawn over the
//! picture on the card; the mouse's left button reaches them while it's free. Their scale is Auto from the screen, never
//! from the window [2026-11-20: "when i maximized it last time, it changed scale"], or the editor's own from its menu.

use crate::camera::Camera;
use crate::card::Card;
use crate::clocks::{CardClock, FrameClock, PassClock};
use crate::doorway::Doorway;
use crate::editor::{Ask, Editor, Side};
use crate::far_eye::FarEye;
use crate::flight::{About, Flight};
use crate::gui;
use crate::marks::{Made, Mark};
use crate::panel::Panel;
use crate::wiring::{self, LaneId, PassId};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
#[cfg(windows)]
use winit::platform::windows::{CornerPreference, WindowAttributesExtWindows};
use winit::window::{CursorGrabMode, CursorIcon, Fullscreen, ResizeDirection, Window, WindowId};

/// The picture's colour where nothing is drawn (old's sky, where it had no world to draw).
const SKY: wgpu::Color = wgpu::Color { r: 0.25, g: 0.34, b: 0.52, a: 1.0 };

pub fn run() {
    let event_loop = EventLoop::new().expect("an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    // The world, through the doorway (the engine never opens a world file itself), and the marks.
    let folder = crate::doorway::test_world();
    let doorway = match Doorway::open(&folder) {
        Ok(doorway) => {
            log::info!("The doorway: open on {}", folder.display());
            Some(doorway)
        }
        Err(error) => {
            log::error!("The doorway couldn't open the world: {error}");
            None
        }
    };
    let mut marks = crate::marks::load().unwrap_or_else(|error| {
        log::error!("The marks couldn't be read: {error}");
        Vec::new()
    });
    // The scatter stage's own marks, after the file's, while it's on.
    if let Some(stage) = doorway.as_ref().and_then(Doorway::stage) {
        marks.extend(stage.marks());
    }
    // The mark the game starts at: `INCUS_MARK` by its name, else the first.
    let first_mark = std::env::var("INCUS_MARK").ok().and_then(|name| {
        let found = marks.iter().position(|mark| mark.name == name);
        if found.is_none() {
            log::warn!("INCUS_MARK={name} is no mark's name; the game starts at the first");
        }
        found
    });
    let flight = crate::flight::wanted().then(|| Flight::new(&marks));
    let mut panel = Panel::new();
    panel.visible = flight.is_none();
    // What the game starts with (`presets.rs`): each page's "default" preset, if David has saved one; the built-in
    // settings on a flight. The Render page's are taken once the eyes are made (`open`).
    let mut camera = Camera::new();
    let (preset, what) = crate::presets::start::<crate::presets::CameraPreset>(&crate::presets::folder(), "camera", flight.is_some());
    log::info!("The Camera page's settings: {what}");
    (camera.pace.speed, camera.pace.multiplier, camera.fov) = (preset.speed, preset.multiplier, preset.field_of_view);
    let mut runner = Runner {
        running: None,
        frame_clock: FrameClock::new(),
        pass_clock: PassClock::new(),
        camera,
        panel,
        editor: Editor::new(),
        held: HashSet::new(),
        captured: false,
        doorway,
        marks,
        first_mark: first_mark.unwrap_or(0),
        at_first_mark: false,
        flight,
        temperatures: crate::temperatures::Watch::start(),
        readout: crate::temperatures::Readout::new(),
        pointer: None,
        scale: 1,
        editor_shown: false,
        editor_full_screen: false,
        pointer_shape: None,
        least: None,
        memory_read: None,
        memory_logged: None,
        tally_at: None,
    };
    event_loop.run_app(&mut runner).expect("the game runs");
    log::info!("Quitting");
}

/// What exists once the window is open.
struct Running {
    window: Arc<Window>,
    card: Card,
    card_clock: Option<CardClock>,
    /// The 95 drawing's sheets on the card.
    sheets: gui::on_card::OnCard,
    /// The screen's refresh rate, a second.
    refresh: f32,
    /// The screen's size (its pixels), for the scale's Auto; and the room on it above the taskbar, as the window's
    /// size when it last filled it (the screen's size till then), for which of the editor's scales it can hold.
    screen: (u32, u32),
    room: (u32, u32),
    /// The far eye (none if the doorway couldn't open the world).
    far: Option<FarEye>,
    /// The near eye (none without a far eye, or with `INCUS_NEAR=0`).
    near: Option<crate::near_eye::NearEye>,
}

/// What a frame's passes hand on to the passes after them.
#[derive(Default)]
struct FrameWork {
    /// The screen's picture to draw on, and the card's work for it.
    texture: Option<wgpu::SurfaceTexture>,
    encoder: Option<wgpu::CommandEncoder>,
    commands: Vec<wgpu::CommandBuffer>,
    /// The card's times that came back this frame.
    card_frames: Vec<Vec<f32>>,
    /// Whether the far eye drew the picture (the panels then go over it; else over the sky's colour).
    far_drew: bool,
    /// Whether the far eye's numbers are this frame's (its start pass recorded), and whether the near eye drew its
    /// picture this frame (its cover and the x-fader go only after it).
    far_ready: bool,
    near_drew: bool,
    /// Whether this frame counted the far layers' longest rays (step 0).
    tallied: bool,
}

struct Runner {
    running: Option<Running>,
    frame_clock: FrameClock,
    pass_clock: PassClock,
    camera: Camera,
    panel: Panel,
    editor: Editor,
    /// The keys held down now.
    held: HashSet<KeyCode>,
    /// Whether the mouse is captured (flying) or free (the panels can be used).
    captured: bool,
    doorway: Option<Doorway>,
    marks: Vec<Mark>,
    /// The mark the game starts at, and whether it's gone there yet.
    first_mark: usize,
    at_first_mark: bool,
    /// The temperatures, read on a thread of their own (none if `INCUS_TEMPERATURES=0`), and drawn over the world.
    temperatures: Option<crate::temperatures::Watch>,
    readout: crate::temperatures::Readout,
    /// Where the mouse's pointer is on the screen (its pixels), and the sheets' scale as last laid out.
    pointer: Option<(f64, f64)>,
    scale: u32,
    /// Whether the editor has been shown yet (the first time, it fills the screen above the taskbar), and whether it
    /// was full screen when it was last left for the game.
    editor_shown: bool,
    editor_full_screen: bool,
    /// The pointer's shape as last set over the editor (a side to size the window from, or the arrow), and the window's
    /// least size as last set (its pixels).
    pointer_shape: Option<Side>,
    least: Option<(u32, u32)>,
    /// When the main memory was last read (once a second on a flight), and last put in the session's log (every half
    /// minute).
    memory_read: Option<Instant>,
    memory_logged: Option<Instant>,
    /// When the far layers' longest rays were last counted (once a second on a flight).
    tally_at: Option<Instant>,
    flight: Option<Flight>,
}

impl Runner {
    fn open(&mut self, event_loop: &ActiveEventLoop) {
        // No frame of Windows' own, ever [David, 2026-11-20: "there should be no wrapper at all"]. Its smaller size, the
        // first time the editor is made smaller, is three quarters of the screen, in its middle (Builder's, for his eye).
        let screen = event_loop.primary_monitor().map_or((2560, 1600), |monitor| (monitor.size().width, monitor.size().height));
        let smaller = (screen.0 * 3 / 4, screen.1 * 3 / 4);
        let attributes = Window::default_attributes()
            .with_title("Incus Artus")
            .with_decorations(false)
            .with_inner_size(winit::dpi::PhysicalSize::new(smaller.0, smaller.1))
            .with_position(winit::dpi::PhysicalPosition::new(((screen.0 - smaller.0) / 2) as i32, ((screen.1 - smaller.1) / 2) as i32))
            .with_maximized(true)
            .with_fullscreen(Some(Fullscreen::Borderless(None)));
        // Windows 11's rounded corners and coloured edge left off: the window's edge is the 95 frame's to the pixel.
        #[cfg(windows)]
        let attributes = attributes.with_corner_preference(CornerPreference::DoNotRound).with_border_color(None);
        let window = Arc::new(event_loop.create_window(attributes).expect("a window"));
        let card = Card::new(window.clone(), self.flight.is_some());
        let card_clock = CardClock::new(&card.device, &card.queue, wiring::card_passes().count());
        let sheets = gui::on_card::OnCard::new(&card.device, card.config.format);
        let refresh = window.current_monitor().and_then(|monitor| monitor.refresh_rate_millihertz()).map_or(60.0, |millihertz| millihertz as f32 / 1000.0);
        log::info!("The window: {} x {}, the screen {refresh:.0} a second", card.config.width, card.config.height);
        // The far eye, its cells summed on the loading lane nearest the first mark first, on all but two of the
        // processor's threads (the frame and the window keep theirs).
        let mut far = self.doorway.as_ref().map(|doorway| {
            let store = crate::bricks::Store::new(&card.device, &card.queue, crate::far::regions_of(doorway), doorway.height(), crate::bricks::brick_side(), crate::bricks::window_regions());
            let near = self.marks.get(self.first_mark).map_or([0.0, 0.0], |mark| mark.at);
            let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).saturating_sub(2).max(1);
            let lane = crate::far::Lane::start(crate::doorway::test_world(), near, threads).map_err(|error| log::error!("The far cells couldn't be summed: {error}")).ok();
            // The rings round the eye (the 16s' and the 8s'), each its maker on a thread of its own (as many as
            // `INCUS_FAR_RING` says: 0 none).
            let rings = crate::far_ring::rings_asked()
                .into_iter()
                .filter_map(|level| crate::far_ring::Ring::start(crate::doorway::test_world(), store.levels[crate::bricks::row(level)], store.kept.side()).map_err(|error| log::error!("A far ring couldn't be started: {error}")).ok())
                .collect();
            FarEye::new(&card.device, card.config.format, store, lane, rings)
        });
        // The near eye: the blocks inside the render radius as triangles (plan 2.3), its lane's threads its own.
        let mut near = (far.is_some() && std::env::var("INCUS_NEAR").as_deref() != Ok("0"))
            .then(|| {
                let mut doorway = Doorway::open(&crate::doorway::test_world()).map_err(|error| log::error!("The near eye's colours couldn't be read: {error}")).ok()?;
                let colours = crate::far::FarColours::load(&mut doorway).map_err(|error| log::error!("The near eye's colours couldn't be read: {error}")).ok()?;
                let textures = crate::textures::load(&doorway, &colours);
                crate::near_eye::NearEye::new(&card.device, &card.queue, card.config.format, crate::doorway::test_world(), &colours.as_light(), &textures).map_err(|error| log::error!("The near eye couldn't start: {error}")).ok()
            })
            .flatten();
        // The Render page's settings the game starts with: its "default" preset, if there is one (`presets.rs`).
        let (preset, what) = crate::presets::start::<crate::presets::RenderPreset>(&crate::presets::folder(), "render", self.flight.is_some());
        log::info!("The Render page's settings: {what}");
        if let Some(far) = &mut far {
            far.look = preset.far;
        }
        if let Some(near) = &mut near {
            near.look = preset.near;
        }
        let screen = screen_of(&window).unwrap_or(screen);
        self.running = Some(Running { window, card, card_clock, sheets, refresh, screen, room: screen, far, near });
    }

    /// The mouse captured for flying, or let go for the panels.
    fn capture(&mut self, captured: bool) {
        let Some(running) = &self.running else { return };
        if captured {
            let grabbed = running.window.set_cursor_grab(CursorGrabMode::Locked).or_else(|_| running.window.set_cursor_grab(CursorGrabMode::Confined));
            if let Err(error) = grabbed {
                log::warn!("The mouse couldn't be captured: {error}");
                return;
            }
        } else {
            let _ = running.window.set_cursor_grab(CursorGrabMode::None);
        }
        running.window.set_cursor_visible(!captured);
        self.captured = captured;
    }

    /// A key pressed (not held down repeating): the editor, the panel, the screen, a menu shut, the mouse, the lantern.
    fn pressed(&mut self, key: KeyCode) {
        match key {
            KeyCode::F2 => self.editor_toggled(),
            // The debug panel is the game's only [David, 2026-11-20: "the debug panel should not appear in the editor"].
            KeyCode::F3 if !self.editor.open => self.panel.visible = !self.panel.visible,
            // Full screen is the editor's only [David, 2026-11-19: "f11 would only effect the editor, the game will stay
            // borderless fullscreen"].
            KeyCode::F11 if self.editor.open => self.full_screen_toggled(),
            KeyCode::Escape if self.editor.open => self.editor.let_go(),
            KeyCode::Tab if !self.editor.open => self.capture(!self.captured),
            // The lantern (build 217) [David, 2026-11-17: "The L key"]: lit or put out, while playing.
            KeyCode::KeyL if self.captured && !self.editor.open => {
                if let Some(near) = self.running.as_mut().and_then(|running| running.near.as_mut()) {
                    near.look.lantern = !near.look.lantern;
                }
            }
            _ => {}
        }
    }

    /// Into the editor, or out of it to the game (F2, File > Return to Game). The editor comes back the way it was last
    /// shown: filling the screen above the taskbar the first time, then as it was left (smaller, or full screen). The
    /// game is borderless full screen.
    fn editor_toggled(&mut self) {
        if let Some(window) = self.running.as_ref().map(|running| running.window.clone()) {
            if self.editor.open {
                self.editor_full_screen = window.fullscreen().is_some();
                window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                window.set_cursor(CursorIcon::Default);
                self.pointer_shape = None;
            } else if !self.editor_full_screen {
                window.set_fullscreen(None);
                if !self.editor_shown {
                    window.set_maximized(true);
                }
            }
        }
        self.editor.toggle();
        if self.editor.open {
            self.editor_shown = true;
            self.capture(false);
            self.held.clear();
        }
    }

    /// The editor full screen, over the taskbar too, or back to the window it was (F11, Window > Full Screen).
    fn full_screen_toggled(&self) {
        if let Some(running) = &self.running {
            let full = running.window.fullscreen().is_some();
            running.window.set_fullscreen((!full).then_some(Fullscreen::Borderless(None)));
        }
    }

    /// What the editor asked of the window, or of the game.
    fn asked(&mut self, ask: Ask, event_loop: &ActiveEventLoop) {
        let Some(window) = self.running.as_ref().map(|running| running.window.clone()) else { return };
        match ask {
            Ask::ReturnToGame => self.editor_toggled(),
            Ask::Quit => event_loop.exit(),
            Ask::Minimise => window.set_minimized(true),
            // The middle button: from full screen or filling the screen, the smaller window; from that, filling.
            Ask::FillOrRestore => {
                if window.fullscreen().is_some() {
                    window.set_fullscreen(None);
                    window.set_maximized(false);
                } else {
                    window.set_maximized(!window.is_maximized());
                }
            }
            Ask::FullScreen => self.full_screen_toggled(),
            // Windows moves and sizes it, from where the pointer was pressed.
            Ask::Move => {
                if let Err(error) = window.drag_window() {
                    log::warn!("The window couldn't be moved: {error}");
                }
            }
            Ask::Size(side) => {
                if let Err(error) = window.drag_resize_window(direction(side)) {
                    log::warn!("The window couldn't be sized: {error}");
                }
            }
            // The editor's scale: it's laid out at it from the next frame, the window's least size with it.
            Ask::Scale => {}
        }
    }

    /// The pointer's shape over the editor: a side's arrows where a press would size the window, else the arrow.
    fn shaped(&mut self, side: Option<Side>) {
        if side == self.pointer_shape {
            return;
        }
        if let Some(running) = &self.running {
            running.window.set_cursor(match side {
                None => CursorIcon::Default,
                Some(Side::North | Side::South) => CursorIcon::NsResize,
                Some(Side::East | Side::West) => CursorIcon::EwResize,
                Some(Side::NorthEast | Side::SouthWest) => CursorIcon::NeswResize,
                Some(Side::NorthWest | Side::SouthEast) => CursorIcon::NwseResize,
            });
        }
        self.pointer_shape = side;
    }

    /// A pass ran from `started` till now, on the processor.
    fn timed(&mut self, pass: PassId, started: Instant) {
        self.ran(pass, started.elapsed().as_secs_f32() * 1000.0);
    }

    /// A pass ran for `ms` on the processor.
    fn ran(&mut self, pass: PassId, ms: f32) {
        self.pass_clock.record(pass, ms);
        if let Some(flight) = &mut self.flight {
            flight.pass(Instant::now(), pass, ms);
        }
    }

    /// The frame: its passes in the wiring's order. A pass that can't go on (no picture to draw on) ends the frame.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let mut work = FrameWork::default();
        for pass in wiring::passes_in(LaneId::Frame) {
            let started = Instant::now();
            let carry_on = match pass.id {
                PassId::GoingToAMark => self.going_to_a_mark(started),
                PassId::Flying => self.flying(started),
                PassId::PuttingTheFarCellsOnTheCard => self.putting_the_far_cells_on_the_card(),
                PassId::PuttingTheNearChunksOnTheCard => self.putting_the_near_chunks_on_the_card(),
                PassId::LayingOutThePanel => self.laying_out_the_panel(),
                PassId::GettingThePicture => self.getting_the_picture(&mut work),
                PassId::TheFarStart => self.the_far_start(&mut work),
                PassId::ListingTheNearChunks => self.listing_the_near_chunks(&mut work),
                PassId::TheFar8 => self.the_far_layer(&mut work, 0, PassId::TheFar8),
                PassId::TheFar4 => self.the_far_layer(&mut work, 1, PassId::TheFar4),
                PassId::TheFar2 => self.the_far_layer(&mut work, 2, PassId::TheFar2),
                PassId::BringingItUp => self.bringing_it_up(&mut work),
                PassId::TheNearFaces | PassId::TheNearWater | PassId::TheNearCover | PassId::TheXFader | PassId::SeeingTheFarLamps | PassId::TheFarLamps => self.the_near(&mut work, pass.id),
                PassId::ThePanel => self.the_panel(&mut work),
                PassId::HandingTheCardItsWork => self.handing_the_card_its_work(&mut work),
                PassId::ShowingIt => self.showing_it(&mut work),
                PassId::Measuring => self.measuring(&mut work, event_loop),
                PassId::HearingTheWindow | PassId::TheDoorway | PassId::SummingTheFarCells | PassId::MakingTheFarTextures | PassId::MakingTheFarLight | PassId::MakingTheFarRing | PassId::SpreadingTheLantern | PassId::ListingTheLamps | PassId::MeshingTheNearChunks | PassId::ReadingTheTemperatures => unreachable!("{} isn't one of the frame's passes", pass.name),
            };
            self.timed(pass.id, started);
            if !carry_on {
                return;
            }
        }
    }

    /// At the start, and on a flight once a mark is measured: the camera taken to the next mark, and its changes made.
    fn going_to_a_mark(&mut self, now: Instant) -> bool {
        let target = match &mut self.flight {
            Some(flight) => flight.due(now).map(|standing| (standing.mark, standing.count)),
            // (Outside a flight a mark's scatter isn't made, unless a count is asked for: `INCUS_SCATTER`, for a look.)
            None if !self.at_first_mark => Some((self.first_mark, std::env::var("INCUS_SCATTER").ok().and_then(|count| count.parse::<u32>().ok()))),
            None => None,
        };
        // A flight's first mark isn't measured till the far cells are all on the card (old's marks waited on their
        // levels too).
        let far_whole = self.running.as_ref().and_then(|running| running.far.as_ref()).is_none_or(FarEye::whole);
        if let Some(flight) = &mut self.flight
            && !far_whole
        {
            flight.hold(now);
        }
        self.at_first_mark = true;
        if let Some((k, count)) = target {
            let (made, eye) = self.go_to(k, count);
            let going = now.elapsed().as_secs_f32() * 1000.0;
            // The doorway's answers on the way, each timed as its own pass.
            let times = self.doorway.as_ref().map(Doorway::take_times).unwrap_or_default();
            if let Some(flight) = &mut self.flight {
                flight.arrived(made, eye, (going, times.len(), times.iter().sum()));
            }
            for ms in times {
                self.ran(PassId::TheDoorway, ms);
            }
        }
        // Nor is a standing mark measured till the near eye's chunks are all in there, the mark's changes in them (a
        // moving mark's keep coming, and aren't waited for).
        if let Some(flight) = &mut self.flight {
            let eye = [self.camera.position.x, self.camera.position.y];
            // (And the far eye's: what's placed there summed, on the card and lit.)
            let far_settled = self.running.as_ref().and_then(|running| running.far.as_ref()).is_none_or(FarEye::changes_settled);
            let whole = self.running.as_ref().and_then(|running| running.near.as_ref()).map(|near| near.whole(eye) && far_settled);
            let moves = flight.speed(&self.marks).is_some();
            flight.hold_for_near(now, whole, moves);
        }
        true
    }

    /// The camera at mark `k` (its scatter at `count`), its changes made through the doorway: what was made, and the eye.
    fn go_to(&mut self, k: usize, count: Option<u32>) -> (Made, Option<[f32; 3]>) {
        let Some(mark) = self.marks.get(k).cloned() else { return (Made::default(), None) };
        let Some(doorway) = &mut self.doorway else { return (Made::default(), None) };
        doorway.forget_changes();
        let mut made = Made::default();
        if let Some(name) = &mark.changes {
            match std::fs::read_to_string(crate::session::project_folder().join(name)) {
                Ok(log) => made = crate::marks::place(doorway, &log),
                Err(error) => log::warn!("{}'s changes couldn't be read from {name}: {error}", mark.name),
            }
        }
        if let (Some(scatter), Some(count)) = (&mark.scatter, count) {
            match crate::marks::scatter(doorway, &mark, scatter, count) {
                Ok(scattered) => {
                    made.blocks += scattered.blocks;
                    made.bars += scattered.bars;
                    made.octs += scattered.octs;
                    made.refused += scattered.refused;
                }
                Err(error) => log::warn!("{error}"),
            }
        }
        let eye = match crate::marks::eye(doorway, &mark) {
            Ok(eye) => {
                self.camera.position = eye;
                self.camera.facing = mark.facing;
                self.camera.pitch = mark.pitch;
                Some(eye.to_array())
            }
            Err(error) => {
                log::warn!("The camera couldn't go to {}: {error}", mark.name);
                None
            }
        };
        // The near eye and the far eye told of the changes as they are now (the mark before's taken back, this one's
        // made).
        if let Some(near) = self.running.as_mut().and_then(|running| running.near.as_mut()) {
            near.changed(doorway.changes());
        }
        if let Some(far) = self.running.as_mut().and_then(|running| running.far.as_mut()) {
            far.changed(doorway.changes());
        }
        let dial = count.map_or(String::new(), |count| format!(", the scatter at {count}"));
        log::info!(
            "At {}{dial}: the eye at {:.1} {:.1} {:.1}; {} whole blocks and {} bars made through the doorway, {} refused",
            mark.name,
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
            made.blocks,
            made.bars,
            made.refused
        );
        (made, eye)
    }

    fn putting_the_far_cells_on_the_card(&mut self) -> bool {
        let eye = [self.camera.position.x, self.camera.position.y];
        if let Some(running) = &mut self.running
            && let Some(far) = &mut running.far
        {
            far.put(&running.card.device, &running.card.queue, eye);
        }
        true
    }

    /// The near eye's chunks: asked for, put on the card, given back; and the far eye told where its rays begin.
    fn putting_the_near_chunks_on_the_card(&mut self) -> bool {
        let eye = [self.camera.position.x, self.camera.position.y];
        let mut chunk_ms = None;
        if let Some(running) = &mut self.running
            && let (Some(near), Some(far)) = (&mut running.near, &mut running.far)
        {
            if let Err(error) = near.put(&running.card.device, &running.card.queue, eye, far) {
                log::error!("The near lane stopped: {error}");
                running.near = None;
                far.near_from = 0.0;
            } else if near.counts.chunk_ms > 0.0 {
                chunk_ms = Some(near.counts.chunk_ms);
            }
        }
        if let Some(ms) = chunk_ms {
            self.ran(PassId::MeshingTheNearChunks, ms);
        }
        true
    }

    /// The near eye's chunks in view listed for the card, and its numbers for the frame: once the far eye's numbers
    /// are the frame's. Whether it has a picture to draw this frame is said to the passes after it.
    fn listing_the_near_chunks(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        let (Some(near), Some(far)) = (&mut running.near, &running.far) else { return true };
        if work.far_ready {
            work.near_drew = near.prepare(&running.card.device, &running.card.queue, (running.card.config.width, running.card.config.height), far, &self.camera);
        }
        true
    }

    /// The near eye's faces into its own picture, then (its second pass) its water over them, then (its third) its
    /// cover for the far eye, all before the march; and after the far picture is brought up (its fourth, the
    /// x-fader) its picture laid over it; then (build 223) the far lamps seen through the far cells, and (build 222)
    /// their dots added.
    fn the_near(&mut self, work: &mut FrameWork, id: PassId) -> bool {
        let Some(running) = &mut self.running else { return false };
        let (Some(near), Some(far), Some(texture), Some(encoder)) = (&mut running.near, &running.far, &work.texture, &mut work.encoder) else { return true };
        // (Its own picture once the far eye's numbers are the frame's; the x-fader once the far picture is brought up.)
        if !(if matches!(id, PassId::TheXFader | PassId::SeeingTheFarLamps | PassId::TheFarLamps) { work.far_drew } else { work.far_ready }) {
            return true;
        }
        if !work.near_drew {
            return true;
        }
        let slot = wiring::card_slot(id);
        let clock = running.card_clock.as_ref();
        match id {
            PassId::TheNearFaces => near.draw_faces(encoder, slot.and_then(|slot| clock.and_then(|clock| clock.render_pass(slot)))),
            PassId::TheNearWater => near.draw_water(encoder, slot.and_then(|slot| clock.and_then(|clock| clock.render_pass(slot)))),
            PassId::TheNearCover => {
                // The cover, and the far eye told it's this frame's: the march, recorded after, skips what it covers.
                if near.cover(&running.card.device, &running.card.queue, encoder, far, slot.and_then(|slot| clock.and_then(|clock| clock.compute_pass(slot)))) {
                    far.covered(&running.card.queue);
                }
            }
            PassId::SeeingTheFarLamps => near.see_lamps(encoder, far, slot.and_then(|slot| clock.and_then(|clock| clock.compute_pass(slot)))),
            PassId::TheFarLamps => near.draw_lamps(encoder, &texture.texture.create_view(&wgpu::TextureViewDescriptor::default()), slot.and_then(|slot| clock.and_then(|clock| clock.render_pass(slot)))),
            _ => near.join(encoder, &texture.texture.create_view(&wgpu::TextureViewDescriptor::default()), slot.and_then(|slot| clock.and_then(|clock| clock.render_pass(slot)))),
        }
        true
    }

    /// The far eye's numbers for the frame (on the processor), then its start pass (on the card).
    fn the_far_start(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        let Some(far) = &mut running.far else { return true };
        let sea_level = self.doorway.as_ref().map_or(128.0, |doorway| doorway.sea_level as f32);
        far.prepare(&running.card.device, &running.card.queue, (running.card.config.width, running.card.config.height), &self.camera, sea_level);
        let Some(encoder) = &mut work.encoder else { return false };
        let timestamps = wiring::card_slot(PassId::TheFarStart).and_then(|slot| running.card_clock.as_ref().and_then(|clock| clock.compute_pass(slot)));
        far.start(encoder, timestamps);
        work.far_ready = true;
        true
    }

    /// The far eye's layer `k` of the pyramid.
    fn the_far_layer(&mut self, work: &mut FrameWork, k: usize, id: PassId) -> bool {
        let Some(running) = &mut self.running else { return false };
        let (Some(far), Some(encoder)) = (&running.far, &mut work.encoder) else { return true };
        let timestamps = wiring::card_slot(id).and_then(|slot| running.card_clock.as_ref().and_then(|clock| clock.compute_pass(slot)));
        far.layer(k, encoder, timestamps);
        // On a flight, once a second after the last layer: the longest rays counted (step 0), outside the layers' timing.
        if k + 1 == crate::far_eye::LAYERS.len() && self.flight.is_some() && self.tally_at.is_none_or(|at| at.elapsed().as_secs_f32() >= 1.0) && far.tally(&running.card.queue, encoder) {
            self.tally_at = Some(Instant::now());
            work.tallied = true;
        }
        true
    }

    /// The far eye's pyramid onto the screen's picture.
    fn bringing_it_up(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        let (Some(far), Some(texture), Some(encoder)) = (&running.far, &work.texture, &mut work.encoder) else { return true };
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let timestamps = wiring::card_slot(PassId::BringingItUp).and_then(|slot| running.card_clock.as_ref().and_then(|clock| clock.render_pass(slot)));
        far.bring_up(encoder, &view, timestamps);
        work.far_drew = true;
        true
    }

    fn flying(&mut self, now: Instant) -> bool {
        let step = self.frame_clock.step(now);
        if !self.editor.open {
            // With the mouse free, Ctrl is the panels' (Ctrl+S saves a preset, build 219): it doesn't fly down, nor S
            // back while it's held, so a save doesn't move the eye.
            let ctrl = self.held.contains(&KeyCode::ControlLeft) || self.held.contains(&KeyCode::ControlRight);
            if self.captured || !ctrl {
                self.camera.fly(&self.held, step);
            } else {
                let held: HashSet<KeyCode> = self.held.iter().copied().filter(|key| !matches!(key, KeyCode::ControlLeft | KeyCode::KeyS)).collect();
                self.camera.fly(&held, step);
            }
        }
        // A moving mark on a flight: straight ahead along its facing, level, at its speed (step 4).
        if let Some(speed) = self.flight.as_ref().and_then(|flight| flight.speed(&self.marks)) {
            let (sin, cos) = self.camera.facing.to_radians().sin_cos();
            self.camera.position += glam::Vec3::new(cos, sin, 0.0) * speed * step;
        }
        // A nodding mark: the camera pitched slowly up and down (the Oracle's step 0).
        if let Some(pitch) = self.flight.as_ref().and_then(|flight| flight.nod(&self.marks, now)) {
            self.camera.pitch = pitch;
        }
        true
    }

    /// The sheets laid out and drawn where anything they show has changed: the editor over the whole screen when it's
    /// open; else the debug panel if it's shown, and the temperatures over the world.
    fn laying_out_the_panel(&mut self) -> bool {
        let Some(running) = &self.running else { return false };
        let picture = (running.card.config.width, running.card.config.height);
        // Auto from the screen, never from the window [David, 2026-11-20: "when i maximized it last time, it changed
        // scale"]; the editor's own if one's chosen in its menu.
        let auto = gui::auto_scale(running.screen);
        let heat = self.temperatures.as_ref().map(crate::temperatures::Watch::latest);
        if self.editor.open {
            self.scale = self.editor.scale.unwrap_or(auto);
            let full_screen = running.window.fullscreen().is_some();
            let window = crate::editor::Window { filling: full_screen || running.window.is_maximized(), full_screen, fits: fits(running.room) };
            // The window no smaller than the editor's smallest frame, at its scale.
            let smallest = Editor::smallest();
            let least = (smallest.0 * self.scale, smallest.1 * self.scale);
            if self.least != Some(least) {
                running.window.set_min_inner_size(Some(winit::dpi::PhysicalSize::new(least.0, least.1)));
                self.least = Some(least);
            }
            // (The window's size in the sheet's pixels, rounded up: a window that isn't a whole number of them is
            // covered to its last pixel.)
            let size = (picture.0.div_ceil(self.scale), picture.1.div_ceil(self.scale));
            self.editor.lay_out(size, heat.map(crate::temperatures::panes).unwrap_or_default(), window);
        } else {
            self.scale = auto;
            if self.panel.visible {
                self.panel.lay_out();
            }
            // The temperatures, top right, always, flights too [David: "while you are running tests, i am sitting here
            // watching, and id like to know what the temps are at while watching"].
            if let Some(heat) = heat {
                self.readout.lay_out(heat);
            }
        }
        true
    }

    /// Where the debug panel is on the screen (its pixels).
    fn panel_at(&self) -> (i32, i32) {
        (crate::panel::AT.0 * self.scale as i32, crate::panel::AT.1 * self.scale as i32)
    }

    /// The screen's next picture (lost or out of date, after a resize or a minimise: set up again, and tried next frame).
    fn getting_the_picture(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        match running.card.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                work.texture = Some(texture);
                work.encoder = Some(running.card.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the frame") }));
                if let Some(clock) = &mut running.card_clock {
                    clock.begin_frame();
                }
                true
            }
            _ => {
                running.card.set_up_again();
                false
            }
        }
    }

    /// The sheets drawn over the picture (cleared to the sky's colour first where no far eye drew it).
    fn the_panel(&mut self, work: &mut FrameWork) -> bool {
        let panel_at = self.panel_at();
        let Some(running) = &mut self.running else { return false };
        let (Some(texture), Some(encoder)) = (&work.texture, &mut work.encoder) else { return false };
        let screen = (running.card.config.width, running.card.config.height);
        let scale = self.scale;
        let mut sheets = Vec::new();
        if self.editor.open {
            sheets.push(gui::on_card::Placed { sheet: self.editor.sheet(), at: (0, 0), scale });
        } else {
            if self.panel.visible {
                sheets.push(gui::on_card::Placed { sheet: self.panel.sheet(), at: panel_at, scale });
            }
            if self.readout.shows() {
                let sheet = self.readout.sheet();
                sheets.push(gui::on_card::Placed { sheet, at: crate::temperatures::at_top_right(screen, sheet, scale), scale });
            }
        }
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let load = if work.far_drew { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(SKY) };
        let timestamps = wiring::card_slot(PassId::ThePanel).and_then(|slot| running.card_clock.as_ref().and_then(|clock| clock.render_pass(slot)));
        running.sheets.draw(&running.card.device, &running.card.queue, encoder, &view, screen, &sheets, load, timestamps, wiring::pass(PassId::ThePanel).name);
        true
    }

    fn handing_the_card_its_work(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        let Some(mut encoder) = work.encoder.take() else { return false };
        if let Some(clock) = &running.card_clock {
            clock.resolve(&mut encoder);
        }
        work.commands.push(encoder.finish());
        running.card.queue.submit(work.commands.drain(..));
        true
    }

    fn showing_it(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        let Some(texture) = work.texture.take() else { return false };
        running.window.pre_present_notify();
        running.card.queue.present(texture);
        true
    }

    fn measuring(&mut self, work: &mut FrameWork, event_loop: &ActiveEventLoop) -> bool {
        let shown = Instant::now();
        let frame_ms = self.frame_clock.shown(shown);
        // The card's times, read back from frames it has finished (this frame's asked for, never waited on).
        if let Some(running) = &mut self.running {
            work.card_frames = running.card_clock.as_mut().map(|clock| clock.after_submit(&running.card.device)).unwrap_or_default();
        }
        // The longest rays (step 0): this frame's count asked for, and any that has come back.
        let far = self.running.as_ref().and_then(|running| running.far.as_ref());
        if work.tallied && let Some(far) = far {
            far.tally_sent();
        }
        let longest = far.and_then(FarEye::tally_read);
        let eye = self.camera.position;
        let reach = self.flight.as_ref().and(self.running.as_ref()).and_then(|running| running.far.as_ref()).and_then(|far| far.usable_reach([eye.x, eye.y]));
        let heat = self.temperatures.as_ref().map(crate::temperatures::Watch::latest);
        // The memory: on a flight the main memory once a second, for each standing's most; always, the main memory and
        // the card's in the session's log every half minute.
        let second = self.memory_read.is_none_or(|at| shown.duration_since(at).as_secs_f32() >= 1.0);
        let main_memory = second.then(crate::memory::MainMemory::now).flatten();
        if second {
            self.memory_read = Some(shown);
        }
        if self.memory_logged.is_none_or(|at| shown.duration_since(at).as_secs() >= 30) {
            self.memory_logged = Some(shown);
            let card = self.running.as_ref().and_then(|running| running.card.memory()).map_or("the card can't say".to_string(), |(used, held)| format!("{used:.0} MB in use, {held:.0} MB held"));
            let main = main_memory.or_else(crate::memory::MainMemory::now).map_or("Windows can't say".to_string(), |m| m.line());
            log::info!("The memory: the main memory {main}; the card's {card}");
        }
        let Some(flight) = &mut self.flight else { return true };
        flight.record(shown, frame_ms, std::mem::take(&mut work.card_frames));
        flight.followed(shown, eye.to_array(), reach);
        if let Some(heat) = heat {
            flight.heat(shown, heat);
        }
        if let Some(memory) = main_memory {
            flight.main_memory(shown, memory.in_use);
        }
        if let Some(longest) = longest {
            flight.longest(shown, &longest);
        }
        if let Some(near) = self.running.as_ref().and_then(|running| running.near.as_ref()) {
            flight.near(shown, near.counts);
        }
        if flight.finished(shown)
            && let Some(running) = &self.running
        {
            flight.report(&About {
                card: &running.card.description,
                picture: (running.card.config.width, running.card.config.height),
                present: running.card.config.present_mode,
                refresh: running.refresh,
                marks: &self.marks,
                far: running.far.as_ref().map_or("none (the doorway couldn't open the world)".to_string(), |far| format!("{}; {}", far.state(), far.settings())),
                memory: running.card.memory(),
                main_memory: crate::memory::MainMemory::now(),
                fog: running.far.as_ref().is_some_and(|far| far.look.fog),
            });
            event_loop.exit();
        }
        true
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_none() {
            self.open(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let WindowEvent::RedrawRequested = event {
            self.frame(event_loop);
            return;
        }
        // "Hearing the window", the window lane's pass.
        let started = Instant::now();
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(running) = &mut self.running {
                    running.card.resize(size.width, size.height);
                    // The screen it's on; and while it fills the room above the taskbar, that room.
                    running.screen = screen_of(&running.window).unwrap_or(running.screen);
                    if running.window.fullscreen().is_none() && running.window.is_maximized() && size.width > 0 && size.height > 0 {
                        running.room = (size.width, size.height);
                    }
                }
            }
            WindowEvent::Moved(_) => {
                if let Some(running) = &mut self.running {
                    running.screen = screen_of(&running.window).unwrap_or(running.screen);
                }
            }
            WindowEvent::Focused(false) => {
                // Alt-tab: the mouse let go, no key left held, the editor's menus shut.
                self.capture(false);
                self.held.clear();
                self.editor.let_go();
            }
            // The pointer, for the sheets: the editor's menus and list follow it, and its shape shows where a press
            // would size the window.
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = Some((position.x, position.y));
                if self.editor.open {
                    let side = self.editor.moved(gui::on_sheet((position.x, position.y), (0, 0), self.scale));
                    self.shaped(side);
                }
            }
            WindowEvent::CursorLeft { .. } => self.pointer = None,
            // The left button: the editor's when it's open, pressed and let go; else the debug panel's while the mouse
            // is free (let go, the panel hears it always, so nothing it started is left held).
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                if let Some(pointer) = self.pointer {
                    if self.editor.open {
                        let at = gui::on_sheet(pointer, (0, 0), self.scale);
                        let ask = match state {
                            ElementState::Pressed => self.editor.pressed(at, Instant::now()),
                            ElementState::Released => self.editor.released(at),
                        };
                        if let Some(ask) = ask {
                            self.asked(ask, event_loop);
                        }
                    } else if self.panel.visible {
                        let at = gui::on_sheet(pointer, self.panel_at(), self.scale);
                        match state {
                            ElementState::Pressed if !self.captured && self.panel.holds(at) => self.panel.pressed(at),
                            ElementState::Released => self.panel.released(at),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, repeat, .. }, .. } => match state {
                ElementState::Pressed => {
                    self.held.insert(key);
                    if !repeat {
                        self.pressed(key);
                    }
                }
                ElementState::Released => {
                    self.held.remove(&key);
                }
            },
            _ => {}
        }
        self.timed(PassId::HearingTheWindow, started);
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if self.captured
            && let DeviceEvent::MouseMotion { delta: (right, down) } = event
        {
            let started = Instant::now();
            self.camera.look(right as f32, down as f32);
            self.timed(PassId::HearingTheWindow, started);
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(running) = &self.running {
            running.window.request_redraw();
        }
    }
}

/// The size of the screen the window is on (its pixels).
fn screen_of(window: &Window) -> Option<(u32, u32)> {
    window.current_monitor().map(|monitor| (monitor.size().width, monitor.size().height)).filter(|(w, h)| *w > 0 && *h > 0)
}

/// Which of the editor's scales, 1x to 5x, the room above the taskbar holds its smallest frame at [David, 2026-11-20:
/// those that can't be had greyed, "4. yes"].
fn fits(room: (u32, u32)) -> [bool; 5] {
    let (w, h) = Editor::smallest();
    std::array::from_fn(|k| {
        let scale = k as u32 + 1;
        room.0 / scale >= w && room.1 / scale >= h
    })
}

/// The way Windows sizes the window from a side.
fn direction(side: Side) -> ResizeDirection {
    match side {
        Side::North => ResizeDirection::North,
        Side::South => ResizeDirection::South,
        Side::East => ResizeDirection::East,
        Side::West => ResizeDirection::West,
        Side::NorthEast => ResizeDirection::NorthEast,
        Side::NorthWest => ResizeDirection::NorthWest,
        Side::SouthEast => ResizeDirection::SouthEast,
        Side::SouthWest => ResizeDirection::SouthWest,
    }
}
