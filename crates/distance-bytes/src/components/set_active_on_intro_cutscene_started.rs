use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `SetActiveOnIntroCutsceneStarted`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SetActiveOnIntroCutsceneStarted {
    pub time_offset: Option<f32>,
}

impl Visit for SetActiveOnIntroCutsceneStarted {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else if version > 0 {
            v.value("TimeOffset", &mut self.time_offset)?;
        }
        Ok(())
    }
}
