//! The 95 drawing: the editor's and the game's windows drawn by us in Windows 95's look, egui gone [David, 2026-11-19:
//! "yeah! lets trash egui"; 2026-11-20: "lets start the day by kicking e-gui to the curb"].
//!
//! A *sheet* is a picture of one window (or of the temperatures over the world), drawn on the processor at its own size
//! (a pixel of it a pixel of his mock-ups) and shown by the card enlarged a whole number of times, every pixel square
//! (`on_card.rs`). It goes to the card again only when it has changed. How it looks is his: the colours, the edges, the
//! marks and the sizes are the shared page's (`journal/docs/editor-look.md`), measured on the control sheet and on his
//! four pictures (`Art/95gui mockups/current/`), and where those two differ, his ruling.
//!
//! Built once, for the editor and the game: `sheet` (the pixels, the colours, Windows' edges), `letters` (Ironquill),
//! `marks` (the small pictures lifted from his), `parts` (what windows are made of), `on_card` (sheets on the screen).

pub mod letters;
pub mod marks;
pub mod on_card;
pub mod parts;
pub mod sheet;

pub use sheet::Sheet;

/// The size his pictures are drawn at: what the scale's Auto fits to the screen.
pub const DRAWN_AT: (u32, u32) = (1280, 800);

/// The scale, Auto: the largest whole number at which the screen holds his pictures' size (at least 1). It's 2 on the
/// laptop's 2,560 x 1,600, so a sheet of 1,280 x 800 fills it and each of its pixels is two of the screen's each way
/// [David, 2026-11-19: Auto "for standard ratios"; the scales' own menus come later].
pub fn auto_scale(screen: (u32, u32)) -> u32 {
    (screen.0 / DRAWN_AT.0).min(screen.1 / DRAWN_AT.1).max(1)
}

/// A point on the screen (its pixels) as a point on a sheet placed at `at`, `scale` times over.
pub fn on_sheet(point: (f64, f64), at: (i32, i32), scale: u32) -> (i32, i32) {
    let along = |p: f64, a: i32| ((p - a as f64) / scale as f64).floor() as i32;
    (along(point.0, at.0), along(point.1, at.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_fits_his_size_to_the_screen() {
        assert_eq!(auto_scale((2560, 1600)), 2);
        assert_eq!(auto_scale((1920, 1080)), 1);
        assert_eq!(auto_scale((3840, 2160)), 2);
        assert_eq!(auto_scale((800, 600)), 1);
        assert_eq!(on_sheet((385.9, 51.0), (0, 0), 2), (192, 25));
        assert_eq!(on_sheet((32.0, 31.0), (32, 32), 2), (0, -1));
    }
}
