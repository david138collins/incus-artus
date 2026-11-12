//! Ballast: made-up work for the card each frame, like sandbags on a test flight [David 2026-11-05:
//! "give it crayons and paper along with a blank screen, just until it has enough load that it becomes
//! linear all the way to full use"]. It lets the frame loop be measured under a realistic load before
//! the real picture exists, and shows whether real-sized work keeps the card awake. **Developer builds
//! only:** this file isn't compiled into a player's copy at all. Switched on with
//! `INCUS_BALLAST=<ms>`, or by the test window (`INCUS_TEST_BALLAST`, `soak.rs`).
//!
//! **Sized in work, not time.** The card picks its clock by how busy it is, so ballast held to so many
//! milliseconds would chase the clock and hide the dozing. Instead a unit of ballast is timed at full
//! clock (the fastest it's seen going, probed in the first seconds, best while uncapped when the card
//! runs flat out), and each frame gets a fixed amount of work: `<ms>` worth at full clock, less the
//! real work, also in full-clock terms. A dozing card takes longer over the same work, so the dozing
//! shows. As the real picture grows, the ballast shrinks, and it's gone once real work passes it.
//!
//! The work reads and writes a 256 MB buffer at random as well as doing arithmetic, so the card's
//! memory is kept busy too, as real drawing does. The card's own timestamps measure the ballast and
//! the real work (the panel's pass), read back a few frames later without waiting.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

/// The buffer the ballast works over: this many `vec4<f32>` (256 MB, more than the card's cache).
const ELEMENTS: u32 = 16 * 1024 * 1024;
/// A unit of ballast: a workgroup of 64 threads, each this many reads and sums.
const STEPS: u32 = 64;
/// While calibrating, this many units a frame, and for this many frames.
const PROBE: u32 = 4096;
const CALIBRATION_FRAMES: usize = 600;
/// The timestamps: the ballast's pass (0, 1) and the panel's (2, 3).
const QUERIES: u32 = 4;

const SHADER: &str = r#"
struct Params { steps: u32, elements: u32, row: u32, seed: u32 }
@group(0) @binding(0) var<storage, read_write> data: array<vec4<f32>>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = (id.x + id.y * params.row + params.seed) % params.elements;
    var v = data[i];
    var j = i;
    for (var k = 0u; k < params.steps; k++) {
        // A read from far away, then sums that depend on it, so nothing can be left out.
        j = (j * 1664525u + 1013904223u) % params.elements;
        let w = data[j];
        v = fract(v * 1.0001 + w * 0.37 + vec4<f32>(0.5));
    }
    data[i] = v;
}
"#;

/// The ballast's state: its pass on the card, the timestamps, and the calibration.
pub struct Ballast {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    params: wgpu::Buffer,
    queries: wgpu::QuerySet,
    resolved: wgpu::Buffer,
    readbacks: Vec<Readback>,
    /// Nanoseconds per timestamp tick.
    period_ns: f32,
    /// Milliseconds per unit, as seen while calibrating; the fastest is the full clock's.
    samples: Vec<f32>,
    /// Milliseconds per unit at full clock, once calibrated.
    per_unit: Option<f32>,
    /// The real work (the panel's pass, ms as it ran), and how much slower than full clock the card is
    /// running (1 or more), both smoothed over a few frames.
    real_ms: f32,
    slowdown: f32,
    /// The readback this frame's timestamps go to, and how many units this frame has.
    pending: Option<usize>,
    units: u32,
    frame: u32,
}

/// A buffer the timestamps are copied to and read from without waiting: free, on its way, or ready.
struct Readback {
    buffer: wgpu::Buffer,
    state: Arc<AtomicU8>,
    units: u32,
    target: f32,
}

const FREE: u8 = 0;
const MAPPING: u8 = 1;
const READY: u8 = 2;

/// The ballast wanted at start (`INCUS_BALLAST=<ms>`), or by the test window.
pub fn wanted() -> bool {
    std::env::var("INCUS_BALLAST").is_ok() || std::env::var("INCUS_TEST_BALLAST").is_ok()
}

/// `INCUS_BALLAST=<ms>`: the ballast's size, in milliseconds at full clock.
pub fn from_start() -> f32 {
    std::env::var("INCUS_BALLAST").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0)
}

impl Ballast {
    /// None if the card can't time its own work (the timestamps are what the ballast is measured by).
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("ballast"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("ballast"), layout: None, module: &module, entry_point: Some("main"), compilation_options: Default::default(), cache: None });
        let data = device.create_buffer(&wgpu::BufferDescriptor { label: Some("ballast data"), size: ELEMENTS as u64 * 16, usage: wgpu::BufferUsages::STORAGE, mapped_at_creation: false });
        let params = device.create_buffer(&wgpu::BufferDescriptor { label: Some("ballast params"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ballast"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: data.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: params.as_entire_binding() }],
        });
        let queries = device.create_query_set(&wgpu::QuerySetDescriptor { label: Some("ballast timestamps"), ty: wgpu::QueryType::Timestamp, count: QUERIES });
        let size = QUERIES as u64 * 8;
        let resolved = device.create_buffer(&wgpu::BufferDescriptor { label: Some("ballast timestamps"), size, usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readbacks = (0..4)
            .map(|_| Readback {
                buffer: device.create_buffer(&wgpu::BufferDescriptor { label: Some("ballast readback"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false }),
                state: Arc::new(AtomicU8::new(FREE)),
                units: 0,
                target: 0.0,
            })
            .collect();
        Some(Self { pipeline, bind_group, params, queries, resolved, readbacks, period_ns: queue.get_timestamp_period(), samples: Vec::new(), per_unit: None, real_ms: 0.0, slowdown: 1.0, pending: None, units: 0, frame: 0 })
    }

    /// The ballast's pass, recorded into the frame before the panel's: `target` milliseconds' worth
    /// at full clock, less the real work. While calibrating, a fixed probe instead.
    pub fn encode(&mut self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder, target: f32) {
        self.frame = self.frame.wrapping_add(1);
        self.units = match self.per_unit {
            None => PROBE,
            Some(_) if target <= 0.0 => 0,
            Some(per_unit) => ((target - self.real_ms / self.slowdown).max(0.0) / per_unit).round() as u32,
        };
        // A free readback for this frame's timestamps (none free: this frame isn't timed).
        self.pending = self.readbacks.iter().position(|readback| readback.state.load(Ordering::Acquire) == FREE);
        let timestamps = self.pending.map(|_| wgpu::ComputePassTimestampWrites { query_set: &self.queries, beginning_of_pass_write_index: Some(0), end_of_pass_write_index: Some(1) });
        let row = 65535u32;
        let (x, y) = (self.units.min(row), self.units.div_ceil(row));
        let params = [STEPS, ELEMENTS, row * 64, self.frame.wrapping_mul(2_654_435_761) % ELEMENTS];
        queue.write_buffer(&self.params, 0, &params.iter().flat_map(|value| value.to_le_bytes()).collect::<Vec<u8>>());
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("ballast"), timestamp_writes: timestamps });
        if self.units > 0 {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(x, y.max(1), 1);
        }
        drop(pass);
        if let Some(k) = self.pending {
            self.readbacks[k].units = self.units;
            self.readbacks[k].target = target;
        }
    }

    /// The timestamps for the panel's pass (the real work), if this frame is timed.
    pub fn panel_timestamps(&self) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.pending.map(|_| wgpu::RenderPassTimestampWrites { query_set: &self.queries, beginning_of_pass_write_index: Some(2), end_of_pass_write_index: Some(3) })
    }

    /// After the panel's pass: the timestamps copied out to this frame's readback.
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(k) = self.pending {
            encoder.resolve_query_set(&self.queries, 0..QUERIES, &self.resolved, 0);
            encoder.copy_buffer_to_buffer(&self.resolved, 0, &self.readbacks[k].buffer, 0, QUERIES as u64 * 8);
        }
    }

    /// After the frame's work is handed over: this frame's readback asked for, and any that have
    /// come back read, without waiting.
    pub fn after_submit(&mut self, device: &wgpu::Device) {
        if let Some(k) = self.pending.take() {
            let state = self.readbacks[k].state.clone();
            state.store(MAPPING, Ordering::Release);
            self.readbacks[k].buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| state.store(if result.is_ok() { READY } else { FREE }, Ordering::Release));
        }
        let _ = device.poll(wgpu::PollType::Poll);
        for k in 0..self.readbacks.len() {
            if self.readbacks[k].state.load(Ordering::Acquire) != READY {
                continue;
            }
            let ticks: Vec<u64> = match self.readbacks[k].buffer.slice(..).get_mapped_range() {
                Ok(view) => view.chunks_exact(8).map(|bytes| u64::from_le_bytes(bytes.try_into().unwrap_or([0; 8]))).collect(),
                Err(_) => Vec::new(),
            };
            self.readbacks[k].buffer.unmap();
            self.readbacks[k].state.store(FREE, Ordering::Release);
            if ticks.len() == QUERIES as usize {
                let ms = |from: u64, to: u64| to.saturating_sub(from) as f32 * self.period_ns / 1_000_000.0;
                let (units, target) = (self.readbacks[k].units, self.readbacks[k].target);
                self.measured(units, target, ms(ticks[0], ticks[1]), ms(ticks[2], ticks[3]));
            }
        }
    }

    /// A frame's timings back: the calibration, the card's slowdown, and the log's figures.
    fn measured(&mut self, units: u32, target: f32, ballast_ms: f32, real_ms: f32) {
        self.real_ms += (real_ms - self.real_ms) * 0.1;
        if units == 0 {
            crate::perf::ballast_frame(target, 0, 0.0, real_ms, self.slowdown);
            return;
        }
        let per_unit = ballast_ms / units as f32;
        match self.per_unit {
            None => {
                self.samples.push(per_unit);
                if self.samples.len() >= CALIBRATION_FRAMES {
                    // The fastest few: the card at full clock.
                    self.samples.sort_by(f32::total_cmp);
                    let fastest = self.samples[self.samples.len() / 50];
                    self.per_unit = Some(fastest);
                    crate::perf::ballast_calibrated(fastest, self.samples[self.samples.len() - 1] / fastest);
                }
            }
            Some(full) => {
                self.slowdown += ((per_unit / full).max(1.0) - self.slowdown) * 0.1;
                crate::perf::ballast_frame(target, units, ballast_ms, real_ms, self.slowdown);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Without its switch, there's no ballast: the card gets no made-up work.
    #[test]
    fn no_ballast_unless_it_is_asked_for() {
        if std::env::var("INCUS_BALLAST").is_err() && std::env::var("INCUS_TEST_BALLAST").is_err() {
            assert!(!wanted());
            assert_eq!(from_start(), 0.0);
        }
    }
}
