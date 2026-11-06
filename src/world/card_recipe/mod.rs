//! The recipe on the card (the regroup, phase 2.3): `whole_recipe.rs` in the shader language
//! (`recipe.wgsl`), reading the world packed into one list of whole numbers (`Pack`), the very numbers the
//! processor reads. Proved by the twin test: the card and the processor bit for bit, every time.
//!
//! Built a piece at a time, each with its own twin test: the noise, curves and noise layers first.

use crate::whole::Fix;
use crate::world::whole_recipe::{WholeCurve, WholeNoise};
use crate::worldgen::noise::{ARROWS, Fade};

const RECIPE: &str = include_str!("recipe.wgsl");

/// The recipe in the shader language: the kit, the noise's arrows, then the recipe's steps.
pub fn source() -> String {
    let arrows: Vec<String> = ARROWS.iter().map(|(x, y)| format!("vec2<i32>({x}, {y})")).collect();
    [crate::whole::shader::source(), format!("var<private> ARROWS: array<vec2<i32>, 64> = array<vec2<i32>, 64>({});\n", arrows.join(", ")), RECIPE.to_string()].concat()
}

/// The world packed for the card: one list of whole numbers, each part's start noted where it's packed.
#[derive(Clone, Debug, Default)]
pub struct Pack {
    pub words: Vec<i32>,
}

fn fade_number(fade: Fade) -> i32 {
    match fade {
        Fade::Smooth => 0,
        Fade::Soft => 1,
        Fade::Straight => 2,
    }
}

impl Pack {
    fn at(&self) -> u32 {
        self.words.len() as u32
    }

    fn push(&mut self, values: &[i32]) {
        self.words.extend_from_slice(values);
    }

    /// A curve: its stretches, its single value, then each stretch's nine numbers.
    pub fn curve(&mut self, curve: &WholeCurve) -> u32 {
        let at = self.at();
        self.push(&[curve.stretches.len() as i32, curve.single.0]);
        for s in &curve.stretches {
            self.push(&[s.x0.0, s.x1.0, s.y0.0, s.y1.0, s.h.0, s.hm0.0, s.hm1.0, s.power.is_some() as i32, s.power.unwrap_or(Fix::ZERO).0]);
        }
        at
    }

    /// A noise layer: its fifteen numbers, then each octave's key, size and weight.
    pub fn noise(&mut self, noise: &WholeNoise) -> u32 {
        let at = self.at();
        let sh = &noise.shaping;
        self.push(&[
            noise.offset_north.0,
            noise.offset_east.0,
            noise.cos.0,
            noise.sin.0,
            noise.over_stretch.0,
            noise.turn_cos.0,
            noise.turn_sin.0,
            noise.octaves.len() as i32,
            noise.solo.map_or(-1, |k| k as i32),
            fade_number(noise.fade),
            sh.contrast.0,
            sh.brightness.0,
            sh.power.is_some() as i32,
            sh.power.unwrap_or(Fix::ZERO).0,
            sh.invert as i32,
        ]);
        for octave in &noise.octaves {
            self.push(&[octave.key as i32, octave.size.0, octave.weight.0]);
        }
        at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::whole::Pos;
    use crate::world::Spline;
    use crate::worldgen::noise::{PerlinSettings, whole_perlin_at, whole_perlin_with_slope_at};

    /// Runs `entry` over `cases` (four numbers each) on the card, with the world packed as `data`; its
    /// answers, one each.
    fn on_the_card(entry: &str, data: &[i32], cases: &[[i32; 4]]) -> Option<Vec<i32>> {
        use wgpu::util::DeviceExt;
        let (device, queue) = crate::whole::shader::card()?;
        let twin = format!(
            "
struct Case {{ a: i32, b: i32, c: i32, d: i32 }}
@group(0) @binding(1) var<storage, read> cases: array<Case>;
@group(0) @binding(2) var<storage, read_write> results: array<i32>;
@compute @workgroup_size(64)
fn twin(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= arrayLength(&cases)) {{
        return;
    }}
    results[i] = {entry}(cases[i]);
}}
"
        );
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the recipe's twin test"), source: wgpu::ShaderSource::Wgsl((source() + TESTS + &twin).into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: None, layout: None, module: &module, entry_point: Some("twin"), compilation_options: Default::default(), cache: None });
        let bytes = |v: &[i32]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let mut packed = data.to_vec();
        if packed.is_empty() {
            packed.push(0);
        }
        let world = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("the world"), contents: &bytes(&packed), usage: wgpu::BufferUsages::STORAGE });
        let flat: Vec<i32> = cases.iter().flatten().copied().collect();
        let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("cases"), contents: &bytes(&flat), usage: wgpu::BufferUsages::STORAGE });
        let size = cases.len() as u64 * 4;
        let output = device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: world.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: input.as_entire_binding() }, wgpu::BindGroupEntry { binding: 2, resource: output.as_entire_binding() }],
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((cases.len() as u32).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&output, 0, &readback, 0, size);
        queue.submit([encoder.finish()]);
        readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).expect("the card finished");
        let view = readback.slice(..).get_mapped_range().expect("the results");
        Some(view.chunks_exact(4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect())
    }

    /// The test's entries into the recipe, case by case.
    const TESTS: &str = "
fn noise_and_curves(c: Case) -> i32 {
    switch c.a {
        case 0: { return curve_at(u32(c.b), c.c); }
        case 1: { return curve_steepness(u32(c.b), c.c); }
        case 2: { return noise_at(u32(c.b), c.c, c.d); }
        case 3: { return noise_raw(u32(c.b), c.c, c.d); }
        case 4: { return perlin_at(0x51u, 0, c.c, c.d); }
        case 5: { return perlin_with_slope_at(0x51u, c.c, c.d).x; }
        case 6: { return perlin_with_slope_at(0x51u, c.c, c.d).y; }
        case 7: { return perlin_with_slope_at(0x51u, c.c, c.d).z; }
        default: { return 0; }
    }
}
";

    fn shuffle() -> impl FnMut() -> i32 {
        let mut state = 0x2545_F491u32;
        move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as i32
        }
    }

    /// The noise, the curves and the noise layers: the card bit for bit the processor.
    #[test]
    fn the_card_reads_the_noise_and_curves_as_the_processor_does() {
        let mut spline = Spline::through(&[(0.0, 0.0), (0.3, 0.6), (0.7, 0.65), (1.0, 1.0)]);
        spline.bends = vec![0.0, 0.5, -0.4];
        let curve = spline.curve().whole();
        let mut settings = PerlinSettings { scale: 410.0, octaves: 5, persistence: 0.45, lacunarity: 2.3, stretch: 1.7, stretch_angle: 33.0, octave_turn: 21.0, offset_north: 90.0, ..PerlinSettings::default() };
        (settings.output.contrast, settings.output.brightness, settings.output.curve) = (2.35, -0.33, 2.3);
        let noise = WholeNoise::new(&settings, 77);
        let mut pack = Pack::default();
        let (curve_at, noise_at) = (pack.curve(&curve), pack.noise(&noise));
        let mut next = shuffle();
        let mut cases = Vec::new();
        for _ in 0..4000 {
            let x = (next() & 0x1_FFFF) - 0x8000;
            cases.push([0, curve_at as i32, x, 0]);
            cases.push([1, curve_at as i32, x, 0]);
            let (north, east) = (next() >> 9, next() >> 9);
            cases.push([2, noise_at as i32, north, east]);
            cases.push([3, noise_at as i32, north, east]);
            for kind in 4..8 {
                cases.push([kind, 0, next(), next()]);
            }
        }
        let Some(answers) = on_the_card("noise_and_curves", &pack.words, &cases) else {
            eprintln!("No graphics card: the recipe's twin test is skipped");
            return;
        };
        let key = 0x51u32;
        let mut wrong = Vec::new();
        for (c, &card) in cases.iter().zip(&answers) {
            let processor = match c[0] {
                0 => curve.at(Fix(c[2])).0,
                1 => curve.steepness(Fix(c[2])).0,
                2 => noise.at(Pos(c[2]), Pos(c[3])).0,
                3 => noise.raw(Pos(c[2]), Pos(c[3])).0,
                4 => whole_perlin_at(key, Fade::Smooth, Fix(c[2]), Fix(c[3])).0,
                5 => whole_perlin_with_slope_at(key, Fix(c[2]), Fix(c[3])).0.0,
                6 => whole_perlin_with_slope_at(key, Fix(c[2]), Fix(c[3])).1.0,
                _ => whole_perlin_with_slope_at(key, Fix(c[2]), Fix(c[3])).2.0,
            };
            if card != processor && wrong.len() < 12 {
                wrong.push(format!("kind {} ({}, {}): card {card}, processor {processor}", c[0], c[2], c[3]));
            }
        }
        assert!(wrong.is_empty(), "the card and the processor disagree:\n{}", wrong.join("\n"));
    }
}
