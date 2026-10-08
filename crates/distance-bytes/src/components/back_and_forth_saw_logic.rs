use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `BackAndForthSawLogic`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BackAndForthSawLogic {
    pub move_extents: Option<f32>,
    pub move_speed_multiplier: Option<f32>,
    pub move_direction: Option<i32>,
}

impl Visit for BackAndForthSawLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        v.value("MoveExtents", &mut self.move_extents)?;
        v.value("MoveSpeedMultiplier", &mut self.move_speed_multiplier)?;
        v.value("MoveDirection", &mut self.move_direction)?;
        Ok(())
    }
}
