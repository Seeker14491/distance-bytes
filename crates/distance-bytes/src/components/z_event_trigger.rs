use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `ZEventTrigger`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZEventTrigger {
    /// Version 1 and earlier.
    pub event_id: Option<i32>,
    /// Version 2 only.
    pub event_name: Option<DString>,
    /// Version 1 and later.
    pub one_shot: Option<bool>,
}

impl Visit for ZEventTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version <= 1 {
            v.value("Event ID", &mut self.event_id)?;
        } else if version == 2 {
            v.value("eventName_", &mut self.event_name)?;
        }
        if version >= 1 {
            v.value("oneShot_", &mut self.one_shot)?;
        }
        Ok(())
    }
}
