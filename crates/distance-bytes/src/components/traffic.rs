use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `Traffic`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Traffic {
    pub scale: Option<[f32; 3]>,
    pub speed: Option<[f32; 3]>,
}

impl Visit for Traffic {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        v.value("scale_", &mut self.scale)?;
        v.value("speed_", &mut self.speed)?;
        Ok(())
    }
}
