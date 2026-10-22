//! Kinematics — the shapes of motion: how movements build up, hold and die away.
//!
//! For now this holds the ADSR envelope (used for the walk → sprint ramp) and its editor.
//! Later, the other motion factors (starting to walk, stopping, turning, mounts, carts…)
//! will use the same envelopes, so everything that moves can be tuned the same way.

mod editor;
mod envelope;

pub use editor::envelope_editor;
pub use envelope::{Envelope, EnvelopeState};
