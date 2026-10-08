use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `QuarantineTrigger`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct QuarantineTrigger {
    /// Version 1 only.
    pub max_bloom_intensity: Option<f32>,
    /// Version 1 only.
    pub min_bloom_intensity: Option<f32>,
    /// Version 1 only.
    pub grain_intensity: Option<f32>,
    /// Version 1 only.
    pub fade_up_time: Option<f32>,
    /// Version 1 only.
    pub fade_down_time: Option<f32>,
}

impl Visit for QuarantineTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("Max Bloom Intensity", &mut self.max_bloom_intensity)?;
            v.value("Min Bloom Intensity", &mut self.min_bloom_intensity)?;
            v.value("Grain Intensity", &mut self.grain_intensity)?;
            v.value("Fade Up Time", &mut self.fade_up_time)?;
            v.value("Fade Down Time", &mut self.fade_down_time)?;
        }
        Ok(())
    }
}
