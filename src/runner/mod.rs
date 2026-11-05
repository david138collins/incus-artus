//! The runner — the frame lane: the window, the card, and the frame loop (the regroup,
//! docs/regroup.md, phase 0). It replaces Bevy's window, renderer and app loop.
//!
//! Each frame, in order, each step timed for the slow-frame log (`perf`):
//! 1. the window's input goes in as the same messages Bevy's own window code sent (keys, mouse
//!    buttons, wheel, motion), so the game's input code didn't change;
//! 2. **the game's update**: Bevy's schedules (its ECS core is kept for now, `docs/regroup.md`), with
//!    the simulation's fixed ticks inside it;
//! 3. **the panel**: egui's frame, the `EguiPrimaryContextPass` systems;
//! 4. **the card**: clear to the sky, draw the panel, hand it over;
//! 5. **the frame cap**: vsync and, for a cap below the refresh rate, the rest of the frame waited
//!    out (as the old `frame_cap.rs` did; 0 = as fast as possible, vsync off).
//!
//! The window's state the game reads (its size, whether it has the focus) is the `WindowState`
//! resource; what the game asks of the window (the mouse captured, a size) it sets there too.

mod keys;

use crate::perf::Timed;
use bevy::app::AppExit;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
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
}

impl Default for WindowState {
    fn default() -> Self {
        Self { size: (1280, 960), scale: 1.0, focused: true, capture_mouse: false, resize_to: None, minimize: None, refresh: 60.0 }
    }
}

/// The frame cap the game wants: a frame every this many of the screen's refreshes (0 = as fast as
/// possible, vsync off). Set by the game from its settings each frame.
#[derive(Resource, Clone, Copy, Debug)]
pub struct FrameCap(pub u32);

impl Default for FrameCap {
    fn default() -> Self {
        Self(2)
    }
}

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
    app.init_resource::<WindowState>().init_resource::<FrameCap>().init_resource::<ScreenshotRequest>().add_message::<FileDropped>();
    // Plugins finish their set-up (Bevy's `App::run` did this).
    while app.plugins_state() == bevy::app::PluginsState::Adding {
        bevy::tasks::tick_global_task_pools_on_main_thread();
    }
    app.finish();
    app.cleanup();
    let event_loop = EventLoop::new().expect("an event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut runner = Runner { app, gpu: None, last_frame: None, due: None, cap: None };
    if let Err(error) = event_loop.run_app(&mut runner) {
        error!("The window's loop stopped: {error}");
    }
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
}

struct Runner {
    app: App,
    gpu: Option<Gpu>,
    /// When the last frame was shown (its length is measured from one to the next).
    last_frame: Option<Instant>,
    /// When the frame now being made should end, under a cap (see `wait_out`).
    due: Option<Instant>,
    /// The frame cap the surface is set up for.
    cap: Option<u32>,
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
        let attributes = Window::default_attributes().with_title("Incus Artus").with_inner_size(winit::dpi::LogicalSize::new(1280, 960)).with_maximized(true);
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
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("Incus Artus"), ..Default::default() })).expect("the card's device");
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
        let refresh = window.current_monitor().and_then(|monitor| monitor.refresh_rate_millihertz()).map_or(60.0, |millihertz| millihertz as f32 / 1000.0);
        {
            let mut state = self.app.world_mut().resource_mut::<WindowState>();
            state.size = (config.width, config.height);
            state.scale = window.scale_factor() as f32;
            state.refresh = refresh;
        }
        self.gpu = Some(Gpu { window, surface, device, queue, config, egui_state, egui_renderer, captured: false });
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
        let Some(gpu) = &mut self.gpu else { return };
        let ctx = self.app.world().resource::<crate::ui::EguiCtx>().0.clone();
        // The panel's frame opens before the game's update, so the game's systems can draw in it.
        ctx.begin_pass(gpu.egui_state.take_egui_input(&gpu.window));
        {
            let _timed = Timed::new("the game's update");
            self.app.update();
        }
        if let Some(exit) = self.app.should_exit() {
            info!("Quitting ({exit:?})");
            event_loop.exit();
            return;
        }
        {
            let _timed = Timed::new("the panel");
            self.app.world_mut().run_schedule(crate::ui::EguiPrimaryContextPass);
        }
        let Some(gpu) = &mut self.gpu else { return };
        let output = ctx.end_pass();
        gpu.egui_state.handle_platform_output(&gpu.window, output.platform_output);
        *self.app.world_mut().resource_mut::<crate::ui::EguiWantsInput>() = crate::ui::EguiWantsInput { keyboard: ctx.egui_wants_keyboard_input() };

        // What the game asks of the window.
        let (capture, resize_to, minimize, cap, screenshot) = {
            let world = self.app.world_mut();
            let cap = world.resource::<FrameCap>().0;
            let screenshot = world.resource_mut::<ScreenshotRequest>().0.take();
            let mut state = world.resource_mut::<WindowState>();
            (state.capture_mouse, state.resize_to.take(), state.minimize.take(), cap, screenshot)
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
        if self.cap != Some(cap) {
            self.cap = Some(cap);
            gpu.config.present_mode = if cap == 0 { wgpu::PresentMode::AutoNoVsync } else { wgpu::PresentMode::AutoVsync };
            gpu.surface.configure(&gpu.device, &gpu.config);
        }

        {
            let _timed = Timed::new("handing the card its work");
            let texture = match gpu.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(texture) | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                _ => {
                    // Lost or out of date (a resize, a minimise): set up again, and try next frame.
                    gpu.surface.configure(&gpu.device, &gpu.config);
                    return;
                }
            };
            let view = texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let screen = egui_wgpu::ScreenDescriptor { size_in_pixels: [gpu.config.width, gpu.config.height], pixels_per_point: output.pixels_per_point };
            let jobs = ctx.tessellate(output.shapes, output.pixels_per_point);
            for (id, deltas) in &output.textures_delta.set {
                for delta in deltas {
                    gpu.egui_renderer.update_texture(&gpu.device, &gpu.queue, *id, delta);
                }
            }
            let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("the frame") });
            let mut commands = gpu.egui_renderer.update_buffers(&gpu.device, &gpu.queue, &mut encoder, &jobs, &screen);
            {
                let attachment = wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(SKY), store: wgpu::StoreOp::Store } };
                let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { label: Some("the panel"), color_attachments: &[Some(attachment)], ..Default::default() });
                gpu.egui_renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
            }
            let shot = screenshot.filter(|_| gpu.config.usage.contains(wgpu::TextureUsages::COPY_SRC)).map(|path| (path, copy_out(&gpu.device, &mut encoder, &texture.texture)));
            commands.push(encoder.finish());
            gpu.queue.submit(commands);
            if let Some((path, (buffer, padded))) = shot {
                save_screenshot(&gpu.device, buffer, padded, (gpu.config.width, gpu.config.height), gpu.config.format, path);
            }
            for id in &output.textures_delta.free {
                gpu.egui_renderer.free_texture(id);
            }
            let _shown = Timed::new("showing it");
            gpu.window.pre_present_notify();
            gpu.queue.present(texture);
        }

        let refresh = self.app.world().resource::<WindowState>().refresh;
        self.wait_out(cap, refresh);
        let now = Instant::now();
        let ms = self.last_frame.map_or(0.0, |last| now.duration_since(last).as_secs_f32() * 1000.0);
        self.last_frame = Some(now);
        let capped = (cap > 0).then(|| cap as f32 * 1000.0 / refresh.max(1.0));
        crate::perf::frame_done(self.app.world_mut(), ms, capped);
    }

    /// Under a cap of 2 or more refreshes a frame, waits out the rest of the frame's time: exactly a
    /// frame's time after the last one was due (so a late wake-up isn't carried into the next frame),
    /// or from now if it's fallen behind (a slow frame isn't made up for by rushing the next ones).
    fn wait_out(&mut self, cap: u32, refresh: f32) {
        if cap <= 1 {
            self.due = None;
            return;
        }
        let frame = Duration::from_secs_f64(cap as f64 / refresh.max(1.0) as f64);
        let now = Instant::now();
        let end = match self.due {
            Some(last) if last + frame > now => last + frame,
            _ => now,
        };
        if end > now {
            let _timed = Timed::new("waiting for the frame cap");
            std::thread::sleep(end - now);
        }
        self.due = Some(end);
    }
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
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.app.world_mut().resource_mut::<WindowState>().scale = scale_factor as f32,
            WindowEvent::Focused(focused) => self.app.world_mut().resource_mut::<WindowState>().focused = focused,
            WindowEvent::KeyboardInput { event, .. } => {
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
        if let Some(gpu) = &self.gpu {
            gpu.window.request_redraw();
        }
    }
}
