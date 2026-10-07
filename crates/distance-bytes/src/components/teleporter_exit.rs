use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TeleporterExit`: Car can exit from here when entering a Teleporter Entrance.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TeleporterExit {
    pub link_id: Option<i32>,
}

impl Visit for TeleporterExit {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else {
            v.value("LinkID", &mut self.link_id)?;
        }
        Ok(())
    }
}
