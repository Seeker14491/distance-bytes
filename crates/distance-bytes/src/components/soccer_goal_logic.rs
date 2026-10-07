use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `SoccerGoalLogic`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SoccerGoalLogic {
    /// `[r, g, b, a]`.
    pub color: Option<[f32; 4]>,
}

impl Visit for SoccerGoalLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else {
            v.value("Color", &mut self.color)?;
        }
        Ok(())
    }
}
