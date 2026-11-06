//! The runner — the frame lane: the window, the card, and the frame loop (the regroup,
//! docs/regroup.md, phase 0). It replaces Bevy's window, renderer and app loop.
//!
//! Each frame, in order, each step timed for the slow-frame log (`perf`):
//! 1. the window's input goes in as the same messages Bevy's own window code sent (keys, mouse
//!    buttons, wheel, motion), so the game's input code didn't change;
//! 2. **the game's update**: Bevy's schedules (its ECS core is kept for now, `docs/regroup.md`), with
//!    the simulation's fixed ticks inside it;
//! 3. **the panel**: egui's frame, the `EguiPrimaryContextPass` systems;
//! 4. **the card**: get the next picture, clear it to the sky, draw the panel, hand it over, show it;
//! 5. **the frame limit** (`FrameLimit`, `pace`): the wait for the next frame's beat, by a clock of
//!    our own.
//!
//! **While the game is in front** it holds the PC's attention (`hardware::Attention`). **When it isn't**
//! (another window clicked, alt-tab), it freezes in place [David 2026-11-05: "i'd rather have the game
//! just freeze (deliberately) and stay there"]: one last frame with a grey veil and "Paused", then
//! nothing is drawn, the world stops, and the attention is given back, until it's clicked again.
//! The window is borderless fullscreen; F11 switches it to a window and back.
//!
//! The window's state the game reads (its size, whether it has the focus) is the `WindowState`
//! resource; what the game asks of the window (the mouse captured, a size) it sets there too.

#[cfg(feature = "dev")]
mod ballast;
pub mod hardware;
mod keys;

use crate::perf::Timed;
use crate::ecs::app::AppExit;
use crate::ecs::input::ButtonState;
use crate::ecs::input::keyboard::{Key, KeyboardInput, NativeKey};
use crate::ecs::input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
use crate::ecs::input::touch::TouchPhase;
use crate::ecs::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowId};

/// The window as the game sees it, and what the game asks of it.
#[derive(Resource, Clone, Debug)]
pub struct WindowState {
    /// The drawable size, in pixels.
    pub size: (u32, u32),
    /// Pixels per point (the screen's scaling).
    pub scale: f32,
    pub focused: bool,
    /// Asked by the game: the mouse captured (hidden and held in place, for looking round).
    pub capture_mouse: bool,
    /// Asked by the game: a new size, in pixels (the soak test's `INCUS_SOAK_WINDOW`).
    pub resize_to: Option<(u32, u32)>,
    /// Asked by the game: minimised (true) or back (false) (the self-test).
    pub minimize: Option<bool>,
    /// The screen's refresh rate (refreshes a second).
    pub refresh: f32,
    /// Set by the game: freeze when the game isn't in front (the setting; never while it's measuring
    /// itself).
    pub freeze_when_away: bool,
    /// The first frame after a freeze: the simulation's clock leaves out the time away (`time/`).
    pub resumed: bool,
}

impl WindowState {
    /// Whether the game is frozen, or freezing, because it isn't in front.
    pub fn away(&self) -> bool {
        !self.focused && self.freeze_when_away
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self { size: (1280, 960), scale: 1.0, focused: true, capture_mouse: false, resize_to: None, minimize: None, refresh: 60.0, freeze_when_away: true, resumed: false }
    }
}

/// How many frames a second the game makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameLimit {
    /// As fast as it can, vsync off: for measuring how much room is left.
    Uncapped,
    /// This many a second, on a steady beat of our own (vsync on): just under the screen's top rate
    /// for a screen with a variable refresh rate (G-SYNC, FreeSync), or any rate chosen.
    Steady(f32),
    /// The screen's own rate, paced by vsync alone: for a screen without a variable refresh rate.
    ScreenRate,
}

/// The frame limit the game wants, set by the game from its settings each frame (`look.rs`).
#[derive(Resource, Clone, Copy, Debug)]
pub struct FrameCap(pub FrameLimit);

impl Default for FrameCap {
    fn default() -> Self {
        Self(FrameLimit::Steady(120.0))
    }
}

/// Set by the test window (`soak.rs`, `INCUS_TEST_WINDOW`): the frame limit it's measuring, over the
/// settings' one, which is left as it was.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TestCap(pub Option<FrameLimit>);

/// The ballast wanted on the card each frame, in milliseconds at full clock (developer builds only:
/// `runner/ballast.rs`; `INCUS_BALLAST`, or the test window). 0: none.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct BallastTarget(pub f32);

/// A file dropped on the window (the World Gen suite takes pictures and block textures this way).
#[derive(Message, Clone, Debug)]
pub struct FileDropped(pub std::path::PathBuf);

/// Asked by the game: the next frame saved as a PNG here (F12, the self-test).
#[derive(Resource, Clone, Debug, Default)]
pub struct ScreenshotRequest(pub Option<std::path::PathBuf>);

/// The colour the picture is cleared to (the sky's, for now: nothing else is drawn yet).
const SKY: wgpu::Color = wgpu::Color { r: 0.25, g: 0.34, b: 0.52, a: 1.0 };

/// Runs the game until it asks to quit or the window is closed.
pub fn run(mut app: App) {
    app.init_resource::<WindowState>().init_resource::<FrameCap>().init_resource::<TestCap>().init_resource::<BallastTarget>().init_resource::<ScreenshotRequest>().add_message::<FileDropped>();
    #[cfg(feature = "dev")]
    app.insert_resource(BallastTarget(ballast::from_start()));
    // Plugins finish their set-up (Bevy's `App::run` did this).
    while app.plugins_state() == crate::ecs::app::PluginsState::Adding {
        crate::ecs::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    let event_loop = EventLoop::new().expect("an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let alarm = hardware::Alarm::new();
    info!("The frame's alarm: {}", if alarm.precise() { "Windows' high-resolution timer" } else { "the plain sleep" });
    let mut runner = Runner { app, gpu: None, last_frame: None, beat: None, limit: None, alarm, attention: hardware::Attention::default(), frozen: false };
    if let Err(error) = event_loop.run_app(&mut runner) {
        error!("The window's loop stopped: {error}");
    }
}

/// The card's device and queue, shared with the loading lane (the world's pack and pipeline, phase 2.0):
/// set once, as the window opens.
static CARD: std::sync::OnceLock<(wgpu::Device, wgpu::Queue)> = std::sync::OnceLock::new();

/// The card, once the window is open.
pub fn card() -> Option<&'static (wgpu::Device, wgpu::Queue)> {
    CARD.get()
}

struct Gpu {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    /// Whether the mouse is captured now (so it's only changed when the game asks for a change).
    captured: bool,
    /// Made-up work for the card (developer builds, when asked for).
    #[cfg(feature = "dev")]
    ballast: Option<ballast::Ballast>,
}

struct Runner {
    app: App,
    gpu: Option<Gpu>,
    /// When the last frame was shown (its length is measured from one to the next).
    last_frame: Option<Instant>,
    /// The beat the frame now being made ends on, under a steady limit (see `pace`).
    beat: Option<Instant>,
    /// The frame limit the surface is set up for.
    limit: Option<FrameLimit>,
    alarm: hardware::Alarm,
    attention: hardware::Attention,
    /// Frozen because the game isn't in front: nothing is drawn till it's clicked again.
    frozen: bool,
}

/// Whether the ballast was asked for (developer builds only).
fn wanted_ballast() -> bool {
    #[cfg(feature = "dev")]
    return ballast::wanted();
    #[cfg(not(feature = "dev"))]
    false
}

/// The screen's refresh rate (refreshes a second), 60 if it can't be had.
fn refresh_of(window: &Window) -> f32 {
    window.current_monitor().and_then(|monitor| monitor.refresh_rate_millihertz()).map_or(60.0, |millihertz| millihertz as f32 / 1000.0)
}

/// The Windows SDK's `dxcompiler.dll`, if it is installed (with the `dxil.dll` beside it): DXC
/// compiles shaders in a fraction of the old compiler's time [found 2026-11-04: some 50 s for the
/// old march with FXC]. `WGPU_DX12_COMPILER` still chooses, for testing.
fn windows_sdk_dxc() -> Option<String> {
    let folder = std::path::Path::new(r"C:\Program Files (x86)\Windows Kits\10\Redist\D3D\x64");
    let dxc = folder.join("dxcompiler.dll");
    (dxc.is_file() && folder.join("dxil.dll").is_file()).then(|| dxc.to_string_lossy().into_owned())
}

impl Runner {
    fn start(&mut self, event_loop: &ActiveEventLoop) {
        // Borderless fullscreen [David 2026-11-05]: the same direct path to the screen as the old
        // exclusive fullscreen, and alt-tab is instant. F11 makes it a (maximised) window and back.
        let attributes = Window::default_attributes()
            .with_title("Incus Artus")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 960))
            .with_maximized(true)
            .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        let window = Arc::new(event_loop.create_window(attributes).expect("a window"));
        // On Windows, DirectX 12 [2026-10-24: Vulkan sometimes stalled handing frames to the screen on
        // this kind of laptop, about 6 frames a second against 240]; `WGPU_BACKEND` still chooses.
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = if cfg!(windows) && std::env::var("WGPU_BACKEND").is_err() { wgpu::Backends::DX12 } else { wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY) };
        if std::env::var("WGPU_DX12_COMPILER").is_err()
            && let Some(dxc) = windows_sdk_dxc()
        {
            descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::DynamicDxc { dxc_path: dxc };
        }
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance.create_surface(window.clone()).expect("a surface to draw on");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, compatible_surface: Some(&surface), ..Default::default() })).expect("a graphics card");
        let info = adapter.get_info();
        info!("Graphics card: {} ({:?}, driver {} {})", info.name, info.backend, info.driver, info.driver_info);
        // The card's own timestamps, for the ballast (developer builds, when it's asked for).
        let timed = cfg!(feature = "dev") && wanted_ballast();
        let required_features = if timed { adapter.features() & wgpu::Features::TIMESTAMP_QUERY } else { wgpu::Features::empty() };
        let required_limits = crate::whole::shader::big_buffers(&adapter);
        info!("The card's largest buffer: {} MB", required_limits.max_storage_buffer_binding_size.min(required_limits.max_buffer_size) >> 20);
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("Incus Artus"), required_features, required_limits, ..Default::default() })).expect("the card's device");
        let _ = CARD.set((device.clone(), queue.clone()));
        let size = window.inner_size();
        let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).expect("the card can draw to this window");
        // The panel's colours are worked out as sRGB; a plain (non-sRGB) surface shows them as made.
        config.format = config.format.remove_srgb_suffix();
        config.view_formats = vec![config.format];
        config.present_mode = wgpu::PresentMode::AutoVsync;
        // Copied from for screenshots, where the card allows it.
        if surface.get_capabilities(&adapter).usages.contains(wgpu::TextureUsages::COPY_SRC) {
            config.usage |= wgpu::TextureUsages::COPY_SRC;
        }
        surface.configure(&device, &config);
        let ctx = self.app.world().resource::<crate::ui::EguiCtx>().0.clone();
        let egui_state = egui_winit::State::new(ctx, egui::ViewportId::ROOT, &window, Some(window.scale_factor() as f32), None, Some(device.limits().max_texture_dimension_2d as usize));
        let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, egui_wgpu::RendererOptions::default());
        {
            let mut state = self.app.world_mut().resource_mut::<WindowState>();
            state.size = (config.width, config.height);
            state.scale = window.scale_factor() as f32;
        }
        #[cfg(feature = "dev")]
        let ballast = if timed {
            let made = ballast::Ballast::new(&device, &queue);
            if made.is_none() {
                warn!("Ballast: the card can't time its own work, so there's none");
            }
            made
        } else {
            None
        };
        self.gpu = Some(Gpu {
            window,
            surface,
            device,
            queue,
            config,
            egui_state,
            egui_renderer,
            captured: false,
            #[cfg(feature = "dev")]
            ballast,
        });
        self.watch_screen();
    }

    /// The screen's refresh rate (again, when the window's moved: it may be on another screen now).
    fn watch_screen(&mut self) {
        let Some(gpu) = &self.gpu else { return };
        let refresh = refresh_of(&gpu.window);
        let mut state = self.app.world_mut().resource_mut::<WindowState>();
        if state.refresh != refresh {
            state.refresh = refresh;
            info!("The screen: {refresh:.0} refreshes a second");
        }
    }

    /// F11: borderless fullscreen, or a window.
    fn toggle_fullscreen(&mut self) {
        let Some(gpu) = &self.gpu else { return };
        let fullscreen = if gpu.window.fullscreen().is_some() { None } else { Some(winit::window::Fullscreen::Borderless(None)) };
        gpu.window.set_fullscreen(fullscreen);
    }

    /// The window has lost the focus: any mouse button the panel thinks is held is let go, since its
    /// release goes to the other window [David 2026-11-05: pressed on the panel's text as a screenshot
    /// took the focus, and the highlight followed the cursor from then on].
    fn let_go_of_the_mouse(&mut self) {
        let Some(gpu) = &mut self.gpu else { return };
        let pos = gpu.egui_state.egui_ctx().input(|input| input.pointer.latest_pos()).unwrap_or_default();
        let input = gpu.egui_state.egui_input_mut();
        for button in [egui::PointerButton::Primary, egui::PointerButton::Secondary, egui::PointerButton::Middle] {
            input.events.push(egui::Event::PointerButton { pos, button, pressed: false, modifiers: egui::Modifiers::default() });
        }
        input.events.push(egui::Event::PointerGone);
        // The game's own idea of the buttons, too.
        for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            self.send(MouseButtonInput { button, state: ButtonState::Released, window: Entity::PLACEHOLDER });
        }
    }

    /// The game's back in front after a freeze: drawing starts again.
    fn unfreeze(&mut self, event_loop: &ActiveEventLoop) {
        if !self.frozen {
            return;
        }
        self.frozen = false;
        crate::diagnostics::away(false);
        self.beat = None;
        self.app.world_mut().resource_mut::<WindowState>().resumed = true;
        event_loop.set_control_flow(ControlFlow::Poll);
        if let Some(gpu) = &self.gpu {
            gpu.window.request_redraw();
        }
        info!("Back in front: the game goes on");
    }

    fn resize(&mut self, width: u32, height: u32) {
        let Some(gpu) = &mut self.gpu else { return };
        if width == 0 || height == 0 {
            return;
        }
        (gpu.config.width, gpu.config.height) = (width, height);
        gpu.surface.configure(&gpu.device, &gpu.config);
        self.app.world_mut().resource_mut::<WindowState>().size = (width, height);
    }

    /// Sends a message into the game, as Bevy's window code did.
    fn send<M: Message>(&mut self, message: M) {
        self.app.world_mut().write_message(message);
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        if self.frozen {
            return;
        }
        // Not in front: this is the last frame till it's clicked again (the veil below). In front:
        // the PC's attention is the game's.
        let away = self.app.world().resource::<WindowState>().away();
        if away {
            self.attention.give_back();
        } else {
            self.attention.take();
        }
        let Some(gpu) = &mut self.gpu else { return };
        let ctx = self.app.world().resource::<crate::ui::EguiCtx>().0.clone();
        // The panel's frame opens before the game's update, so the game's systems can draw in it.
        ctx.begin_pass(gpu.egui_state.take_egui_input(&gpu.window));
        {
            let _timed = Timed::new("the game's update");
            self.app.update();
        }
        self.app.world_mut().resource_mut::<WindowState>().resumed = false;
        if let Some(exit) = self.app.should_exit() {
            info!("Quitting ({exit:?})");
            event_loop.exit();
            return;
        }
        {
            let _timed = Timed::new("the panel");
            self.app.world_mut().run_schedule(crate::ui::EguiPrimaryContextPass);
        }
        if away {
            draw_the_veil(&ctx);
        }
        let Some(gpu) = &mut self.gpu else { return };
        let output = ctx.end_pass();
        gpu.egui_state.handle_platform_output(&gpu.window, output.platform_output);
        *self.app.world_mut().resource_mut::<crate::ui::EguiWantsInput>() = crate::ui::EguiWantsInput { keyboard: ctx.egui_wants_keyboard_input() };

        // What the game asks of the window.
        #[cfg(feature = "dev")]
        let ballast_target = self.app.world().resource::<BallastTarget>().0;
        let (capture, resize_to, minimize, limit, screenshot) = {
            let world = self.app.world_mut();
            let limit = world.resource::<TestCap>().0.unwrap_or(world.resource::<FrameCap>().0);
            let screenshot = world.resource_mut::<ScreenshotRequest>().0.take();
            let mut state = world.resource_mut::<WindowState>();
            (state.capture_mouse, state.resize_to.take(), state.minimize.take(), limit, screenshot)
        };
        if let Some(minimized) = minimize {
            gpu.window.set_minimized(minimized);
        }
        if capture != gpu.captured {
            gpu.captured = capture;
            if capture {
                let locked = gpu.window.set_cursor_grab(CursorGrabMode::Locked).or_else(|_| gpu.window.set_cursor_grab(CursorGrabMode::Confined));
                if let Err(error) = locked {
                    warn!("The mouse couldn't be captured: {error}");
                }
            } else {
                let _ = gpu.window.set_cursor_grab(CursorGrabMode::None);
            }
            gpu.window.set_cursor_visible(!capture);
        }
        if let Some((width, height)) = resize_to {
            let _ = gpu.window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
        }
        if self.limit != Some(limit) {
            self.limit = Some(limit);
            gpu.config.present_mode = if limit == FrameLimit::Uncapped { wgpu::PresentMode::AutoNoVsync } else { wgpu::PresentMode::AutoVsync };
            gpu.surface.configure(&gpu.device, &gpu.config);
        }

        {
            // The card's steps are timed one by one, so a stall in handing over says which it was in.
            let fetched = {
                let _timed = Timed::new("getting the next picture");
                gpu.surface.get_current_texture()
            };
            let texture = match fetched {
                wgpu::CurrentSurfaceTexture::Success(texture) | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                _ => {
                    // Lost or out of date (a resize, a minimise): set up again, and try next frame.
                    gpu.surface.configure(&gpu.device, &gpu.config);
                    return;
                }
            };
            let drawing = Timed::new("drawing the panel for the card");
            let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let screen = egui_wgpu::ScreenDescriptor { size_in_pixels: [gpu.config.width, gpu.config.height], pixels_per_point: output.pixels_per_point };
            let jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
            for (id, deltas) in &output.textures_delta.set {
                for delta in deltas {
                    gpu.egui_renderer.update_texture(&gpu.device, &gpu.queue, *id, delta);
                }
            }
            let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the frame") });
            #[cfg(feature = "dev")]
            if let Some(ballast) = &mut gpu.ballast {
                ballast.encode(&gpu.queue, &mut encoder, ballast_target);
            }
            let mut commands = gpu.egui_renderer.update_buffers(&gpu.device, &gpu.queue, &mut encoder, &jobs, &screen);
            {
                let attachment = wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(SKY), store: wgpu::StoreOp::Store } };
                #[cfg(feature = "dev")]
                let timestamp_writes = gpu.ballast.as_ref().and_then(|ballast| ballast.panel_timestamps());
                #[cfg(not(feature = "dev"))]
                let timestamp_writes = None;
                let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the panel"), color_attachments: &[Some(attachment)], timestamp_writes, ..Default::default() });
                gpu.egui_renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
            }
            let shot = screenshot.filter(|_| gpu.config.usage.contains(wgpu::TextureUsages::COPY_SRC)).map(|path| (path, copy_out(&gpu.device, &mut encoder, &texture.texture)));
            #[cfg(feature = "dev")]
            if let Some(ballast) = &gpu.ballast {
                ballast.resolve(&mut encoder);
            }
            commands.push(encoder.finish());
            drop(drawing);
            {
                let _timed = Timed::new("handing the card its work");
                gpu.queue.submit(commands);
            }
            #[cfg(feature = "dev")]
            if let Some(ballast) = &mut gpu.ballast {
                ballast.after_submit(&gpu.device);
            }
            if let Some((path, (buffer, padded))) = shot {
                // (F12 and the self-test only: the frame waits for the copy.)
                save_screenshot(&gpu.device, buffer, padded, (gpu.config.width, gpu.config.height), gpu.config.format, path);
            }
            for id in &output.textures_delta.free {
                gpu.egui_renderer.free_texture(id);
            }
            let _shown = Timed::new("showing it");
            gpu.window.pre_present_notify();
            gpu.queue.present(texture);
        }

        if away {
            // Frozen in place: the veiled frame stays on the screen, the event loop sleeps till
            // something happens, and nothing more is drawn till the game is clicked again.
            self.frozen = true;
            crate::diagnostics::away(true);
            self.last_frame = None;
            event_loop.set_control_flow(ControlFlow::Wait);
            info!("Not in front: the game is frozen till it's clicked again");
            return;
        }
        let refresh = self.app.world().resource::<WindowState>().refresh;
        self.pace(limit);
        let now = Instant::now();
        let ms = self.last_frame.map_or(0.0, |last| now.duration_since(last).as_secs_f32() * 1000.0);
        self.last_frame = Some(now);
        let capped = match limit {
            FrameLimit::Uncapped => None,
            FrameLimit::Steady(fps) => Some(1000.0 / fps.max(1.0)),
            FrameLimit::ScreenRate => Some(1000.0 / refresh.max(1.0)),
        };
        if cfg!(feature = "dev") {
            crate::perf::frame_core(hardware::current_core());
        }
        crate::perf::frame_done(self.app.world_mut(), ms, capped);
    }

    /// Under a steady limit, waits for the next frame's beat, by a clock of our own: the processor's
    /// own counter, which doesn't waver [David 2026-11-05: "what other things could we use as a
    /// metronome"]. Each beat is the last one plus a frame's length, so a late wake-up is never carried
    /// into the next frame and the tempo can't drift; a frame that ran past its beat starts the beats
    /// again from now (a slow frame isn't made up for by rushing the next ones). The alarm is Windows'
    /// high-resolution timer, and how late it wakes is measured every time (`perf::woke`). On a screen
    /// with a variable refresh rate the screen shows each frame the moment it's ready, so the beat is
    /// what's seen. The simulation's clock is untouched by any of it: its time is real time (`time/`).
    fn pace(&mut self, limit: FrameLimit) {
        let FrameLimit::Steady(fps) = limit else {
            self.beat = None;
            return;
        };
        let length = Duration::from_secs_f64(1.0 / fps.max(1.0) as f64);
        let now = Instant::now();
        let beat = match self.beat {
            Some(last) if last + length > now => last + length,
            _ => now,
        };
        if beat > now {
            {
                let _timed = Timed::new("waiting for the beat");
                self.alarm.sleep_until(beat);
            }
            if cfg!(feature = "dev") {
                crate::perf::woke(Instant::now().saturating_duration_since(beat).as_secs_f32() * 1000.0);
            }
        }
        self.beat = Some(beat);
    }
}

/// The veil over the last frame before a freeze: grey over everything, and "Paused".
fn draw_the_veil(ctx: &egui::Context) {
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("the veil")));
    let screen = ctx.content_rect();
    painter.rect_filled(screen, 0.0, egui::Color32::from_black_alpha(150));
    painter.text(screen.center(), egui::Align2::CENTER_CENTER, "Paused", egui::FontId::proportional(32.0), egui::Color32::from_gray(225));
    painter.text(screen.center() + egui::vec2(0.0, 30.0), egui::Align2::CENTER_CENTER, "click the game to go on", egui::FontId::proportional(15.0), egui::Color32::from_gray(170));
}

/// A copy of the picture into a buffer the processor can read (rows padded to the card's 256 bytes).
fn copy_out(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::Texture) -> (wgpu::Buffer, u32) {
    let (width, height) = (texture.width(), texture.height());
    let padded = (width * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("a screenshot"), size: (padded * height) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let layout = wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: None };
    encoder.copy_texture_to_buffer(texture.as_image_copy(), wgpu::TexelCopyBufferInfo { buffer: &buffer, layout }, wgpu::Extent3d { width, height, depth_or_array_layers: 1 });
    (buffer, padded)
}

/// Waits for the copy (a screenshot is rare, so the frame may wait for it), then writes the PNG on a
/// thread of its own.
fn save_screenshot(device: &wgpu::Device, buffer: wgpu::Buffer, padded: u32, (width, height): (u32, u32), format: wgpu::TextureFormat, path: std::path::PathBuf) {
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    if let Err(error) = device.poll(wgpu::PollType::wait_indefinitely()) {
        warn!("A screenshot couldn't be read back: {error}");
        return;
    }
    let bgra = matches!(format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    {
        let data = match buffer.slice(..).get_mapped_range() {
            Ok(data) => data,
            Err(error) => {
                warn!("A screenshot couldn't be read back: {error:?}");
                return;
            }
        };
        for row in 0..height {
            let start = (row * padded) as usize;
            for pixel in data[start..start + (width * 4) as usize].chunks_exact(4) {
                rgba.extend_from_slice(&if bgra { [pixel[2], pixel[1], pixel[0], 255] } else { [pixel[0], pixel[1], pixel[2], 255] });
            }
        }
    }
    buffer.unmap();
    std::thread::spawn(move || {
        let written = std::fs::File::create(&path).map_err(|error| error.to_string()).and_then(|file| {
            let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header().and_then(|mut writer| writer.write_image_data(&rgba)).map_err(|error| error.to_string())
        });
        match written {
            Ok(()) => info!("Screenshot saved to {}", path.display()),
            Err(error) => warn!("The screenshot couldn't be saved to {}: {error}", path.display()),
        }
    });
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            self.start(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(gpu) = &mut self.gpu {
            let _ = gpu.egui_state.on_window_event(&gpu.window, &event);
        }
        let window = Entity::PLACEHOLDER;
        match event {
            WindowEvent::CloseRequested => {
                self.send(AppExit::Success);
                event_loop.exit();
            }
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::Moved(_) => self.watch_screen(),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.app.world_mut().resource_mut::<WindowState>().scale = scale_factor as f32,
            WindowEvent::Focused(focused) => {
                self.app.world_mut().resource_mut::<WindowState>().focused = focused;
                if focused {
                    self.unfreeze(event_loop);
                } else {
                    self.let_go_of_the_mouse();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F11) && event.state == ElementState::Pressed && !event.repeat {
                    self.toggle_fullscreen();
                }
                let key_code = keys::convert_physical_key_code(event.physical_key);
                let state = if event.state == ElementState::Pressed { ButtonState::Pressed } else { ButtonState::Released };
                let text = event.text.as_ref().map(|text| text.as_str().into());
                self.send(KeyboardInput { key_code, logical_key: Key::Unidentified(NativeKey::Unidentified), state, text, repeat: event.repeat, window });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    winit::event::MouseButton::Back => MouseButton::Back,
                    winit::event::MouseButton::Forward => MouseButton::Forward,
                    winit::event::MouseButton::Other(other) => MouseButton::Other(other),
                };
                let state = if state == ElementState::Pressed { ButtonState::Pressed } else { ButtonState::Released };
                self.send(MouseButtonInput { button, state, window });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (unit, x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (MouseScrollUnit::Line, x, y),
                    MouseScrollDelta::PixelDelta(position) => (MouseScrollUnit::Pixel, position.x as f32, position.y as f32),
                };
                self.send(MouseWheel { unit, x, y, window, phase: TouchPhase::Moved });
            }
            WindowEvent::DroppedFile(path) => self.send(FileDropped(path)),
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta: (x, y) } = event {
            self.send(MouseMotion { delta: Vec2::new(x as f32, y as f32) });
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(gpu) = &self.gpu
            && !self.frozen
        {
            gpu.window.request_redraw();
        }
    }
}
