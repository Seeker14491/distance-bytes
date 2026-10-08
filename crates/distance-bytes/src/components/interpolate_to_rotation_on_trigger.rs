use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `InterpolateToRotationOnTrigger`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InterpolateToRotationOnTrigger {
    pub actually_interpolate: Option<bool>,
    pub relative: Option<bool>,
    pub local_movement: Option<bool>,
    /// A quaternion `[x, y, z, w]`.
    pub end_rot: Option<[f32; 4]>,
    pub time: Option<f32>,
}

impl Visit for InterpolateToRotationOnTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("ActuallyInterpolate", &mut self.actually_interpolate)?;
            v.value("Relative", &mut self.relative)?;
            v.value("LocalMovement", &mut self.local_movement)?;
            v.value("EndRot", &mut self.end_rot)?;
            v.value("Time", &mut self.time)?;
        }
        Ok(())
    }
}
