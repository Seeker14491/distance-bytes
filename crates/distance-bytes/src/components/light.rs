use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `Light` (the game's `LightWrapper`): adds light to the scene.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Light {
    /// Unity's `LightType`: 0 spot, 1 directional, 2 point. Versions 0 and 1; later versions keep
    /// the prefab's type.
    pub r#type: Option<i32>,
    /// `[r, g, b, a]`. From version 3 the game scales the color to a maximum component of 1,
    /// moving the factor into the intensity.
    pub color: Option<[f32; 4]>,
    pub intensity: Option<f32>,
    /// Version 0 only, ignored by the game.
    pub shadows: Option<i32>,
    /// Version 0 only, ignored by the game.
    pub shadow_strength: Option<f32>,
    /// Version 0 only, ignored by the game.
    pub shadow_bias: Option<f32>,
    /// Version 0 only, ignored by the game.
    pub shadow_softness: Option<f32>,
    /// Version 0 only, ignored by the game.
    pub shadow_softness_fade: Option<f32>,
    pub range: Option<f32>,
    /// In degrees. See `has_spot_angle`.
    pub spot_angle: Option<f32>,
    /// Whether the body has `spot_angle`. Versions 0 and 1 always have it; later versions only
    /// when the light is a spot light, which depends on the prefab, so it's read from the length
    /// of the body.
    pub has_spot_angle: bool,
    /// Version 0 only, ignored by the game.
    pub cookie_name: Option<DString>,
    /// Version 0 only, ignored by the game.
    pub flare_name: Option<DString>,
    /// Version 0 only, ignored by the game.
    pub render_mode: Option<i32>,
    /// Version 0 only, ignored by the game.
    pub culling_mask: Option<i32>,
    /// Version 4 and later.
    pub detailed_light: Option<bool>,
}

impl Visit for Light {
    const VERSIONS: RangeInclusive<i32> = 0..=4;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version <= 1 {
            v.value("Type", &mut self.r#type)?;
        }
        v.value("Color", &mut self.color)?;
        v.value("Intensity", &mut self.intensity)?;
        if version == 0 {
            v.value("Shadows", &mut self.shadows)?;
            v.value("ShadowStrength", &mut self.shadow_strength)?;
            v.value("ShadowBias", &mut self.shadow_bias)?;
            v.value("ShadowSoftness", &mut self.shadow_softness)?;
            v.value("ShadowSoftnessFade", &mut self.shadow_softness_fade)?;
        }
        v.value("Range", &mut self.range)?;
        if version <= 1 {
            self.has_spot_angle = true;
        } else if let Some(left) = v.remaining() {
            // The spot angle takes 4 bytes, and "Detailed Light" (written as a plain bool) 1.
            self.has_spot_angle = left >= if version >= 4 { 5 } else { 4 };
        }
        if self.has_spot_angle {
            v.value("SpotAngle", &mut self.spot_angle)?;
        }
        if version == 0 {
            v.value("CookieName", &mut self.cookie_name)?;
            v.value("FlareName", &mut self.flare_name)?;
            v.value("RenderMode", &mut self.render_mode)?;
            v.value("CullingMask", &mut self.culling_mask)?;
        }
        if version >= 4 {
            v.value("Detailed Light", &mut self.detailed_light)?;
        }
        Ok(())
    }
}
