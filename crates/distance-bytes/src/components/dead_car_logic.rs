use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `DeadCarLogic`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeadCarLogic {
    pub enable_collision: Option<bool>,
    pub weightless: Option<bool>,
    /// Named `ShowWheelBL` in version 0 only.
    pub show_wheels: Option<bool>,
    /// Version 0 only.
    pub show_wheel_br: Option<bool>,
    /// Version 0 only.
    pub show_wheel_fl: Option<bool>,
    /// Version 0 only.
    pub show_wheel_fr: Option<bool>,
    /// Version 1 and later.
    pub copy_player_car: Option<bool>,
    /// Version 1 and later.
    pub copy_player_car_screen: Option<bool>,
    /// Version 1 and later.
    pub disable_emissive: Option<bool>,
    /// Version 2 only.
    pub wheel_rotation_speed: Option<f32>,
}

impl Visit for DeadCarLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("Enable Collision", &mut self.enable_collision)?;
            v.value("Weightless", &mut self.weightless)?;
        }
        if version == 0 {
            v.value("ShowWheelBL", &mut self.show_wheels)?;
            v.value("ShowWheelBR", &mut self.show_wheel_br)?;
            v.value("ShowWheelFL", &mut self.show_wheel_fl)?;
            v.value("ShowWheelFR", &mut self.show_wheel_fr)?;
        }
        if version >= 1 {
            v.value("showWheels_", &mut self.show_wheels)?;
            v.value("copyPlayerCar_", &mut self.copy_player_car)?;
            v.value("copyPlayerCarScreen_", &mut self.copy_player_car_screen)?;
            v.value("disableEmissive_", &mut self.disable_emissive)?;
        }
        if version >= 2 {
            v.value("wheelRotationSpeed_", &mut self.wheel_rotation_speed)?;
        }
        Ok(())
    }
}
