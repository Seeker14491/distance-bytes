use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `InfiniteCooldownTrigger`: Enables infinite cooldown when car enters.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InfiniteCooldownTrigger {
    /// Version 1 only.
    pub disable_jump: Option<bool>,
    /// Version 1 only.
    pub disable_boost: Option<bool>,
    /// Version 1 only.
    pub disable_wings: Option<bool>,
    pub disable_hover_screen_emitter: Option<bool>,
}

impl Visit for InfiniteCooldownTrigger {
    const VERSIONS: RangeInclusive<i32> = 1..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 1 {
            v.value("DisableJump", &mut self.disable_jump)?;
            v.value("DisableBoost", &mut self.disable_boost)?;
            v.value("DisableWings", &mut self.disable_wings)?;
        }
        if version >= 1 {
            v.value(
                "DisableHoverScreenEmitter",
                &mut self.disable_hover_screen_emitter,
            )?;
        }
        Ok(())
    }
}
