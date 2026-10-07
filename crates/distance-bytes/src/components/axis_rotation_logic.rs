use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `AxisRotationLogic`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AxisRotationLogic {
    pub angular_speed: Option<f32>,
    /// `AxisRotationLogic.Axis`.
    pub axis_of_rotation: Option<i32>,
    pub limit_rotation: Option<bool>,
    pub rotation_bounds: Option<f32>,
    pub starting_angle_offset: Option<f32>,
}

impl Visit for AxisRotationLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        v.value("AngularSpeed", &mut self.angular_speed)?;
        v.value("AxisOfRotation", &mut self.axis_of_rotation)?;
        v.value("LimitRotation", &mut self.limit_rotation)?;
        v.value("RotationBounds", &mut self.rotation_bounds)?;
        v.value("StartingAngleOffset", &mut self.starting_angle_offset)?;
        Ok(())
    }
}
