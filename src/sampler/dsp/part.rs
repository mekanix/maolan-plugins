use crate::common::tuning::Tuning;
use crate::sampler::dsp::bus::{AuxSend, Bus};
use crate::sampler::dsp::group::Group;
use crate::sampler::dsp::zone::Zone;

#[derive(Debug, Clone)]
pub struct Part {
    pub name: String,
    pub groups: Vec<Group>,

    pub midi_channel: u8,

    pub transpose: i8,

    pub tuning: f32,

    pub gain_db: f32,

    pub pan: f32,

    pub poly_limit: usize,

    pub mpe_enabled: bool,

    pub bus: Bus,

    pub aux_sends: [AuxSend; 4],

    pub microtuning: Option<Tuning>,

    pub last_keyswitch_note: Option<u8>,

    pub previous_note: Option<u8>,
}

impl Default for Part {
    fn default() -> Self {
        Self {
            name: String::new(),
            groups: Vec::new(),
            midi_channel: 255,
            transpose: 0,
            tuning: 0.0,
            gain_db: 0.0,
            pan: 0.0,
            poly_limit: 0,
            mpe_enabled: false,
            bus: Bus::default(),
            aux_sends: [AuxSend::default(); 4],
            microtuning: None,
            last_keyswitch_note: None,
            previous_note: None,
        }
    }
}

impl Part {
    pub fn find_zone(
        &self,
        note: u8,
        velocity: u8,
        channel: u8,
        cc_values: &[u8; 128],
        pitch_bend_raw: i16,
        held_notes: &[bool; 128],
    ) -> Option<(usize, &Group, &Zone)> {
        for (gi, group) in self.groups.iter().enumerate() {
            if !crate::sampler::dsp::group::group_is_active(group, self, cc_values, held_notes) {
                continue;
            }
            if let Some(zone) = group.find_zone(note, velocity, channel, cc_values, pitch_bend_raw)
            {
                return Some((gi, group, zone));
            }
        }
        None
    }
}
