//! A field — one number for every pixel of a square patch of the world. The raw material: no
//! forests or oceans yet, just values from 0 (black) to 1 (white).
//!
//! Pixels are numbered from the north-west corner: `col` goes east, `row` goes south, so the field
//! reads like a paper map with north at the top. Noise is worked out in **blocks** (game X north,
//! Y east, from the centre), so changing the resolution shows the same field in more or less
//! detail rather than a different field.

#[derive(Clone, Debug)]
pub struct Field {
    pub width: usize,
    pub height: usize,
    pub blocks_per_pixel: f32,
    pub values: Vec<f32>,
}

impl Field {
    /// An empty (all black) square field `size_blocks` across.
    pub fn new(size_blocks: f32, blocks_per_pixel: f32) -> Self {
        let pixels = (size_blocks / blocks_per_pixel).ceil().max(1.0) as usize;
        Self { width: pixels, height: pixels, blocks_per_pixel, values: vec![0.0; pixels * pixels] }
    }

    pub fn index(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    /// The centre of a pixel in blocks: (X north, Y east), from the field's centre.
    pub fn blocks_at(&self, col: usize, row: usize) -> (f32, f32) {
        let north = (self.height as f32 / 2.0 - row as f32 - 0.5) * self.blocks_per_pixel;
        let east = (col as f32 + 0.5 - self.width as f32 / 2.0) * self.blocks_per_pixel;
        (north, east)
    }

    /// Works out every pixel's value from its position in blocks, using all the computer's cores
    /// (each takes a band of rows). The rule must give the same answer in any order, which is
    /// what keeps a seed's field the same every time.
    pub fn fill(&mut self, rule: impl Fn(f32, f32) -> f32 + Sync) {
        let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
        let rows_each = self.height.div_ceil(cores).max(1);
        let (width, height, blocks) = (self.width, self.height, self.blocks_per_pixel);
        std::thread::scope(|scope| {
            for (band, chunk) in self.values.chunks_mut(rows_each * width).enumerate() {
                let rule = &rule;
                scope.spawn(move || {
                    for (i, value) in chunk.iter_mut().enumerate() {
                        let (col, row) = (i % width, band * rows_each + i / width);
                        let north = (height as f32 / 2.0 - row as f32 - 0.5) * blocks;
                        let east = (col as f32 + 0.5 - width as f32 / 2.0) * blocks;
                        *value = rule(north, east);
                    }
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn north_is_up_and_east_is_right() {
        let field = Field::new(1000.0, 100.0);
        let (north, east) = field.blocks_at(0, 0);
        assert!(north > 0.0 && east < 0.0, "the top-left pixel is the north-west corner");
        let (north, east) = field.blocks_at(9, 9);
        assert!(north < 0.0 && east > 0.0, "the bottom-right pixel is the south-east corner");
    }

    #[test]
    fn fill_gives_every_pixel_the_value_for_its_own_position() {
        let mut field = Field::new(1000.0, 10.0);
        field.fill(|north, east| north * 1000.0 + east);
        for row in 0..field.height {
            for col in 0..field.width {
                let (north, east) = field.blocks_at(col, row);
                assert_eq!(field.values[field.index(col, row)], north * 1000.0 + east);
            }
        }
    }
}
