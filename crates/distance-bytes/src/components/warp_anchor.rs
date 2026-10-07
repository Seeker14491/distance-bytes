use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `WarpAnchor`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WarpAnchor {
    /// `WarpAnchor.TransitionEffect`.
    pub transition_effect: Option<i32>,
    pub my_id: Option<i32>,
    pub other_id: Option<i32>,
    /// Named `Time` in version 9 and earlier.
    pub time_before_returning: Option<f32>,
    /// Named `Delay` in version 9 and earlier.
    pub delay_time: Option<f32>,
    /// Version 9 and earlier.
    pub repetitions: Option<i32>,
    /// Named `Primary` in version 9 and earlier.
    pub is_primary: Option<bool>,
    /// Version 9 and earlier.
    pub one_way: Option<bool>,
    /// Version 9 and earlier.
    pub slowmo: Option<bool>,
    /// Version 9 and earlier.
    pub slowmo_delay: Option<bool>,
    pub action_index_before: Option<i32>,
    pub action_index_during: Option<i32>,
    pub action_index_after: Option<i32>,
    /// Named `VirusSpiritID` in version 9 and earlier.
    pub archaic_id: Option<i32>,
    /// Named `SlowmoSpeed` in version 9 and earlier.
    pub time_scale: Option<f32>,
    /// Named `SlowmoDelaySpeed` in version 9 and earlier.
    pub delay_time_scale: Option<f32>,
    pub shake_before: Option<bool>,
    pub shake_after: Option<bool>,
    pub shake_strength: Option<f32>,
    /// Named `EmpAfter` in version 9 and earlier.
    pub enable_emp_after_warp: Option<bool>,
    /// Named `VhsDuring` in version 9 and earlier.
    pub vhs_during: Option<bool>,
    pub glitch_during: Option<bool>,
    pub glitch_during_delay: Option<bool>,
    pub shake_during_delay: Option<bool>,
    pub shake_during_delay_strength: Option<f32>,
    /// Named `EmpRadius` in version 9 and earlier.
    pub emp_radius: Option<f32>,
    pub delay_before_turning_on_after_emp: Option<f32>,
    pub add_noise: Option<bool>,
    pub snap_to_exit: Option<bool>,
    /// Version 9 and earlier.
    pub delay_other_id: Option<i32>,
    /// Version 9 and earlier.
    pub delay_warp_time: Option<f32>,
    /// Version 9 and earlier.
    pub delay_warp_min: Option<f32>,
    /// Version 9 and earlier.
    pub delay_warp_max: Option<f32>,
    /// Version 9 and earlier.
    pub delay_for_delay_warp: Option<f32>,
    /// Version 9 and earlier.
    pub glitch_during_delay_warp: Option<bool>,
    /// Version 9 and earlier.
    pub vhs_during_delay_warp: Option<bool>,
    /// Version 9 and earlier.
    pub audio_event_during_delay_warp: Option<DString>,
    /// Version 9 and earlier.
    pub audio_event_after_delay_warp: Option<DString>,
    pub audio_event_before: Option<DString>,
    pub audio_event_during: Option<DString>,
    pub audio_event_after: Option<DString>,
    pub audio_high_pass_during: Option<bool>,
    pub audio_low_pass_after: Option<bool>,
    /// Named `NewAnchorAfterID` in version 9 and earlier.
    pub new_anchor_after_warp: Option<i32>,
    /// Version 2 and later.
    pub archaic_audio: Option<DString>,
    /// Versions 3 to 9.
    pub delay_other_id2: Option<i32>,
    /// Versions 3 to 9.
    pub delay_other_id3: Option<i32>,
    /// Version 4 and later.
    pub bloom_out: Option<bool>,
    /// Version 5 and later.
    pub one_time_use: Option<bool>,
    /// Versions 6 to 9.
    pub filter_colors_during_glitch: Option<bool>,
    /// `TimeWidget.State`. Version 7 and later.
    pub car_timer_state_before: Option<i32>,
    /// `TimeWidget.State`. Version 7 and later.
    pub car_timer_state_during: Option<i32>,
    /// `TimeWidget.State`. Version 7 and later.
    pub car_timer_state_after: Option<i32>,
    /// Version 8 and later.
    pub emp_selection_randomness: Option<i32>,
    /// Version 9 and later.
    pub enable_corruption_effect: Option<bool>,
    /// Version 9 and later.
    pub glitch_intensity: Option<f32>,
    /// Version 9 and later.
    pub glitch_filter_colors: Option<bool>,
    /// Version 9 and later.
    pub glitch_up: Option<bool>,
    /// Version 9 and later.
    pub glitch_down: Option<bool>,
    /// Version 9 and later.
    pub glitch_displace: Option<bool>,
    /// Version 9 and later.
    pub glitch_scale: Option<bool>,
    /// `WarpAnchor.Type`. Version 15 and later.
    pub r#type: Option<i32>,
    /// Version 17 and later.
    pub ignore_in_arcade: Option<bool>,
    /// Version 18 only.
    pub ignore_in_adventure: Option<bool>,
    /// `WarpAnchor.WarpStyle`. Version 10 and later.
    pub type_of_warp: Option<i32>,
    /// Version 10 and later.
    pub enable_delay: Option<bool>,
    /// Version 10 and later.
    pub connect_to_archaic: Option<bool>,
    /// Version 10 and later.
    pub archaic_tractor_beam: Option<bool>,
    /// Version 10 and later.
    pub archaic_tractor_beam_time: Option<f32>,
    /// `[r, g, b, a]`. Version 14 and later.
    pub corruption_effect_color: Option<[f32; 4]>,
    /// Version 11 and later.
    pub shake_before_delay: Option<bool>,
    /// Version 12 and later.
    pub show_countdown_and_gps_after: Option<bool>,
    /// Version 13 and later.
    pub snap_countdown_to_time: Option<f32>,
    /// Version 13 and later.
    pub set_countdown_time_scale: Option<bool>,
    /// Version 13 and later.
    pub countdown_time_scale: Option<f32>,
    /// Version 13 and later.
    pub disable_countdown: Option<bool>,
    /// Version 16 and later.
    pub bank_id: Option<i32>,
}

impl Visit for WarpAnchor {
    const VERSIONS: RangeInclusive<i32> = 1..=18;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if (1..=9).contains(&version) {
            v.value("TransitionEffect", &mut self.transition_effect)?;
            v.value("MyID", &mut self.my_id)?;
            v.value("OtherID", &mut self.other_id)?;
            v.value("Time", &mut self.time_before_returning)?;
            v.value("Delay", &mut self.delay_time)?;
            v.value("Repetitions", &mut self.repetitions)?;
            v.value("Primary", &mut self.is_primary)?;
            v.value("OneWay", &mut self.one_way)?;
            v.value("Slowmo", &mut self.slowmo)?;
            v.value("SlowmoDelay", &mut self.slowmo_delay)?;
            v.value("ActionIndexBefore", &mut self.action_index_before)?;
            v.value("ActionIndexDuring", &mut self.action_index_during)?;
            v.value("ActionIndexAfter", &mut self.action_index_after)?;
            v.value("VirusSpiritID", &mut self.archaic_id)?;
            v.value("SlowmoSpeed", &mut self.time_scale)?;
            v.value("SlowmoDelaySpeed", &mut self.delay_time_scale)?;
            v.value("ShakeBefore", &mut self.shake_before)?;
            v.value("ShakeAfter", &mut self.shake_after)?;
            v.value("ShakeStrength", &mut self.shake_strength)?;
            v.value("EmpAfter", &mut self.enable_emp_after_warp)?;
            v.value("VhsDuring", &mut self.vhs_during)?;
            v.value("GlitchDuring", &mut self.glitch_during)?;
            v.value("GlitchDuringDelay", &mut self.glitch_during_delay)?;
            v.value("ShakeDuringDelay", &mut self.shake_during_delay)?;
            v.value("ShakeDuringDelayStrength", &mut self.shake_during_delay_strength)?;
            v.value("EmpRadius", &mut self.emp_radius)?;
            v.value("DelayBeforeTurningOnAfterEMP", &mut self.delay_before_turning_on_after_emp)?;
            v.value("AddNoise", &mut self.add_noise)?;
            v.value("SnapToExit", &mut self.snap_to_exit)?;
            v.value("DelayOtherID", &mut self.delay_other_id)?;
            v.value("DelayWarpTime", &mut self.delay_warp_time)?;
            v.value("DelayWarpMin", &mut self.delay_warp_min)?;
            v.value("DelayWarpMax", &mut self.delay_warp_max)?;
            v.value("DelayForDelayWarp", &mut self.delay_for_delay_warp)?;
            v.value("GlitchDuringDelayWarp", &mut self.glitch_during_delay_warp)?;
            v.value("VhsDuringDelayWarp", &mut self.vhs_during_delay_warp)?;
            v.value("AudioEventDuringDelayWarp", &mut self.audio_event_during_delay_warp)?;
            v.value("AudioEventAfterDelayWarp", &mut self.audio_event_after_delay_warp)?;
            v.value("AudioEventBefore", &mut self.audio_event_before)?;
            v.value("AudioEventDuring", &mut self.audio_event_during)?;
            v.value("AudioEventAfter", &mut self.audio_event_after)?;
            v.value("AudioHighPassDuring", &mut self.audio_high_pass_during)?;
            v.value("AudioLowPassAfter", &mut self.audio_low_pass_after)?;
            v.value("NewAnchorAfterID", &mut self.new_anchor_after_warp)?;
        }
        if (2..10).contains(&version) {
            v.value("ArchaicAudio", &mut self.archaic_audio)?;
        }
        if (3..10).contains(&version) {
            v.value("DelayOtherID2", &mut self.delay_other_id2)?;
            v.value("DelayOtherID3", &mut self.delay_other_id3)?;
        }
        if (4..10).contains(&version) {
            v.value("BloomOut", &mut self.bloom_out)?;
        }
        if (5..10).contains(&version) {
            v.value("OneTimeUse", &mut self.one_time_use)?;
        }
        if (6..10).contains(&version) {
            v.value("FilterColorsDuringGlitch", &mut self.filter_colors_during_glitch)?;
        }
        if (7..10).contains(&version) {
            v.value("CarTimerStateBefore", &mut self.car_timer_state_before)?;
            v.value("CarTimerStateDuring", &mut self.car_timer_state_during)?;
            v.value("CarTimerStateAfter", &mut self.car_timer_state_after)?;
        }
        if (8..10).contains(&version) {
            v.value("EMPSelectionRandomness", &mut self.emp_selection_randomness)?;
        }
        if (9..10).contains(&version) {
            v.value("EnableCorruptionEffect", &mut self.enable_corruption_effect)?;
            v.value("GlitchIntensity", &mut self.glitch_intensity)?;
            v.value("GlitchFilterColors", &mut self.glitch_filter_colors)?;
            v.value("GlitchUp", &mut self.glitch_up)?;
            v.value("GlitchDown", &mut self.glitch_down)?;
            v.value("GlitchDisplace", &mut self.glitch_displace)?;
            v.value("GlitchScale", &mut self.glitch_scale)?;
        }
        if version >= 10 {
            if version >= 15 {
                v.value("Type", &mut self.r#type)?;
            }
            v.value("MyID", &mut self.my_id)?;
            v.value("OtherID", &mut self.other_id)?;
            v.value("IsPrimary", &mut self.is_primary)?;
            v.value("SnapToExit", &mut self.snap_to_exit)?;
            v.value("OneTimeUse", &mut self.one_time_use)?;
            if version >= 17 {
                v.value("ignoreInArcade_", &mut self.ignore_in_arcade)?;
            }
            if version >= 18 {
                v.value("ignoreInAdventure_", &mut self.ignore_in_adventure)?;
            }
            v.value("Type of Warp", &mut self.type_of_warp)?;
            v.value("TimeBeforeReturning", &mut self.time_before_returning)?;
            v.value("TimeScale", &mut self.time_scale)?;
            v.value("NewAnchorAfterWarp", &mut self.new_anchor_after_warp)?;
            v.value("Enable Delay", &mut self.enable_delay)?;
            v.value("DelayTime", &mut self.delay_time)?;
            v.value("DelayTimeScale", &mut self.delay_time_scale)?;
            v.value("GlitchDuringDelay", &mut self.glitch_during_delay)?;
            v.value("ShakeDuringDelay", &mut self.shake_during_delay)?;
            v.value("ShakeDuringDelayStrength", &mut self.shake_during_delay_strength)?;
            v.value("ConnectToArchaic", &mut self.connect_to_archaic)?;
            v.value("ArchaicID", &mut self.archaic_id)?;
            v.value("ActionIndexBefore", &mut self.action_index_before)?;
            v.value("ActionIndexDuring", &mut self.action_index_during)?;
            v.value("ActionIndexAfter", &mut self.action_index_after)?;
            v.value("ArchaicTractorBeam", &mut self.archaic_tractor_beam)?;
            v.value("ArchaicTractorBeamTime", &mut self.archaic_tractor_beam_time)?;
            v.value("TransitionEffect", &mut self.transition_effect)?;
            v.value("EnableCorruptionEffect", &mut self.enable_corruption_effect)?;
            if version >= 14 {
                v.value("CorruptionEffectColor", &mut self.corruption_effect_color)?;
            }
            v.value("AddNoise", &mut self.add_noise)?;
            v.value("BloomOut", &mut self.bloom_out)?;
            v.value("VHS During", &mut self.vhs_during)?;
            if version >= 11 {
                v.value("ShakeBeforeDelay", &mut self.shake_before_delay)?;
            }
            v.value("ShakeBefore", &mut self.shake_before)?;
            v.value("ShakeAfter", &mut self.shake_after)?;
            v.value("ShakeStrength", &mut self.shake_strength)?;
            v.value("GlitchDuring", &mut self.glitch_during)?;
            v.value("GlitchIntensity", &mut self.glitch_intensity)?;
            v.value("GlitchFilterColors", &mut self.glitch_filter_colors)?;
            v.value("GlitchUp", &mut self.glitch_up)?;
            v.value("GlitchDown", &mut self.glitch_down)?;
            v.value("GlitchDisplace", &mut self.glitch_displace)?;
            v.value("GlitchScale", &mut self.glitch_scale)?;
            v.value("Enable EMP After Warp", &mut self.enable_emp_after_warp)?;
            v.value("EMP Radius", &mut self.emp_radius)?;
            v.value("DelayBeforeTurningOnAfterEMP", &mut self.delay_before_turning_on_after_emp)?;
            v.value("EMPSelectionRandomness", &mut self.emp_selection_randomness)?;
            v.value("AudioHighPassDuring", &mut self.audio_high_pass_during)?;
            v.value("AudioLowPassAfter", &mut self.audio_low_pass_after)?;
            v.value("AudioEventBefore", &mut self.audio_event_before)?;
            v.value("AudioEventDuring", &mut self.audio_event_during)?;
            v.value("AudioEventAfter", &mut self.audio_event_after)?;
            v.value("ArchaicAudio", &mut self.archaic_audio)?;
            v.value("CarTimerStateBefore", &mut self.car_timer_state_before)?;
            v.value("CarTimerStateDuring", &mut self.car_timer_state_during)?;
            v.value("CarTimerStateAfter", &mut self.car_timer_state_after)?;
        }
        if version >= 12 {
            v.value("ShowCountdownAndGPSAfter", &mut self.show_countdown_and_gps_after)?;
        }
        if version >= 13 {
            v.value("SnapCountdownToTime", &mut self.snap_countdown_to_time)?;
            v.value("SetCountdownTimeScale", &mut self.set_countdown_time_scale)?;
            v.value("CountdownTimeScale", &mut self.countdown_time_scale)?;
            v.value("DisableCountdown", &mut self.disable_countdown)?;
        }
        if version >= 16 {
            v.value("bankID_", &mut self.bank_id)?;
        }
        Ok(())
    }
}
