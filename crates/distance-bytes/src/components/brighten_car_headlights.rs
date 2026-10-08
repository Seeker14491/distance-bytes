use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `BrightenCarHeadlights`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct BrightenCarHeadlights {
    pub intensity: Option<f32>,
    pub spot_range: Option<f32>,
    pub spot_angle: Option<f32>,
    pub flicker: Option<bool>,
    pub lerp: Option<bool>,
    pub lerp_time: Option<f32>,
    /// Version 2 only.
    pub on_time: Option<f32>,
    /// Version 2 only.
    pub flicker_time: Option<f32>,
    /// Version 2 only.
    pub flicker_mod_number: Option<i32>,
}

impl Visit for BrightenCarHeadlights {
    const VERSIONS: RangeInclusive<i32> = 1..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("Intensity", &mut self.intensity)?;
            v.value("Spot Range", &mut self.spot_range)?;
            v.value("Spot Angle", &mut self.spot_angle)?;
            v.value("Flicker", &mut self.flicker)?;
            v.value("Lerp", &mut self.lerp)?;
            v.value("LerpTime", &mut self.lerp_time)?;
        }
        if version >= 2 {
            v.value("On Time", &mut self.on_time)?;
            v.value("Flicker Time", &mut self.flicker_time)?;
            v.value("Flicker Mod Number", &mut self.flicker_mod_number)?;
        }
        Ok(())
    }
}
