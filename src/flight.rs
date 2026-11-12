//! A measured flight (`INCUS_FLIGHT=1`): the camera stands at each mark in turn (`data/marks.toml`; a mark with a
//! scatter once for each count of its dial), the frames settle, then they're measured. A report is written to
//! `logs/flight-<seconds since 1970>.txt`, and the game quits.
//!
//! Uncapped (frames handed over as soon as they're made) and with the panel hidden, so what's measured is the frame
//! alone.
//!
//! Each standing has a section of the report, and in it each pass a line of its own, "    <its name>: ...". The software
//! map's test reads its costs from the first standing's.

use crate::clocks::Spread;
use crate::marks::{Made, Mark};
use crate::temperatures::Temperatures;
use crate::wiring::{self, PASSES, PassId, Where};
use std::time::{Duration, Instant};

/// How long the frames settle at a mark before they're measured.
const SETTLE: Duration = Duration::from_secs(5);
/// How much longer the first mark settles: the session's warm-up [build 119's flight: for about its first 25 s the whole
/// session ran slower, the card's pass too (0.018 against 0.011 ms), so the first mark read 20% slower than the rest].
const WARM_UP: Duration = Duration::from_secs(30);
/// How long they're measured at each (old's standing marks: 20 s).
const MEASURE: Duration = Duration::from_secs(20);

/// Whether this session is a flight.
pub fn wanted() -> bool {
    std::env::var_os("INCUS_FLIGHT").is_some()
}

/// One standing of the flight: a mark (by its place in the marks), at a count of its scatter's dial if it has one.
pub struct Standing {
    pub mark: usize,
    pub count: Option<u32>,
    /// What was made there through the doorway, and where the eye stood (said by going to the mark).
    made: Made,
    eye: Option<[f32; 3]>,
    /// What going there cost: its time (ms), and the doorway's tiles answered on the way and their time (ms).
    going: Option<(f32, usize, f32)>,
    /// The measured frames' lengths, ms.
    frames: Vec<f32>,
    /// Each pass's measured times on the processor, by its id.
    passes: Vec<Vec<f32>>,
    /// The measured frames' card times: each card slot's ms, the card's whole last.
    card: Vec<Vec<f32>>,
    /// Where the eye was at the last measured frame (a moving mark's end), and the least the window's usable 32s and
    /// 64s reached round it over the measured frames (blocks; step 4).
    end: Option<[f32; 3]>,
    reach: Option<[f32; 2]>,
    /// The hottest the CPU area and the GPU read over the measured frames, and the GPU's slowest clock.
    hottest: Temperatures,
    /// The most main memory the game held over the measured frames (bytes, read once a second).
    most_main_memory: Option<u64>,
}

pub struct Flight {
    standings: Vec<Standing>,
    /// The standing now, and when it began.
    now: Option<(usize, Instant)>,
}

/// What the report says about the session.
pub struct About<'a> {
    pub card: &'a str,
    pub picture: (u32, u32),
    pub present: wgpu::PresentMode,
    pub refresh: f32,
    pub marks: &'a [Mark],
    /// How the far cells came in.
    pub far: String,
    /// The card's memory the game holds at the end, MB: in use, and held.
    pub memory: Option<(f64, f64)>,
    /// The main memory the game holds at the end, and the most it has (none where Windows can't say).
    pub main_memory: Option<crate::memory::MainMemory>,
}

impl Flight {
    pub fn new(marks: &[Mark]) -> Self {
        let mut standings = Vec::new();
        for (k, mark) in marks.iter().enumerate() {
            let counts: Vec<Option<u32>> = match &mark.scatter {
                Some(scatter) => scatter.counts.iter().map(|count| Some(*count)).collect(),
                None => vec![None],
            };
            for count in counts {
                standings.push(Standing { mark: k, count, made: Made::default(), eye: None, going: None, frames: Vec::new(), passes: vec![Vec::new(); PASSES.len()], card: Vec::new(), end: None, reach: None, hottest: Temperatures::default(), most_main_memory: None });
            }
        }
        log::info!("A flight: {} standings at {} marks, {} s settling and {} s measured at each (the first settling {} s more)", standings.len(), marks.len(), SETTLE.as_secs(), MEASURE.as_secs(), WARM_UP.as_secs());
        Self { standings, now: None }
    }

    /// How long standing `k` settles before it's measured.
    fn settle(k: usize) -> Duration {
        if k == 0 { SETTLE + WARM_UP } else { SETTLE }
    }

    /// The standing to go to now, if it's time to move: the first at the start, the next once one is measured.
    pub fn due(&mut self, now: Instant) -> Option<&Standing> {
        let next = match self.now {
            None => 0,
            Some((k, since)) if now.duration_since(since) >= Self::settle(k) + MEASURE && k + 1 < self.standings.len() => k + 1,
            Some(_) => return None,
        };
        self.now = Some((next, now));
        Some(&self.standings[next])
    }

    /// The first standing held back (its settling begun again) while something it waits for isn't ready yet: the far
    /// cells all on the card.
    pub fn hold(&mut self, now: Instant) {
        if let Some((0, since)) = &mut self.now {
            *since = now;
        }
    }

    /// Going to the standing now made this through the doorway, put the eye here, and cost this (`going`: its time in
    /// ms, and the doorway's tiles answered on the way and their time in ms).
    pub fn arrived(&mut self, made: Made, eye: Option<[f32; 3]>, going: (f32, usize, f32)) {
        if let Some((k, _)) = self.now {
            self.standings[k].made = made;
            self.standings[k].eye = eye;
            self.standings[k].going = Some(going);
        }
    }

    /// The standing being measured now, if its frames have settled.
    fn measuring(&mut self, now: Instant) -> Option<&mut Standing> {
        let (k, since) = self.now?;
        let since = now.duration_since(since);
        (since >= Self::settle(k) && since < Self::settle(k) + MEASURE).then(|| &mut self.standings[k])
    }

    /// A pass ran for `ms` on the processor.
    pub fn pass(&mut self, now: Instant, pass: PassId, ms: f32) {
        if let Some(standing) = self.measuring(now) {
            standing.passes[pass as usize].push(ms);
        }
    }

    /// A frame shown (its length), and the card's times that came back with it.
    pub fn record(&mut self, now: Instant, frame_ms: Option<f32>, card: Vec<Vec<f32>>) {
        if let Some(standing) = self.measuring(now) {
            standing.frames.extend(frame_ms);
            standing.card.extend(card);
        }
    }

    /// The speed the camera flies at now (blocks a second), if the standing's mark moves.
    pub fn speed(&self, marks: &[crate::marks::Mark]) -> Option<f32> {
        let (k, _) = self.now?;
        marks.get(self.standings[k].mark)?.speed
    }

    /// Where the eye is this frame, and how far the window's usable 32s and 64s reach round it (none without a window).
    pub fn followed(&mut self, now: Instant, eye: [f32; 3], reach: Option<[f32; 2]>) {
        if let Some(standing) = self.measuring(now) {
            standing.end = Some(eye);
            if let Some(reach) = reach {
                let least = standing.reach.get_or_insert(reach);
                *least = [least[0].min(reach[0]), least[1].min(reach[1])];
            }
        }
    }

    /// The temperatures as last read: the hottest kept over the measured frames, and the GPU's slowest clock.
    pub fn heat(&mut self, now: Instant, reading: Temperatures) {
        if let Some(standing) = self.measuring(now) {
            let most = |kept: Option<f32>, new: Option<f32>| match (kept, new) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
            let least = |kept: Option<f32>, new: Option<f32>| match (kept, new) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            let h = &mut standing.hottest;
            *h = Temperatures {
                cpu_area: most(h.cpu_area, reading.cpu_area),
                gpu: most(h.gpu, reading.gpu),
                gpu_clock: least(h.gpu_clock, reading.gpu_clock),
                cpu_core: most(h.cpu_core, reading.cpu_core),
                cpu_watts: most(h.cpu_watts, reading.cpu_watts),
                cpu_load: most(h.cpu_load, reading.cpu_load),
                gpu_watts: most(h.gpu_watts, reading.gpu_watts),
                gpu_load: most(h.gpu_load, reading.gpu_load),
            };
        }
    }

    /// The main memory the game holds now (read once a second): the most kept over the measured frames.
    pub fn main_memory(&mut self, now: Instant, in_use: u64) {
        if let Some(standing) = self.measuring(now) {
            standing.most_main_memory = Some(standing.most_main_memory.map_or(in_use, |most| most.max(in_use)));
        }
    }

    /// Whether the last standing is measured.
    pub fn finished(&self, now: Instant) -> bool {
        matches!(self.now, Some((k, since)) if k + 1 == self.standings.len() && now.duration_since(since) >= Self::settle(k) + MEASURE)
    }

    /// The report, written to `logs/`.
    pub fn report(&self, about: &About) {
        let mut text = String::new();
        let code = if crate::session::BUILD_CHANGED.is_empty() { "the code committed".to_string() } else { format!("the code with changes: {}", crate::session::BUILD_CHANGED) };
        text += &format!("The flight: Incus Artus, {}; {code}\n", crate::session::build_line());
        text += &format!("The card: {}\n", about.card);
        text += &format!("The picture {} x {}, the screen's refresh {:.0}, uncapped; handed to the screen: {:?}; the panel hidden\n", about.picture.0, about.picture.1, about.refresh, about.present);
        // The switches set for this session. Not the build's own stamp (`INCUS_BUILD...`): `cargo run` hands the build
        // script's values to the game as it starts, and they're in the first line already.
        let mut switches: Vec<String> = std::env::vars()
            .filter(|(name, _)| (name.starts_with("INCUS_") && !name.starts_with("INCUS_BUILD")) || name.starts_with("WGPU_"))
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        switches.sort();
        text += &format!("Switches: {}\n", switches.join(", "));
        text += &format!("The far cells: {} (the first mark held till all were on the card)\n", about.far);
        text += &match about.memory {
            Some((used, held)) => format!("The card's memory: {used:.0} MB in use, {held:.0} MB held (the library's allocator; the screen's own pictures not in it)\n"),
            None => "The card's memory: the card can't say\n".to_string(),
        };
        text += &match about.main_memory {
            Some(memory) => format!("The main memory: {}\n", memory.line()),
            None => "The main memory: Windows can't say\n".to_string(),
        };
        text += &format!("The marks: data/marks.toml; {} s settling, then {} s measured at each (the first settling {} s more: the session's warm-up)\n", SETTLE.as_secs(), MEASURE.as_secs(), WARM_UP.as_secs());
        let spread = |spread: Spread| format!("median {:.3}, 95th {:.3}, 99th {:.3}, worst {:.3}", spread.median, spread.p95, spread.p99, spread.worst);
        for standing in &self.standings {
            let mark = &about.marks[standing.mark];
            let dial = standing.count.map_or(String::new(), |count| format!(", the scatter at {count}"));
            let eye = standing.eye.map_or("not placed (the doorway couldn't answer)".to_string(), |[north, east, up]| format!("north {north:.1}, east {east:.1}, up {up:.1}"));
            text += &format!("\nAt {}{dial} ({} eye; fog {}): the eye at {eye}, facing {:.0}°, pitch {:.1}°\n", mark.name, mark.eyes.join(" and "), if mark.fog { "on" } else { "off" }, mark.facing, mark.pitch);
            if let Some((ms, tiles, tiles_ms)) = standing.going {
                text += &format!("  going there: {ms:.1} ms (the doorway answered {tiles} tiles in {tiles_ms:.1} ms)\n");
            }
            if let (Some(speed), Some(begun), Some(end)) = (mark.speed, standing.eye, standing.end) {
                let gone = ((end[0] - begun[0]).powi(2) + (end[1] - begun[1]).powi(2) + (end[2] - begun[2]).powi(2)).sqrt();
                text += &format!("  moving at {speed:.0} blocks a second, straight ahead and level: the eye ended at north {:.1}, east {:.1}, up {:.1}, {gone:.0} blocks from where it began\n", end[0], end[1], end[2]);
            }
            if let Some(most) = standing.most_main_memory {
                text += &format!("  the main memory: at most {:.0} MB in use\n", most as f64 / 1e6);
            }
            let heat = standing.hottest;
            if heat != Temperatures::default() {
                text += &format!("  the heat, each at its most (the GPU's clock at its least): {}\n", heat.line());
            }
            if let Some(reach) = standing.reach {
                text += &format!("  the window round the eye: the usable 32s reached at least {:.0} blocks round it over the measured frames, the 64s {:.0}\n", reach[0], reach[1]);
            }
            let made = &standing.made;
            if made.blocks + made.bars + made.refused > 0 {
                text += &format!("  made through the doorway: {} whole blocks, {} bars, {} refused\n", made.blocks, made.bars, made.refused);
            }
            let frames = Spread::of(standing.frames.iter().copied());
            let fps = if frames.median > 0.0 { 1000.0 / frames.median } else { 0.0 };
            text += &format!(
                "  the frame: {} frames, {fps:.0} a second (median); ms median {:.3}, 95th {:.3}, 99th {:.3}, 99.9th {:.3}, worst {:.3}\n",
                frames.count, frames.median, frames.p95, frames.p99, frames.p999, frames.worst
            );
            text += &format!("  each pass (ms; {} frames timed on the card):\n", standing.card.len());
            for pass in &PASSES {
                let times = &standing.passes[pass.id as usize];
                let on_processor = if pass.on == Where::Card { "recording it on the processor" } else { "on the processor" };
                let mut line = format!("    {}: {on_processor} {}", pass.name, if times.is_empty() { "not run".to_string() } else { spread(Spread::of(times.iter().copied())) });
                if let Some(slot) = wiring::card_slot(pass.id) {
                    let card = Spread::of(standing.card.iter().filter_map(|frame| frame.get(slot).copied()));
                    line += &if card.count == 0 { "; on the card not timed".to_string() } else { format!("; on the card {}", spread(card)) };
                }
                text += &line;
                text += "\n";
            }
            if !standing.card.is_empty() {
                text += &format!("  the card's whole: {}\n", spread(Spread::of(standing.card.iter().filter_map(|frame| frame.last().copied()))));
            }
        }
        let path = crate::session::logs_folder().join(format!("flight-{}.txt", crate::session::unix_seconds()));
        match std::fs::write(&path, &text) {
            Ok(()) => log::info!("The flight's report: {}", path.display()),
            Err(error) => log::error!("The flight's report couldn't be written to {}: {error}", path.display()),
        }
        for line in text.lines() {
            log::info!("{line}");
        }
    }
}
