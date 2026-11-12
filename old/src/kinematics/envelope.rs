//! Envelopes — how a motion builds up and dies away over time.
//!
//! This is the same idea as a synth's ADSR envelope, applied to movement instead of volume.
//! While a "gate" is held (for sprinting: Ctrl held while moving), the envelope's level runs:
//!
//! ```text
//!   level
//!   1 ┤    ╱╲
//!     │   ╱  ╲______         Attack:  rises from 0 to 1 over `attack` seconds
//!     │  ╱          ╲        Decay:   falls from 1 to the sustain level over `decay` seconds
//!     │ ╱            ╲       Sustain: holds that level for as long as the gate is held
//!   0 ┼╱──────────────╲──    Release: once let go, falls to 0 over `release` seconds
//!      A   D    S     R
//! ```
//!
//! The level (0 to 1) is then used to blend between two values: for sprinting, between
//! walking speed (0) and full sprint speed (1).
//!
//! Each moving part has a *curve* from −1 to +1: 0 is a straight line, + starts fast and
//! eases in to the end, − starts slowly and speeds up towards the end.
//!
//! If the gate is pressed again during the release, the attack starts from the current level
//! instead of jumping back to 0, so there's never a sudden jump in speed.

use serde::{Deserialize, Serialize};

/// The shape of an envelope (the tunable part).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Envelope {
    /// Seconds to rise from 0 to 1.
    pub attack: f32,
    pub attack_curve: f32,
    /// Seconds to fall from 1 to the sustain level.
    pub decay: f32,
    pub decay_curve: f32,
    /// The level held while the gate stays on (0 to 1).
    pub sustain: f32,
    /// Seconds to fall to 0 after the gate is let go.
    pub release: f32,
    pub release_curve: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 1.0,
            attack_curve: 0.0,
            decay: 0.0,
            decay_curve: 0.0,
            sustain: 1.0,
            release: 1.0,
            release_curve: 0.0,
        }
    }
}

/// Bends a 0‥1 progress value. `curve` 0 = straight, + = fast start, − = slow start.
pub fn shape(progress: f32, curve: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if curve.abs() < 1e-3 {
        return progress;
    }
    // +1 → power 1/8 (shoots up then levels off), −1 → power 8 (creeps then shoots up).
    progress.powf(2f32.powf(-3.0 * curve.clamp(-1.0, 1.0)))
}

/// Which part of the envelope is playing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// Gate off and fully released: level 0.
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// One running copy of an envelope (each actor has its own).
#[derive(Clone, Copy, Debug, Default)]
pub struct EnvelopeState {
    pub stage: Stage,
    /// The current output, 0 to 1.
    pub level: f32,
    /// Seconds since the current stage began.
    elapsed: f32,
    /// The level when the current stage began.
    from: f32,
    /// True for one update after the gate was pressed (the envelope "fired").
    triggered: bool,
}

impl EnvelopeState {
    /// Seconds since the current stage began (for drawing the playhead).
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// How long the current stage lasts in total, for drawing the playhead.
    pub fn stage_length(&self, envelope: &Envelope) -> f32 {
        match self.stage {
            Stage::Attack => envelope.attack * (1.0 - self.from),
            Stage::Decay => envelope.decay,
            Stage::Release => envelope.release,
            Stage::Idle | Stage::Sustain => 0.0,
        }
    }

    fn enter(&mut self, stage: Stage) {
        self.stage = stage;
        self.elapsed = 0.0;
        self.from = self.level;
    }

    /// Did the most recent update start a new attack? (A hook for footsteps and the like.)
    pub fn just_triggered(&self) -> bool {
        self.triggered
    }

    /// Move the envelope on by `dt` seconds with the gate on or off. Returns the new level.
    pub fn update(&mut self, envelope: &Envelope, gate: bool, dt: f32) -> f32 {
        self.triggered = false;
        match (gate, self.stage) {
            (true, Stage::Idle | Stage::Release) => {
                self.enter(Stage::Attack);
                self.triggered = true;
            }
            (false, Stage::Attack | Stage::Decay | Stage::Sustain) => self.enter(Stage::Release),
            _ => {}
        }
        self.elapsed += dt.max(0.0);
        let sustain = envelope.sustain.clamp(0.0, 1.0);

        // A loop, because one update can finish a short stage and carry on into the next.
        loop {
            let length = self.stage_length(envelope);
            let progress = if length > 0.0 { self.elapsed / length } else { 1.0 };
            match self.stage {
                Stage::Idle => {
                    self.level = 0.0;
                    self.elapsed = 0.0;
                    break;
                }
                Stage::Sustain => {
                    self.level = sustain;
                    break;
                }
                _ if progress >= 1.0 => {
                    // This stage is over: land exactly on its end level, and carry the
                    // left-over time into the next stage.
                    let (end, next) = match self.stage {
                        Stage::Attack => (1.0, Stage::Decay),
                        Stage::Decay => (sustain, Stage::Sustain),
                        _ => (0.0, Stage::Idle),
                    };
                    let left_over = self.elapsed - length;
                    self.level = end;
                    self.enter(next);
                    self.elapsed = left_over;
                }
                Stage::Attack => {
                    self.level = self.from + (1.0 - self.from) * shape(progress, envelope.attack_curve);
                    break;
                }
                Stage::Decay => {
                    self.level = self.from + (sustain - self.from) * shape(progress, envelope.decay_curve);
                    break;
                }
                Stage::Release => {
                    self.level = self.from * (1.0 - shape(progress, envelope.release_curve));
                    break;
                }
            }
        }
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICK: f32 = 1.0 / 30.0;

    fn run(state: &mut EnvelopeState, envelope: &Envelope, gate: bool, seconds: f32) {
        let ticks = (seconds / TICK).round() as u32;
        for _ in 0..ticks {
            state.update(envelope, gate, TICK);
        }
    }

    #[test]
    fn rises_holds_and_falls() {
        let envelope = Envelope {
            attack: 2.0,
            decay: 1.0,
            sustain: 0.5,
            release: 1.0,
            ..Envelope::default()
        };
        let mut state = EnvelopeState::default();
        run(&mut state, &envelope, true, 1.0);
        assert!((state.level - 0.5).abs() < 0.02, "half way up the attack: {}", state.level);
        run(&mut state, &envelope, true, 1.1);
        assert_eq!(state.stage, Stage::Decay);
        run(&mut state, &envelope, true, 3.0);
        assert_eq!(state.stage, Stage::Sustain);
        assert_eq!(state.level, 0.5);
        run(&mut state, &envelope, false, 1.1);
        assert_eq!(state.stage, Stage::Idle);
        assert_eq!(state.level, 0.0);
    }

    #[test]
    fn pressing_again_during_release_never_jumps() {
        let envelope = Envelope::default();
        let mut state = EnvelopeState::default();
        run(&mut state, &envelope, true, 2.0);
        run(&mut state, &envelope, false, 0.5);
        let before = state.level;
        state.update(&envelope, true, TICK);
        assert!(state.level >= before && state.level - before < 0.1);
    }

    #[test]
    fn fires_once_per_press() {
        let envelope = Envelope::default();
        let mut state = EnvelopeState::default();
        state.update(&envelope, true, TICK);
        assert!(state.just_triggered());
        state.update(&envelope, true, TICK);
        assert!(!state.just_triggered(), "holding doesn't fire again");
        state.update(&envelope, false, TICK);
        state.update(&envelope, true, TICK);
        assert!(state.just_triggered(), "a quick re-tap during the release fires");
    }

    #[test]
    fn zero_length_stages_are_instant() {
        let envelope = Envelope {
            attack: 0.0,
            decay: 0.0,
            sustain: 0.7,
            release: 0.0,
            ..Envelope::default()
        };
        let mut state = EnvelopeState::default();
        assert_eq!(state.update(&envelope, true, TICK), 0.7);
        assert_eq!(state.update(&envelope, false, TICK), 0.0);
    }

    #[test]
    fn curves_keep_their_end_points() {
        for curve in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            assert_eq!(shape(0.0, curve), 0.0);
            assert_eq!(shape(1.0, curve), 1.0);
        }
        assert!(shape(0.5, 1.0) > 0.5, "+ starts fast");
        assert!(shape(0.5, -1.0) < 0.5, "− starts slow");
    }
}
