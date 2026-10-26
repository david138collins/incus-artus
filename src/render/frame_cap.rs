//! Frame cap — how many frames a second the game draws, as a share of the screen's refresh rate.
//!
//! The screen shows a new picture every refresh (240 times a second on David's laptop). Drawing a
//! frame for every refresh is the smoothest, but costs the most power and heat; drawing one every
//! 2nd, 3rd or 4th refresh shows each frame for exactly that many refreshes, so motion stays even.
//! (A cap that isn't a whole share, say 100 on a 240 screen, would show some frames for 2
//! refreshes and some for 3: a judder.)
//!
//! How: vsync keeps every frame on a refresh, and when the cap is a share, the main loop also
//! waits out the rest of each frame's time at the end of the frame. "Uncapped" turns vsync off
//! and draws as fast as it can (for measuring the game's top speed; it can tear).

use super::RenderSettings;
use bevy::prelude::*;
use bevy::window::{Monitor, PresentMode, PrimaryMonitor, PrimaryWindow};
use std::time::{Duration, Instant};

/// The screen's refresh rate (refreshes a second), or 60 if the screen doesn't say.
pub fn refresh_rate(monitors: impl IntoIterator<Item = Option<u32>>) -> f32 {
    monitors.into_iter().flatten().next().map_or(60.0, |millihertz| millihertz as f32 / 1000.0)
}

/// Keeps the window's vsync matching the setting, and waits out the rest of each frame when the
/// cap is a share of the refresh rate. Runs last in the frame.
pub fn cap_frames(
    settings: Res<RenderSettings>,
    monitors: Query<&Monitor, With<PrimaryMonitor>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut due: Local<Option<Instant>>,
) {
    let present = if settings.frame_cap == 0 { PresentMode::AutoNoVsync } else { PresentMode::AutoVsync };
    if let Ok(mut window) = windows.single_mut()
        && window.present_mode != present
    {
        window.present_mode = present;
    }
    if settings.frame_cap <= 1 {
        *due = None;
        return;
    }
    let refresh = refresh_rate(monitors.iter().map(|monitor| monitor.refresh_rate_millihertz));
    let frame = Duration::from_secs_f64(settings.frame_cap as f64 / refresh as f64);
    let now = Instant::now();
    // When this frame should end: exactly a frame's time after the last one was due (so a late
    // wake-up isn't carried into the next frame), or from now if we've fallen behind (a slow
    // frame isn't made up for by rushing the next ones).
    let end = match *due {
        Some(last) if last + frame > now => last + frame,
        _ => now,
    };
    if end > now {
        std::thread::sleep(end - now);
    }
    *due = Some(end);
}
