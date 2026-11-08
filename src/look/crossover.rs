//! The crossover: the seams between sight's layers on a track, like a multiband compressor's [David 2026-11-08: "a
//! track, with a slider for each intersection between layers, and bars that come from the side as i drag the slider
//! for each intersection up and down, the band gets wider. sort of like a multiband compressor"; "oh yeah, a
//! crossover! exactly"].
//!
//! Each layer is a coloured zone, finest on the left; where two meet, the seam's blend is a band of the two colours
//! mixed as the shader mixes them. A seam's handle dragged left or right moves the seam, up or down widens or
//! narrows its band; double-clicked, it goes back to its default. Two tracks, because the seams are of two kinds
//! (`march.wgsl`, `worth_of`): among the blocks near the eye, by distance; past them, by a cell's size on the screen.

/// One seam between two layers.
pub struct Seam {
    /// The finer layer's factor (the seam's name).
    pub finer: f32,
    /// The coarser layer it fades into, if the seam is in use (both layers in the pyramid).
    pub coarser: Option<f32>,
    /// Its fade, as the settings keep it: among the blocks, from and to (shares of their reach); past them, where
    /// it starts (the finer pixel so many times its cells) and over how many doublings.
    pub fade: [f32; 2],
    pub default: [f32; 2],
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Track {
    /// Among the blocks near the eye: shares of the blocks' reach, from the eye (0) to its edge (1).
    Blocks,
    /// Past them: a cell's size on the screen, in screen pixels (the axis in doublings, log2).
    Cells,
}

/// Each seam's lane on a track (pixels high).
const LANE: f32 = 14.0;

/// About how far the blocks reach (blocks; the live level's half window), for reading the near track.
const BLOCKS_REACH: f32 = 336.0;

impl Track {
    /// The axis, left to right.
    fn axis(self) -> [f32; 2] {
        match self {
            Track::Blocks => [0.0, 1.0],
            Track::Cells => [-3.0, 5.0],
        }
    }

    /// Where a seam's blend runs on the axis: from where the finer layer is all there (left) to where it's gone
    /// (right).
    pub fn band(self, finer: f32, fade: [f32; 2]) -> [f32; 2] {
        match self {
            Track::Blocks => [fade[0].min(fade[1]), fade[1]],
            Track::Cells => {
                let gone = (finer / fade[0].max(0.01)).log2();
                [gone - fade[1], gone]
            }
        }
    }

    /// The fade a band on the axis stands for (`band` the other way), kept to what the settings allow.
    pub fn fade(self, finer: f32, band: [f32; 2]) -> [f32; 2] {
        match self {
            Track::Blocks => {
                let from = band[0].clamp(0.0, 1.0);
                [from, band[1].clamp(from, 1.0)]
            }
            Track::Cells => [(finer / band[1].exp2()).clamp(0.25, 4.0), (band[1] - band[0]).clamp(0.1, 4.0)],
        }
    }

    /// How much of the finer layer shows at a place on the axis (0 to 1), as `worth_of` in `march.wgsl` works it out.
    pub fn finer_share(self, finer: f32, fade: [f32; 2], at: f32) -> f32 {
        match self {
            Track::Blocks => {
                let begins = fade[0].min(fade[1]);
                ((fade[1] - at) / (fade[1] - begins).max(1e-3)).clamp(0.0, 1.0)
            }
            Track::Cells => {
                // The finer pixel against the cell: its factor in screen pixels, the cell 2^at of them.
                let ratio = finer / at.exp2();
                ((ratio / fade[0].max(0.1) - 1.0) / (fade[1].max(0.05).exp2() - 1.0).max(1e-3)).clamp(0.0, 1.0)
            }
        }
    }

    /// How much a band widens for a pixel dragged up.
    fn widening(self) -> f32 {
        match self {
            Track::Blocks => 1.0 / 200.0,
            Track::Cells => 1.0 / 40.0,
        }
    }

    /// The narrowest a band goes (on the axis).
    fn narrowest(self) -> f32 {
        match self {
            Track::Blocks => 0.0,
            Track::Cells => 0.1,
        }
    }

    /// A seam's numbers, in words.
    fn words(self, seam: &Seam) -> String {
        let names = format!("{} into {}", name(seam.finer), seam.coarser.map_or("nothing".to_string(), name));
        match self {
            Track::Blocks => {
                let [from, to] = seam.fade;
                format!("{names}: fading from {from:.2} to {to:.2} of the blocks' reach (about {:.0} to {:.0} blocks)", from * BLOCKS_REACH, to * BLOCKS_REACH)
            }
            Track::Cells => {
                let [start, doublings] = seam.fade;
                let [all_in, gone] = self.band(seam.finer, seam.fade).map(f32::exp2);
                format!("{names}: coming in where a cell is {gone:.2} screen pixels across, all in by {all_in:.2} (starts at {start:.2}, over {doublings:.2} doublings)")
            }
        }
    }
}

/// A layer's name.
fn name(factor: f32) -> String {
    format!("{factor} × {factor}")
}

/// Each layer's colour on the track.
fn colour(factor: f32) -> egui::Color32 {
    if factor >= 6.0 {
        egui::Color32::from_rgb(92, 78, 150)
    } else if factor >= 3.0 {
        egui::Color32::from_rgb(52, 112, 150)
    } else if factor >= 1.9 {
        egui::Color32::from_rgb(60, 138, 84)
    } else if factor >= 1.4 {
        egui::Color32::from_rgb(160, 140, 52)
    } else {
        egui::Color32::from_rgb(176, 96, 52)
    }
}

fn mix(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    egui::Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// Draws a track for the pyramid's `layers` (coarsest first) and its `seams`; returns the seams whose fades were
/// changed (dragged, or double-clicked back to their defaults).
pub fn track(ui: &mut egui::Ui, id: egui::Id, track: Track, layers: &[f32], seams: &mut [Seam]) -> Vec<usize> {
    let [low, high] = track.axis();
    // The seams in use, finest first (left to right on both tracks, and top to bottom: each its own lane, so seams
    // at the same place can each be grabbed).
    let mut used: Vec<usize> = (0..seams.len()).filter(|&k| seams[k].coarser.is_some()).collect();
    used.sort_by(|&a, &b| seams[a].finer.total_cmp(&seams[b].finer));
    let coarsest = layers.first().copied().unwrap_or(8.0);

    let width = ui.available_width().clamp(200.0, 620.0);
    let (outer, _) = ui.allocate_exact_size(egui::vec2(width, 38.0 + LANE * used.len() as f32), egui::Sense::hover());
    let painter = ui.painter_at(outer);
    let bar = egui::Rect::from_min_max(egui::pos2(outer.left() + 8.0, outer.top() + 4.0), egui::pos2(outer.right() - 8.0, outer.bottom() - 16.0));
    let x_of = |at: f32| bar.left() + (at - low) / (high - low) * bar.width();
    let at_of = |x: f32| low + (x - bar.left()) / bar.width() * (high - low);

    // The zones: each column the colours mixed as the layers are, the coarsest under, each finer over it by its
    // share (a finer layer only where the one above it is).
    let mut x = bar.left();
    while x < bar.right() {
        let at = at_of(x + 1.0);
        let mut shown = colour(coarsest);
        let mut share = 1.0;
        for &k in used.iter().rev() {
            share *= track.finer_share(seams[k].finer, seams[k].fade, at);
            shown = mix(shown, colour(seams[k].finer), share);
        }
        painter.rect_filled(egui::Rect::from_min_max(egui::pos2(x, bar.top()), egui::pos2((x + 2.0).min(bar.right()), bar.bottom())), 0.0, shown);
        x += 2.0;
    }

    // The layers' names, each in the middle of its zone (between the seams on either side).
    let centre = |seams: &[Seam], k: usize| {
        let [a, b] = track.band(seams[k].finer, seams[k].fade);
        x_of(0.5 * (a + b)).clamp(bar.left(), bar.right())
    };
    let mut edges = vec![bar.left()];
    edges.extend(used.iter().map(|&k| centre(seams, k)));
    edges.push(bar.right());
    let mut names: Vec<f32> = used.iter().map(|&k| seams[k].finer).collect();
    names.push(coarsest);
    for (k, &factor) in names.iter().enumerate() {
        if edges[k + 1] - edges[k] > 34.0 {
            painter.text(egui::pos2(0.5 * (edges[k] + edges[k + 1]), bar.top() + 2.0), egui::Align2::CENTER_TOP, name(factor), egui::FontId::proportional(11.0), egui::Color32::from_white_alpha(200));
        }
    }

    // The axis underneath.
    let marks: &[(f32, &str)] = match track {
        Track::Blocks => &[(0.0, "you"), (0.25, "84"), (0.5, "168"), (0.75, "252"), (1.0, "336 blocks")],
        Track::Cells => &[(-2.0, "¼ px"), (0.0, "1 px"), (2.0, "4 px"), (4.0, "16 px")],
    };
    for &(at, label) in marks {
        let x = x_of(at);
        painter.line_segment([egui::pos2(x, bar.bottom()), egui::pos2(x, bar.bottom() + 3.0)], egui::Stroke::new(1.0, egui::Color32::GRAY));
        let align = if at <= low { egui::Align2::LEFT_TOP } else if at >= high { egui::Align2::RIGHT_TOP } else { egui::Align2::CENTER_TOP };
        painter.text(egui::pos2(x, bar.bottom() + 3.0), align, label, egui::FontId::proportional(10.0), egui::Color32::GRAY);
    }

    // The seams' bands and handles: bars at the band's edges, joined through the handle.
    let mut changed = Vec::new();
    let mut said = None;
    for (place, &k) in used.iter().enumerate() {
        let [a, b] = track.band(seams[k].finer, seams[k].fade);
        let handle_x = centre(seams, k);
        let lane = bar.top() + 16.0 + LANE * (place as f32 + 0.5);
        let response = ui.interact(egui::Rect::from_center_size(egui::pos2(handle_x, lane), egui::vec2(18.0, LANE)), id.with(k), egui::Sense::click_and_drag());
        if response.double_clicked() {
            seams[k].fade = seams[k].default;
            changed.push(k);
        } else if response.dragged() {
            let delta = response.drag_delta();
            if delta != egui::Vec2::ZERO {
                // Between its neighbours' centres: the bands never cross.
                let left = if place > 0 { at_of(centre(seams, used[place - 1])) } else { low };
                let right = if place + 1 < used.len() { at_of(centre(seams, used[place + 1])) } else { high };
                let middle_at = (0.5 * (a + b) + delta.x / bar.width() * (high - low)).clamp(left, right);
                let half = 0.5 * ((b - a) - delta.y * track.widening()).max(track.narrowest());
                seams[k].fade = track.fade(seams[k].finer, [middle_at - half, middle_at + half]);
                changed.push(k);
            }
        }
        let lit = response.hovered() || response.dragged();
        if lit {
            said = Some(track.words(&seams[k]));
        }
        let stroke = egui::Stroke::new(if lit { 2.0 } else { 1.5 }, if lit { egui::Color32::WHITE } else { egui::Color32::from_white_alpha(170) });
        let [left, right] = [x_of(a).clamp(bar.left(), bar.right()), x_of(b).clamp(bar.left(), bar.right())];
        for x in [left, right] {
            painter.line_segment([egui::pos2(x, lane - 0.4 * LANE), egui::pos2(x, lane + 0.4 * LANE)], stroke);
        }
        painter.line_segment([egui::pos2(left, lane), egui::pos2(right, lane)], stroke);
        painter.circle_filled(egui::pos2(handle_x, lane), if lit { 6.0 } else { 5.0 }, if lit { egui::Color32::WHITE } else { egui::Color32::from_gray(210) });
        painter.circle_stroke(egui::pos2(handle_x, lane), 5.0, egui::Stroke::new(1.0, egui::Color32::from_gray(30)));
    }

    // The seam under the pointer in words, or which seams aren't in use (kept, greyed: not hidden).
    match said {
        Some(words) => {
            ui.small(words);
        }
        None => {
            let idle: Vec<String> = seams.iter().filter(|s| s.coarser.is_none()).map(|s| name(s.finer)).collect();
            let text = if used.is_empty() { "One layer only: no seams.".to_string() } else { String::new() };
            let text = if idle.is_empty() { text } else { format!("{text}Not in use with these layers (kept as set): {}.", idle.join(", ")) };
            ui.add_enabled(false, egui::Label::new(egui::RichText::new(if text.is_empty() { " ".to_string() } else { text }).small()));
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_band_and_its_fade_are_the_same_thing() {
        for track in [Track::Blocks, Track::Cells] {
            for finer in [4.0, 2.0, 1.5, 1.0] {
                let fade = if track == Track::Blocks { [0.33, 0.52] } else { [0.68, 0.32] };
                let back = track.fade(finer, track.band(finer, fade));
                assert!((back[0] - fade[0]).abs() < 1e-4 && (back[1] - fade[1]).abs() < 1e-4, "{track:?} {finer}: {back:?}");
            }
        }
    }

    #[test]
    fn the_finer_layer_is_all_there_left_of_its_band_and_gone_right_of_it() {
        for track in [Track::Blocks, Track::Cells] {
            for finer in [4.0, 2.0, 1.0] {
                let fade = if track == Track::Blocks { [0.4, 0.6] } else { [1.0, 1.0] };
                let [a, b] = track.band(finer, fade);
                assert!(track.finer_share(finer, fade, a - 0.01) > 0.999, "{track:?} {finer}");
                assert!(track.finer_share(finer, fade, b + 0.01) < 0.001, "{track:?} {finer}");
                let half = track.finer_share(finer, fade, 0.5 * (a + b));
                assert!(half > 0.05 && half < 0.95, "{track:?} {finer}: {half}");
            }
        }
    }

    #[test]
    fn by_default_the_finer_seams_lie_to_the_left() {
        // Past the blocks the defaults put each seam a doubling apart, finest leftmost (the order the handles keep).
        let centre = |finer: f32| {
            let [a, b] = Track::Cells.band(finer, crate::sight::draw::CELLS_FADE);
            0.5 * (a + b)
        };
        assert!(centre(1.0) < centre(1.5) && centre(1.5) < centre(2.0) && centre(2.0) < centre(4.0));
    }
}
