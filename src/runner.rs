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
use crate::editor::Editor;
use crate::flight::{About, Flight};
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
    let flight = crate::flight::wanted().then(Flight::new);
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
        let egui_state = egui_winit::State::new(egui_ctx.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, Some(card.device.limits().max_texture_dimension_2d as usize));
        let egui_renderer = egui_wgpu::Renderer::new(&card.device, card.config.format, egui_wgpu::RendererOptions::default());
        let refresh = window.current_monitor().and_then(|monitor| monitor.refresh_rate_millihertz()).map_or(60.0, |millihertz| millihertz as f32 / 1000.0);
        log::info!("The window: {} x {}, the screen {refresh:.0} a second", card.config.width, card.config.height);
        self.running = Some(Running { window, card, card_clock, egui_ctx, egui_state, egui_renderer, to_free: Vec::new(), refresh });
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

    /// A pass ran for `ms` on the processor.
    fn timed(&mut self, pass: PassId, started: Instant) {
        let ms = started.elapsed().as_secs_f32() * 1000.0;
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
                PassId::Flying => self.flying(started),
                PassId::LayingOutThePanel => self.laying_out_the_panel(&mut work),
                PassId::GettingThePicture => self.getting_the_picture(&mut work),
                PassId::ThePanel => self.the_panel(&mut work),
                PassId::HandingOver => self.handing_over(&mut work),
                PassId::Measuring => self.measuring(&mut work, event_loop),
                PassId::HearingTheWindow => unreachable!("hearing the window is the window's pass, not the frame's"),
            };
            self.timed(pass.id, started);
            if !carry_on {
                return;
            }
        }
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
        };
        self.panel.show(&ctx, &readings);
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
        let attachment = wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(SKY), store: wgpu::StoreOp::Store } };
        let timestamp_writes = wiring::card_slot(PassId::ThePanel).and_then(|slot| running.card_clock.as_ref().and_then(|clock| clock.render_pass(slot)));
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some(wiring::pass(PassId::ThePanel).name), color_attachments: &[Some(attachment)], timestamp_writes, ..Default::default() });
        running.egui_renderer.render(&mut pass.forget_lifetime(), &work.jobs, &screen);
        true
    }

    fn handing_over(&mut self, work: &mut FrameWork) -> bool {
        let Some(running) = &mut self.running else { return false };
        let (Some(texture), Some(mut encoder)) = (work.texture.take(), work.encoder.take()) else { return false };
        if let Some(clock) = &running.card_clock {
            clock.resolve(&mut encoder);
        }
        work.commands.push(encoder.finish());
        running.card.queue.submit(work.commands.drain(..));
        work.card_frames = running.card_clock.as_mut().map(|clock| clock.after_submit(&running.card.device)).unwrap_or_default();
        for id in running.to_free.drain(..) {
            running.egui_renderer.free_texture(&id);
        }
        running.window.pre_present_notify();
        running.card.queue.present(texture);
        true
    }

    fn measuring(&mut self, work: &mut FrameWork, event_loop: &ActiveEventLoop) -> bool {
        let shown = Instant::now();
        let frame_ms = self.frame_clock.shown(shown);
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
                camera: &self.camera,
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
        if !self.captured
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
