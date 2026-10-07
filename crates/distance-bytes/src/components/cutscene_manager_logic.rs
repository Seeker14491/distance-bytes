use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `CutsceneManagerLogic`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CutsceneManagerLogic {
    pub time_to_switch_to_next_level: Option<f32>,
    pub delay_before_fade_in: Option<f32>,
    pub fade_in_time: Option<f32>,
    pub fade_out_time: Option<f32>,
    pub aspect_ratio_21_9: Option<bool>,
    /// Version 2 and later.
    pub event_to_trigger_on_start: Option<DString>,
    /// Version 2 and earlier.
    pub music_cue_id: Option<i32>,
    /// Version 3 only.
    pub music_track: Option<DString>,
}

impl Visit for CutsceneManagerLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("TimeToSwitchToNextLevel", &mut self.time_to_switch_to_next_level)?;
            v.value("Delay Before Fade In", &mut self.delay_before_fade_in)?;
            v.value("Fade In Time", &mut self.fade_in_time)?;
            v.value("Fade Out Time", &mut self.fade_out_time)?;
            v.value("21:9 Aspect Ratio", &mut self.aspect_ratio_21_9)?;
            if version >= 2 {
                v.value("eventToTriggerOnStart_", &mut self.event_to_trigger_on_start)?;
            }
        }
        if version < 3 {
            v.value("MusicCueID", &mut self.music_cue_id)?;
        } else {
            v.value("Music Track", &mut self.music_track)?;
        }
        Ok(())
    }
}
