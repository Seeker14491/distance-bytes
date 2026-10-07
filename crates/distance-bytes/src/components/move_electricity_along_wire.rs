use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `MoveElectricityAlongWire`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MoveElectricityAlongWire {
    /// Named `StartOnWarpEvent` in version 1 only.
    pub animate_on_warp_event: Option<bool>,
    /// Version 2 only.
    pub time: Option<f32>,
}

impl Visit for MoveElectricityAlongWire {
    const VERSIONS: RangeInclusive<i32> = 1..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 1 {
            v.value("StartOnWarpEvent", &mut self.animate_on_warp_event)?;
        }
        if version == 2 {
            v.value("Time", &mut self.time)?;
            v.value("AnimateOnWarpEvent", &mut self.animate_on_warp_event)?;
        }
        Ok(())
    }
}
