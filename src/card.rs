//! The graphics card: chosen, set up, and joined to the window.
//!
//! What old learnt, carried as results:
//! - DirectX 12 on Windows [2026-10-24: Vulkan sometimes stalled handing frames to the screen on this kind of laptop,
//!   about 6 frames a second against 240]. `WGPU_BACKEND` still chooses.
//! - DXC compiles the shaders, the Windows SDK's own when it's installed, with its validator loaded by path first
//!   [2026-11-04: DXC took a fraction of the old compiler's time; 2026-11-09: under Nsight a mismatched validator
//!   failed every shader]. `WGPU_DX12_COMPILER` still chooses.
//! - The card is chosen by `WGPU_ADAPTER_NAME` (part of its name), else `WGPU_POWER_PREF` (`low`, `high`), else the
//!   high-performance one. The laptop's Intel card is flown with `WGPU_ADAPTER_NAME=intel`.
//! - The card's own timestamps where it has them: the card's clock (`clocks.rs`) times each pass.
//! - The surface isn't sRGB: the panel works its colours out as sRGB and they're shown as made.
//! - Limits are wgpu's defaults, which every card meets: raised only on purpose, and said so [portable software,
//!   no per-card tricks: David, 2026-11-08]. Raised: the largest buffer, for the far bricks (the whole world's far
//!   cells on the card, about a gigabyte; both the laptop's cards allow 2 GB); and the storage buffers a pass may
//!   read, 8 to 16, for the far eye's ninth (the window's usable regions, plan 2.2 step 4; DirectX 12 cards allow 64
//!   or more).

use std::sync::Arc;
use winit::window::Window;

/// The largest buffer asked for (2 GB, less a byte: both the laptop's cards allow it).
const BIGGEST: u64 = (2 << 30) - 1;
/// The storage buffers a pass may read, asked for (wgpu's default is 8; the far eye reads 9).
pub const STORAGE_BUFFERS: u32 = 16;

/// The card, its window's picture, and how they were chosen.
pub struct Card {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    /// The card in a line: its name, kind, backend, driver, and what chose it.
    pub description: String,
}

impl Card {
    /// Chooses the card and sets it up to draw to the window; `uncapped` hands frames over as fast as they're made.
    pub fn new(window: Arc<Window>, uncapped: bool) -> Self {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = if cfg!(windows) && std::env::var("WGPU_BACKEND").is_err() { wgpu::Backends::DX12 } else { wgpu::Backends::from_env().unwrap_or(wgpu::Backends::PRIMARY) };
        if std::env::var("WGPU_DX12_COMPILER").is_err()
            && let Some(dxc) = windows_sdk_dxc()
        {
            descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::DynamicDxc { dxc_path: dxc };
        }
        let instance = wgpu::Instance::new(descriptor);
        let surface = instance.create_surface(window.clone()).expect("a surface to draw on");
        let (adapter, chosen_by) = choose(&instance, &surface);
        let info = adapter.get_info();
        let description = format!("{} ({:?}, {:?}, driver {} {}; chosen by {chosen_by})", info.name, info.device_type, info.backend, info.driver, info.driver_info);
        log::info!("The card: {description}");
        let required_features = adapter.features() & wgpu::Features::TIMESTAMP_QUERY;
        if required_features.is_empty() {
            log::warn!("This card can't time its own work: the card's clock is off");
        }
        let largest = adapter.limits().max_buffer_size.min(adapter.limits().max_storage_buffer_binding_size).min(BIGGEST);
        log::info!("The card's largest buffer, raised on purpose for the far bricks: {} MB (wgpu's default {} MB)", largest >> 20, wgpu::Limits::default().max_storage_buffer_binding_size >> 20);
        let storage_buffers = adapter.limits().max_storage_buffers_per_shader_stage.min(STORAGE_BUFFERS);
        log::info!("The storage buffers a pass may read, raised on purpose for the far eye: {storage_buffers} (wgpu's default {})", wgpu::Limits::default().max_storage_buffers_per_shader_stage);
        let required_limits = wgpu::Limits { max_buffer_size: largest, max_storage_buffer_binding_size: largest, max_storage_buffers_per_shader_stage: storage_buffers, ..wgpu::Limits::default() };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("Incus Artus"), required_features, required_limits, ..Default::default() }))
            .expect("the card's device");
        let size = window.inner_size();
        let mut config = surface.get_default_config(&adapter, size.width.max(1), size.height.max(1)).expect("the card can draw to this window");
        config.format = config.format.remove_srgb_suffix();
        config.view_formats = vec![config.format];
        config.present_mode = present_mode(&surface.get_capabilities(&adapter).present_modes, uncapped);
        surface.configure(&device, &config);
        log::info!("The picture: {} x {}, handed to the screen {:?}", config.width, config.height, config.present_mode);
        Self { surface, device, queue, config, description }
    }

    /// The window's new size: the picture set up again to match.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// The card's memory the game holds, MB: in use, and held in blocks (the library's allocator; the screen's own
    /// pictures, about 50 MB at 2560 x 1600, aren't in it). None where the card can't say.
    pub fn memory(&self) -> Option<(f64, f64)> {
        self.device.generate_allocator_report().map(|report| (report.total_allocated_bytes as f64 / 1e6, report.total_reserved_bytes as f64 / 1e6))
    }

    /// The picture set up again as it was (it was lost, or out of date).
    pub fn set_up_again(&self) {
        self.surface.configure(&self.device, &self.config);
    }
}

/// The card: by name, by power, or the high-performance one.
fn choose(instance: &wgpu::Instance, surface: &wgpu::Surface) -> (wgpu::Adapter, String) {
    if let Ok(wanted) = std::env::var("WGPU_ADAPTER_NAME") {
        let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let names: Vec<String> = adapters.iter().map(|adapter| adapter.get_info().name).collect();
        let Some(adapter) = adapters.into_iter().find(|adapter| adapter.get_info().name.to_lowercase().contains(&wanted.to_lowercase()) && adapter.is_surface_supported(surface)) else {
            panic!("WGPU_ADAPTER_NAME={wanted} matches no graphics card that can draw here; there are: {}", names.join(", "));
        };
        return (adapter, format!("WGPU_ADAPTER_NAME={wanted}"));
    }
    let (power_preference, chosen_by) = match wgpu::PowerPreference::from_env() {
        Some(power) => (power, format!("WGPU_POWER_PREF={}", std::env::var("WGPU_POWER_PREF").unwrap_or_default())),
        None => (wgpu::PowerPreference::HighPerformance, "the default, high performance".to_string()),
    };
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference, compatible_surface: Some(surface), ..Default::default() })).expect("a graphics card");
    (adapter, chosen_by)
}

/// How the picture is handed to the screen: uncapped, as soon as it's made (Immediate, else Mailbox, else Fifo); capped,
/// in step with the screen (FifoRelaxed, else Fifo).
fn present_mode(allowed: &[wgpu::PresentMode], uncapped: bool) -> wgpu::PresentMode {
    use wgpu::PresentMode::{Fifo, FifoRelaxed, Immediate, Mailbox};
    let order: &[wgpu::PresentMode] = if uncapped { &[Immediate, Mailbox, Fifo] } else { &[FifoRelaxed, Fifo] };
    order.iter().copied().find(|mode| allowed.contains(mode)).unwrap_or(Fifo)
}

/// The Windows SDK's `dxcompiler.dll`, if it's installed with its `dxil.dll` beside it; the validator is loaded first, by
/// its path, so DXC gets the one made with it.
fn windows_sdk_dxc() -> Option<String> {
    let folder = std::path::Path::new(r"C:\Program Files (x86)\Windows Kits\10\Redist\D3D\x64");
    let (dxc, dxil) = (folder.join("dxcompiler.dll"), folder.join("dxil.dll"));
    if !(dxc.is_file() && dxil.is_file()) {
        return None;
    }
    #[cfg(windows)]
    load_by_path(&dxil);
    Some(dxc.to_string_lossy().into_owned())
}

/// A library loaded by its full path, kept for the game's life.
#[cfg(windows)]
fn load_by_path(path: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: a nul-terminated path that outlives the call; the library is never freed.
    let _ = unsafe { windows::Win32::System::LibraryLoader::LoadLibraryW(windows::core::PCWSTR(wide.as_ptr())) };
}
