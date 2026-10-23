//! Layers — the stack of fields, like the layers of a graphics editor.
//!
//! Layers are **opaque**: you see the top visible layer, and the one below shows through only
//! where the top one doesn't cover (outside an image, or where it's see-through). Below all of
//! them is a dark checkerboard: nothing there. Blending layers together is a tool for later.
//!
//! Each layer has its own name, settings, colouring, and (for noise) its own seed, so two Perlin
//! layers with the same settings still make different patterns.

use super::image::{Picture, Placement};
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
    pub colouring: Colouring,
    pub result: Option<Finished>,
    /// Goes up each time a new result arrives (so the picture knows to repaint).
    pub version: u64,
    /// The picture as last painted, and what it was painted from.
    pub painted: Vec<[u8; 3]>,
    painted_for: Option<(u64, Colouring)>,
    /// Goes up each time the picture is repainted (so the stacked picture knows to rebuild).
    pub paint_version: u64,

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
            colouring: Colouring::default(),
            result: None,
            version: 0,
            painted: Vec::new(),
            painted_for: None,
            paint_version: 0,
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

    /// Repaints the layer's picture if its result or colouring changed.
    pub fn repaint(&mut self) {
        let Some(result) = &self.result else { return };
        let wanted = (self.version, self.colouring.clone());
        if self.painted_for.as_ref() == Some(&wanted) {
            return;
        }
        let made = &result.made;
        self.painted = self.colouring.paint(&made.shaped.values, &made.sorted, made.original.as_deref());
        self.painted_for = Some(wanted);
        self.paint_version += 1;
    }
}

/// Picks out the next free "Perlin noise field 3"-style name.
pub fn next_name(layers: &[Layer], type_name: &str) -> String {
    (1..)
        .map(|n| format!("{type_name} {n}"))
        .find(|name| !layers.iter().any(|layer| &layer.name == name))
        .unwrap_or_default()
}

/// The finished picture: for each pixel, the top visible layer that covers it. `None` where no
/// layer does (drawn as a checkerboard on screen, black in exports).
pub fn composite(layers: &[Layer], cells: usize) -> Vec<Option<[u8; 3]>> {
    let mut picture = vec![None; cells];
    // Bottom first, so higher layers paint over lower ones.
    for layer in layers.iter().filter(|layer| layer.visible) {
        let Some(result) = &layer.result else { continue };
        if layer.painted.len() != cells || result.made.shaped.values.len() != cells {
            continue; // still being remade at a new size
        }
        for (i, pixel) in picture.iter_mut().enumerate() {
            if result.made.covers(i) {
                *pixel = Some(layer.painted[i]);
            }
        }
    }
    picture
}
