use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `EngageBrokenPieces`: Add movement, rotation, and/or explosion forces.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EngageBrokenPieces {
    pub r#move: Option<bool>,
    pub move_speed: Option<f32>,
    pub rotate: Option<bool>,
    pub rotate_speed: Option<f32>,
    pub explode: Option<bool>,
    pub explode_speed: Option<f32>,
}

impl Visit for EngageBrokenPieces {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("move_", &mut self.r#move)?;
            v.value("moveSpeed_", &mut self.move_speed)?;
            v.value("rotate_", &mut self.rotate)?;
            v.value("rotateSpeed_", &mut self.rotate_speed)?;
            v.value("explode_", &mut self.explode)?;
            v.value("explodeSpeed_", &mut self.explode_speed)?;
        }
        Ok(())
    }
}
