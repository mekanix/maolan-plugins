use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

/// Surge-style unison detune: per-voice frequency ratio.
///
/// Voices are distributed linearly across the detune range so that
/// `detune = 1.0` spreads the outermost voices by ±1 semitone (±100 cents).
/// A single voice is never detuned (ratio 1.0).
pub fn detune_ratio(i: usize, voices: usize, detune: f32) -> f32 {
    if voices <= 1 {
        return 1.0;
    }
    let pos = i as f32 / (voices as f32 - 1.0);
    let offset = (pos - 0.5) * 2.0 * detune.clamp(0.0, 1.0);
    2.0f32.powf(offset / 12.0)
}

/// Surge-style unison stereo spread: equal-power pan gains for one voice.
///
/// The pan angle spans `±spread * π/4` around the center with a constant-
/// power law. Gains are normalized by `1/√2` so a centered voice keeps the
/// legacy `(0.5, 0.5)` mixing gain; `spread = 0.0` collapses every voice to
/// the center, `spread = 1.0` hard-pans the outermost voices. A single
/// voice is always centered.
pub fn pan_gains(i: usize, voices: usize, spread: f32) -> (f32, f32) {
    if voices <= 1 {
        return (0.5, 0.5);
    }
    let pos = i as f32 / (voices as f32 - 1.0);
    let theta = (pos - 0.5) * spread.clamp(0.0, 1.0) * FRAC_PI_2;
    let norm = FRAC_PI_4.cos();
    (
        (theta + FRAC_PI_4).cos() * norm,
        (theta + FRAC_PI_4).sin() * norm,
    )
}
