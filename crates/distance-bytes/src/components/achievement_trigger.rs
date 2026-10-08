use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `AchievementTrigger`.
///
/// Version 2, which the game writes, has no values.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AchievementTrigger {
    /// `EAchievements`. Versions 0 and 1 only, ignored by the game.
    pub achievement: Option<i32>,
    /// Versions 0 and 1 only, ignored by the game.
    pub only_trigger_in_adventure_mode: Option<bool>,
}

impl Visit for AchievementTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version < 2 {
            v.value("Achievement", &mut self.achievement)?;
            v.value(
                "OnlyTriggerInAdventureMode",
                &mut self.only_trigger_in_adventure_mode,
            )?;
        }
        Ok(())
    }
}
