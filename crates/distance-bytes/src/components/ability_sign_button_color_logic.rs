use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `AbilitySignButtonColorLogic`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AbilitySignButtonColorLogic {
    /// Version 1 only.
    pub use_controller_button_color: Option<bool>,
}

impl Visit for AbilitySignButtonColorLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value(
                "Use Controller Button Color",
                &mut self.use_controller_button_color,
            )?;
        }
        Ok(())
    }
}
