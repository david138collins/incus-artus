//! A slider that also turns with the mouse wheel: hover over it (the bar or its number) and roll
//! the wheel to nudge the value, without dragging. Otherwise exactly egui's slider: it's used
//! everywhere in its place, built the same way (`Slider::new(&mut value, range).text("…")`).
//!
//! **How far one notch moves it:**
//! - whole numbers: by one;
//! - logarithmic sliders (ones spanning, say, 1 to 10,000): by 5% of the value, so small and large
//!   values both move sensibly;
//! - others: by 1% of the slider's range.
//!
//! Hold **Shift** for finer steps (a tenth as far; whole numbers still move by one).

use bevy_egui::egui::{self, Response, Ui, Vec2, Widget, WidgetText, emath::Numeric};
use std::ops::RangeInclusive;

pub struct Slider<'a, N: Numeric> {
    value: &'a mut N,
    range: RangeInclusive<N>,
    text: Option<WidgetText>,
    logarithmic: bool,
    suffix: Option<String>,
    integer: bool,
}

impl<'a, N: Numeric> Slider<'a, N> {
    pub fn new(value: &'a mut N, range: RangeInclusive<N>) -> Self {
        Self { value, range, text: None, logarithmic: false, suffix: None, integer: false }
    }

    /// Whole numbers only (even when the value is stored as a decimal).
    pub fn integer(mut self) -> Self {
        self.integer = true;
        self
    }

    pub fn text(mut self, text: impl Into<WidgetText>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn logarithmic(mut self, logarithmic: bool) -> Self {
        self.logarithmic = logarithmic;
        self
    }

    pub fn suffix(mut self, suffix: impl ToString) -> Self {
        self.suffix = Some(suffix.to_string());
        self
    }
}

/// Where a value goes after `notches` turns of the wheel (+ = up), on a slider from `low` to
/// `high`.
fn turned(value: f64, low: f64, high: f64, notches: f64, integral: bool, logarithmic: bool, fine: bool) -> f64 {
    let (low, high) = (low.min(high), low.max(high));
    let next = if integral {
        value + notches
    } else if logarithmic {
        // Up from nothing on a log slider: start from a thousandth of the range.
        let from = if value > 0.0 { value } else { (high - low) * 0.001 };
        from * (if fine { 1.005f64 } else { 1.05 }).powf(notches)
    } else {
        value + notches * (high - low) / if fine { 1_000.0 } else { 100.0 }
    };
    next.clamp(low, high)
}

impl<N: Numeric> Widget for Slider<'_, N> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut slider = egui::Slider::new(&mut *self.value, self.range.clone()).logarithmic(self.logarithmic);
        if let Some(text) = self.text {
            slider = slider.text(text);
        }
        if let Some(suffix) = self.suffix {
            slider = slider.suffix(suffix);
        }
        if self.integer {
            slider = slider.integer();
        }
        let mut response = ui.add(slider);
        if response.hovered() {
            // The wheel's turns this frame. (Shift turns it sideways on some systems, so take
            // whichever way it turned.)
            let (turn, fine) = ui.input(|input| {
                let mut turn = 0.0f32;
                for event in &input.events {
                    if let egui::Event::MouseWheel { delta, .. } = event {
                        turn += if delta.y != 0.0 { delta.y } else { delta.x };
                    }
                }
                (turn, input.modifiers.shift)
            });
            if turn != 0.0 {
                let value = turned(self.value.to_f64(), self.range.start().to_f64(), self.range.end().to_f64(), turn.signum() as f64, N::INTEGRAL || self.integer, self.logarithmic, fine);
                *self.value = N::from_f64(value);
                response.mark_changed();
            }
            // While over a slider, the wheel is for the slider: don't also scroll the panel.
            ui.input_mut(|input| input.smooth_scroll_delta = Vec2::ZERO);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::turned;

    #[test]
    fn a_notch_moves_by_a_sensible_amount_and_stays_in_range() {
        // Linear: 1% of the range (a tenth of that with Shift).
        assert!((turned(0.5, 0.0, 1.0, 1.0, false, false, false) - 0.51).abs() < 1e-9);
        assert!((turned(0.5, 0.0, 1.0, -1.0, false, false, true) - 0.499).abs() < 1e-9);
        // Whole numbers: by one.
        assert_eq!(turned(3.0, 0.0, 30.0, 1.0, true, false, false), 4.0);
        // Logarithmic: by 5% of the value.
        assert!((turned(100.0, 1.0, 10_000.0, 1.0, false, true, false) - 105.0).abs() < 1e-9);
        // Never past the ends.
        assert_eq!(turned(1.0, 0.0, 1.0, 1.0, false, false, false), 1.0);
        assert_eq!(turned(0.0, 0.0, 1.0, -1.0, false, false, false), 0.0);
        // Up from zero on a log slider gets going.
        assert!(turned(0.0, 0.0, 10.0, 1.0, false, true, false) > 0.0);
    }
}
