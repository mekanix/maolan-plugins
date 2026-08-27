use maolan_plugins::tuner::dsp::Tuner;

const SR: f32 = 48_000.0;

fn sine(freq: f32, n: usize, amp: f32) -> Vec<f32> {
    (0..n)
        .map(|i| amp * (2.0 * std::f32::consts::PI * freq * i as f32 / SR).sin())
        .collect()
}

#[test]
fn detects_a4() {
    let input = sine(440.0, 8192, 0.5);
    let mut tuner = Tuner::new(SR as f64);
    let result = tuner
        .feed_mono(&input)
        .expect("expected a detection result");
    assert!(result.detected, "pitch not detected");
    assert!(
        (result.frequency_hz - 440.0).abs() < 2.0,
        "detected {} Hz",
        result.frequency_hz
    );
    assert!(
        (result.note - 69.0).abs() < 0.1,
        "detected note {}",
        result.note
    );
}

#[test]
fn detects_low_e() {
    let input = sine(82.41, 16384, 0.5);
    let mut tuner = Tuner::new(SR as f64);
    let result = tuner
        .feed_mono(&input)
        .expect("expected a detection result");
    assert!(result.detected, "pitch not detected");
    assert!(
        (result.frequency_hz - 82.41).abs() < 2.0,
        "detected {} Hz",
        result.frequency_hz
    );
}

#[test]
fn silence_is_not_detected() {
    let input = vec![0.0_f32; 8192];
    let mut tuner = Tuner::new(SR as f64);
    let result = tuner.feed_mono(&input);
    assert!(result.is_none() || !result.unwrap().detected);
}
