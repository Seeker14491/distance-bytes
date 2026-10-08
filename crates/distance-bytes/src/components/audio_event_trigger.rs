use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `AudioEventTrigger`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AudioEventTrigger {
    pub event: Option<DString>,
    pub one_time_trigger: Option<bool>,
    pub is_adventure_music_trigger: Option<bool>,
    /// Version 2 and earlier.
    pub music_cue_id: Option<i32>,
    /// Version 3 only.
    pub music_track: Option<DString>,
    /// Version 2 and later.
    pub trigger_on_level_finish: Option<bool>,
}

impl Visit for AudioEventTrigger {
    const VERSIONS: RangeInclusive<i32> = 1..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("Event", &mut self.event)?;
            v.value("OneTimeTrigger", &mut self.one_time_trigger)?;
            v.value(
                "IsAdventureMusicTrigger_",
                &mut self.is_adventure_music_trigger,
            )?;
            if version < 3 {
                v.value("MusicCueID", &mut self.music_cue_id)?;
            } else {
                v.value("Music Track", &mut self.music_track)?;
            }
        }
        if version >= 2 {
            v.value("triggerOnLevelFinish_", &mut self.trigger_on_level_finish)?;
        }
        Ok(())
    }
}
