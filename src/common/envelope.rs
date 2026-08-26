#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Shared ADSR/DAHDSR parameter bundle.
///
/// Plugins with multiple envelope generators can use this to pass stage
/// times/levels around without listing individual fields. The vel2/key2 fields
/// are SFZ-style per-voice offsets that are applied when the envelope is
/// triggered (velocity 0 gets the full offset, velocity 127 gets none).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdsrParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub delay: f32,
    pub hold: f32,
    pub start: f32,
    pub end: f32,
    pub attack_shape: AttackShape,
    pub decay_shape: DecayReleaseShape,
    pub release_shape: DecayReleaseShape,
    pub vel2_attack: f32,
    pub vel2_decay: f32,
    pub vel2_sustain: f32,
    pub vel2_release: f32,
    pub vel2_delay: f32,
    pub vel2_hold: f32,
    pub vel2_start: f32,
    pub vel2_end: f32,
    pub key2_attack: f32,
    pub key2_decay: f32,
    pub key2_sustain: f32,
    pub key2_release: f32,
    pub key2_delay: f32,
    pub key2_hold: f32,
    pub key2_start: f32,
    pub key2_end: f32,
}

impl Default for AdsrParams {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.2,
            sustain: 1.0,
            release: 0.3,
            delay: 0.0,
            hold: 0.0,
            start: 0.0,
            end: 0.0,
            attack_shape: AttackShape::Convex,
            decay_shape: DecayReleaseShape::Linear,
            release_shape: DecayReleaseShape::Linear,
            vel2_attack: 0.0,
            vel2_decay: 0.0,
            vel2_sustain: 0.0,
            vel2_release: 0.0,
            vel2_delay: 0.0,
            vel2_hold: 0.0,
            vel2_start: 0.0,
            vel2_end: 0.0,
            key2_attack: 0.0,
            key2_decay: 0.0,
            key2_sustain: 0.0,
            key2_release: 0.0,
            key2_delay: 0.0,
            key2_hold: 0.0,
            key2_start: 0.0,
            key2_end: 0.0,
        }
    }
}

impl AdsrParams {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            ..Default::default()
        }
    }

    /// Return a copy with SFZ vel2/key2 offsets applied for the given note and velocity.
    pub fn scaled_for_voice(&self, note: u8, velocity: u8) -> Self {
        let vel = velocity as f32 / 127.0;
        let key = note as f32 / 127.0;
        let vel_factor = 1.0 - vel;
        let key_factor = 1.0 - key;

        let scale_time =
            |base: f32, v2: f32, k2: f32| (base + v2 * vel_factor + k2 * key_factor).max(0.0);
        let scale_level = |base: f32, v2: f32, k2: f32| {
            (base + v2 * vel_factor + k2 * key_factor).clamp(0.0, 1.0)
        };

        Self {
            attack: scale_time(self.attack, self.vel2_attack, self.key2_attack),
            decay: scale_time(self.decay, self.vel2_decay, self.key2_decay),
            release: scale_time(self.release, self.vel2_release, self.key2_release),
            delay: scale_time(self.delay, self.vel2_delay, self.key2_delay),
            hold: scale_time(self.hold, self.vel2_hold, self.key2_hold),
            sustain: scale_level(self.sustain, self.vel2_sustain, self.key2_sustain),
            start: scale_level(self.start, self.vel2_start, self.key2_start),
            end: scale_level(self.end, self.vel2_end, self.key2_end),
            ..*self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeMode {
    Digital,
    Analog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeRetriggerMode {
    Reset = 0,
    Continue = 1,
}

impl EnvelopeRetriggerMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => EnvelopeRetriggerMode::Reset,
            _ => EnvelopeRetriggerMode::Continue,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackShape {
    Convex = 0,
    Linear = 1,
    Concave = 2,
}

impl AttackShape {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => AttackShape::Linear,
            2 => AttackShape::Concave,
            _ => AttackShape::Convex,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecayReleaseShape {
    Linear = 0,
    Quadratic = 1,
    Cubic = 2,
}

impl DecayReleaseShape {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => DecayReleaseShape::Quadratic,
            2 => DecayReleaseShape::Cubic,
            _ => DecayReleaseShape::Linear,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EnvelopeState {
    Delay,
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
    Idle,
}

#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    sample_rate: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    delay: f32,
    hold: f32,
    start: f32,
    end: f32,
    mode: EnvelopeMode,
    attack_shape: AttackShape,
    decay_shape: DecayReleaseShape,
    release_shape: DecayReleaseShape,
    retrigger_mode: EnvelopeRetriggerMode,
    tempo_sync: bool,
    tempo_bpm: f32,
    uber_release: f32,
    gated_release: bool,
    correct_analog_mode: bool,

    state: EnvelopeState,
    phase: f32,
    decay_phase: f32,
    hold_phase: f32,
    release_phase: f32,
    output: f32,
    gate: bool,
    released: bool,
    release_start_level: f32,
}

impl AdsrEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
            delay: 0.0,
            hold: 0.0,
            start: 0.0,
            end: 0.0,
            mode: EnvelopeMode::Digital,
            attack_shape: AttackShape::Convex,
            decay_shape: DecayReleaseShape::Linear,
            release_shape: DecayReleaseShape::Linear,
            retrigger_mode: EnvelopeRetriggerMode::Reset,
            tempo_sync: false,
            tempo_bpm: 120.0,
            uber_release: 0.0,
            gated_release: false,
            correct_analog_mode: false,
            state: EnvelopeState::Idle,
            phase: 0.0,
            decay_phase: 0.0,
            hold_phase: 0.0,
            release_phase: 0.0,
            output: 0.0,
            gate: false,
            released: false,
            release_start_level: 0.0,
        }
    }

    pub fn value(&self) -> f32 {
        self.output
    }

    pub fn set_params(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.attack = attack.max(0.0);
        self.decay = decay.max(0.0);
        self.sustain = sustain.clamp(0.0, 1.0);
        self.release = release.max(0.0);
    }

    pub fn set_extended_params(&mut self, params: AdsrParams) {
        self.attack = params.attack.max(0.0);
        self.decay = params.decay.max(0.0);
        self.sustain = params.sustain.clamp(0.0, 1.0);
        self.release = params.release.max(0.0);
        self.delay = params.delay.max(0.0);
        self.hold = params.hold.max(0.0);
        self.start = params.start.clamp(0.0, 1.0);
        self.end = params.end.clamp(0.0, 1.0);
        self.attack_shape = params.attack_shape;
        self.decay_shape = params.decay_shape;
        self.release_shape = params.release_shape;
    }

    pub fn params(&self) -> AdsrParams {
        AdsrParams {
            attack: self.attack,
            decay: self.decay,
            sustain: self.sustain,
            release: self.release,
            delay: self.delay,
            hold: self.hold,
            start: self.start,
            end: self.end,
            attack_shape: self.attack_shape,
            decay_shape: self.decay_shape,
            release_shape: self.release_shape,
            ..Default::default()
        }
    }

    pub fn set_mode(&mut self, mode: EnvelopeMode) {
        self.mode = mode;
    }

    pub fn set_shapes(
        &mut self,
        attack: AttackShape,
        decay: DecayReleaseShape,
        release: DecayReleaseShape,
    ) {
        self.attack_shape = attack;
        self.decay_shape = decay;
        self.release_shape = release;
    }

    pub fn set_retrigger_mode(&mut self, mode: EnvelopeRetriggerMode) {
        self.retrigger_mode = mode;
    }

    pub fn retrigger_mode(&self) -> EnvelopeRetriggerMode {
        self.retrigger_mode
    }

    pub fn set_attack(&mut self, attack: f32) {
        self.attack = attack.max(0.0);
    }

    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.max(0.0);
    }

    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain.clamp(0.0, 1.0);
    }

    pub fn set_release(&mut self, release: f32) {
        self.release = release.max(0.0);
    }

    pub fn set_delay(&mut self, delay: f32) {
        self.delay = delay.max(0.0);
    }

    pub fn set_hold(&mut self, hold: f32) {
        self.hold = hold.max(0.0);
    }

    pub fn set_start(&mut self, start: f32) {
        self.start = start.clamp(0.0, 1.0);
    }

    pub fn set_end(&mut self, end: f32) {
        self.end = end.clamp(0.0, 1.0);
    }

    pub fn set_tempo_sync(&mut self, sync: bool) {
        self.tempo_sync = sync;
    }

    pub fn set_tempo(&mut self, tempo_bpm: f32) {
        self.tempo_bpm = tempo_bpm;
    }

    pub fn set_uber_release(&mut self, uber: f32) {
        self.uber_release = uber.clamp(0.0, 1.0);
    }

    pub fn set_gated_release(&mut self, gated: bool) {
        self.gated_release = gated;
    }

    pub fn set_correct_analog_mode(&mut self, v: bool) {
        self.correct_analog_mode = v;
    }

    #[allow(dead_code)]
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.phase = 0.0;
        self.decay_phase = 0.0;
        self.hold_phase = 0.0;
        self.release_phase = 0.0;
        self.output = 0.0;
        self.gate = false;
        self.released = false;
        self.release_start_level = 0.0;
    }

    pub fn trigger(&mut self) {
        self.gate = true;
        self.released = false;
        match self.retrigger_mode {
            EnvelopeRetriggerMode::Reset => {
                self.state = EnvelopeState::Delay;
                self.phase = 0.0;
                self.decay_phase = 0.0;
                self.hold_phase = 0.0;
                self.release_phase = 0.0;
                self.output = self.start;
            }
            EnvelopeRetriggerMode::Continue => {
                self.state = EnvelopeState::Delay;
                self.phase = 0.0;
                self.decay_phase = 0.0;
                self.hold_phase = 0.0;
                self.release_phase = 0.0;

                if self.output > self.start && self.attack > 0.0 {
                    self.phase = ((self.output - self.start) / (1.0 - self.start)).clamp(0.0, 1.0);
                } else {
                    self.output = self.start;
                }
            }
        }
        self.skip_zero_time_stages();
    }

    fn skip_zero_time_stages(&mut self) {
        let beat_to_sec = |beats: f32| -> f32 {
            if self.tempo_sync && self.tempo_bpm > 0.0 {
                beats * 60.0 / self.tempo_bpm
            } else {
                beats
            }
        };

        if beat_to_sec(self.delay) <= 0.0 {
            self.state = EnvelopeState::Attack;
        } else {
            return;
        }

        if beat_to_sec(self.attack) <= 0.0 {
            self.output = 1.0;
            self.state = EnvelopeState::Hold;
        } else {
            return;
        }

        if beat_to_sec(self.hold) <= 0.0 {
            self.state = EnvelopeState::Decay;
        } else {
            return;
        }

        if beat_to_sec(self.decay) <= 0.0 {
            self.output = self.sustain;
            self.state = EnvelopeState::Sustain;
        }
    }

    pub fn release(&mut self) {
        self.gate = false;
        if !self.gated_release && self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.release_start_level = self.output;
            self.release_phase = 0.0;
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f32 {
        let beat_to_sec = |beats: f32| -> f32 {
            if self.tempo_sync && self.tempo_bpm > 0.0 {
                beats * 60.0 / self.tempo_bpm
            } else {
                beats
            }
        };
        let attack = beat_to_sec(self.attack);
        let decay = beat_to_sec(self.decay);
        let release = beat_to_sec(self.release);
        let delay = beat_to_sec(self.delay);
        let hold = beat_to_sec(self.hold);
        let dt = 1.0 / self.sample_rate;

        match self.state {
            EnvelopeState::Delay => {
                if !self.gate {
                    self.state = EnvelopeState::Release;
                    self.release_start_level = self.output;
                    self.release_phase = 0.0;
                } else if delay > 0.0 {
                    self.phase += dt / delay;
                    if self.phase >= 1.0 {
                        self.phase = 0.0;
                        self.state = EnvelopeState::Attack;
                    }
                } else {
                    self.state = EnvelopeState::Attack;
                }
                self.output = self.start;
            }
            EnvelopeState::Attack => {
                if !self.gate {
                    self.state = EnvelopeState::Release;
                    self.release_start_level = self.output;
                    self.release_phase = 0.0;
                } else if attack <= 0.0 {
                    self.phase = 1.0;
                    self.output = 1.0;
                    self.state = EnvelopeState::Hold;
                } else if self.mode == EnvelopeMode::Analog {
                    let time_const = if self.correct_analog_mode { 7.0 } else { 5.0 };
                    let attack_coef = 1.0 - (-time_const / (attack * self.sample_rate)).exp();
                    let target = 1.0f32.max(self.start);
                    self.output += (target - self.output) * attack_coef;
                    if self.output >= 0.999 {
                        self.output = 1.0;
                        self.phase = 1.0;
                        self.state = EnvelopeState::Hold;
                    }
                } else {
                    let attack_rate = 1.0 / (attack * self.sample_rate);
                    self.phase += attack_rate;
                    if self.phase >= 1.0 {
                        self.phase = 1.0;
                        self.output = 1.0;
                        self.state = EnvelopeState::Hold;
                    } else {
                        let shaped = self.attack_shape_value(self.phase);
                        self.output = self.start + (1.0 - self.start) * shaped;
                    }
                }
            }
            EnvelopeState::Hold => {
                if !self.gate {
                    self.state = EnvelopeState::Release;
                    self.release_start_level = self.output;
                    self.release_phase = 0.0;
                } else if hold > 0.0 {
                    self.hold_phase += dt / hold;
                    if self.hold_phase >= 1.0 {
                        self.hold_phase = 1.0;
                        self.state = EnvelopeState::Decay;
                    }
                } else {
                    self.state = EnvelopeState::Decay;
                }
                self.output = 1.0;
            }
            EnvelopeState::Decay => {
                if !self.gate {
                    self.state = EnvelopeState::Release;
                    self.release_start_level = self.output;
                    self.release_phase = 0.0;
                } else {
                    let decay_rate = if decay > 0.0 {
                        1.0 / (decay * self.sample_rate)
                    } else {
                        1.0
                    };
                    let target = self.sustain;
                    if self.mode == EnvelopeMode::Analog {
                        let time_const = if self.correct_analog_mode { 7.0 } else { 5.0 };
                        let coef = (-decay_rate * time_const).exp();
                        self.output = target + (self.output - target) * coef;
                        if (self.output - target).abs() < 0.001 {
                            self.output = target;
                            self.state = EnvelopeState::Sustain;
                        }
                    } else {
                        self.decay_phase += decay_rate;
                        if self.decay_phase >= 1.0 {
                            self.decay_phase = 1.0;
                            self.output = target;
                            self.state = EnvelopeState::Sustain;
                        } else {
                            let shaped = self.decay_shape_value(self.decay_phase);
                            self.output = target + (1.0 - target) * (1.0 - shaped);
                        }
                    }
                }
            }
            EnvelopeState::Sustain => {
                if !self.gate {
                    self.state = EnvelopeState::Release;
                    self.release_start_level = self.output;
                    self.release_phase = 0.0;
                }
                self.output = self.sustain;
            }
            EnvelopeState::Release => {
                if release <= 0.0 || self.output <= self.end {
                    self.output = self.end;
                    self.state = EnvelopeState::Idle;
                } else {
                    let release_rate = 1.0 / (release * self.sample_rate);
                    let uber_mul = 1.0 + self.uber_release * 9.0;
                    let threshold = self.release_start_level * 0.1;
                    let use_uber = self.uber_release > 0.0 && self.output < threshold;
                    if self.mode == EnvelopeMode::Analog {
                        let time_const = if self.correct_analog_mode { 7.0 } else { 5.0 };
                        let coef = if use_uber {
                            (-release_rate * time_const * uber_mul).exp()
                        } else {
                            (-release_rate * time_const).exp()
                        };
                        let target = self.end;
                        self.output = target + (self.output - target) * coef;
                        if (self.output - target).abs() < 0.001 {
                            self.output = target;
                            self.state = EnvelopeState::Idle;
                        }
                    } else {
                        let effective_rate = if use_uber {
                            release_rate * uber_mul
                        } else {
                            release_rate
                        };
                        self.release_phase += effective_rate;
                        if self.release_phase >= 1.0 {
                            self.output = self.end;
                            self.state = EnvelopeState::Idle;
                        } else {
                            let shaped = self.release_shape_value(self.release_phase);
                            self.output = self.release_start_level
                                + (self.end - self.release_start_level) * shaped;
                        }
                    }
                }
            }
            EnvelopeState::Idle => {
                self.output = self.end;
            }
        }
        self.output
    }

    #[allow(dead_code)]
    pub fn process_block(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            *sample = self.next();
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle || self.output > 1.0e-6 || self.gate
    }

    fn attack_shape_value(&self, phase: f32) -> f32 {
        let p = phase.clamp(0.0, 1.0);
        match self.attack_shape {
            AttackShape::Convex => p.sqrt(),
            AttackShape::Linear => p,
            AttackShape::Concave => p * p,
        }
    }

    fn decay_shape_value(&self, phase: f32) -> f32 {
        let p = phase.clamp(0.0, 1.0);
        match self.decay_shape {
            DecayReleaseShape::Linear => p,
            DecayReleaseShape::Quadratic => p * p,
            DecayReleaseShape::Cubic => p * p * p,
        }
    }

    fn release_shape_value(&self, phase: f32) -> f32 {
        let p = phase.clamp(0.0, 1.0);
        match self.release_shape {
            DecayReleaseShape::Linear => p,
            DecayReleaseShape::Quadratic => p * p,
            DecayReleaseShape::Cubic => p * p * p,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvPoint {
    pub t: f32,
    pub v: f32,
    pub cp_t: f32,
    pub cp_v: f32,
}

impl EnvPoint {
    pub const fn new(t: f32, v: f32) -> Self {
        Self {
            t,
            v,
            cp_t: 0.33,
            cp_v: 0.0,
        }
    }

    pub const fn with_control(t: f32, v: f32, cp_t: f32, cp_v: f32) -> Self {
        Self { t, v, cp_t, cp_v }
    }
}

#[derive(Debug, Clone)]
pub struct Envelope {
    points: Vec<EnvPoint>,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            points: vec![EnvPoint::new(0.0, 1.0), EnvPoint::new(1.0, 0.0)],
        }
    }
}

impl Envelope {
    pub fn new(points: Vec<EnvPoint>) -> Self {
        let mut env = Self { points };
        env.sort_and_dedup();
        env
    }

    pub fn with_default_adsr(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        let total = attack + decay + release;
        if total <= 0.0 {
            return Self::default();
        }
        let mut points = vec![
            EnvPoint::new(0.0, 0.0),
            EnvPoint::new(attack / total, 1.0),
            EnvPoint::new((attack + decay) / total, sustain.clamp(0.0, 1.0)),
            EnvPoint::new(1.0, 0.0),
        ];
        if attack <= 0.0 {
            points.remove(0);
        }
        Self::new(points)
    }

    pub fn flat(value: f32) -> Self {
        Self::new(vec![EnvPoint::new(0.0, value), EnvPoint::new(1.0, value)])
    }

    fn sort_and_dedup(&mut self) {
        self.points.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());
        self.points.dedup_by(|a, b| (a.t - b.t).abs() < 1.0e-6);
    }

    pub fn value(&self, t: f32) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if t <= self.points[0].t {
            return self.points[0].v;
        }
        if t >= self.points.last().unwrap().t {
            return self.points.last().unwrap().v;
        }
        for i in 1..self.points.len() {
            let p0 = &self.points[i - 1];
            let p1 = &self.points[i];
            if t >= p0.t && t <= p1.t {
                let dt = p1.t - p0.t;
                if dt < 1.0e-9 {
                    return p0.v;
                }
                let frac = (t - p0.t) / dt;
                let cp0_v = p0.v + p0.cp_v;
                let cp1_v = p1.v - p1.cp_v;
                return cubic_bezier(frac, p0.v, cp0_v, cp1_v, p1.v);
            }
        }
        self.points.last().unwrap().v
    }

    pub fn linear_value(&self, t: f32) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if t <= self.points[0].t {
            return self.points[0].v;
        }
        if t >= self.points.last().unwrap().t {
            return self.points.last().unwrap().v;
        }
        for i in 1..self.points.len() {
            let p0 = &self.points[i - 1];
            let p1 = &self.points[i];
            if t >= p0.t && t <= p1.t {
                return linear_between(t, p0, p1);
            }
        }
        self.points.last().unwrap().v
    }

    pub fn fill_buffer(&self, out: &mut [f32], dt_per_sample: f32) {
        if out.is_empty() {
            return;
        }

        if let Some(first) = self.points.first()
            && self.points.iter().all(|p| (p.v - first.v).abs() < 1.0e-9)
        {
            out.fill(first.v);
            return;
        }

        if self.points.len() == 1 {
            out.fill(self.points[0].v);
            return;
        }

        let mut seg = 0usize;
        for (i, s) in out.iter_mut().enumerate() {
            let t = i as f32 * dt_per_sample;

            while seg + 1 < self.points.len() && t > self.points[seg + 1].t {
                seg += 1;
            }
            *s = self.value_at_segment(t, seg);
        }
    }

    pub fn fill_buffer_linear(&self, out: &mut [f32], dt_per_sample: f32) {
        if out.is_empty() {
            return;
        }

        if let Some(first) = self.points.first()
            && self.points.iter().all(|p| (p.v - first.v).abs() < 1.0e-9)
        {
            out.fill(first.v);
            return;
        }

        if self.points.len() == 1 {
            out.fill(self.points[0].v);
            return;
        }

        let mut seg = 0usize;
        for (i, s) in out.iter_mut().enumerate() {
            let t = i as f32 * dt_per_sample;

            while seg + 1 < self.points.len() && t > self.points[seg + 1].t {
                seg += 1;
            }
            *s = self.linear_value_at_segment(t, seg);
        }
    }

    #[inline]
    fn value_at_segment(&self, t: f32, seg: usize) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if t <= self.points[0].t {
            return self.points[0].v;
        }
        let last = self.points.len() - 1;
        if t >= self.points[last].t {
            return self.points[last].v;
        }
        let i = seg.min(last);
        let p0 = &self.points[i];
        let p1 = &self.points[(i + 1).min(last)];
        let dt = p1.t - p0.t;
        if dt < 1.0e-9 {
            return p0.v;
        }
        let frac = ((t - p0.t) / dt).clamp(0.0, 1.0);
        let cp0_v = p0.v + p0.cp_v;
        let cp1_v = p1.v - p1.cp_v;
        cubic_bezier(frac, p0.v, cp0_v, cp1_v, p1.v)
    }

    #[inline]
    fn linear_value_at_segment(&self, t: f32, seg: usize) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if t <= self.points[0].t {
            return self.points[0].v;
        }
        let last = self.points.len() - 1;
        if t >= self.points[last].t {
            return self.points[last].v;
        }
        let i = seg.min(last);
        let p0 = &self.points[i];
        let p1 = &self.points[(i + 1).min(last)];
        linear_between(t, p0, p1)
    }

    pub fn points(&self) -> &[EnvPoint] {
        &self.points
    }

    pub fn points_mut(&mut self) -> &mut Vec<EnvPoint> {
        &mut self.points
    }
}

#[inline]
fn cubic_bezier(t: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    let u = 1.0 - t;
    let u2 = u * u;
    let t2 = t * t;
    u2 * u * p0 + 3.0 * u2 * t * p1 + 3.0 * u * t2 * p2 + t2 * t * p3
}

#[inline]
fn linear_between(t: f32, p0: &EnvPoint, p1: &EnvPoint) -> f32 {
    let dt = p1.t - p0.t;
    if dt < 1.0e-9 {
        return p0.v;
    }
    let frac = ((t - p0.t) / dt).clamp(0.0, 1.0);
    p0.v + (p1.v - p0.v) * frac
}

#[cfg(test)]
mod envelope_tests {
    use super::*;

    #[test]
    fn envelope_default() {
        let env = Envelope::default();
        assert!((env.value(0.0) - 1.0).abs() < 1.0e-6);
        assert!((env.value(1.0) - 0.0).abs() < 1.0e-6);
    }

    #[test]
    fn envelope_default_adsr() {
        let env = Envelope::with_default_adsr(10.0, 50.0, 0.5, 40.0);
        assert!(env.value(0.0).abs() < 1.0e-6);
        assert!((env.value(10.0 / 100.0) - 1.0).abs() < 1.0e-6);
        assert!((env.value(60.0 / 100.0) - 0.5).abs() < 1.0e-6);
    }

    #[test]
    fn curved_value_uses_control_points() {
        let env = Envelope::new(vec![
            EnvPoint::with_control(0.0, 0.0, 0.33, 0.8),
            EnvPoint::with_control(1.0, 1.0, 0.33, 0.0),
        ]);
        let v = env.value(0.5);

        assert!(v > 0.5, "control points should curve above linear: {v}");
    }

    #[test]
    fn adsr_envelope_reaches_sustain_immediately_with_zero_stages() {
        let mut eg = AdsrEnvelope::new(48000.0);
        eg.trigger();
        assert!(eg.next() > 0.99);
    }

    #[test]
    fn adsr_envelope_delay_stage_holds_start_level() {
        let params = AdsrParams {
            delay: 0.01,
            start: 0.25,
            ..Default::default()
        };

        let mut eg = AdsrEnvelope::new(48000.0);
        eg.set_extended_params(params);
        eg.trigger();

        // First samples should stay at the start level during the delay stage.
        for _ in 0..400 {
            let v = eg.next();
            assert!(
                (v - 0.25).abs() < 0.01,
                "expected start level 0.25, got {v}"
            );
        }
    }

    #[test]
    fn adsr_envelope_hold_stage_keeps_peak() {
        let params = AdsrParams {
            attack: 0.0,
            hold: 0.01,
            ..Default::default()
        };

        let mut eg = AdsrEnvelope::new(48000.0);
        eg.set_extended_params(params);
        eg.trigger();

        // After zero attack, output should stay at 1.0 during hold.
        assert!(eg.next() > 0.99);
        for _ in 0..400 {
            assert!(eg.next() > 0.99);
        }
    }

    #[test]
    fn adsr_envelope_end_level_after_release() {
        let params = AdsrParams {
            attack: 0.0,
            decay: 0.0,
            release: 0.001,
            end: 0.2,
            ..Default::default()
        };

        let mut eg = AdsrEnvelope::new(48000.0);
        eg.set_extended_params(params);
        eg.trigger();
        eg.next();
        eg.release();

        // Process enough samples to finish the release.
        for _ in 0..200 {
            eg.next();
        }
        assert!((eg.value() - 0.2).abs() < 0.01, "expected end level 0.2");
    }

    #[test]
    fn adsr_envelope_vel2_scales_attack_by_velocity() {
        let params = AdsrParams {
            attack: 0.01,
            vel2_attack: 0.01,
            ..Default::default()
        };

        let low_vel = params.scaled_for_voice(60, 0).attack;
        let high_vel = params.scaled_for_voice(60, 127).attack;

        assert!(
            (low_vel - 0.02).abs() < 0.001,
            "low velocity attack should be doubled"
        );
        assert!(
            (high_vel - 0.01).abs() < 0.001,
            "high velocity attack should be unchanged"
        );
    }

    #[test]
    fn adsr_envelope_key2_scales_sustain_by_key() {
        let params = AdsrParams {
            sustain: 0.5,
            key2_sustain: 0.5,
            ..Default::default()
        };

        let low_key = params.scaled_for_voice(0, 100).sustain;
        let high_key = params.scaled_for_voice(127, 100).sustain;

        assert!(
            (low_key - 1.0).abs() < 0.001,
            "low key sustain should be boosted"
        );
        assert!(
            (high_key - 0.5).abs() < 0.001,
            "high key sustain should be unchanged"
        );
    }
}
