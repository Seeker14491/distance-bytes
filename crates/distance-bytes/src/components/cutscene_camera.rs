use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `CutsceneCamera`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CutsceneCamera {
    pub id: Option<i32>,
    pub movement_speed: Option<f32>,
    pub shake_amount: Option<f32>,
    pub fov: Option<f32>,
    pub time_scale: Option<f32>,
    pub glitch_intensity: Option<f32>,
    pub use_vhs: Option<bool>,
    pub text_typing: Option<bool>,
}

impl Visit for CutsceneCamera {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version > 0 {
            v.value("ID", &mut self.id)?;
            v.value("MovementSpeed", &mut self.movement_speed)?;
            v.value("ShakeAmount", &mut self.shake_amount)?;
            v.value("FOV", &mut self.fov)?;
            v.value("TimeScale", &mut self.time_scale)?;
            v.value("GlitchIntensity", &mut self.glitch_intensity)?;
            v.value("UseVHS", &mut self.use_vhs)?;
            v.value("TextTyping", &mut self.text_typing)?;
        }
        Ok(())
    }
}
