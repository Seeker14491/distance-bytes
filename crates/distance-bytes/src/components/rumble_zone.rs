use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `RumbleZone`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct RumbleZone {
    pub camera_shake_intensity: Option<f32>,
    pub archaic_ramp_amount: Option<f32>,
}

impl Visit for RumbleZone {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("Camera Shake Intensity", &mut self.camera_shake_intensity)?;
            v.value("Archaic Ramp Amount", &mut self.archaic_ramp_amount)?;
        }
        Ok(())
    }
}
