use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TeleporterEntrance`: When entering this, the Car will be instantly transported to a linked
/// Teleporter Exit.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TeleporterEntrance {
    /// Named `LinkID` in version 1 only.
    pub teleports_to: Option<i32>,
    /// Version 3 and later.
    pub include_in_track_calculation: Option<bool>,
    /// Version 4 only.
    pub bank_id: Option<i32>,
}

impl Visit for TeleporterEntrance {
    const VERSIONS: RangeInclusive<i32> = 1..=4;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 1 {
            v.value("LinkID", &mut self.teleports_to)?;
            return Ok(());
        }
        v.value("Teleports To", &mut self.teleports_to)?;
        if version >= 3 {
            v.value(
                "includeInTrackCalculation",
                &mut self.include_in_track_calculation,
            )?;
        }
        if version >= 4 {
            v.value("bankID_", &mut self.bank_id)?;
        }
        Ok(())
    }
}
