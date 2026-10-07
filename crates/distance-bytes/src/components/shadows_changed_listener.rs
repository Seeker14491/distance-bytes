use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `ShadowsChangedListener`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShadowsChangedListener {
    pub override_shadows_off_intensity: Option<bool>,
    pub shadows_off_intensity: Option<f32>,
}

impl Visit for ShadowsChangedListener {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value(
                "overrideShadowsOffIntensity_",
                &mut self.override_shadows_off_intensity,
            )?;
            v.value("shadowsOffIntensity_", &mut self.shadows_off_intensity)?;
        }
        Ok(())
    }
}
