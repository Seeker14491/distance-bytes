use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `RollingBarrelDropperLogic`: Drops Rollers when triggered. Add an EventListener to use this.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RollingBarrelDropperLogic {
    pub drop_rate_per_minute: Option<i32>,
    /// Version 1 and earlier.
    pub drop_time_offset: Option<f32>,
    /// Named `InitialFiringForce` in version 1 and earlier.
    pub initial_firing_speed: Option<f32>,
    /// Version 1 and earlier.
    pub roller_torque: Option<f32>,
    /// Version 1 and earlier.
    pub roller_max_angular_speed: Option<f32>,
    /// `[r, g, b, a]`.
    pub roller_emit_color: Option<[f32; 4]>,
    /// `[r, g, b, a]`. Version 1 only.
    pub pre_drop_pulse_color: Option<[f32; 4]>,
    /// Version 2 only.
    pub barrel_lifetime: Option<f32>,
}

impl Visit for RollingBarrelDropperLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version < 2 {
            if version >= 0 {
                v.value("DropRatePerMinute", &mut self.drop_rate_per_minute)?;
                v.value("DropTimeOffset", &mut self.drop_time_offset)?;
                v.value("InitialFiringForce", &mut self.initial_firing_speed)?;
                v.value("RollerTorque", &mut self.roller_torque)?;
                v.value("RollerMaxAngularSpeed", &mut self.roller_max_angular_speed)?;
                v.value("RollerEmitColor", &mut self.roller_emit_color)?;
            }
            if version >= 1 {
                v.value("PreDropPulseColor", &mut self.pre_drop_pulse_color)?;
            }
        } else {
            v.value("DropRatePerMinute", &mut self.drop_rate_per_minute)?;
            v.value("barrelLifetime_", &mut self.barrel_lifetime)?;
            v.value("initialFiringSpeed_", &mut self.initial_firing_speed)?;
            v.value("RollerEmitColor", &mut self.roller_emit_color)?;
        }
        Ok(())
    }
}
