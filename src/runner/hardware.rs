//! What the frame asks of the hardware through Windows directly:
//! - **the alarm** (`Alarm`): wakes the frame on its beat, by Windows' high-resolution timer;
//! - **the game's claim on the PC's attention** (`Attention`), held while it's in front and given back
//!   when it isn't [David 2026-11-05: "relaxed but ready to act instantly without any distraction";
//!   "a full screen only thing that tyrannizes the PC, as long as people can at least tab back to
//!   discord quickly"];
//! - **which processor core the frame is on** (`current_core`, `core_classes`);
//! - **background threads below the frame** (`background_thread`).
//!
//! Off Windows the alarm is the plain sleep, there's no claim, and no core.
//!
//! (Build 62 paced the cap by the screen's own refreshes instead. On David's laptop the screen has a
//! variable refresh rate: in a game it waits for each frame, so its refreshes followed the game and
//! made a poor clock [docs/regroup.md, build 62's test window]. Taken out in build 63.)

use std::time::{Duration, Instant};

#[cfg(windows)]
mod windows_only {
    use std::time::Duration;
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
    use windows::Win32::System::Threading::{
        CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, GetCurrentProcess, GetCurrentThread, INFINITE, PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
        ProcessPowerThrottling, SetProcessInformation, SetThreadPriority, SetWaitableTimer, THREAD_PRIORITY_BELOW_NORMAL, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_NORMAL, TIMER_ALL_ACCESS, WaitForSingleObject,
    };

    /// Windows' high-resolution waitable timer (Windows 10 from 2018 on): wakes within a fraction of
    /// a millisecond, without speeding up the system's tick.
    pub struct Timer(HANDLE);

    impl Timer {
        pub fn new() -> Option<Self> {
            unsafe { CreateWaitableTimerExW(None, windows::core::PCWSTR::null(), CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, TIMER_ALL_ACCESS.0) }.ok().map(Timer)
        }

        /// Sleeps for `wait`. False if the timer couldn't be set.
        pub fn sleep(&self, wait: Duration) -> bool {
            // In hundreds of nanoseconds, negative for "from now".
            let due = -((wait.as_nanos() / 100).max(1) as i64);
            if unsafe { SetWaitableTimer(self.0, &due, 0, None, None, false) }.is_err() {
                return false;
            }
            unsafe { WaitForSingleObject(self.0, INFINITE) };
            true
        }
    }

    impl Drop for Timer {
        fn drop(&mut self) {
            let _ = unsafe { CloseHandle(self.0) };
        }
    }

    /// Takes the PC's attention for the game (on the frame's thread): Windows' tick at 1 ms (since
    /// 2020 it applies to this program alone, and Windows 11 drops it while the window is hidden), the
    /// frame's thread first in line, and Windows told not to slow the game down to save power (which
    /// would also move it to the efficient cores).
    pub fn take_attention() {
        unsafe {
            timeBeginPeriod(1);
            let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
        }
        power_throttling(false);
    }

    /// Gives it all back, so the PC is everyone else's.
    pub fn give_back_attention() {
        unsafe {
            timeEndPeriod(1);
            let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_NORMAL);
        }
        power_throttling(true);
    }

    /// A background thread (the loading lane's workers, the World Compiler): below the frame's
    /// priority, so it never takes the frame's core.
    pub fn background_thread() {
        let _ = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL) };
    }

    /// Whether Windows may slow the game down to save power (true: it decides, as for any program).
    fn power_throttling(allowed: bool) {
        let state = PROCESS_POWER_THROTTLING_STATE { Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION, ControlMask: if allowed { 0 } else { PROCESS_POWER_THROTTLING_EXECUTION_SPEED }, StateMask: 0 };
        let _ = unsafe { SetProcessInformation(GetCurrentProcess(), ProcessPowerThrottling, &state as *const _ as *const core::ffi::c_void, std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32) };
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
    use std::time::Duration;

    pub struct Timer;

    impl Timer {
        pub fn new() -> Option<Self> {
            None
        }

        pub fn sleep(&self, _wait: Duration) -> bool {
            false
        }
    }

    pub fn take_attention() {}

    pub fn give_back_attention() {}

    pub fn background_thread() {}

    pub fn current_core() -> u32 {
        0
    }

    pub fn core_classes() -> Vec<u8> {
        Vec::new()
    }
}

pub use windows_only::{background_thread, core_classes, current_core};

/// Wakes the frame at a moment: Windows' high-resolution timer where there is one, else the plain
/// sleep.
pub struct Alarm {
    timer: Option<windows_only::Timer>,
}

impl Alarm {
    pub fn new() -> Self {
        Self { timer: windows_only::Timer::new() }
    }

    /// Whether it's Windows' high-resolution timer.
    pub fn precise(&self) -> bool {
        self.timer.is_some()
    }

    /// Sleeps until `when` (straight back if it's gone by).
    pub fn sleep_until(&self, when: Instant) {
        let now = Instant::now();
        if when <= now {
            return;
        }
        let wait: Duration = when - now;
        if !self.timer.as_ref().is_some_and(|timer| timer.sleep(wait)) {
            std::thread::sleep(wait);
        }
    }
}

/// The game's claim on the PC's attention: taken while it's in front, given back when it isn't.
#[derive(Default)]
pub struct Attention {
    held: bool,
}

impl Attention {
    pub fn take(&mut self) {
        if !self.held {
            windows_only::take_attention();
            self.held = true;
        }
    }

    pub fn give_back(&mut self) {
        if self.held {
            windows_only::give_back_attention();
            self.held = false;
        }
    }
}

impl Drop for Attention {
    fn drop(&mut self) {
        self.give_back();
    }
}
