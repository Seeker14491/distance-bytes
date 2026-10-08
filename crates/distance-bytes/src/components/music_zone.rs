use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `MusicZone`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct MusicZone {
    /// Version 0 only.
    pub music_cue_id: Option<i32>,
    /// Version 1 only.
    pub music_track: Option<DString>,
    pub advanced_dev_options: Option<bool>,
    pub pause_previous_track: Option<bool>,
}

impl Visit for MusicZone {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version < 1 {
            v.value("MusicCueID", &mut self.music_cue_id)?;
        } else {
            v.value("Music Track", &mut self.music_track)?;
        }
        v.value("Advanced Dev Options", &mut self.advanced_dev_options)?;
        v.value("Pause Previous Track", &mut self.pause_previous_track)?;
        Ok(())
    }
}
