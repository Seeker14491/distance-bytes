use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `LevelImageCamera`: Creates level preview image from this position.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelImageCamera {
    /// Version 1 and later.
    pub field_of_view: Option<f32>,
}

impl Visit for LevelImageCamera {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("fieldOfView_", &mut self.field_of_view)?;
        }
        Ok(())
    }
}
