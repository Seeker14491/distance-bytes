use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `LookAtCamera`: Rotates the object towards the player (Camera or Car).
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LookAtCamera {
    pub rotation_speed: Option<f32>,
    /// Version 1 and later.
    pub flip_z_axis: Option<bool>,
    /// `LookAtCamera.TargetType`. Version 2 and later.
    pub target_type: Option<i32>,
    /// Version 3 only.
    pub billboard: Option<bool>,
}

impl Visit for LookAtCamera {
    const VERSIONS: RangeInclusive<i32> = 0..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("rotationSpeed_", &mut self.rotation_speed)?;
        }
        if version >= 1 {
            v.value("Flip Z Axis", &mut self.flip_z_axis)?;
        }
        if version >= 2 {
            v.value("targetType_", &mut self.target_type)?;
        }
        if version >= 3 {
            v.value("billboard_", &mut self.billboard)?;
        }
        Ok(())
    }
}
