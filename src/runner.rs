//! The window and the frame.
//!
//! The frame runs its passes in the order the wiring lists them (`wiring.rs`, the frame's lane), each timed on the
//! processor; the card's clock times the passes on the card. The window's events are the other lane: each one is heard
//! as it comes, and timed too.
//!
//! Keys (old's): F2 the editor, F3 the panel, F11 window or full screen, Tab the mouse (`camera.rs` has the flying keys).
//! The window opens borderless and full screen [David, 2026-11-05: the same direct path to the screen as exclusive
//! full screen, and alt-tab is instant]; F11 makes it a maximised window and back.

use crate::camera::Camera;
use crate::card::Card;
use crate::clocks::{CardClock, FrameClock, PassClock, PassTimes};
use crate::doorway::Doorway;
use crate::editor::Editor;
use crate::far_eye::FarEye;
use crate::flight::{About, Flight};
use crate::marks::{Made, Mark};
use crate::panel::{Panel, Readings};
use crate::wiring::{self, LaneId, PassId};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

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
    let marks = crate::marks::load().unwrap_or_else(|error| {
        log::error!("The marks couldn't be read: {error}");
        Vec::new()
    });
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
    let mut runner = Runner {
        running: None,
        frame_clock: FrameClock::new(),
        pass_clock: PassClock::new(),
        camera: Camera::new(),
        panel,
        editor: Editor::new(),
        held: HashSet::new(),
        captured: false,
        doorway,
        marks,
        first_mark: first_mark.unwrap_or(0),
        at_first_mark: false,
        flight,
    };
    event_loop.run_app(&mut runner).expect("the game runs");
    log::info!("Quitting");
}

/// What exists once the window is open.
struct Running {
    window: Arc<Window>,
    card: Card,
    card_clock: Option<CardClock>,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    /// The panels' textures to let go of once a frame has been handed over.
    to_free: Vec<egui::TextureId>,
    /// The screen's refresh rate, a second.
    refresh: f32,
    /// The far eye (none if the doorway couldn't open the world).
    far: Option<FarEye>,
    /// The card's memory, as last asked (once a second while a panel shows it), and when.
    memory: Option<(f64, f64)>,
    memory_at: Option<Instant>,
}

/// What a frame's passes hand on to the passes after them.
#[derive(Default)]
struct FrameWork {
    /// The panels cut into shapes for the card, and their size on the screen.
    jobs: Vec<egui::ClippedPrimitive>,
    pixels_per_point: f32,
    /// The screen's picture to draw on, and the card's work for it.
    texture: Option<wgpu::SurfaceTexture>,
    encoder: Option<wgpu::CommandEncoder>,
    commands: Vec<wgpu::CommandBuffer>,
    /// The card's times that came back this frame.
    card_frames: Vec<Vec<f32>>,
    /// Whether the far eye drew the picture (the panels then go over it; else over the sky's colour).
    far_drew: bool,
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
    flight: Option<Flight>,
}

impl Runner {
    fn open(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = Window::default_attributes()
            .with_title("Incus Artus")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 960))
            .with_maximized(true)
            .with_fullscreen(Some(Fullscreen::Borderless(None)));
        let window = Arc::new(event_loop.create_window(attributes).expect("a window"));
        let card = Card::new(window.clone(), self.flight.is_some());
        let card_clock = CardClock::new(&card.device, &card.queue, wiring::card_passes().count());
        let egui_ctx = egui::Context::default();
        // No selecting the panels' text by dragging [David, 2026-11-12: "i can start highlighting text in the panel, and
        // it never lets go"].
        egui_ctx.all_styles_mut(|style| style.interaction.selectable_labels = false);
        let egui_state = egui_winit::State::new(egui_ctx.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, Some(card.device.limits().max_texture_dimension_2d as usize));
        let egui_renderer = egui_wgpu::Renderer::new(&card.device, card.config.format, egui_wgpu::RendererOptions::default());
        let refresh = window.current_monitor().and_then(|monitor| monitor.refresh_rate_millihertz()).map_or(60.0, |millihertz| millihertz as f32 / 1000.0);
        log::info!("The window: {} x {}, the screen {refresh:.0} a second", card.config.width, card.config.height);
        // The far eye, its cells summed on the loading lane nearest the first mark first, on all but two of the
        // processor's threads (the frame and the window keep theirs).
        let far = self.doorway.as_ref().map(|doorway| {
            let store = crate::bricks::Store::new(&card.device, &card.queue, crate::far::regions_of(doorway), doorway.height());
            let near = self.marks.get(self.first_mark).map_or([0.0, 0.0], |mark| mark.at);
            let threads = std::thread::available_parallelism().map_or(2, |n| n.get()).saturating_sub(2).max(1);
            let lane = crate::far::Lane::start(crate::doorway::test_world(), near, threads).map_err(|error| log::error!("The far cells couldn't be summed: {error}")).ok();
            FarEye::new(&card.device, card.config.format, store, lane)
        });
        self.running = Some(Running { window, card, card_clock, egui_ctx, egui_state, egui_renderer, to_free: Vec::new(), refresh, far, memory: None, memory_at: None });
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

    /// A key pressed (not held down repeating): the editor, the panel, the screen, the mouse.
    fn pressed(&mut self, key: KeyCode) {
        match key {
            KeyCode::F2 => {
                self.editor.open = !self.editor.open;
                if self.editor.open {
                    self.capture(false);
                    self.held.clear();
                }
            }
            KeyCode::F3 => self.panel.visible = !self.panel.visible,
            KeyCode::F11 => {
                if let Some(running) = &self.running {
                    if running.window.fullscreen().is_some() {
                        running.window.set_fullscreen(None);
                        running.window.set_maximized(true);
                    } else {
                        running.window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
                }
            }
            KeyCode::Tab if !self.editor.open => self.capture(!self.captured),
            _ => {}
        }
    }

    /// Each pass's recent times, by its id.
    fn pass_times(&self) -> (Vec<PassTimes>, Option<crate::clocks::Spread>) {
        let card = self.running.as_ref().and_then(|running| running.card_clock.as_ref()).map(CardClock::recent);
        let times = wiring::PASSES
            .iter()
            .map(|pass| PassTimes { processor: self.pass_clock.recent(pass.id), card: wiring::card_slot(pass.id).and_then(|slot| card.as_ref().and_then(|card| card.get(slot).copied())) })
            .collect();
        (times, card.and_then(|card| card.last().copied()))
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
                PassId::LayingOutThePanel => self.laying_out_the_panel(&mut work),
                PassId::GettingThePicture => self.getting_the_picture(&mut work),
                PassId::TheFarStart => self.the_far_start(&mut work),
                PassId::TheFar8 => self.the_far_layer(&mut work, 0, PassId::TheFar8),
                PassId::TheFar4 => self.the_far_layer(&mut work, 1, PassId::TheFar4),
                PassId::TheFar2 => self.the_far_layer(&mut work, 2, PassId::TheFar2),
                PassId::BringingItUp => self.bringing_it_up(&mut work),
                PassId::ThePanel => self.the_panel(&mut work),
                PassId::HandingTheCardItsWork => self.handing_the_card_its_work(&mut work),
                PassId::ShowingIt => self.showing_it(&mut work),
                PassId::Measuring => self.measuring(&mut work, event_loop),
                PassId::HearingTheWindow | PassId::TheDoorway | PassId::SummingTheFarCells => unreachable!("{} isn't one of the frame's passes", pass.name),
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
            None if !self.at_first_mark => Some((self.first_mark, None)),
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
        let Some((k, count)) = target else { return true };
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
        if let Some(running) = &mut self.running
            && let Some(far) = &mut running.far
        {
            far.put(&running.card.device, &running.card.queue);
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
        true
    }

    /// The far eye's layer `k` of the pyramid.
    fn the_far_layer(&mut self, work: &mut FrameWork, k: usize, id: PassId) -> bool {
        let Some(running) = &mut self.running else { return false };
        let (Some(far), Some(encoder)) = (&running.far, &mut work.encoder) else { return true };
        let timestamps = wiring::card_slot(id).and_then(|slot| running.card_clock.as_ref().and_then(|clock| clock.compute_pass(slot)));
        far.layer(k, encoder, timestamps);
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
            self.camera.fly(&self.held, step);
        }
        true
    }

    fn laying_out_the_panel(&mut self, work: &mut FrameWork) -> bool {
        // The figures are worked out only when a panel shows them (not on a flight, where they'd be measured too).
        let (times, card_whole, frame) = if self.panel.visible || self.editor.open {
            let (times, card_whole) = self.pass_times();
            (times, card_whole, self.frame_clock.recent())
        } else {
            (vec![PassTimes::default(); wiring::PASSES.len()], None, crate::clocks::Spread::default())
        };
        let Some(running) = &mut self.running else { return false };
        if (self.panel.visible || self.editor.open) && running.memory_at.is_none_or(|at| at.elapsed().as_secs_f32() >= 1.0) {
            running.memory = running.card.memory();
            running.memory_at = Some(Instant::now());
        }
        let ctx = running.egui_ctx.clone();
        ctx.begin_pass(running.egui_state.take_egui_input(&running.window));
        self.editor.show(&ctx, &times);
        let readings = Readings {
            card: &running.card.description,
            picture: (running.card.config.width, running.card.config.height),
            present: running.card.config.present_mode,
            refresh: running.refresh,
            frame,
            passes: &times,
            card_whole,
            camera: &self.camera,
            captured: self.captured,
            memory: running.memory,
        };
        self.panel.show(&ctx, &readings, running.far.as_mut().map(|far| &mut far.look));
        let output = ctx.end_pass();
        running.egui_state.handle_platform_output(&running.window, output.platform_output);
        work.jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
        work.pixels_per_point = output.pixels_per_point;
        // The panels' textures go to the card now, whether or not this frame gets a picture to draw on, so none is lost.
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                running.egui_renderer.update_texture(&running.card.device, &running.card.queue, *id, delta);
            }
        }
        running.to_free.extend(output.textures_delta.free.iter().copied());
        true
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

    fn the_panel(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        let (Some(texture), Some(encoder)) = (&work.texture, &mut work.encoder) else { return false };
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let screen = egui_wgpu::ScreenDescriptor { size_in_pixels: [running.card.config.width, running.card.config.height], pixels_per_point: work.pixels_per_point };
        work.commands = running.egui_renderer.update_buffers(&running.card.device, &running.card.queue, encoder, &work.jobs, &screen);
        let load = if work.far_drew { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(SKY) };
        let attachment = wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load, store: wgpu::StoreOp::Store } };
        let timestamp_writes = wiring::card_slot(PassId::ThePanel).and_then(|slot| running.card_clock.as_ref().and_then(|clock| clock.render_pass(slot)));
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some(wiring::pass(PassId::ThePanel).name), color_attachments: &[Some(attachment)], timestamp_writes, ..Default::default() });
        running.egui_renderer.render(&mut pass.forget_lifetime(), &work.jobs, &screen);
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
        for id in running.to_free.drain(..) {
            running.egui_renderer.free_texture(&id);
        }
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
        let Some(flight) = &mut self.flight else { return true };
        flight.record(shown, frame_ms, std::mem::take(&mut work.card_frames));
        if flight.finished(shown)
            && let Some(running) = &self.running
        {
            flight.report(&About {
                card: &running.card.description,
                picture: (running.card.config.width, running.card.config.height),
                present: running.card.config.present_mode,
                refresh: running.refresh,
                marks: &self.marks,
                far: running.far.as_ref().map_or("none (the doorway couldn't open the world)".to_string(), FarEye::state),
                memory: running.card.memory(),
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
        // The panels hear the window while the mouse is free, but never Tab (it would move the panel's focus to a
        // button, which Space would then press while flying).
        let is_tab = matches!(&event, WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(KeyCode::Tab), .. }, .. });
        // A mouse button let go always reaches them, even while flying, so nothing they started (a drag) is left held.
        let let_go = matches!(&event, WindowEvent::MouseInput { state: ElementState::Released, .. });
        if (!self.captured || let_go)
            && !is_tab
            && let Some(running) = &mut self.running
        {
            let _ = running.egui_state.on_window_event(&running.window, &event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(running) = &mut self.running {
                    running.card.resize(size.width, size.height);
                }
            }
            WindowEvent::Focused(false) => {
                // Alt-tab: the mouse let go, and no key left held.
                self.capture(false);
                self.held.clear();
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
