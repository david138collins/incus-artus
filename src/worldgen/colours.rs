//! Colours — how layers become pixels. Deliberately plain and high-contrast (graphing calculator,
//! not art) until the map gets its real look.
//!
//! - **Kinds** use the **palette**: one colour per kind *name*, shared by every step, editable in
//!   the suite and saved in presets. A kind with no colour yet shows as magenta, so it can't hide.
//! - **Numbers** use a **ramp**, chosen per layer in the view.
//! - **Regions** get a random bright colour each (from their number), and the borders between
//!   regions can be drawn over any layer.

use super::map::{LayerData, Map};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KindColour {
    pub kind: String,
    pub rgb: [u8; 3],
}

/// David's starting colours, 23 Oct 2026.
pub fn default_palette() -> Vec<KindColour> {
    [
        ("off the world", [0, 0, 0]),
        ("ocean", [16, 36, 110]),
        ("river", [110, 175, 255]),
        ("plains", [150, 220, 105]),
        ("forest", [28, 105, 38]),
        ("mountains", [140, 140, 140]),
        ("road", [245, 220, 50]),
    ]
    .into_iter()
    .map(|(kind, rgb)| KindColour { kind: kind.to_string(), rgb })
    .collect()
}

const MISSING: [u8; 3] = [255, 0, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Ramp {
    /// Black (lowest) to white (highest).
    #[default]
    Grey,
    /// Black, red, yellow, white: small differences stand out.
    Heat,
    /// Grey in 12 hard steps, like contour bands.
    Bands,
}

impl Ramp {
    pub const ALL: [Ramp; 3] = [Ramp::Grey, Ramp::Heat, Ramp::Bands];

    pub fn name(self) -> &'static str {
        match self {
            Ramp::Grey => "grey",
            Ramp::Heat => "heat",
            Ramp::Bands => "bands",
        }
    }

    /// `t` from 0 to 1.
    fn colour(self, t: f32) -> [u8; 3] {
        let t = t.clamp(0.0, 1.0);
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;
        match self {
            Ramp::Grey => [byte(t); 3],
            Ramp::Heat => [byte(t * 3.0), byte(t * 3.0 - 1.0), byte(t * 3.0 - 2.0)],
            Ramp::Bands => {
                let band = (t * 12.0).floor().min(11.0) / 11.0;
                [byte(band); 3]
            }
        }
    }
}

/// What to draw: a layer, how, and what goes on top.
pub struct Look<'a> {
    pub layer: &'a str,
    pub ramp: Ramp,
    pub palette: &'a [KindColour],
    pub borders: bool,
}

/// The map as RGBA pixels, ready for the screen. Also returns the lowest and highest value of a
/// numbers layer (the ramp is stretched between them), for the legend.
pub fn paint(map: &Map, look: &Look) -> (Vec<u8>, Option<(f32, f32)>) {
    let mut pixels = vec![0u8; map.cells() * 4];
    let mut range = None;
    let mut put = |i: usize, rgb: [u8; 3]| {
        pixels[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    };
    match map.layer(look.layer).map(|layer| &layer.data) {
        Some(LayerData::Numbers(values)) => {
            let (low, high) = values.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
            let span = (high - low).max(1e-6);
            for (i, value) in values.iter().enumerate() {
                put(i, look.ramp.colour((value - low) / span));
            }
            range = Some((low, high));
        }
        Some(LayerData::Kinds { values, names }) => {
            let colours: Vec<[u8; 3]> = names
                .iter()
                .map(|name| look.palette.iter().find(|c| &c.kind == name).map_or(MISSING, |c| c.rgb))
                .collect();
            for (i, value) in values.iter().enumerate() {
                put(i, colours.get(*value as usize).copied().unwrap_or(MISSING));
            }
        }
        Some(LayerData::Regions { ids, .. }) => {
            for (i, id) in ids.iter().enumerate() {
                put(i, if *id == 0 { [0, 0, 0] } else { region_colour(*id) });
            }
        }
        None => {}
    }
    if look.borders {
        draw_borders(map, &mut pixels);
    }
    (pixels, range)
}

/// A bright colour picked from a region's number (the same number always gets the same colour).
fn region_colour(id: u32) -> [u8; 3] {
    let h = crate::fate::planet_hash(0, 0, id as i64, 0, 0);
    let channel = |shift: u32| 70 + ((h >> shift) & 0xFF) as u8 % 186;
    [channel(0), channel(8), channel(16)]
}

/// Draws the edges between regions (from the map's last regions layer) in black.
fn draw_borders(map: &Map, pixels: &mut [u8]) {
    let Some(ids) = map.layers.iter().rev().find_map(|layer| match &layer.data {
        LayerData::Regions { ids, .. } => Some(ids),
        _ => None,
    }) else {
        return;
    };
    for row in 0..map.height {
        for col in 0..map.width {
            let i = map.index(col, row);
            // Only the east and south neighbours, so each border is one cell thick.
            let differs = (col + 1 < map.width && ids[i + 1] != ids[i]) || (row + 1 < map.height && ids[i + map.width] != ids[i]);
            if differs && ids[i] != 0 {
                pixels[i * 4..i * 4 + 3].copy_from_slice(&[0, 0, 0]);
            }
        }
    }
}
