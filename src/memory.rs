//! The main memory: how much the game holds, beside the card's, in a flight's report and the session's log [David,
//! 2026-11-12, via Counsel: "tell builder to begin tracking ram use, and start looking for opportunities where using it
//! is almost a free win or takes some heat/load off something else"; he thinks "basically everyone will have at least
//! 8gb"].
//!
//! Windows' own count for the game's process (`GetProcessMemoryInfo`), no program started and no waiting:
//! - **In use:** the private bytes, what the game has asked for and holds, whether in memory or paged out; and the most
//!   it has held at once since it started.
//! - **Working set:** what of it is in memory now, and the most at once.
//!
//! And the machine's: its memory and how much is free (`GlobalMemoryStatusEx`).

/// The game's main memory and the machine's, in bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MainMemory {
    pub in_use: u64,
    pub most_in_use: u64,
    pub working_set: u64,
    pub most_working_set: u64,
    pub machine: u64,
    pub machine_free: u64,
}

impl MainMemory {
    /// As it stands now (none where Windows can't say).
    #[cfg(windows)]
    pub fn now() -> Option<Self> {
        use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX};
        use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        use windows::Win32::System::Threading::GetCurrentProcess;
        let mut counters = PROCESS_MEMORY_COUNTERS_EX { cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32, ..Default::default() };
        // The extended counters, read through the plain ones' door (Windows fills the rest, `cb` saying there's room).
        unsafe { GetProcessMemoryInfo(GetCurrentProcess(), (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(), counters.cb) }.ok()?;
        let mut machine = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
        unsafe { GlobalMemoryStatusEx(&mut machine) }.ok()?;
        Some(Self {
            in_use: counters.PrivateUsage as u64,
            most_in_use: counters.PeakPagefileUsage as u64,
            working_set: counters.WorkingSetSize as u64,
            most_working_set: counters.PeakWorkingSetSize as u64,
            machine: machine.ullTotalPhys,
            machine_free: machine.ullAvailPhys,
        })
    }

    #[cfg(not(windows))]
    pub fn now() -> Option<Self> {
        None
    }

    /// One line, in MB and GB, for a report or the log.
    pub fn line(&self) -> String {
        let mb = |bytes: u64| bytes as f64 / 1e6;
        format!(
            "{:.0} MB in use ({:.0} MB at most), {:.0} MB of it in memory ({:.0} MB at most); the machine {:.1} GB, {:.1} GB free",
            mb(self.in_use),
            mb(self.most_in_use),
            mb(self.working_set),
            mb(self.most_working_set),
            self.machine as f64 / 1e9,
            self.machine_free as f64 / 1e9
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Windows answers for the game's own process: something in use, no more than it ever held, and a machine with room.
    #[test]
    fn the_main_memory_is_read() {
        let now = MainMemory::now().expect("Windows says how much memory the process holds");
        // Written to and looked at, so the build can't leave it out.
        let held = std::hint::black_box(vec![1u8; 64 << 20]);
        let after = MainMemory::now().unwrap();
        assert_eq!(std::hint::black_box(&held)[12345], 1);
        assert!(now.in_use > 0 && now.in_use <= now.most_in_use && now.working_set <= now.most_working_set, "{now:?}");
        assert!(after.in_use >= now.in_use + (60 << 20), "64 MB held shows: {} then {}", now.in_use, after.in_use);
        assert!(now.machine > (1 << 30) && now.machine_free <= now.machine, "{now:?}");
        drop(held);
        eprintln!("The test's process: {}", after.line());
    }
}
