use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `AdventureFinishTrigger`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AdventureFinishTrigger {
    pub time_scale_0_0_00: Option<f32>,
    pub time_scale_1_1_90: Option<f32>,
    pub time_scale_2_4_74: Option<f32>,
    pub time_scale_3_6_39: Option<f32>,
    pub time_scale_4_8_31: Option<f32>,
}

impl Visit for AdventureFinishTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("TimeScale 0 (0.00)", &mut self.time_scale_0_0_00)?;
            v.value("TimeScale 1 (1.90)", &mut self.time_scale_1_1_90)?;
            v.value("TimeScale 2 (4.74)", &mut self.time_scale_2_4_74)?;
            v.value("TimeScale 3 (6.39)", &mut self.time_scale_3_6_39)?;
            v.value("TimeScale 4 (8.31)", &mut self.time_scale_4_8_31)?;
        }
        Ok(())
    }
}
