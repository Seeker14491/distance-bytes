use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `GPSTrigger`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GPSTrigger {
    pub glitch_intensity: Option<f32>,
    pub glitch_duration: Option<f32>,
    pub delay_duration: Option<f32>,
}

impl Visit for GPSTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("glitchIntensity_", &mut self.glitch_intensity)?;
            v.value("glitchDuration_", &mut self.glitch_duration)?;
            v.value("delayDuration_", &mut self.delay_duration)?;
        }
        Ok(())
    }
}
