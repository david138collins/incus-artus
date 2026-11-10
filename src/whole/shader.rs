//! The kit's copy for the card (`kit.wgsl`), with the processor's own tables put in front of it, so both
//! read the very same numbers. Shaders that make the world include `source()` first.

use super::tables::{ATAN, EXP2, LOG2, SIN};

const KIT: &str = include_str!("kit.wgsl");

/// The kit in the shader language: its tables, then its steps.
pub fn source() -> String {
    let table = |name: &str, values: &[i32]| {
        let listed: Vec<String> = values.iter().map(|v| v.to_string()).collect();
        format!("var<private> {name}: array<i32, {}> = array<i32, {}>({});\n", values.len(), values.len(), listed.join(", "))
    };
    [table("LOG2", &LOG2), table("EXP2", &EXP2), table("SIN", &SIN), table("ATAN", &ATAN), KIT.to_string()].concat()
}

/// The card tests take turns: each holds the card for as long as its thread lives [2026-11-09: run all at once,
/// each with a device of its own, six of sight's picture tests failed together that pass one at a time].
#[cfg(test)]
static CARD_TURN: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
thread_local! {
    /// This thread's turn at the card, once it has asked for one (a test asking twice keeps the one it has).
    static TURN: std::cell::RefCell<Option<std::sync::MutexGuard<'static, ()>>> = const { std::cell::RefCell::new(None) };
}

/// A graphics card to run a twin test on, if there is one (DirectX 12 on Windows, as the game), once it's this
/// test's turn.
#[cfg(test)]
pub fn card() -> Option<(wgpu::Device, wgpu::Queue)> {
    TURN.with(|turn| {
        let mut turn = turn.borrow_mut();
        if turn.is_none() {
            // A test that failed holding it gives it up all the same.
            *turn = Some(CARD_TURN.lock().unwrap_or_else(|failed| failed.into_inner()));
        }
    });
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    if cfg!(windows) {
        descriptor.backends = wgpu::Backends::DX12;
        // DXC, as the game uses (`runner::windows_sdk_dxc`): the old compiler gives up on the recipe.
        let folder = std::path::Path::new(r"C:\Program Files (x86)\Windows Kits\10\Redist\D3D\x64");
        let dxc = folder.join("dxcompiler.dll");
        if dxc.is_file() && folder.join("dxil.dll").is_file() {
            descriptor.backend_options.dx12.shader_compiler = wgpu::Dx12Compiler::DynamicDxc { dxc_path: dxc.to_string_lossy().into_owned() };
        }
    }
    let instance = wgpu::Instance::new(descriptor);
    // `INCUS_TWIN_CARD=software`: Microsoft's WARP, the software DirectX 12, a different card altogether
    // [Counsel 2026-11-05: a second card for the twin test at no cost].
    let software = std::env::var("INCUS_TWIN_CARD").is_ok_and(|card| card == "software");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: software, ..Default::default() })).ok()?;
    eprintln!("The twin test's card: {} ({:?})", adapter.get_info().name, adapter.get_info().device_type);
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("the twin test"), required_limits: big_buffers(&adapter), ..Default::default() })).ok()
}

/// The storage buffers sight's march reads at once (`march.wgsl`): one more than wgpu's usual 8 since the placed
/// lights (build 95). Every desktop card asks for far more [David 2026-11-09: "probably getting more is better if
/// its more performant or clean"].
pub const STORAGE_BUFFERS_NEEDED: u32 = 10;

/// wgpu's usual limits, but buffers as big as the card allows (a world's pack is larger than the usual 128 MB
/// [2026-11-05: 182 MB, the default bake's maps at full size]), and as many storage buffers a shader as it allows.
pub fn big_buffers(adapter: &wgpu::Adapter) -> wgpu::Limits {
    let card = adapter.limits();
    wgpu::Limits {
        max_storage_buffer_binding_size: card.max_storage_buffer_binding_size,
        max_buffer_size: card.max_buffer_size,
        max_storage_buffers_per_shader_stage: card.max_storage_buffers_per_shader_stage,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::card;
    use super::source;

    /// The steps the twin test runs, by number: the card's `twin` below does the same.
    fn on_the_processor(op: i32, a: i32, b: i32, c: i32) -> i32 {
        let (a, b, c) = (Fix(a), Fix(b), Fix(c));
        match op {
            0 => (a + b).0,
            1 => (a - b).0,
            2 => (a * b).0,
            3 => (a / b).0,
            4 => sqrt(a).0,
            5 => hypot(a, b).0,
            6 => log2(a).0,
            7 => exp2(a).0,
            8 => pow(a, b).0,
            9 => sin(a.0).0,
            10 => cos(a.0).0,
            11 => atan2(a, b),
            12 => smooth(a).0,
            13 => mix(a, b, c).0,
            14 => Pos(a.0).per(Pos(b.0)).0,
            _ => 0,
        }
    }

    const TWIN: &str = r#"
struct Case { op: i32, a: i32, b: i32, c: i32 }
@group(0) @binding(0) var<storage, read> cases: array<Case>;
@group(0) @binding(1) var<storage, read_write> results: array<i32>;

@compute @workgroup_size(64)
fn twin(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= arrayLength(&cases)) {
        return;
    }
    let c = cases[i];
    var r = 0;
    switch c.op {
        case 0: { r = c.a + c.b; }
        case 1: { r = c.a - c.b; }
        case 2: { r = fix_mul(c.a, c.b); }
        case 3: { r = fix_div(c.a, c.b); }
        case 4: { r = fix_sqrt(c.a); }
        case 5: { r = fix_hypot(c.a, c.b); }
        case 6: { r = fix_log2(c.a); }
        case 7: { r = fix_exp2(c.a); }
        case 8: { r = fix_pow(c.a, c.b); }
        case 9: { r = fix_sin(c.a); }
        case 10: { r = fix_cos(c.a); }
        case 11: { r = fix_atan2(c.a, c.b); }
        case 12: { r = fix_smooth(c.a); }
        case 13: { r = fix_mix(c.a, c.b, c.c); }
        case 14: { r = pos_per(c.a, c.b); }
        default: { r = 0; }
    }
    results[i] = r;
}
"#;

    /// Runs `cases` (op, a, b, c) on the card and gives back its answers.
    fn on_the_card(device: &wgpu::Device, queue: &wgpu::Queue, cases: &[[i32; 4]]) -> Vec<i32> {
        use wgpu::util::DeviceExt;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("the twin test"), source: wgpu::ShaderSource::Wgsl((source() + TWIN).into()) });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("the twin test"), layout: None, module: &module, entry_point: Some("twin"), compilation_options: Default::default(), cache: None });
        let bytes: Vec<u8> = cases.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
        let input = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("cases"), contents: &bytes, usage: wgpu::BufferUsages::STORAGE });
        let size = cases.len() as u64 * 4;
        let output = device.create_buffer(&wgpu::BufferDescriptor { label: Some("results"), size, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false });
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: Some("readback"), size, usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: input.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: output.as_entire_binding() }],
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
        view.chunks_exact(4).map(|b| i32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect()
    }

    /// The cases: the edges of every step (0, ±1, the range's ends, powers of two) and many made by a
    /// fixed shuffle, in the ranges each step is used in.
    fn cases() -> Vec<[i32; 4]> {
        let mut state = 0x9E37_79B9u32;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as i32
        };
        let edges = [0, 1, -1, ONE, -ONE, ONE / 2, i32::MAX, i32::MIN, i32::MAX - 1, i32::MIN + 1, 1 << 20, -(1 << 20), 65_535, 65_537, 3, -3];
        let mut cases = Vec::new();
        for op in 0..15 {
            for &a in &edges {
                for &b in &edges {
                    cases.push([op, a, b, edges[(a as u32 % 16) as usize]]);
                }
            }
            for _ in 0..4000 {
                let (a, b, c) = (next(), next(), next());
                // Each step's useful range, as well as anything at all.
                let (a, b) = match op {
                    6 | 8 => (a & 0x7FFF_FFFF, (b >> 12) & 0x000F_FFFF),
                    7 => (a >> 10, b),
                    12 | 13 => (a >> 14, b),
                    _ => (a, b),
                };
                cases.push([op, a, b, c >> 15]);
            }
        }
        cases
    }

    /// The twin test for the kit: every step on the card, bit for bit the processor's. Skipped (with a
    /// note) where there's no graphics card to run it on.
    #[test]
    fn the_card_and_the_processor_agree_bit_for_bit() {
        let Some((device, queue)) = card() else {
            eprintln!("No graphics card: the kit's twin test is skipped");
            return;
        };
        let cases = cases();
        let answers = on_the_card(&device, &queue, &cases);
        let names = ["+", "−", "×", "÷", "sqrt", "hypot", "log2", "exp2", "pow", "sin", "cos", "atan2", "smooth", "mix", "per"];
        let mut wrong = Vec::new();
        for (case, &card) in cases.iter().zip(&answers) {
            let processor = on_the_processor(case[0], case[1], case[2], case[3]);
            if card != processor && wrong.len() < 20 {
                wrong.push(format!("{}({}, {}, {}): card {card}, processor {processor}", names[case[0] as usize], case[1], case[2], case[3]));
            }
        }
        assert!(wrong.is_empty(), "the card and the processor disagree:\n{}", wrong.join("\n"));
    }
}
