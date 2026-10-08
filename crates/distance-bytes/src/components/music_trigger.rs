use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `MusicTrigger`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MusicTrigger {
    /// Version 2 and earlier.
    pub music_cue_id: Option<i32>,
    /// Version 3 only.
    pub music_track: Option<DString>,
    pub one_time_trigger: Option<bool>,
    pub reset_before_trigger: Option<bool>,
    pub disable_music_trigger: Option<bool>,
    /// Version 2 and later.
    pub stop_previous_music: Option<bool>,
}

impl Visit for MusicTrigger {
    const VERSIONS: RangeInclusive<i32> = 1..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            if version < 3 {
                v.value("MusicCueID", &mut self.music_cue_id)?;
            } else {
                v.value("Music Track", &mut self.music_track)?;
            }
            v.value("OneTimeTrigger", &mut self.one_time_trigger)?;
            v.value("ResetBeforeTrigger", &mut self.reset_before_trigger)?;
            v.value("DisableMusicTrigger", &mut self.disable_music_trigger)?;
        }
        if version >= 2 {
            v.value("stopPreviousMusic_", &mut self.stop_previous_music)?;
        }
        Ok(())
    }
}
