use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `EmpireProximityDoorLogic`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EmpireProximityDoorLogic {
    pub global_speed: Option<f32>,
    pub move_outwardin_tri_angles: Option<bool>,
    pub init_delay_time: Option<f32>,
    pub first_open_amount: Option<f32>,
    pub first_open_time: Option<f32>,
    pub first_delay_time: Option<f32>,
    pub rotation_angle: Option<[f32; 3]>,
    pub rotation_time: Option<f32>,
    pub second_open_amount: Option<f32>,
    pub second_open_time: Option<f32>,
    pub second_delay_time: Option<f32>,
}

impl Visit for EmpireProximityDoorLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        v.value("globalSpeed_", &mut self.global_speed)?;
        v.value("moveOutwardinTriAngles_", &mut self.move_outwardin_tri_angles)?;
        v.value("initDelayTime_", &mut self.init_delay_time)?;
        v.value("firstOpenAmount_", &mut self.first_open_amount)?;
        v.value("firstOpenTime_", &mut self.first_open_time)?;
        v.value("firstDelayTime_", &mut self.first_delay_time)?;
        v.value("rotationAngle_", &mut self.rotation_angle)?;
        v.value("rotationTime_", &mut self.rotation_time)?;
        v.value("secondOpenAmount_", &mut self.second_open_amount)?;
        v.value("secondOpenTime_", &mut self.second_open_time)?;
        v.value("secondDelayTime_", &mut self.second_delay_time)?;
        Ok(())
    }
}
