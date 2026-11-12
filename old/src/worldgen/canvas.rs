//! The camera over a field: fit it to the screen, drag to pan, scroll to zoom, double-click to fit
//! again. Shared by the Graphics Kit and World Gen.

use egui::{Pos2, Rect, Vec2};

pub struct Camera {
    /// Field position (in pixels) at the middle of the screen, and screen points per pixel.
    pub centre: Vec2,
    pub zoom: f32,
    fitted_for: (usize, usize),
    /// Frames left in which to keep fitting the field to the screen (the panels take a few frames
    /// to settle into their sizes when the suite opens).
    settling: u32,
    /// Dragging doesn't pan (the drag is drawing something, such as a cross-section).
    pub hold_pan: bool,
    /// The screen area the field was last fitted to, while it hasn't been panned or zoomed since:
    /// if the area changes size (a panel opened or folded), the field is fitted again.
    fitted_to: Option<Vec2>,
}

impl Default for Camera {
    fn default() -> Self {
        Self { centre: Vec2::ZERO, zoom: 1.0, fitted_for: (0, 0), settling: 10, hold_pan: false, fitted_to: None }
    }
}

impl Camera {
    /// Fits, pans and zooms for a field `width × height` pixels shown in `rect`. Returns the field
    /// pixel under the mouse.
    pub fn update(&mut self, ui: &egui::Ui, rect: Rect, response: &egui::Response, (width, height): (usize, usize)) -> Option<(usize, usize)> {
        let size = Vec2::new(width as f32, height as f32);
        if self.settling > 0 {
            self.settling -= 1;
            self.fitted_for = (0, 0);
            ui.ctx().request_repaint();
        }
        // Fit the whole field on screen when its size changes (and on double-click), and when the
        // screen area changes size while it's still as fitted (opening a panel would otherwise
        // leave it fitted to a sliver).
        if self.fitted_to.is_some_and(|fitted| (fitted - rect.size()).length() > 1.0) {
            self.fitted_for = (0, 0);
        }
        if response.double_clicked() || self.fitted_for != (width, height) {
            if self.fitted_for != (0, 0) && !response.double_clicked() {
                // Same view, new pixel size: keep looking at the same place at the same size.
                let ratio = width as f32 / self.fitted_for.0 as f32;
                self.centre *= ratio;
                self.zoom /= ratio;
            } else {
                self.zoom = (rect.width() / size.x).min(rect.height() / size.y) * 0.95;
                self.centre = size / 2.0;
                self.fitted_to = Some(rect.size());
            }
            self.fitted_for = (width, height);
        }

        // Drag to pan, scroll to zoom (keeping the point under the mouse still).
        if response.dragged() && !self.hold_pan {
            self.centre -= response.drag_delta() / self.zoom;
            self.fitted_to = None;
        }
        let mouse = response.hover_pos()?;
        let scroll = ui.input(|input| input.smooth_scroll_delta.y);
        if scroll != 0.0 {
            let under = self.centre + (mouse - rect.center()) / self.zoom;
            // Never jumps: zoomed out past the least (fitted to a tiny screen area), a scroll
            // carries on from where it is.
            self.zoom = (self.zoom * (scroll * 0.003).exp()).clamp(0.1f32.min(self.zoom), 256.0);
            self.fitted_to = None;
            self.centre = under - (mouse - rect.center()) / self.zoom;
        }
        let pixel = self.centre + (mouse - rect.center()) / self.zoom;
        (pixel.x >= 0.0 && pixel.y >= 0.0 && pixel.x < size.x && pixel.y < size.y).then_some((pixel.x as usize, pixel.y as usize))
    }

    /// Where a field position (in pixels) is on screen.
    pub fn to_screen(&self, rect: Rect, pixel: Vec2) -> Pos2 {
        rect.center() + (pixel - self.centre) * self.zoom
    }

    /// Where the whole field is on screen.
    pub fn field_rect(&self, rect: Rect, (width, height): (usize, usize)) -> Rect {
        Rect::from_min_size(self.to_screen(rect, Vec2::ZERO), Vec2::new(width as f32, height as f32) * self.zoom)
    }

    /// The field pixels showing on screen: (first column, first row, width, height).
    pub fn visible_pixels(&self, rect: Rect, (width, height): (usize, usize)) -> Option<(usize, usize, usize, usize)> {
        let half = rect.size() / 2.0 / self.zoom;
        let (left, top) = ((self.centre.x - half.x).floor().max(0.0), (self.centre.y - half.y).floor().max(0.0));
        let (right, bottom) = ((self.centre.x + half.x).ceil().min(width as f32), (self.centre.y + half.y).ceil().min(height as f32));
        (right > left && bottom > top).then_some((left as usize, top as usize, (right - left) as usize, (bottom - top) as usize))
    }

    /// Outlines the pixel under the mouse when zoomed in far enough to see it.
    pub fn outline(&self, painter: &egui::Painter, rect: Rect, hover: Option<(usize, usize)>, colour: egui::Color32) {
        if let Some((col, row)) = hover
            && self.zoom >= 4.0
        {
            let cell = Rect::from_min_size(self.to_screen(rect, Vec2::new(col as f32, row as f32)), Vec2::splat(self.zoom));
            painter.rect_stroke(cell, 0.0, egui::Stroke::new(1.0, colour), egui::StrokeKind::Outside);
        }
    }
}
