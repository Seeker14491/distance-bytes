use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `SetActiveOnIntroCutsceneStarted`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetActiveOnIntroCutsceneStarted {
    pub time_offset: Option<f32>,
}

impl Visit for SetActiveOnIntroCutsceneStarted {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else if version > 0 {
            v.value("TimeOffset", &mut self.time_offset)?;
        }
        Ok(())
    }
}
