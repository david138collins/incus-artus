//! What the hardware says through Windows directly: the screen's refreshes (`ScreenClock`, the frame
//! cap's clock) and which processor core the frame is on (`current_core`, `core_classes`). Off Windows
//! there's no screen clock (the frame cap falls back to the timer) and no core.

#[cfg(windows)]
mod windows_only {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, IDXGIOutput};

    /// The screen's own clock: waits for its next refresh (its vertical blank).
    pub struct ScreenClock {
        output: IDXGIOutput,
        monitor: isize,
    }

    impl ScreenClock {
        /// The clock of the screen `monitor` (winit's handle for it), from whichever graphics adapter
        /// drives it: on a laptop that may not be the card that draws.
        pub fn for_monitor(monitor: isize) -> Option<Self> {
            let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.ok()?;
            for a in 0.. {
                let Ok(adapter) = (unsafe { factory.EnumAdapters1(a) }) else { break };
                for o in 0.. {
                    let Ok(output) = (unsafe { adapter.EnumOutputs(o) }) else { break };
                    if let Ok(desc) = unsafe { output.GetDesc() }
                        && desc.Monitor.0 as isize == monitor
                    {
                        return Some(Self { output, monitor });
                    }
                }
            }
            None
        }

        pub fn monitor(&self) -> isize {
            self.monitor
        }

        /// Sleeps until the screen's next refresh begins. False if the screen couldn't say.
        pub fn wait_for_refresh(&self) -> bool {
            unsafe { self.output.WaitForVBlank() }.is_ok()
        }
    }

    /// The processor core this thread is running on now.
    pub fn current_core() -> u32 {
        unsafe { windows::Win32::System::Threading::GetCurrentProcessorNumber() }
    }

    /// Each core's efficiency class, by its number: the highest are the performance cores (P), lower
    /// ones the efficient cores (E). All the same on a processor without the two kinds.
    pub fn core_classes() -> Vec<u8> {
        use windows::Win32::System::SystemInformation::{CpuSetInformation, GetSystemCpuSetInformation, SYSTEM_CPU_SET_INFORMATION};
        let mut length = 0u32;
        let _ = unsafe { GetSystemCpuSetInformation(None, 0, &mut length, None, None) };
        if length == 0 {
            return Vec::new();
        }
        // u64s, so the entries are aligned.
        let mut buffer = vec![0u64; (length as usize).div_ceil(8)];
        if !unsafe { GetSystemCpuSetInformation(Some(buffer.as_mut_ptr() as *mut SYSTEM_CPU_SET_INFORMATION), length, &mut length, None, None) }.as_bool() {
            return Vec::new();
        }
        let bytes = buffer.as_ptr() as *const u8;
        let mut classes = Vec::new();
        let mut at = 0usize;
        while at + std::mem::size_of::<SYSTEM_CPU_SET_INFORMATION>() <= length as usize {
            let entry = unsafe { std::ptr::read_unaligned(bytes.add(at) as *const SYSTEM_CPU_SET_INFORMATION) };
            if entry.Size == 0 {
                break;
            }
            if entry.Type == CpuSetInformation {
                let set = unsafe { entry.Anonymous.CpuSet };
                if set.Group == 0 {
                    let core = set.LogicalProcessorIndex as usize;
                    if classes.len() <= core {
                        classes.resize(core + 1, 0);
                    }
                    classes[core] = set.EfficiencyClass;
                }
            }
            at += entry.Size as usize;
        }
        classes
    }
}

#[cfg(not(windows))]
mod windows_only {
    pub struct ScreenClock;

    impl ScreenClock {
        pub fn for_monitor(_monitor: isize) -> Option<Self> {
            None
        }

        pub fn monitor(&self) -> isize {
            0
        }

        pub fn wait_for_refresh(&self) -> bool {
            false
        }
    }

    pub fn current_core() -> u32 {
        0
    }

    pub fn core_classes() -> Vec<u8> {
        Vec::new()
    }
}

pub use windows_only::*;

/// winit's handle for the screen the window is on (Windows' `HMONITOR`).
pub fn monitor_of(window: &winit::window::Window) -> Option<isize> {
    #[cfg(windows)]
    {
        use winit::platform::windows::MonitorHandleExtWindows;
        window.current_monitor().map(|monitor| monitor.hmonitor())
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        None
    }
}
