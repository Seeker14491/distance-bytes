use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `InterpolateToPositionOnTrigger`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InterpolateToPositionOnTrigger {
    pub actually_interpolate: Option<bool>,
    pub relative: Option<bool>,
    pub end_pos: Option<[f32; 3]>,
    pub time: Option<f32>,
    /// Version 2 only.
    pub local_movement: Option<bool>,
}

impl Visit for InterpolateToPositionOnTrigger {
    const VERSIONS: RangeInclusive<i32> = 1..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("ActuallyInterpolate", &mut self.actually_interpolate)?;
            v.value("Relative", &mut self.relative)?;
            v.value("EndPos", &mut self.end_pos)?;
            v.value("Time", &mut self.time)?;
        }
        if version >= 2 {
            v.value("LocalMovement", &mut self.local_movement)?;
        }
        Ok(())
    }
}
