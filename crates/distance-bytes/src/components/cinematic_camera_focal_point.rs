use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `CinematicCameraFocalPoint`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CinematicCameraFocalPoint {
    pub id: Option<i32>,
}

impl Visit for CinematicCameraFocalPoint {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("ID", &mut self.id)?;
        }
        Ok(())
    }
}
