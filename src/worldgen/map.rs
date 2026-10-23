//! The map — what the steps of a generator read and write.
//!
//! The world is divided into square **cells** (one pixel each on screen), `blocks_per_cell` blocks
//! wide. Steps write **layers**: one value per cell, of one of three sorts:
//! - **Numbers**: height, moisture, temperature... any amount. Shown as a colour ramp.
//! - **Kinds**: one of a short named list (ocean, plains, forest...). Shown in the palette's colours.
//! - **Regions**: which named area each cell belongs to (0 = none). Shown as patches with borders.
//!
//! Steps can also put down **marks**: dots with labels (a region's name, later sites and roads).
//!
//! Cells are numbered from the north-west corner: `col` goes east, `row` goes south, so the map
//! reads like a paper map with north at the top. Steps should work in **blocks** (game X north,
//! Y east, from the world's centre) wherever they can, so changing the map's resolution shows the
//! same world in more or less detail instead of a different world.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub blocks_per_cell: f32,
    /// The world's edge: cells further than this from the centre are off the world.
    pub world_radius: f32,
    pub layers: Vec<Layer>,
    pub marks: Vec<Mark>,
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub data: LayerData,
}

#[derive(Clone, Debug)]
pub enum LayerData {
    Numbers(Vec<f32>),
    Kinds { values: Vec<u8>, names: Vec<String> },
    /// `names[id - 1]` is region `id`'s name.
    Regions { ids: Vec<u32>, names: Vec<String> },
}

/// A dot with a label. `group` lets the view show or hide a whole set ("region names", "sites").
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mark {
    pub col: f32,
    pub row: f32,
    pub label: String,
    pub group: String,
}

impl Map {
    /// An empty map of a round world `world_diameter` blocks across.
    pub fn new(world_diameter: f32, blocks_per_cell: f32) -> Self {
        let cells = (world_diameter / blocks_per_cell).ceil().max(1.0) as usize;
        Self {
            width: cells,
            height: cells,
            blocks_per_cell,
            world_radius: world_diameter / 2.0,
            layers: Vec::new(),
            marks: Vec::new(),
        }
    }

    pub fn cells(&self) -> usize {
        self.width * self.height
    }

    pub fn index(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    /// The centre of a cell in blocks: (X north, Y east), from the world's centre.
    pub fn blocks_at(&self, col: usize, row: usize) -> (f32, f32) {
        let north = (self.height as f32 / 2.0 - row as f32 - 0.5) * self.blocks_per_cell;
        let east = (col as f32 + 0.5 - self.width as f32 / 2.0) * self.blocks_per_cell;
        (north, east)
    }

    /// How far out a cell is, from 0 (the centre) to 1 (the world's edge) and beyond.
    pub fn distance_out(&self, col: usize, row: usize) -> f32 {
        let (north, east) = self.blocks_at(col, row);
        (north * north + east * east).sqrt() / self.world_radius
    }

    pub fn in_world(&self, col: usize, row: usize) -> bool {
        self.distance_out(col, row) <= 1.0
    }

    pub fn layer(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.name == name)
    }

    pub fn numbers(&self, name: &str) -> Option<&[f32]> {
        match self.layer(name).map(|layer| &layer.data) {
            Some(LayerData::Numbers(values)) => Some(values),
            _ => None,
        }
    }

    pub fn kinds(&self, name: &str) -> Option<(&[u8], &[String])> {
        match self.layer(name).map(|layer| &layer.data) {
            Some(LayerData::Kinds { values, names }) => Some((values, names)),
            _ => None,
        }
    }

    /// Adds a layer, replacing any earlier one of the same name.
    pub fn put(&mut self, name: &str, data: LayerData) {
        self.layers.retain(|layer| layer.name != name);
        self.layers.push(Layer { name: name.to_string(), data });
    }

    /// Works out a number for every cell, from its column, row and position in blocks, using all
    /// the computer's cores (each takes a band of rows). The rule must give the same answer in
    /// any order, which is what keeps a seed's world the same every time.
    pub fn fill(&self, rule: impl Fn(usize, usize, f32, f32) -> f32 + Sync) -> Vec<f32> {
        let mut values = vec![0.0; self.cells()];
        let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
        let rows_each = self.height.div_ceil(cores).max(1);
        std::thread::scope(|scope| {
            for (band, chunk) in values.chunks_mut(rows_each * self.width).enumerate() {
                let rule = &rule;
                scope.spawn(move || {
                    for (i, value) in chunk.iter_mut().enumerate() {
                        let (col, row) = (i % self.width, band * rows_each + i / self.width);
                        let (north, east) = self.blocks_at(col, row);
                        *value = rule(col, row, north, east);
                    }
                });
            }
        });
        values
    }

    /// The four cells next to a cell (fewer at the map's edges).
    pub fn neighbours(&self, col: usize, row: usize) -> impl Iterator<Item = (usize, usize)> {
        let (width, height) = (self.width as i64, self.height as i64);
        [(0i64, -1i64), (1, 0), (0, 1), (-1, 0)].into_iter().filter_map(move |(dc, dr)| {
            let (c, r) = (col as i64 + dc, row as i64 + dr);
            (c >= 0 && r >= 0 && c < width && r < height).then_some((c as usize, r as usize))
        })
    }

    /// Everything known about one cell, as lines of text (for the inspector under the mouse).
    pub fn describe_cell(&self, col: usize, row: usize) -> Vec<(String, String)> {
        let i = self.index(col, row);
        self.layers
            .iter()
            .map(|layer| {
                let value = match &layer.data {
                    LayerData::Numbers(values) => format!("{:.3}", values[i]),
                    LayerData::Kinds { values, names } => {
                        names.get(values[i] as usize).cloned().unwrap_or_else(|| format!("kind {}", values[i]))
                    }
                    LayerData::Regions { ids, names } => match ids[i] {
                        0 => "(none)".to_string(),
                        id => format!("{} (#{id})", names.get(id as usize - 1).map_or("?", |n| n.as_str())),
                    },
                };
                (layer.name.clone(), value)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn north_is_up_and_east_is_right() {
        let map = Map::new(1000.0, 100.0);
        let (north, east) = map.blocks_at(0, 0);
        assert!(north > 0.0 && east < 0.0, "the top-left cell is the north-west corner");
        let (north, east) = map.blocks_at(9, 9);
        assert!(north < 0.0 && east > 0.0, "the bottom-right cell is the south-east corner");
    }

    #[test]
    fn fill_gives_every_cell_its_own_value_in_order() {
        let map = Map::new(1000.0, 10.0);
        let values = map.fill(|col, row, _, _| (row * 1000 + col) as f32);
        for row in 0..map.height {
            for col in 0..map.width {
                assert_eq!(values[map.index(col, row)], (row * 1000 + col) as f32);
            }
        }
    }
}
