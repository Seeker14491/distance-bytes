use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `AudioEffectZone`.
///
/// Versions 1 and later have no values: the game takes the radius from the object's scale.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AudioEffectZone {
    /// Version 0 only.
    pub radius: Option<f32>,
}

impl Visit for AudioEffectZone {
    const VERSIONS: RangeInclusive<i32> = 0..=5;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.value("Radius", &mut self.radius)?;
        }
        Ok(())
    }
}
