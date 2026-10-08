use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `ExcludeFromEMP`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct ExcludeFromEMP {
    pub exclude_renderer: Option<bool>,
    pub exclude_lights: Option<bool>,
    pub exclude_particles: Option<bool>,
}

impl Visit for ExcludeFromEMP {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        v.value("Exclude Renderer", &mut self.exclude_renderer)?;
        v.value("Exclude Lights", &mut self.exclude_lights)?;
        v.value("Exclude Particles", &mut self.exclude_particles)
    }
}
