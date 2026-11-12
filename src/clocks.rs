//! The clocks: how long each frame takes, and how long each pass takes, on the processor and on the card.
//!
//! The card's clock times every pass on its own, from the first build [plan 2.1: old timed its whole march as one
//! pass, so it couldn't say which layer the bare plain's cost was in (1.7)]. Each pass writes the card's own
//! timestamps at its start and end; they're copied out and read a few frames later, never waited for.

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Instant;

/// How many recent frames the panel's figures are taken over.
pub const RECENT: usize = 240;

/// A spread of times: how many, the middle one, the slow ones, and the slowest (milliseconds).
#[derive(Clone, Copy, Default)]
pub struct Spread {
    pub count: usize,
    pub median: f32,
    pub p95: f32,
    pub p99: f32,
    pub p999: f32,
    pub worst: f32,
}

impl Spread {
    pub fn of(samples: impl IntoIterator<Item = f32>) -> Self {
        let mut sorted: Vec<f32> = samples.into_iter().collect();
        if sorted.is_empty() {
            return Self::default();
        }
        sorted.sort_by(f32::total_cmp);
        let at = |share: f32| sorted[((sorted.len() - 1) as f32 * share).round() as usize];
        Self { count: sorted.len(), median: at(0.5), p95: at(0.95), p99: at(0.99), p999: at(0.999), worst: sorted[sorted.len() - 1] }
    }
}

/// The frame's clock: from one frame handed to the screen to the next, and from one frame's step to the next.
pub struct FrameClock {
    last: Option<Instant>,
    stepped: Option<Instant>,
    recent: VecDeque<f32>,
}

impl FrameClock {
    pub fn new() -> Self {
        Self { last: None, stepped: None, recent: VecDeque::with_capacity(RECENT) }
    }

    /// A frame handed over now: its length in milliseconds (none for the first).
    pub fn shown(&mut self, now: Instant) -> Option<f32> {
        let ms = self.last.map(|last| now.duration_since(last).as_secs_f32() * 1000.0);
        self.last = Some(now);
        if let Some(ms) = ms {
            if self.recent.len() == RECENT {
                self.recent.pop_front();
            }
            self.recent.push_back(ms);
        }
        ms
    }

    /// The seconds since the last frame's step, for moving things (none before the first, and at most a tenth of a
    /// second, so a stall doesn't throw the camera across the world). From step to step, a whole frame [build 132's
    /// moving flight: it was from the last frame handed over to this frame's start, the sliver between frames, so the
    /// camera had flown at about 1% of its speed since the rebuild began (David: "too slow to notice")].
    pub fn step(&mut self, now: Instant) -> f32 {
        let seconds = self.stepped.map_or(0.0, |last| now.duration_since(last).as_secs_f32().min(0.1));
        self.stepped = Some(now);
        seconds
    }

    /// The recent frames' spread.
    pub fn recent(&self) -> Spread {
        Spread::of(self.recent.iter().copied())
    }
}

/// A buffer the timestamps are copied to and read from without waiting: free, on its way, or ready.
struct Readback {
    buffer: wgpu::Buffer,
    state: Arc<AtomicU8>,
}

const FREE: u8 = 0;
const MAPPING: u8 = 1;
const READY: u8 = 2;

/// How many frames' timestamps can be on their way back at once.
pub const READBACKS: usize = 4;

/// The card's clock: each of the frame's passes on the card timed there, in the order they run (`wiring::card_slot`).
pub struct CardClock {
    /// How many passes it times.
    slots: usize,
    queries: wgpu::QuerySet,
    resolved: wgpu::Buffer,
    readbacks: Vec<Readback>,
    /// The readback this frame's timestamps go to (none free: this frame isn't timed).
    pending: Option<usize>,
    /// Nanoseconds a tick.
    period_ns: f32,
    /// Each pass's recent times, and the card's whole (first start to last end) after them.
    recent: Vec<VecDeque<f32>>,
}

impl CardClock {
    /// None if the card can't time its own work.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, slots: usize) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) || slots == 0 {
            return None;
        }
        let count = 2 * slots as u32;
        let size = count as u64 * 8;
        let queries = device.create_query_set(&wgpu::QuerySetDescriptor { label: Some("the card's clock"), ty: wgpu::QueryType::Timestamp, count });
        let resolved = device.create_buffer(&wgpu::BufferDescriptor { label: Some("the card's clock"), size, usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readbacks = (0..READBACKS)
            .map(|_| Readback {
                buffer: device.create_buffer(&wgpu::BufferDescriptor { label: Some("the card's clock, read back"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }),
                state: Arc::new(AtomicU8::new(FREE)),
            })
            .collect();
        let recent = vec![VecDeque::with_capacity(RECENT); slots + 1];
        Some(Self { slots, queries, resolved, readbacks, pending: None, period_ns: queue.get_timestamp_period(), recent })
    }

    /// A new frame: timed if a readback is free.
    pub fn begin_frame(&mut self) {
        self.pending = self.readbacks.iter().position(|readback| readback.state.load(Ordering::Acquire) == FREE);
    }

    /// The timestamps for a render pass (its slot), if this frame is timed.
    pub fn render_pass(&self, pass: usize) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.pending.map(|_| wgpu::RenderPassTimestampWrites { query_set: &self.queries, beginning_of_pass_write_index: Some(2 * pass as u32), end_of_pass_write_index: Some(2 * pass as u32 + 1) })
    }

    /// The timestamps for a compute pass (its slot), if this frame is timed.
    pub fn compute_pass(&self, pass: usize) -> Option<wgpu::ComputePassTimestampWrites<'_>> {
        self.pending.map(|_| wgpu::ComputePassTimestampWrites { query_set: &self.queries, beginning_of_pass_write_index: Some(2 * pass as u32), end_of_pass_write_index: Some(2 * pass as u32 + 1) })
    }

    /// After the frame's last pass: its timestamps copied out to this frame's readback.
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(k) = self.pending {
            let count = 2 * self.slots as u32;
            encoder.resolve_query_set(&self.queries, 0..count, &self.resolved, 0);
            encoder.copy_buffer_to_buffer(&self.resolved, 0, &self.readbacks[k].buffer, 0, count as u64 * 8);
        }
    }

    /// After the frame's work is handed over: this frame's readback asked for, and any that have come back read. Each
    /// frame that came back gives each slot's milliseconds, and the card's whole last.
    pub fn after_submit(&mut self, device: &wgpu::Device) -> Vec<Vec<f32>> {
        if let Some(k) = self.pending.take() {
            let state = self.readbacks[k].state.clone();
            state.store(MAPPING, Ordering::Release);
            self.readbacks[k].buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| state.store(if result.is_ok() { READY } else { FREE }, Ordering::Release));
        }
        let _ = device.poll(wgpu::PollType::Poll);
        let mut frames = Vec::new();
        for k in 0..self.readbacks.len() {
            if self.readbacks[k].state.load(Ordering::Acquire) != READY {
                continue;
            }
            let ticks: Vec<u64> = match self.readbacks[k].buffer.slice(..).get_mapped_range() {
                Ok(view) => view.as_chunks::<8>().0.iter().map(|bytes| u64::from_le_bytes(*bytes)).collect(),
                Err(_) => Vec::new(),
            };
            self.readbacks[k].buffer.unmap();
            self.readbacks[k].state.store(FREE, Ordering::Release);
            if ticks.len() != 2 * self.slots {
                continue;
            }
            let ms = |from: u64, to: u64| to.saturating_sub(from) as f32 * self.period_ns / 1_000_000.0;
            let mut times: Vec<f32> = (0..self.slots).map(|pass| ms(ticks[2 * pass], ticks[2 * pass + 1])).collect();
            times.push(ms(ticks[0], ticks[ticks.len() - 1]));
            for (recent, &time) in self.recent.iter_mut().zip(&times) {
                if recent.len() == RECENT {
                    recent.pop_front();
                }
                recent.push_back(time);
            }
            frames.push(times);
        }
        frames
    }

    /// Each slot's recent spread, and the card's whole last.
    pub fn recent(&self) -> Vec<Spread> {
        self.recent.iter().map(|recent| Spread::of(recent.iter().copied())).collect()
    }
}

/// The passes' clock on the processor: how long each pass took there (for a pass on the card, recording its work).
pub struct PassClock {
    recent: Vec<VecDeque<f32>>,
}

impl PassClock {
    pub fn new() -> Self {
        Self { recent: vec![VecDeque::with_capacity(RECENT); crate::wiring::PASSES.len()] }
    }

    /// A pass ran for `ms`.
    pub fn record(&mut self, pass: crate::wiring::PassId, ms: f32) {
        let recent = &mut self.recent[pass as usize];
        if recent.len() == RECENT {
            recent.pop_front();
        }
        recent.push_back(ms);
    }

    /// A pass's recent spread.
    pub fn recent(&self, pass: crate::wiring::PassId) -> Spread {
        Spread::of(self.recent[pass as usize].iter().copied())
    }
}

/// A pass's recent times: on the processor, and on the card if it runs there.
#[derive(Clone, Copy, Default)]
pub struct PassTimes {
    pub processor: Spread,
    pub card: Option<Spread>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// A step is the whole time from one frame's step to the next, whenever the frame between is handed over.
    #[test]
    fn a_step_is_a_whole_frame() {
        let mut clock = FrameClock::new();
        let start = Instant::now();
        assert_eq!(clock.step(start), 0.0, "nothing before the first");
        clock.shown(start + Duration::from_millis(49));
        let step = clock.step(start + Duration::from_millis(50));
        assert!((step - 0.050).abs() < 1e-4, "a 50 ms frame steps 0.05 s, not {step}");
        clock.shown(start + Duration::from_millis(1049));
        assert_eq!(clock.step(start + Duration::from_millis(1050)), 0.1, "a stall steps a tenth of a second at most");
    }
}
