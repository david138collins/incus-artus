//! The dyno's flight (`docs/dyno.md`, the instruments): the same route over the same world every time, measured
//! frame by frame, so one build can be set against the next. Started with `INCUS_DYNO=1` (with the soak's
//! `INCUS_START_SCREEN=worldgen` and `INCUS_SELFTEST_SUITE=compiler`, so the world is the World Compiler's defaults);
//! flown by `soak.rs`, which has the game's side of it. This is the route and the sums.
//!
//! **The route**, found on the map from the middle, the same for the same map:
//! 1. **Over the land** to the highest ground within 2,500 blocks, 40 over the ground, a little down.
//! 2. **Round the peak**, half a circle 300 out.
//! 3. **Out to sea**: to the nearest sea from there and 800 on over it, 20 over the water, looking at the horizon
//!    (the longest rays there are).
//! 4. **Quick** back towards the middle at 300 blocks a second (David's "creaking when you move too quick": the near
//!    levels refilled, the bake chasing).
//! 5. **A cave mouth**: up to the nearest cave mouth, slowly, looking in (the near walk, placed light if any).
//! 6. **Home** to the middle.
//!
//! **The standing marks** (the Oracle's experiment 1, 2026-11-11), after the route: the camera held still at three
//! places that matter, each measured once the live levels round it are whole:
//! 1. **On the ground at the spawn**, a player's eye over the ground, looking north and level.
//! 2. **The mountain top**, the highest ground within 2,500 blocks, looking out from the middle, on a clear day (fog off).
//! 3. **Beside the quarter-block build** David made on 2026-11-10 (build 108), replayed from that session's event log
//!    (`kept logs/events-1794341728.log`), standing 8 blocks off its north side, looking at it.
//!
//! **Measured:** each frame's time, each pass's on the card (`sight::draw::take_card_readings`), and the loading lanes'
//! work on the card (`world::card_recipe::take_lane_times`: the tiles and the bake, outside sight's timer), kept by
//! phase (baking: the bake not done yet; baked), by leg and by mark; the temperatures and the card's clock every few
//! seconds. **The sums:** the median, 95th and 99th percentile of each, as a report in `logs/` and in the log.

use std::collections::BTreeMap;

/// One leg of the route: its name, where it ends (north, east), how high (over the ground, or a height of its own),
/// how fast (blocks a second), and how far down it looks (degrees).
#[derive(Clone, Debug, PartialEq)]
pub struct Leg {
    pub name: &'static str,
    pub to: [f32; 2],
    pub height: Height,
    pub speed: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Height {
    /// This much over the ground (or the sea).
    Above(f32),
    /// At this height.
    At(f32),
}

/// The route, from the middle, on a map whose ground is `ground(north, east)` (`None` past it), with its sea at
/// `sea`, and the cave mouths there are (each a place at the surface).
/// The highest ground within 2,500 of the middle, on a grid of 50 (the middle if there's none).
fn peak_of(ground: &dyn Fn(f32, f32) -> Option<f32>) -> [f32; 2] {
    let mut peak = ([0.0f32, 0.0], f32::MIN);
    for i in -50..=50 {
        for j in -50..=50 {
            let at = [i as f32 * 50.0, j as f32 * 50.0];
            if at[0].hypot(at[1]) > 2500.0 {
                continue;
            }
            if let Some(z) = ground(at[0], at[1])
                && z > peak.1
            {
                peak = (at, z);
            }
        }
    }
    peak.0
}

pub fn route(ground: &dyn Fn(f32, f32) -> Option<f32>, sea: f32, mouths: &[[f32; 3]]) -> Vec<Leg> {
    let mut legs = Vec::new();
    let land = |north: f32, east: f32| ground(north, east).map(|z| z.max(sea));
    // 1. The highest ground within 2,500, on a grid of 50.
    let peak = peak_of(ground);
    // Stand off 300 short of it, on the way from the middle (or north of it, if it is the middle).
    let from_middle = normalised(peak).unwrap_or([1.0, 0.0]);
    let short = [peak[0] - from_middle[0] * 300.0, peak[1] - from_middle[1] * 300.0];
    legs.push(Leg { name: "over the land", to: short, height: Height::Above(40.0), speed: 60.0, pitch: -8.0 });
    // 2. Half round it, 300 out, in steps of 30°.
    let start = (short[1] - peak[1]).atan2(short[0] - peak[0]);
    for k in 1..=6 {
        let angle = start + (k as f32 * 30.0).to_radians();
        legs.push(Leg { name: "round the peak", to: [peak[0] + 300.0 * angle.cos(), peak[1] + 300.0 * angle.sin()], height: Height::Above(40.0), speed: 60.0, pitch: -10.0 });
    }
    let here = legs.last().map_or([0.0, 0.0], |leg| leg.to);
    // 3. The nearest sea (ground 4 under the sea at least) from there, on rings of 100 out to 6,000, then 800 on.
    let mut sea_at = None;
    'rings: for ring in 1..=60 {
        let r = ring as f32 * 100.0;
        for k in 0..(ring * 8) {
            let angle = k as f32 / (ring * 8) as f32 * std::f32::consts::TAU;
            let at = [here[0] + r * angle.cos(), here[1] + r * angle.sin()];
            if ground(at[0], at[1]).is_some_and(|z| z < sea - 4.0) {
                sea_at = Some(at);
                break 'rings;
            }
        }
    }
    if let Some(at) = sea_at {
        let way = normalised([at[0] - here[0], at[1] - here[1]]).unwrap_or([1.0, 0.0]);
        legs.push(Leg { name: "out to sea", to: at, height: Height::Above(20.0), speed: 80.0, pitch: -4.0 });
        let on = [at[0] + way[0] * 800.0, at[1] + way[1] * 800.0];
        if land(on[0], on[1]).is_some() {
            legs.push(Leg { name: "out to sea", to: on, height: Height::Above(20.0), speed: 80.0, pitch: -2.0 });
        }
    }
    // 4. Quick, most of the way back towards the middle.
    let here = legs.last().map_or([0.0, 0.0], |leg| leg.to);
    legs.push(Leg { name: "quick", to: [here[0] * 0.25, here[1] * 0.25], height: Height::Above(60.0), speed: 300.0, pitch: -10.0 });
    // 5. The nearest cave mouth to the middle within 3,000: up to 25 short of it, at its height, slowly.
    let here = legs.last().map_or([0.0, 0.0], |leg| leg.to);
    let mouth = mouths.iter().filter(|m| m[0].hypot(m[1]) <= 3000.0).min_by(|a, b| a[0].hypot(a[1]).total_cmp(&b[0].hypot(b[1])));
    if let Some(mouth) = mouth {
        let way = normalised([mouth[0] - here[0], mouth[1] - here[1]]).unwrap_or([1.0, 0.0]);
        let near = [mouth[0] - way[0] * 80.0, mouth[1] - way[1] * 80.0];
        legs.push(Leg { name: "to the cave mouth", to: near, height: Height::At(mouth[2] + 30.0), speed: 120.0, pitch: -10.0 });
        legs.push(Leg { name: "at the cave mouth", to: [mouth[0] - way[0] * 25.0, mouth[1] - way[1] * 25.0], height: Height::At(mouth[2] + 3.0), speed: 15.0, pitch: -3.0 });
    }
    // 6. Home.
    legs.push(Leg { name: "home", to: [0.0, 0.0], height: Height::Above(40.0), speed: 120.0, pitch: -8.0 });
    legs
}

fn normalised(v: [f32; 2]) -> Option<[f32; 2]> {
    let length = v[0].hypot(v[1]);
    (length > 1e-3).then(|| [v[0] / length, v[1] / length])
}

/// Where the route is `seconds` in (it starts at the middle): the place across (north, east), its leg, and how far
/// along that leg (0 to 1); `None` once it's done.
pub fn along(legs: &[Leg], seconds: f32) -> Option<([f32; 2], usize, f32)> {
    let mut from = [0.0f32, 0.0];
    let mut left = seconds;
    for (k, leg) in legs.iter().enumerate() {
        let length = (leg.to[0] - from[0]).hypot(leg.to[1] - from[1]);
        let takes = length / leg.speed.max(1.0);
        if left <= takes {
            let f = if takes > 0.0 { left / takes } else { 1.0 };
            return Some(([from[0] + (leg.to[0] - from[0]) * f, from[1] + (leg.to[1] - from[1]) * f], k, f));
        }
        left -= takes;
        from = leg.to;
    }
    None
}

// ── The standing marks ──

/// A place the camera is held still to be measured: where (north, east; the eye a player's height over the ground
/// there), which way it looks, whether the day is clear (fog off), and what's built there first.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub name: &'static str,
    pub at: [f32; 2],
    /// Compass degrees (0 north, 90 east).
    pub heading: f32,
    /// Degrees down (negative) or up, unless it looks at `look_at`.
    pub pitch: f32,
    /// A place looked at (north, east, height): the pitch then worked out from the eye.
    pub look_at: Option<[f32; 3]>,
    /// The clear day: fog off while it's measured.
    pub clear: bool,
    /// Placings replayed before it's measured (the quarter-block build).
    pub build: Vec<Placing>,
    /// How far over a player's eye the camera stands (blocks; 0 a player's own eye, over the ground or the sea).
    pub above: f32,
    /// Measured standing (the marks), or only pictured (the references' other places).
    pub measured: bool,
}

/// The references' places besides the marks (`INCUS_DYNO_REFERENCES`: pictures of old to set the new engine's
/// beside), only pictured: high over the spawn looking out, low over the sea at the route's far point looking at the
/// horizon (`sea`, if the route has one), and the mountain top again with fog (the marks' is the clear day).
pub fn reference_places(spawn: [i32; 2], top: &Mark, sea: Option<[f32; 2]>) -> Vec<Mark> {
    let place = |name, at, heading, pitch, clear, above| Mark { name, at, heading, pitch, look_at: None, clear, build: Vec::new(), above, measured: false };
    let mut places = vec![place("high over the spawn", [spawn[0] as f32 + 0.5, spawn[1] as f32 + 0.5], 0.0, -30.0, false, 400.0)];
    if let Some(sea) = sea {
        let heading = if sea[0].hypot(sea[1]) > 1.0 { sea[1].atan2(sea[0]).to_degrees().rem_euclid(360.0) } else { 0.0 };
        places.push(place("low over the sea", sea, heading, -2.0, false, 20.0));
    }
    places.push(place("the mountain top with fog", top.at, top.heading, top.pitch, false, 0.0));
    places
}

/// The times of day the references are pictured at: a name and the sun's height (degrees).
pub const REFERENCE_TIMES: [(&str, f32); 4] = [("noon", 40.0), ("low sun", 10.0), ("dusk", -4.0), ("night", -25.0)];

/// One placing as the event log tells it (`world/edits.rs`, an "edit request"): a whole block at a cell, or a bar of a
/// block from its corner (half blocks) along an axis (0 north–south, 1 east–west, 2 upright), in a cell.
#[derive(Clone, Debug, PartialEq)]
pub enum Placing {
    Block { cell: [i32; 3], block: String },
    Bar { cell: [i32; 3], at: [i32; 3], axis: u8, block: String },
}

impl Placing {
    pub fn cell(&self) -> [i32; 3] {
        match self {
            Placing::Block { cell, .. } | Placing::Bar { cell, .. } => *cell,
        }
    }
}

/// The placings in an event log, in order (blocks and bars; other edits aren't read).
pub fn placings(log: &str) -> Vec<Placing> {
    let three = |text: &str| -> Option<[i32; 3]> {
        let mut numbers = text.split_whitespace().map(|word| word.parse::<i32>().ok());
        Some([numbers.next()??, numbers.next()??, numbers.next()??])
    };
    log.lines()
        .filter(|line| line.contains("edit request"))
        .filter_map(|line| {
            let what = line.split_once("asks to place ")?.1;
            if let Some(bar) = what.strip_prefix("a ").filter(|rest| rest.contains(" bar of ")) {
                let (axis, rest) = bar.split_once(" bar of ")?;
                let axis = ["north–south", "east–west", "upright"].iter().position(|name| *name == axis)? as u8;
                let (block, rest) = rest.split_once(" from ")?;
                let (at, cell) = rest.split_once(" (half blocks) at ")?;
                Some(Placing::Bar { cell: three(cell)?, at: three(at)?, axis, block: block.to_string() })
            } else {
                let (block, cell) = what.split_once(" at ")?;
                // Octants, blocks on the octant grid and objects say more; only plain blocks here.
                (!block.contains(' ')).then_some(())?;
                Some(Placing::Block { cell: three(cell)?, block: block.to_string() })
            }
        })
        .collect()
}

/// The standing marks on a map whose ground is `ground(north, east)`, with the spawn at `spawn` (a block column) and
/// the build's placings (none: no build mark).
pub fn marks(ground: &dyn Fn(f32, f32) -> Option<f32>, spawn: [i32; 2], build: Vec<Placing>) -> Vec<Mark> {
    let mut marks = Vec::new();
    // 1. On the ground at the spawn, looking north and level.
    marks.push(Mark { name: "on the ground at the spawn", at: [spawn[0] as f32 + 0.5, spawn[1] as f32 + 0.5], heading: 0.0, pitch: 0.0, look_at: None, clear: false, build: Vec::new(), above: 0.0, measured: true });
    // 2. The mountain top: the route's peak, then its highest block column within 50, looking out from the middle.
    let rough = peak_of(ground);
    let mut top = (rough, f32::MIN);
    for i in -50..=50 {
        for j in -50..=50 {
            let at = [rough[0] + i as f32, rough[1] + j as f32];
            if let Some(z) = ground(at[0], at[1])
                && z > top.1
            {
                top = (at, z);
            }
        }
    }
    let top = [top.0[0].floor() + 0.5, top.0[1].floor() + 0.5];
    let heading = if top[0].hypot(top[1]) > 1.0 { top[1].atan2(top[0]).to_degrees().rem_euclid(360.0) } else { 0.0 };
    marks.push(Mark { name: "the mountain top on a clear day", at: top, heading, pitch: -5.0, look_at: None, clear: true, build: Vec::new(), above: 0.0, measured: true });
    // 3. Beside the build: 8 blocks off its north side, half way along it, looking at its middle.
    if !build.is_empty() {
        let cells: Vec<[i32; 3]> = build.iter().map(Placing::cell).collect();
        let least = cells.iter().fold([i32::MAX; 3], |a, c| [a[0].min(c[0]), a[1].min(c[1]), a[2].min(c[2])]);
        let most = cells.iter().fold([i32::MIN; 3], |a, c| [a[0].max(c[0]), a[1].max(c[1]), a[2].max(c[2])]);
        let middle = [(least[0] + most[0] + 1) as f32 / 2.0, (least[1] + most[1] + 1) as f32 / 2.0, (least[2] + most[2] + 1) as f32 / 2.0];
        let at = [most[0] as f32 + 1.0 + 8.0, middle[1]];
        marks.push(Mark { name: "beside the quarter-block build", at, heading: 180.0, pitch: 0.0, look_at: Some(middle), clear: false, build, above: 0.0, measured: true });
    }
    marks
}

/// How long the route takes (seconds).
pub fn duration(legs: &[Leg]) -> f32 {
    let mut from = [0.0f32, 0.0];
    let mut total = 0.0;
    for leg in legs {
        total += (leg.to[0] - from[0]).hypot(leg.to[1] - from[1]) / leg.speed.max(1.0);
        from = leg.to;
    }
    total
}

// ── The sums ──

/// What's kept for a phase, a leg or a mark: each frame's time (ms), sight's on the card and each pass's (ms), and the
/// loading lanes' work on the card.
#[derive(Default, Clone, Debug)]
pub struct Samples {
    pub frames: Vec<f32>,
    pub sight: Vec<f32>,
    pub passes: BTreeMap<String, Vec<f32>>,
    /// Beside each of sight's readings, the lanes' work on the card since the last reading (ms; the lanes' clock reads
    /// a pass when it's done, so this is the frame it was read in, not the one it ran in), and the two together.
    pub lanes: Vec<f32>,
    pub whole: Vec<f32>,
    /// The lanes' work by kind (`world::card_recipe::LANE_KINDS`), all told (ms), and the pieces timed.
    pub lane_kinds: [f64; 3],
    pub lane_pieces: u64,
}

impl Samples {
    pub fn frame(&mut self, ms: f32) {
        self.frames.push(ms);
    }

    pub fn card(&mut self, sight: f32, passes: &[(String, f32)]) {
        self.sight.push(sight);
        for (name, ms) in passes {
            self.passes.entry(name.clone()).or_default().push(*ms);
        }
    }

    /// One of sight's readings with the lanes' work since the last (`card` takes the reading itself).
    pub fn with_lanes(&mut self, sight: f32, lanes: f32) {
        self.lanes.push(lanes);
        self.whole.push(sight + lanes);
    }

    /// The lanes' work read this frame, by kind (ms), and its pieces.
    pub fn lane_work(&mut self, by_kind: [f32; 3], pieces: u64) {
        for (total, ms) in self.lane_kinds.iter_mut().zip(by_kind) {
            *total += ms as f64;
        }
        self.lane_pieces += pieces;
    }
}

/// The value `share` of the way up the sorted values (0.5 the median, 0.95 the 95th percentile).
pub fn percentile(values: &[f32], share: f32) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let k = ((sorted.len() - 1) as f32 * share).round() as usize;
    Some(sorted[k.min(sorted.len() - 1)])
}

/// A line for `samples`: frames (count, fps from the median, median, 95th, 99th and 99.9th, and the worst, ms), sight
/// on the card (median, 95th and 99th), the card's whole with the lanes' work (median, 95th, 99th), the lanes' work
/// (by kind, a second of flight, and its share of the card's), and each pass's median.
pub fn line(samples: &Samples) -> String {
    let p = |values: &[f32], share: f32| percentile(values, share).map_or("-".to_string(), |v| format!("{v:.2}"));
    let fps = percentile(&samples.frames, 0.5).map_or("-".to_string(), |ms| format!("{:.0}", 1000.0 / ms.max(1e-3)));
    let passes: Vec<String> = samples.passes.iter().map(|(name, values)| format!("{name} {}", p(values, 0.5))).collect();
    let seconds = samples.frames.iter().map(|&ms| ms as f64).sum::<f64>() / 1000.0;
    let lanes_total: f64 = samples.lane_kinds.iter().sum();
    let sight_total: f64 = samples.sight.iter().map(|&ms| ms as f64).sum();
    let by_kind: Vec<String> = crate::world::card_recipe::LANE_KINDS.iter().zip(samples.lane_kinds).map(|(name, ms)| format!("{name} {:.1}", ms / seconds.max(1e-3))).collect();
    let share = if lanes_total + sight_total > 0.0 { format!("{:.0}%", 100.0 * lanes_total / (lanes_total + sight_total)) } else { "-".into() };
    format!(
        "{} frames, {fps} fps (median); frame ms median {}, 95th {}, 99th {}, 99.9th {}, worst {}; sight on the card ms median {}, 95th {}, 99th {}; the card's whole (sight and the lanes) ms median {}, 95th {}, 99th {}; the lanes' work on the card {} pieces, ms a second {} ({:.1} in all), {share} of the card's; passes (medians, ms): {}",
        samples.frames.len(),
        p(&samples.frames, 0.5),
        p(&samples.frames, 0.95),
        p(&samples.frames, 0.99),
        p(&samples.frames, 0.999),
        p(&samples.frames, 1.0),
        p(&samples.sight, 0.5),
        p(&samples.sight, 0.95),
        p(&samples.sight, 0.99),
        p(&samples.whole, 0.5),
        p(&samples.whole, 0.95),
        p(&samples.whole, 0.99),
        samples.lane_pieces,
        by_kind.join(", "),
        lanes_total / seconds.max(1e-3),
        passes.join(", ")
    )
}

/// Sight on the card for the march and its twin side by side (`INCUS_SIGHT_TWIN`, frame by frame): each one's
/// median, 95th and 99th (ms) and readings, and the twin's against the march's (per cent: below 0 the twin's faster).
pub fn twin_line(march: &Samples, twin: &Samples) -> String {
    let show = |values: &[f32], share: f32| percentile(values, share).map_or("-".to_string(), |v| format!("{v:.2}"));
    let against = |share: f32| match (percentile(&march.sight, share), percentile(&twin.sight, share)) {
        (Some(a), Some(b)) if a > 0.0 => format!("{:+.1}%", (b / a - 1.0) * 100.0),
        _ => "-".to_string(),
    };
    format!(
        "march median {}, 95th {}, 99th {} ({} readings); twin median {}, 95th {}, 99th {} ({}); the twin against the march: median {}, 95th {}, 99th {}",
        show(&march.sight, 0.5),
        show(&march.sight, 0.95),
        show(&march.sight, 0.99),
        march.sight.len(),
        show(&twin.sight, 0.5),
        show(&twin.sight, 0.95),
        show(&twin.sight, 0.99),
        twin.sight.len(),
        against(0.5),
        against(0.95),
        against(0.99)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The twin's line: each side's numbers, and the twin's against the march's.
    #[test]
    fn the_twin_line_sets_the_card_s_times_side_by_side() {
        let mut march = Samples::default();
        let mut twin = Samples::default();
        for k in 1..=100 {
            march.card(k as f32, &[]);
            twin.card(k as f32 * 0.8, &[]);
        }
        let line = twin_line(&march, &twin);
        assert!(line.contains("march median 51.00"), "{line}");
        assert!(line.contains("(100 readings)"), "{line}");
        assert!(line.contains("the twin against the march: median -20.0%, 95th -20.0%, 99th -20.0%"), "{line}");
    }

    /// A made-up world: a hill 300 high at (1,000, 500), sea west of east −2,000.
    fn ground(north: f32, east: f32) -> Option<f32> {
        if north.abs() > 8000.0 || east.abs() > 8000.0 {
            return None;
        }
        if east < -2000.0 {
            return Some(100.0);
        }
        let d = (north - 1000.0).hypot(east - 500.0);
        Some(130.0 + (300.0 - d * 0.2).max(0.0))
    }

    #[test]
    fn the_route_visits_the_peak_the_sea_and_a_cave_mouth() {
        let legs = route(&ground, 128.0, &[[200.0, -300.0, 140.0]]);
        let names: Vec<&str> = legs.iter().map(|leg| leg.name).collect();
        eprintln!("The route: {names:?}, {:.0} s", duration(&legs));
        for name in ["over the land", "round the peak", "out to sea", "quick", "to the cave mouth", "at the cave mouth", "home"] {
            assert!(names.contains(&name), "the route has {name}: {names:?}");
        }
        // Round the peak: every point 300 from it.
        for leg in legs.iter().filter(|leg| leg.name == "round the peak") {
            assert!(((leg.to[0] - 1000.0).hypot(leg.to[1] - 500.0) - 300.0).abs() < 1.0, "{leg:?}");
        }
        // Out to sea, it ends over the sea.
        let sea = legs.iter().rfind(|leg| leg.name == "out to sea").unwrap();
        assert!(ground(sea.to[0], sea.to[1]).unwrap() < 128.0);
        // The same map, the same route.
        assert_eq!(legs, route(&ground, 128.0, &[[200.0, -300.0, 140.0]]));
        // Along it: the middle at the start, home at the end, nothing after.
        let total = duration(&legs);
        assert_eq!(along(&legs, 0.0).map(|(at, k, _)| (at, k)), Some(([0.0, 0.0], 0)));
        let (end, k, _) = along(&legs, total - 1e-3).unwrap();
        assert_eq!(k, legs.len() - 1);
        assert!(end[0].hypot(end[1]) < 1.0);
        assert!(along(&legs, total + 1.0).is_none());
    }

    /// The event log's placings read back as they were asked for, in order; other lines and edits passed over.
    #[test]
    fn the_build_is_read_from_the_event_log() {
        let log = "tick     9514  edit request    player 0 asks to place a north–south bar of incus:stone_brick from -7980 -2381 272 (half blocks) at -3990 -1191 136\n\
                   tick     9515  block changed   -3990 -1191 136: incus:air → pieces (by player 0)\n\
                   tick    10885  edit request    player 0 asks to place incus:gabbro at -3992 -1188 137\n\
                   tick    10886  edit request    player 0 asks to place an octant of incus:gabbro at 1 2 3 (half blocks) at 0 1 1\n\
                   tick    10887  edit request    player 0 asks to place a upright bar of incus:stone_brick from -7974 -2391 273 (half blocks) at -3987 -1196 136\n\
                   tick    10888  edit request    player 0 asks to break at 4 5 6\n";
        assert_eq!(
            placings(log),
            vec![
                Placing::Bar { cell: [-3990, -1191, 136], at: [-7980, -2381, 272], axis: 0, block: "incus:stone_brick".into() },
                Placing::Block { cell: [-3992, -1188, 137], block: "incus:gabbro".into() },
                Placing::Bar { cell: [-3987, -1196, 136], at: [-7974, -2391, 273], axis: 2, block: "incus:stone_brick".into() },
            ]
        );
    }

    /// David's quarter-block build of 2026-11-10, as the build mark replays it: his 169 placings (146 bars of stone
    /// brick, 23 blocks of gabbro), within the cells he built in.
    #[test]
    fn the_kept_build_is_all_there() {
        let log = std::fs::read_to_string(crate::debug::project_folder().join("kept logs/events-1794341728.log")).expect("the kept event log");
        let build = placings(&log);
        assert_eq!(build.len(), 169);
        assert_eq!(build.iter().filter(|p| matches!(p, Placing::Bar { .. })).count(), 146);
        assert!(build.iter().all(|p| (-4000..=-3984).contains(&p.cell()[0]) && (-1200..=-1185).contains(&p.cell()[1]) && (135..=139).contains(&p.cell()[2])));
    }

    /// The marks: the spawn's column, the hill's top (the made-up world's highest ground) looking out, and 8 blocks
    /// off the build's north side looking south at it.
    #[test]
    fn the_marks_stand_where_they_should() {
        let build = vec![Placing::Block { cell: [100, 200, 140], block: "incus:gabbro".into() }, Placing::Block { cell: [103, 205, 142], block: "incus:gabbro".into() }];
        let marks = marks(&ground, [3, -4], build.clone());
        assert_eq!(marks.len(), 3);
        assert_eq!(marks[0].at, [3.5, -3.5]);
        assert_eq!((marks[0].heading, marks[0].pitch, marks[0].clear), (0.0, 0.0, false));
        let top = marks[1].at;
        assert!((top[0] - 1000.5).abs() < 1.0 && (top[1] - 500.5).abs() < 1.0, "the hill's top: {top:?}");
        assert!(marks[1].clear);
        assert!((marks[1].heading - 500.5f32.atan2(1000.5).to_degrees()).abs() < 1.0, "looking out from the middle: {}", marks[1].heading);
        assert_eq!(marks[2].at, [112.0, 203.0], "8 blocks north of the build's north side, half way along");
        assert_eq!(marks[2].heading, 180.0);
        assert_eq!(marks[2].look_at, Some([102.0, 203.0, 141.5]));
        assert_eq!(marks[2].build, build);
        assert_eq!(super::marks(&ground, [0, 0], Vec::new()).len(), 2, "no build, no build mark");
    }

    /// The references' other places: high over the spawn looking out, low over the sea looking away from the middle,
    /// the mountain top again with fog; all only pictured; no sea, no sea place.
    #[test]
    fn the_references_places_are_where_they_should_be() {
        let top = &marks(&ground, [3, -4], Vec::new())[1];
        let places = reference_places([3, -4], top, Some([0.0, -3000.0]));
        let names: Vec<&str> = places.iter().map(|place| place.name).collect();
        assert_eq!(names, ["high over the spawn", "low over the sea", "the mountain top with fog"]);
        assert!(places.iter().all(|place| !place.measured));
        assert_eq!((places[0].at, places[0].above, places[0].pitch), ([3.5, -3.5], 400.0, -30.0));
        assert_eq!(places[1].above, 20.0);
        assert!((places[1].heading - 270.0).abs() < 0.01, "looking away from the middle, west: {}", places[1].heading);
        assert_eq!((places[2].at, places[2].clear), (top.at, false));
        assert_eq!(reference_places([0, 0], top, None).len(), 2);
        assert!(REFERENCE_TIMES.windows(2).all(|pair| pair[0].1 > pair[1].1), "noon down to night");
    }

    #[test]
    fn percentiles_are_the_sorted_values() {
        let values: Vec<f32> = (1..=100).map(|v| v as f32).collect();
        assert_eq!(percentile(&values, 0.5), Some(51.0));
        assert_eq!(percentile(&values, 0.95), Some(95.0));
        assert_eq!(percentile(&values, 0.99), Some(99.0));
        assert_eq!(percentile(&[], 0.5), None);
    }
}
