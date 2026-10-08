use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `WarningPulseLight`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WarningPulseLight {
    pub speed: Option<f32>,
    pub shader_property: Option<DString>,
    pub min_intensity_scale: Option<f32>,
}

impl Visit for WarningPulseLight {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("Speed", &mut self.speed)?;
            v.value("Shader Property", &mut self.shader_property)?;
            v.value("Min Intensity Scale", &mut self.min_intensity_scale)?;
        }
        Ok(())
    }
}
