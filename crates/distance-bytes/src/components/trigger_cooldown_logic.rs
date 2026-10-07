use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TriggerCooldownLogic`: Cools down car when entering.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TriggerCooldownLogic {
    /// Version 1 only.
    pub include_in_track_calculation: Option<bool>,
}

impl Visit for TriggerCooldownLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value(
                "includeInTrackCalculation_",
                &mut self.include_in_track_calculation,
            )?;
        }
        Ok(())
    }
}
