use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `WingCorruptionZone`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WingCorruptionZone {
    pub show_ability_alert: Option<bool>,
}

impl Visit for WingCorruptionZone {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("ShowAbilityAlert", &mut self.show_ability_alert)?;
        }
        Ok(())
    }
}
