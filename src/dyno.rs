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
//! **Measured:** each frame's time, and each pass's on the card (`sight::draw::take_card_readings`), kept by phase
//! (baking: the bake not done yet; baked) and by leg; the temperatures and the card's clock every few seconds.
//! **The sums:** the median, 95th and 99th percentile of each, as a report in `logs/` and in the log.

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
pub fn route(ground: &dyn Fn(f32, f32) -> Option<f32>, sea: f32, mouths: &[[f32; 3]]) -> Vec<Leg> {
    let mut legs = Vec::new();
    let land = |north: f32, east: f32| ground(north, east).map(|z| z.max(sea));
    // 1. The highest ground within 2,500, on a grid of 50.
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
    let peak = peak.0;
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

/// What's kept for a phase or a leg: each frame's time (ms), sight's on the card and each pass's (ms).
#[derive(Default, Clone, Debug)]
pub struct Samples {
    pub frames: Vec<f32>,
    pub sight: Vec<f32>,
    pub passes: BTreeMap<String, Vec<f32>>,
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
/// on the card (median and 95th), and each pass's median.
pub fn line(samples: &Samples) -> String {
    let p = |values: &[f32], share: f32| percentile(values, share).map_or("-".to_string(), |v| format!("{v:.2}"));
    let fps = percentile(&samples.frames, 0.5).map_or("-".to_string(), |ms| format!("{:.0}", 1000.0 / ms.max(1e-3)));
    let passes: Vec<String> = samples.passes.iter().map(|(name, values)| format!("{name} {}", p(values, 0.5))).collect();
    format!(
        "{} frames, {fps} fps (median); frame ms median {}, 95th {}, 99th {}, 99.9th {}, worst {}; sight on the card ms median {}, 95th {}; passes (medians, ms): {}",
        samples.frames.len(),
        p(&samples.frames, 0.5),
        p(&samples.frames, 0.95),
        p(&samples.frames, 0.99),
        p(&samples.frames, 0.999),
        p(&samples.frames, 1.0),
        p(&samples.sight, 0.5),
        p(&samples.sight, 0.95),
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

    #[test]
    fn percentiles_are_the_sorted_values() {
        let values: Vec<f32> = (1..=100).map(|v| v as f32).collect();
        assert_eq!(percentile(&values, 0.5), Some(51.0));
        assert_eq!(percentile(&values, 0.95), Some(95.0));
        assert_eq!(percentile(&values, 0.99), Some(99.0));
        assert_eq!(percentile(&[], 0.5), None);
    }
}
