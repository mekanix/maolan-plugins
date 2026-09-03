# Maolan Synth vs. Surge XT — Sound Quality Improvement Plan

Analysis date: 2026-09-03. Maolan Synth sources: `plugins/src/synth/` and
`plugins/src/common/`. Surge reference: `~/repos/surge` (Surge XT sources;
note the `sst-filters`/`sst-effects`/`sst-basic-blocks`/`sst-waveshapers`
submodules are not checked out there, so comparisons of those internals are
based on Surge's surrounding architecture and documented behavior).

Maolan Synth is a pure-Rust, Surge-inspired reimplementation. Architecturally
it mirrors Surge closely (oscillator types, filter families, LFO/MSEG
modulation, analog-mode ADSR), but the sound-producing cores are simplified or
naive exactly where Surge invests most of its quality, and several features
are outright broken. Items below are ordered roughly by audible impact.

## P0 — Broken features (silence, frozen modulation, dead controls)

1. **Wavetable and Window oscillators output permanent silence.**
   Nothing ever loads a wavetable: no `Wavetable::from_*` call exists, so
   `wavetable` stays `None` and both oscillators return zero
   (`plugins/src/common/oscillator.rs:1698`, `:1971`). There is also no GUI
   loader.
   Plan: add a wavetable file loader (`.wt`/`vawt` and wav files), a bundled
   factory wavetable set, GUI selection, and per-patch wavetable
   persistence in `state.rs`.

2. **Scene LFOs are effectively frozen.**
   `engine.rs:687-694` advances each scene LFO once per *block* and hands the
   frozen value to all voices, while `Lfo::next` advances
   `rate_hz/sample_rate` per call (`common/lfo.rs:649-660`) — so scene LFOs
   run ~buffer-size times too slow (e.g. 512× at 512 frames).
   Plan: advance scene LFOs per sample inside the voice loop (as Surge does
   with SIMD per-sample modulation), or compute per-sample scene-LFO values
   into a block buffer and index it per sample.

3. **MIDI pitch bend is never delivered.**
   The synth MIDI handler matches CC (0xB0) and channel aftertouch (0xD0)
   only (`synth/plugin.rs:2786`, `:2797`); status 0xE0 is ignored and
   `engine.set_pitch_bend` (`engine.rs:542`) is dead code.
   Plan: handle 0xE0, feed the existing `pitch_bend` params with smoothing
   (Surge uses a configurable `Modulator::SmoothingMode` for controllers).

4. **Filter type switching across families is silently broken.**
   `Filter::set_filter_type` is a no-op for Ladder/VintageLadder/DiodeLadder/
   TriPole/SampleHold/OB-Xd/Notch24 (`common/filter.rs:2064-2073`), and the
   enum is never rebuilt on type change (`voice.rs:1585+`). Switching
   families leaves the old DSP running; switching Ladder→SVF makes the SVF
   fall through to passthrough (`filter.rs:431`).
   Plan: rebuild the `Filter` enum variant (reallocating DSP state) whenever
   the requested type maps to a different family; reset state cleanly.

5. **Twist oscillator: only models 0–5 reachable, models 6–15 dead code.**
   `set_shape` maps the shape knob via `TwistModel::from_u8((shape*5.0) as
   u8)` (`common/oscillator.rs:3590`), so Chords, Vowels, Kick, Snare, HiHat,
   Modal, etc. can never be selected.
   Plan: map the shape parameter across all 16 models.

6. **Unison spread parameter is a no-op; detune is capped at ±5 cents.**
   `unison_spread` is stored but never used — `ClassicOsc::update_voices`
   hard-pans by index regardless (`common/oscillator.rs:255-272`), and
   `detune_scale = unison_detune * 0.05` semitones caps detune at ±5 cents
   vs. Surge's ~±100 cents (8 sites, e.g. `oscillator.rs:257`).
   Plan: implement Surge-style unison (`sst::basic_blocks::dsp::UnisonSetup`
   equivalent): per-voice detune bias/offset, pan-law stereo spread honoring
   `unison_spread`, loudness compensation, and widen the detune range.

## P1 — Sound quality gaps vs. Surge

7. **No oversampling in the voice path.**
   Surge renders the entire voice (oscillators, QuadFilterChain, waveshaper,
   feedback) at 2× Fs with steep 6-pole halfband up/down filters
   (`surge/src/common/globals.h:53-54`,
   `SurgeSynthesizer.cpp:4739,4855,4874`). Maolan runs everything at host
   rate; `common/oversample.rs` exists but is unused by the synth. Nonlinear
   stages (waveshaper, ring mod, FM, filter feedback) alias at base rate.
   Plan: add optional 2× oversampling around the per-voice render path
   (reuse the halfband code in `common/oversample.rs`), at minimum around
   waveshaper/feedback/FM-heavy paths. Gate behind a quality parameter.

8. **Oscillators are not band-limited (except Classic saw/pulse).**
   - "Modern" is 7 fixed polyBLEP saws with a hard-coded detune ladder
     (`common/oscillator.rs:2076-2220`) — Surge's Modern is fully
     band-limited DPW (Differentiated Polynomial Waveforms), computed from
     the instantaneous frequency so it stays alias-free under FM/sync
     (`surge/.../ModernOscillator.cpp:106-169`).
   - Twist `next_va` is a completely naive saw/square (`common/twist.rs:364`).
   - Classic hard sync resets phase with no BLEP on the reset edge
     (`oscillator.rs:340-348`) → clicky, aliased sync.
   - Wavetable mipmaps are built with naive 2-tap averaging
     (`common/wavetable.rs:172`) instead of band-limited (FFT) decimation
     with pitch-dependent mipmap selection as in
     `WavetableOscillator.cpp:338-352`.
   Plan: add BLEP to sync edges; implement DPW or BLIT/sinc-convolution
   (Surge Classic uses windowed-sinc tables) for Modern; anti-alias Twist
   models; build wavetable mipmaps with FFT band-limiting.

9. **Cheap nonlinearity: `fast_tanh(x) = x/(1+|x|)`.**
   All filter nonlinearities use this dull, coarse approximation
   (`common/filter.rs:2225`) where Surge uses real tanh / `softclip_ps`
   inside filter feedback (`QuadFilterChain.cpp:107-148`). Filter feedback
   is also a 1-sample-delay loop with no internal saturation
   (`voice.rs:3197`) — it can blow up at fb=1.0.
   Plan: use a proper tanh (or `wide`-SIMD polynomial tanh), and add soft
   clipping inside the feedback loop. With 2× oversampling (#7) this becomes
   the "weight" of the filter sound.

10. **Cutoff/EG/LFO modulation in linear Hz instead of octave space.**
    EG adds ×10000 Hz, LFO ×5000 Hz, keytrack 50 Hz/key
    (`voice.rs:3207-3213`). Surge modulates pitch-space (octaves), which is
    why its sweeps feel musical at any base cutoff.
    Plan: convert filter-cutoff modulation to octave/pitch space
    (multiply-by-ratio rather than add-Hz).

11. **No denormal protection, no output limiter.**
    IIR states and decaying envelopes will hit denormals (no FTZ/DAZ, no DC
    offset in the synth path), and the voice sum has no normalization or
    clip protection (`engine.rs:735-738`) — high polyphony exceeds ±1.
    Surge sets FTZ/DAZ around processing and hard-clips scenes/master at
    +18 dB / 0 dBFS (`SurgeSynthesizer.cpp:4844-4848, 5051-5062`,
    `surge-xt/SurgeSynthProcessor.cpp:569`).
    Plan: enable FTZ/DAZ (or add small DC offsets) in the audio thread;
    add a final soft/hard clip or limiter on the engine output.

12. **StringOsc filter-state bugs.**
    L/R channels share one tone-filter instance (state crosstalk,
    `common/oscillator.rs:2804-2807`), and the stiffness/compliance biquad
    reuses a single state variable across all four delay slots
    (`oscillator.rs:2777-2785`) — a broken biquad.
    Plan: give each channel and each delay slot its own filter state.

## P2 — Polish toward Surge parity

13. **Sample-rate change leaves oscillators stale.**
    `Voice::set_sample_rate` rebuilds filters/EGs/LFOs but not oscillators
    (`voice.rs:2118-2136`) — latent pitch bug if a host changes rate without
    re-activation. Plan: add `set_sample_rate` to the `Oscillator` enum and
    propagate.

14. **No sample-accurate event timing.** All note/CC events apply at block
    start. Plan: sort events by sample offset and split block processing at
    event boundaries (also fixes zipper/stepping for fast automation).

15. **Voice stealing uses a hard restart.** Stolen voices restart with a
    64-sample fade; Surge uses an "uber release" (ultra-fast envelope fade,
    `SurgeSynthesizer.cpp:546-578`) and mono modes reclaim ringing envelopes
    for click-free legato. Plan: add a fast-release steal mode.

16. **SineOsc/WindowOsc clone the L filter per sample to fake the R channel**
    (`common/oscillator.rs:941-943`, `:2035-2041`) — wasteful, and the R
    state never persists. Plan: keep a real second filter instance.

17. **Portamento, microtuning, drift polish.** Surge adds constant-rate
    portamento with log/exp curves and glissando, SCL/KBM + MTS-ESP
    microtuning, and per-unison DriftLFO decorrelation with non-zero drift
    on note-on (`SurgeVoice.cpp:547,666-693`). Maolan has basic drift
    (`voice.rs:3617-3632`) but none of the rest. Plan: implement portamento
    modes first (highest musical value), then optional SCL/KBM tuning.

18. **Minor correctness:** `set_note_tuning` divides by `pitch_bend_range`
    which can be 0 → inf (`engine.rs:613`). Guard the division.

## Suggested order of work

1. P0 items 1–6 (restore broken features) — biggest perceived improvement
   per effort.
2. P1 item 7 (2× voice oversampling) + item 9 (proper tanh + feedback
   softclip) — the single largest "sounds like Surge" change.
3. P1 items 8, 10–12 (band-limited oscillators, octave-space modulation,
   denormal/limiter, StringOsc state).
