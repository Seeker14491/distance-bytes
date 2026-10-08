use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `UltraPlanet`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct UltraPlanet {
    /// `[r, g, b, a]`.
    pub cloud_color: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub cloud_ambient: Option<[f32; 4]>,
    pub cloud_cover: Option<f32>,
    /// Version 1 only.
    pub cloud_bump: Option<f32>,
    pub city_daytime_light: Option<f32>,
    pub animation_speed: Option<f32>,
}

impl Visit for UltraPlanet {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("cloudColor_", &mut self.cloud_color)?;
        v.value("cloudAmbient_", &mut self.cloud_ambient)?;
        v.value("cloudCover_", &mut self.cloud_cover)?;
        if version >= 1 {
            v.value("cloudBump_", &mut self.cloud_bump)?;
        }
        v.value("cityDaytimeLight_", &mut self.city_daytime_light)?;
        v.value("animationSpeed_", &mut self.animation_speed)?;
        Ok(())
    }
}
