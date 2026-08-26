# Maolan Sampler: SFZ support status

This document tracks which SFZ features are implemented in the Sampler engine/GUI and which are missing or only partially done.

## Implemented (from SFZ import)

- Preprocessing: comments, `#include`, `#define`, `#if/#else/#endif`.
- Header precedence: `control` → `global` → `master` → `group` → `region`.
- Global / master offsets: `global_volume`, `global_pan`, `global_tune`, `master_volume`, `master_pan`, `master_tune`.
- Sample definition: `sample`, `default_path`, `*silence`.
- Key mapping: `key`, `lokey`, `hikey`, `pitch_keycenter`, `keylabel`.
- Velocity mapping: `lovel`, `hivel`.
- Key/velocity fades: `xfin_lokey`/`xfin_hikey`, `xfout_lokey`/`xfout_hikey`, `xfin_lovel`/`xfin_hivel`, `xfout_lovel`/`xfout_hivel`.
- Tuning: `tune`, `transpose`, `keytracking`, `bend_up`, `bend_down`.
- Amplitude & panning: `volume`, `pan`, `width`, `position`, `amp_keytrack`, `amp_veltrack`.
- Group-level envelopes, LFOs, and filters: `ampeg_*`, `fileg_*`, `amplfo_*`, `fillfo_*`, `pitchlfo_*`, `lfo01_*`, `cutoff`, `resonance`, `fil_type`, `fil_keytrack`, `fil_subtype`, `fil_veltrack` are parsed on `<group>` and applied to voices triggered in that group (overriding global GUI settings for that voice).
- Extended EG opcodes: `delay`, `hold`, `start`, `end`, `vel2attack`, `vel2decay`, `vel2sustain`, `vel2release`, `vel2delay`, `vel2hold`, `vel2start`, `vel2end`, `key2attack`, `key2decay`, `key2sustain`, `key2release`, `key2delay`, `key2hold`, `key2start`, `key2end`, and `ampeg_attackccN`/`fileg_attackccN`/etc curve opcodes.
- LFO `delay` and `fade` opcodes.
- Filter velocity tracking and per-filter CC modulation: `fil_veltrack`, `cutoff_oncc`, `resonance_oncc`.
- Playback: `offset`, `direction`, `loop_mode`, `loop_start`, `loop_end`, `loop_crossfade`, `loop_count`, `loop_direction`.
- Triggering / grouping: `polyphony`, `group`, `off_by`, `exclusive_group`, `count`, `off_mode`, `trigger=first/legato`.
- Keyswitches: `sw_last`, `sw_down`, `sw_up`, `sw_up` (default when not held), `sw_previous`, `sw_lolast`, `sw_hilast`, `sw_default`, `sw_label`.
- Round-robin / random: `seq_length`, `lorand`/`hirand`.
- CC modulation (per zone): `volume_oncc`, `pan_oncc`, `tune_oncc`, `cutoff_oncc`, `resonance_oncc`, `offset_oncc`, `delay_oncc`, `curvecc`.
- Full mod-matrix sources (per zone): editable sources now include `Velocity`, `KeyTrack`, `PitchBend`, `ModWheel`, `ChannelPressure`, `Expression`, `LFO1`..`LFO4`, `EG1`/`EG2`, `Random`, `PlaybackPosition`, `IsGated`, `IsReleased`, and `MidiCc`.
- Output routing: `output=` on `<group>` and `<region>`.
- Zone-level GUI editing: `output`, `velcurve`, `off_mode`, `count`, `amp_veltrack`.
- Group-level GUI editing: `output`, keyswitches (`sw_last`, `sw_down`, `sw_up`, `sw_previous`, `sw_lolast`, `sw_hilast`, `sw_default`, `sw_label`), group EGs (`ampeg_*`, `fileg_*`), group LFOs (`lfo01`..`lfo04`), and group filter (`cutoff`, `resonance`, `fil_type`, `fil_subtype`, `fil_keytrack`, `fil_veltrack`).

## Not implemented or only partially implemented

The GUI now covers all items that were previously listed as gaps. Remaining limitations are minor:

- The group EG/LFO/filter editor exposes the most commonly edited parameters (EG attack/decay/sustain/release, LFO rate/amount/wave, filter cutoff/resonance/type/subtype/key tracking/velocity tracking). Advanced EG stages (`delay`/`hold`/`start`/`end`, `vel2*`/`key2*`, curve shapes) and LFO `delay`/`fade`/`phase`/`trigger`/`sync` can still be set via the group **Extra SFZ** text field or imported from SFZ.
- Group-level mod-matrix routes are parsed from SFZ but do not yet have a dedicated GUI editor.
