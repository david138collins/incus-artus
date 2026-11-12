//! The window and the frame.
//!
//! Each frame: the camera flies by the keys held, the panel is laid out, the card clears the picture and draws the
//! panel over it, and the picture is handed to the screen. The card's clock times each pass; the frame's clock times
//! the frame.
//!
//! Keys (old's): F3 the panel, F11 window or full screen, Tab the mouse (`camera.rs` has the flying keys).
//! The window opens borderless and full screen [David, 2026-11-05: the same direct path to the screen as exclusive
//! full screen, and alt-tab is instant]; F11 makes it a maximised window and back.

use crate::camera::Camera;
use crate::card::Card;
use crate::clocks::{CardClock, FrameClock};
use crate::flight::{About, Flight};
use crate::panel::{Panel, Readings};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

/// The frame's passes on the card, in the order they run: one today, the picture cleared and the panel drawn over it.
const PASSES: &[&str] = &["the panel"];

/// The picture's colour where nothing is drawn (old's sky, where it had no world to draw).
const SKY: wgpu::Color = wgpu::Color { r: 0.25, g: 0.34, b: 0.52, a: 1.0 };

pub fn run() {
    let event_loop = EventLoop::new().expect("an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let flight = crate::flight::wanted().then(Flight::new);
    let mut panel = Panel::new();
    panel.visible = flight.is_none();
    let mut runner = Runner { running: None, frame_clock: FrameClock::new(), camera: Camera::new(), panel, held: HashSet::new(), captured: false, flight };
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
    /// The screen's refresh rate, a second.
    refresh: f32,
}

struct Runner {
    running: Option<Running>,
    frame_clock: FrameClock,
    camera: Camera,
    panel: Panel,
    /// The keys held down now.
    held: HashSet<KeyCode>,
    /// Whether the mouse is captured (flying) or free (the panel can be used).
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
        let card_clock = CardClock::new(&card.device, &card.queue, PASSES);
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(egui_ctx.clone(), egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, Some(card.device.limits().max_texture_dimension_2d as usize));
        let egui_renderer = egui_wgpu::Renderer::new(&card.device, card.config.format, egui_wgpu::RendererOptions::default());
        let refresh = window.current_monitor().and_then(|monitor| monitor.refresh_rate_millihertz()).map_or(60.0, |millihertz| millihertz as f32 / 1000.0);
        log::info!("The window: {} x {}, the screen {refresh:.0} a second", card.config.width, card.config.height);
        self.running = Some(Running { window, card, card_clock, egui_ctx, egui_state, egui_renderer, refresh });
    }

    /// The mouse captured for flying, or let go for the panel.
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

    /// A key pressed (not held down repeating): the panel, the screen, the mouse.
    fn pressed(&mut self, key: KeyCode) {
        match key {
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
            KeyCode::Tab => self.capture(!self.captured),
            _ => {}
        }
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let step = self.frame_clock.step(now);
        self.camera.fly(&self.held, step);
        let Some(running) = &mut self.running else { return };

        // The panel, laid out.
        let input = running.egui_state.take_egui_input(&running.window);
        running.egui_ctx.begin_pass(input);
        let readings = Readings {
            card: &running.card.description,
            picture: (running.card.config.width, running.card.config.height),
            present: running.card.config.present_mode,
            refresh: running.refresh,
            frame: self.frame_clock.recent(),
            passes: PASSES,
            card_times: running.card_clock.as_ref().map(CardClock::recent),
            camera: &self.camera,
            captured: self.captured,
        };
        self.panel.show(&running.egui_ctx, &readings);
        let output = running.egui_ctx.end_pass();
        running.egui_state.handle_platform_output(&running.window, output.platform_output);

        // The picture to draw on (lost or out of date, after a resize or a minimise: set up again, and tried next frame).
        let texture = match running.card.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            _ => {
                running.card.set_up_again();
                return;
            }
        };
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let screen = egui_wgpu::ScreenDescriptor { size_in_pixels: [running.card.config.width, running.card.config.height], pixels_per_point: output.pixels_per_point };
        let jobs = running.egui_ctx.tessellate(output.shapes, output.pixels_per_point);
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                running.egui_renderer.update_texture(&running.card.device, &running.card.queue, *id, delta);
            }
        }
        let mut encoder = running.card.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the frame") });
        let mut commands = running.egui_renderer.update_buffers(&running.card.device, &running.card.queue, &mut encoder, &jobs, &screen);
        if let Some(clock) = &mut running.card_clock {
            clock.begin_frame();
        }
        {
            // Pass 0, "the panel": the picture cleared, and the panel drawn over it.
            let attachment = wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(SKY), store: wgpu::StoreOp::Store } };
            let timestamp_writes = running.card_clock.as_ref().and_then(|clock| clock.render_pass(0));
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some(PASSES[0]), color_attachments: &[Some(attachment)], timestamp_writes, ..Default::default() });
            running.egui_renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
        }
        if let Some(clock) = &running.card_clock {
            clock.resolve(&mut encoder);
        }
        commands.push(encoder.finish());
        running.card.queue.submit(commands);
        let card_frames = running.card_clock.as_mut().map(|clock| clock.after_submit(&running.card.device)).unwrap_or_default();
        for id in &output.textures_delta.free {
            running.egui_renderer.free_texture(id);
        }
        running.window.pre_present_notify();
        running.card.queue.present(texture);

        let shown = Instant::now();
        let frame_ms = self.frame_clock.shown(shown);
        if let Some(flight) = &mut self.flight {
            flight.record(shown, frame_ms, card_frames);
            if flight.finished(shown) {
                flight.report(&About {
                    card: &running.card.description,
                    picture: (running.card.config.width, running.card.config.height),
                    present: running.card.config.present_mode,
                    refresh: running.refresh,
                    passes: PASSES,
                    camera: &self.camera,
                });
                event_loop.exit();
            }
        }
    }
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_none() {
            self.open(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // The panel hears the window while the mouse is free, but never Tab (it would move the panel's focus to a
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
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if self.captured
            && let DeviceEvent::MouseMotion { delta: (right, down) } = event
        {
            self.camera.look(right as f32, down as f32);
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(running) = &self.running {
            running.window.request_redraw();
        }
    }
}
