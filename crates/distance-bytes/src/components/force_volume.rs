use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `ForceVolume`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ForceVolume {
    pub direction: Option<[f32; 3]>,
    pub global_force: Option<bool>,
    /// `ForceVolume.ForceType`.
    pub force_type: Option<i32>,
    pub magnitude: Option<f32>,
    pub disable_global_gravity: Option<bool>,
    pub wind_speed: Option<f32>,
    pub drag_multiplier: Option<f32>,
}

impl Visit for ForceVolume {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        v.value("direction_", &mut self.direction)?;
        v.value("globalForce_", &mut self.global_force)?;
        v.value("forceType_", &mut self.force_type)?;
        v.value("magnitude_", &mut self.magnitude)?;
        v.value("disableGlobalGravity_", &mut self.disable_global_gravity)?;
        v.value("windSpeed_", &mut self.wind_speed)?;
        v.value("dragMultiplier_", &mut self.drag_multiplier)?;
        Ok(())
    }
}
