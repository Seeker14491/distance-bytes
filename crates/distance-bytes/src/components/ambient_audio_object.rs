use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `AmbientAudioObject`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AmbientAudioObject {
    pub event: Option<DString>,
    /// Version 3 and later.
    pub say_mode_time: Option<bool>,
    /// Version 3 and later.
    pub event_after_time: Option<DString>,
    /// Version 3 and later.
    pub r#loop: Option<bool>,
    /// Version 4 and later.
    pub infected: Option<bool>,
    /// Version 5 and later.
    pub medium_range: Option<bool>,
    pub has_subtitles: Option<bool>,
    /// Version 2 and later.
    pub use_doppler: Option<bool>,
    /// Version 6 and later.
    pub is_2_d: Option<bool>,
    /// Version 7 only.
    pub max_attenuation_override: Option<f32>,
    pub listen_for_warp_event: Option<bool>,
    pub warp_anchor_id: Option<i32>,
    /// `AmbientAudioObject.PlayState`.
    pub state_after_warp: Option<i32>,
    /// `AmbientAudioObject.WarpEvent`.
    pub warp_anchor_event: Option<i32>,
}

impl Visit for AmbientAudioObject {
    const VERSIONS: RangeInclusive<i32> = 1..=7;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Event", &mut self.event)?;
        if version >= 3 {
            v.value("Say Mode Time", &mut self.say_mode_time)?;
            v.value("Event After Time", &mut self.event_after_time)?;
            v.value("Loop", &mut self.r#loop)?;
            if version >= 4 {
                v.value("Infected", &mut self.infected)?;
            }
            if version >= 5 {
                v.value("mediumRange_", &mut self.medium_range)?;
            }
        }
        v.value("Has Subtitles", &mut self.has_subtitles)?;
        if version >= 2 {
            v.value("UseDoppler", &mut self.use_doppler)?;
        }
        if version >= 6 {
            v.value("Is 2D", &mut self.is_2_d)?;
        }
        if version >= 7 {
            v.value(
                "Max Attenuation Override",
                &mut self.max_attenuation_override,
            )?;
        }
        v.value("ListenForWarpEvent", &mut self.listen_for_warp_event)?;
        v.value("WarpAnchor ID", &mut self.warp_anchor_id)?;
        v.value("State After Warp", &mut self.state_after_warp)?;
        v.value("WarpAnchorEvent", &mut self.warp_anchor_event)?;
        Ok(())
    }
}
