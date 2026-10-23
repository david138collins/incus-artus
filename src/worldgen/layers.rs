//! Layers — the stack of fields, like the layers of a graphics editor.
//!
//! Each layer **mixes** with the layers below it (normal, add, multiply… see `blend.rs`), faded by
//! its **opacity**. They mix their numbers, not their colours: the stack makes one field of values,
//! and the result's palette colours it. Where a layer doesn't cover (outside an image, or where
//! it's see-through), the layers below show through. Where no layer covers at all is a dark
//! checkerboard: nothing there.
//!
//! Each layer has its own name, settings, colouring, and (for noise) its own seed, so two Perlin
//! layers with the same settings still make different patterns. A layer's own colouring is what
//! you see when it's shown alone.

use super::blend::Mix;
use super::field::Field;
use super::image::{Picture, Placement};
use super::made::{Made, Shaping};
use super::noise::PerlinSettings;
use super::palette::Colouring;
use super::runner::{Finished, Job, Poll};
use std::sync::Arc;

pub enum LayerKind {
    Perlin { seed: u64, settings: PerlinSettings },
    Image { file: String, picture: Arc<Picture>, placement: Placement },
}

impl LayerKind {
    pub fn type_name(&self) -> &'static str {
        match self {
            LayerKind::Perlin { .. } => "Perlin noise field",
            LayerKind::Image { .. } => "Image",
        }
    }
}

/// Everything that decides what a layer makes, to notice when it changes. (An image's pixels are
/// compared by which loaded picture it is, not pixel by pixel.)
#[derive(Clone, PartialEq)]
enum Recipe {
    Perlin(u64, PerlinSettings),
    Image(usize, Placement),
}

pub struct Layer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub kind: LayerKind,
    /// How it combines with the layers below, and how strongly (0 to 1).
    pub mix: Mix,
    pub opacity: f32,
    pub colouring: Colouring,
    pub result: Option<Finished>,
    /// Goes up each time a new result arrives (so the picture knows to rebuild).
    pub version: u64,

    job: Option<Job>,
    seen: Option<(f32, f32, Recipe)>,
    wanted: bool,
}

impl Layer {
    pub fn new(id: u32, name: String, kind: LayerKind) -> Self {
        Self {
            id,
            name,
            visible: true,
            kind,
            mix: Mix::Normal,
            opacity: 1.0,
            colouring: Colouring::default(),
            result: None,
            version: 0,
            job: None,
            seen: None,
            wanted: true,
        }
    }

    pub fn is_making(&self) -> bool {
        self.job.is_some()
    }

    fn recipe(&self) -> Recipe {
        match &self.kind {
            LayerKind::Perlin { seed, settings } => Recipe::Perlin(*seed, settings.clone()),
            LayerKind::Image { picture, placement, .. } => Recipe::Image(Arc::as_ptr(picture) as usize, placement.clone()),
        }
    }

    /// Notices changes, collects a finished result, and starts the next one when wanted.
    ///
    /// While a slider is being dragged, the running job is left to finish (so the picture keeps
    /// up as fast as layers can be made) and the next one starts straight after with the newest
    /// settings. A slow one (over a quarter of a second) is abandoned instead.
    pub fn drive(&mut self, size_blocks: f32, blocks_per_pixel: f32) {
        let now = (size_blocks, blocks_per_pixel, self.recipe());
        if self.seen.as_ref() != Some(&now) {
            self.seen = Some(now);
            self.wanted = true;
        }
        if let Some(job) = &self.job {
            match job.poll() {
                Poll::Done(finished) => {
                    self.result = Some(*finished);
                    self.version += 1;
                    self.job = None;
                }
                Poll::Over => self.job = None,
                Poll::Waiting if self.wanted && job.started.elapsed().as_secs_f32() > 0.25 => {
                    job.stop();
                    self.job = None;
                }
                Poll::Waiting => return,
            }
        }
        if self.wanted {
            self.wanted = false;
            self.job = Some(match &self.kind {
                LayerKind::Perlin { seed, settings } => {
                    let (seed, settings) = (*seed, settings.clone());
                    Job::start(move |stop| super::noise::make(&settings, seed, size_blocks, blocks_per_pixel, stop))
                }
                LayerKind::Image { picture, placement, .. } => {
                    let (picture, placement) = (picture.clone(), placement.clone());
                    Job::start(move |_| Some(super::image::make(&picture, &placement, size_blocks, blocks_per_pixel)))
                }
            });
        }
    }
}

/// Picks out the next free "Perlin noise field 3"-style name.
pub fn next_name(layers: &[Layer], type_name: &str) -> String {
    (1..)
        .map(|n| format!("{type_name} {n}"))
        .find(|name| !layers.iter().any(|layer| &layer.name == name))
        .unwrap_or_default()
}

/// The stack's result: every visible layer mixed into the ones below it, bottom first, as one
/// field of values with its own numbers (range, histogram). Pixels no layer covers are left
/// uncovered.
pub fn stack(layers: &[Layer], width: usize, blocks_per_pixel: f32) -> Made {
    let cells = width * width;
    let mut values = vec![0.0; cells];
    let mut covered = vec![false; cells];
    for layer in layers.iter().filter(|layer| layer.visible) {
        let Some(result) = &layer.result else { continue };
        let this = &result.made.shaped.values;
        if this.len() != cells {
            continue; // still being remade at a new size
        }
        for i in 0..cells {
            if !result.made.covers(i) {
                continue;
            }
            // Where nothing is below, the layer's own value is used as it is.
            values[i] = if covered[i] { layer.mix.over(values[i], this[i], layer.opacity) } else { this[i] };
            covered[i] = true;
        }
    }
    let field = Field { width, height: width, blocks_per_pixel, values };
    let covered = covered.contains(&false).then_some(covered);
    // Already 0 to 1, so no shaping: the result's numbers are worked out as they are.
    let as_it_is = Shaping { fill_range: false, ..Shaping::default() };
    Made::finish(field, covered, None, &as_it_is, |v| v)
}

/// One colour per pixel from a layer's (or the stack's) values. `None` where it doesn't cover
/// (drawn as a checkerboard on screen, see-through in exports).
pub fn colour(made: &Made, colouring: &Colouring) -> Vec<Option<[u8; 3]>> {
    let painted = colouring.paint(&made.shaped.values, &made.sorted, made.original.as_deref());
    painted.into_iter().enumerate().map(|(i, colour)| made.covers(i).then_some(colour)).collect()
}
