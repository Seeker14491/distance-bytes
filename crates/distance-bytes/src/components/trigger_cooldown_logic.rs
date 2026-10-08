use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TriggerCooldownLogic`: Cools down car when entering.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
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
