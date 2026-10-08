use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `AmbientColors`: the ambient light colors, each `[r, g, b, a]`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AmbientColors {
    pub sky: Option<[f32; 4]>,
    pub equator: Option<[f32; 4]>,
    pub ground: Option<[f32; 4]>,
}

impl AmbientColors {
    pub(crate) fn visit(&mut self, v: &mut impl Visitor) -> Result<()> {
        v.value("AmbientSkyColor", &mut self.sky)?;
        v.value("AmbientEquatorColor", &mut self.equator)?;
        v.value("AmbientGroundColor", &mut self.ground)
    }
}

/// `LevelSettings.DirectionLight`: the directional light.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct DirectionLight {
    /// `[r, g, b, a]`.
    pub color: Option<[f32; 4]>,
    pub intensity: Option<f32>,
    /// In degrees. Not in version 0 of `Biodome`.
    pub longitude: Option<f32>,
    /// In degrees. Not in version 0 of `Biodome`.
    pub latitude: Option<f32>,
}

impl DirectionLight {
    pub(crate) fn visit(&mut self, v: &mut impl Visitor, with_direction: bool) -> Result<()> {
        v.value("DirectionLightColor", &mut self.color)?;
        v.value("DirectionLightIntensity", &mut self.intensity)?;
        if with_direction {
            v.value("Longitude", &mut self.longitude)?;
            v.value("Latitude", &mut self.latitude)?;
        }
        Ok(())
    }
}

/// `LevelSettings.Skybox`: the sky. Versions before 20 (of both `LevelSettings` and `Biodome`)
/// have the old skybox, later ones the new one and a background layer.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Skybox {
    /// `LevelSettings.Skybox.Old.Type`. Old skybox only.
    pub r#type: Option<i32>,
    /// `[r, g, b, a]`. Old skybox only.
    pub color: Option<[f32; 4]>,
    /// Old skybox only.
    pub hue: Option<f32>,
    /// Old skybox only.
    pub saturation: Option<f32>,
    /// New skybox only, as are the fields up to `color_bottom`.
    pub use_sun: Option<bool>,
    pub sun_size: Option<f32>,
    pub sun_brightness: Option<f32>,
    pub use_stars: Option<bool>,
    /// `[r, g, b, a]`.
    pub star_color1: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub star_color2: Option<[f32; 4]>,
    /// Version 23 and later.
    pub star_variation: Option<f32>,
    pub star_brightness: Option<f32>,
    pub star_glow: Option<f32>,
    pub use_storm: Option<bool>,
    /// `[r, g, b, a]`.
    pub storm_color: Option<[f32; 4]>,
    pub storm_density: Option<f32>,
    pub storm_cutoff: Option<f32>,
    pub storm_motion: Option<f32>,
    pub storm_phase: Option<f32>,
    /// `[r, g, b, a]`.
    pub color_top: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub color_middle: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub color_bottom: Option<[f32; 4]>,
    pub exposure: Option<f32>,
    /// Not in the old skybox of the earliest versions.
    pub sky_far: Option<f32>,
    pub sky_fade_distance: Option<f32>,
    pub fog_amount: Option<f32>,
    /// New skybox only, as are the fields after it.
    pub use_background_layer: Option<bool>,
    pub background_layer_name: Option<DString>,
    pub background_scroll: Option<f32>,
    /// Version 22 and later.
    pub background_fog_amount: Option<f32>,
}

impl Skybox {
    /// `Skybox.OnVisitOld`.
    pub(crate) fn visit_old(&mut self, v: &mut impl Visitor, sky_far: bool) -> Result<()> {
        v.value("Skybox", &mut self.r#type)?;
        v.value("SkyboxColor", &mut self.color)?;
        v.value("SkyboxHue", &mut self.hue)?;
        v.value("SkyboxSaturation", &mut self.saturation)?;
        self.visit_common(v, sky_far)
    }

    /// `Skybox.OnVisit`, given the version of the component visiting it.
    pub(crate) fn visit_new(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Sun", &mut self.use_sun)?;
        v.value("SunSize", &mut self.sun_size)?;
        v.value("SunBrightness", &mut self.sun_brightness)?;
        v.value("Stars", &mut self.use_stars)?;
        v.value("StarColor1", &mut self.star_color1)?;
        v.value("StarColor2", &mut self.star_color2)?;
        if version >= 23 {
            v.value("StarVariation", &mut self.star_variation)?;
        }
        v.value("StarBrightness", &mut self.star_brightness)?;
        v.value("StarGlow", &mut self.star_glow)?;
        v.value("Storm", &mut self.use_storm)?;
        v.value("StormColor", &mut self.storm_color)?;
        v.value("StormDensity", &mut self.storm_density)?;
        v.value("StormCutoff", &mut self.storm_cutoff)?;
        v.value("StormMotion", &mut self.storm_motion)?;
        v.value("StormPhase", &mut self.storm_phase)?;
        v.value("SkyboxColorTop", &mut self.color_top)?;
        v.value("SkyboxColorMiddle", &mut self.color_middle)?;
        v.value("SkyboxColorBottom", &mut self.color_bottom)?;
        self.visit_common(v, true)?;
        v.value("Background Layer", &mut self.use_background_layer)?;
        v.value("Background Layer Name", &mut self.background_layer_name)?;
        v.value("Background Scroll", &mut self.background_scroll)?;
        if version >= 22 {
            v.value("Background Fog Amount", &mut self.background_fog_amount)?;
        }
        Ok(())
    }

    fn visit_common(&mut self, v: &mut impl Visitor, sky_far: bool) -> Result<()> {
        v.value("SkyboxExposure", &mut self.exposure)?;
        if sky_far {
            v.value("SkyFar", &mut self.sky_far)?;
        }
        v.value("SkyFadeDistance", &mut self.sky_fade_distance)?;
        v.value("SkyboxFogAmount", &mut self.fog_amount)
    }
}

/// `LevelSettings.Fog`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Fog {
    /// `[r, g, b, a]`.
    pub color: Option<[f32; 4]>,
    pub brightness: Option<f32>,
    pub density: Option<f32>,
    pub max_distance: Option<f32>,
    pub near: Option<f32>,
    pub far: Option<f32>,
    pub bottom: Option<f32>,
    pub bottom_penumbra: Option<f32>,
    pub top: Option<f32>,
    pub top_penumbra: Option<f32>,
}

impl Fog {
    pub(crate) fn visit(&mut self, v: &mut impl Visitor) -> Result<()> {
        v.value("FogColor", &mut self.color)?;
        v.value("FogBrightness", &mut self.brightness)?;
        v.value("FogDensity", &mut self.density)?;
        v.value("FogMaxDistance", &mut self.max_distance)?;
        v.value("FogNear", &mut self.near)?;
        v.value("FogFar", &mut self.far)?;
        v.value("FogBottom", &mut self.bottom)?;
        v.value("FogBottomPenumbra", &mut self.bottom_penumbra)?;
        v.value("FogTop", &mut self.top)?;
        v.value("FogTopPenumbra", &mut self.top_penumbra)
    }
}

/// `Biodome`: a zone with its own ambient light, directional light, sky and fog.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Biodome {
    /// Version 0 only.
    pub radius: Option<f32>,
    /// `Biodome.Type`: 0 sphere, 1 cube. Version 1 and later.
    pub r#type: Option<i32>,
    pub fade_distance: Option<f32>,
    pub enable_ambient_light: Option<bool>,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub ambient_colors: AmbientColors,
    pub has_directional_light: Option<bool>,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub direction_light: DirectionLight,
    pub enable_skybox: Option<bool>,
    /// Version 4 only, ignored by the game.
    pub background_layer: Option<DString>,
    /// The old skybox before version 20, without `sky_far` before version 3.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub skybox: Skybox,
    pub enable_fog: Option<bool>,
    /// Version 2 and later.
    pub fog_height_is_relative: Option<bool>,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub fog: Fog,
}

impl Visit for Biodome {
    const VERSIONS: RangeInclusive<i32> = 0..=23;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.value("Radius", &mut self.radius)?;
        } else {
            v.value("Type", &mut self.r#type)?;
        }
        v.value("FadeDistance", &mut self.fade_distance)?;
        v.value("EnableAmbientLight", &mut self.enable_ambient_light)?;
        self.ambient_colors.visit(v)?;
        v.value("HasDirectionalLight", &mut self.has_directional_light)?;
        self.direction_light.visit(v, version > 0)?;
        v.value("EnableSkybox", &mut self.enable_skybox)?;
        if version == 4 {
            v.value("Background Layer", &mut self.background_layer)?;
        }
        if version < 20 {
            self.skybox.visit_old(v, version >= 3)?;
        } else {
            self.skybox.visit_new(v, version)?;
        }
        v.value("EnableFog", &mut self.enable_fog)?;
        if version >= 2 {
            v.value("FogHeightIsRelative", &mut self.fog_height_is_relative)?;
        }
        self.fog.visit(v)
    }
}
