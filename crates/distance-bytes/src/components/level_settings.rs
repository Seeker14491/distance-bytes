use std::ops::RangeInclusive;

use super::environment::{AmbientColors, DirectionLight, Fog, Skybox};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `LevelSettings`: a level's name, modes, environment, medal times and editor state.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct LevelSettings {
    /// When the level was saved, as .NET's `DateTime.ToBinary`.
    pub date_time: Option<i64>,
    pub level_name: Option<DString>,
    /// Version 24 and later.
    pub level_description: Option<DString>,
    /// Version 25 and later.
    pub level_creator_name: Option<DString>,
    /// The game modes the level supports, as `GameModeID` and whether it's enabled, in file
    /// order.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub modes: Vec<(Option<i32>, Option<bool>)>,
    /// Version 27 and later.
    pub music_track: Option<DString>,
    /// `MusicCueID`. Version 26 and earlier.
    pub music_cue_id: Option<i32>,
    /// Version 3 and earlier, ignored by the game.
    pub skybox_material: Option<DString>,
    /// `[r, g, b, a]`, for all three ambient colors. Version 3 and earlier.
    pub ambient_light_color: Option<[f32; 4]>,
    /// Version 4 and later.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub ambient_colors: AmbientColors,
    /// Version 5 and later.
    pub enable_directional_light: Option<bool>,
    /// Version 5 and later.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub direction_light: DirectionLight,
    /// Version 5 and later.
    pub enable_shadows: Option<bool>,
    /// Version 5 and later.
    pub enable_sun_shafts: Option<bool>,
    /// Only the color in version 3 and earlier; the old skybox before version 20, without
    /// `sky_far` before version 7.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub skybox: Skybox,
    pub fog_enabled: Option<bool>,
    /// Only the color, near and far in version 3 and earlier.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub fog: Fog,
    pub diamond_time: Option<f32>,
    pub diamond_points: Option<i32>,
    pub gold_time: Option<f32>,
    pub gold_points: Option<i32>,
    pub silver_time: Option<f32>,
    pub silver_points: Option<i32>,
    pub bronze_time: Option<f32>,
    pub bronze_points: Option<i32>,
    /// Version 1, and 3 and later, as are the other ability settings.
    pub infinite_cooldown: Option<bool>,
    pub disable_flying: Option<bool>,
    pub disable_jumping: Option<bool>,
    pub disable_boosting: Option<bool>,
    pub disable_jet_rotating: Option<bool>,
    /// `LevelDifficulty`. Version 2 and later.
    pub difficulty: Option<i32>,
    /// Versions 9 to 25.
    pub disable_announcer: Option<bool>,
    /// Version 8 and later.
    pub creator_version: Option<i32>,
    /// Version 6 and later.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub editor: LevelEditorPrivateSettings,
}

/// `LevelSettings.LevelEditorPrivateSettings`: the level editor's cursor and camera.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct LevelEditorPrivateSettings {
    pub cursor_position: Option<[f32; 3]>,
    /// A quaternion `[x, y, z, w]`.
    pub cursor_rotation: Option<[f32; 4]>,
    pub camera_position: Option<[f32; 3]>,
    /// A quaternion `[x, y, z, w]`.
    pub camera_rotation: Option<[f32; 4]>,
    pub camera_orthographic: Option<bool>,
    pub camera_target_distance: Option<f32>,
}

impl Visit for LevelSettings {
    const VERSIONS: RangeInclusive<i32> = 0..=27;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("DateTime", &mut self.date_time)?;
        v.value("LevelName", &mut self.level_name)?;
        if version > 23 {
            v.value("LevelDescription", &mut self.level_description)?;
        }
        if version > 24 {
            v.value("LevelCreatorName", &mut self.level_creator_name)?;
        }
        v.dictionary("Modes", &mut self.modes, |v, (mode, enabled)| {
            v.value("Key", mode)?;
            v.value("Val", enabled)
        })?;
        if version >= 27 {
            v.value("Music Track", &mut self.music_track)?;
        } else {
            v.value("MusicCueID", &mut self.music_cue_id)?;
        }
        if version <= 3 {
            v.value("SkyboxMaterial", &mut self.skybox_material)?;
            v.value("SkyboxColor", &mut self.skybox.color)?;
            v.value("AmbientLightColor", &mut self.ambient_light_color)?;
            v.value("FogEnabled", &mut self.fog_enabled)?;
            v.value("FogColor", &mut self.fog.color)?;
            v.value("FogNear", &mut self.fog.near)?;
            v.value("FogFar", &mut self.fog.far)?;
        } else {
            self.ambient_colors.visit(v)?;
            if version >= 5 {
                v.value("EnableDirectionalLight", &mut self.enable_directional_light)?;
                self.direction_light.visit(v, true)?;
                v.value("EnableShadows", &mut self.enable_shadows)?;
                v.value("EnableSunShafts", &mut self.enable_sun_shafts)?;
            }
            if version < 20 {
                self.skybox.visit_old(v, version >= 7)?;
            } else {
                self.skybox.visit_new(v, version)?;
            }
            v.value("FogEnabled", &mut self.fog_enabled)?;
            self.fog.visit(v)?;
        }
        v.value("DiamondTime", &mut self.diamond_time)?;
        v.value("DiamondPoints", &mut self.diamond_points)?;
        v.value("GoldTime", &mut self.gold_time)?;
        v.value("GoldPoints", &mut self.gold_points)?;
        v.value("SilverTime", &mut self.silver_time)?;
        v.value("SilverPoints", &mut self.silver_points)?;
        v.value("BronzeTime", &mut self.bronze_time)?;
        v.value("BronzePoints", &mut self.bronze_points)?;
        if version == 1 || version >= 3 {
            v.value("InfiniteCooldown", &mut self.infinite_cooldown)?;
            v.value("DisableFlying", &mut self.disable_flying)?;
            v.value("DisableJumping", &mut self.disable_jumping)?;
            v.value("DisableBoosting", &mut self.disable_boosting)?;
            v.value("DisableJetRotating", &mut self.disable_jet_rotating)?;
        }
        if version >= 2 {
            v.value("Difficulty", &mut self.difficulty)?;
        }
        if (9..=25).contains(&version) {
            v.value("Disable Announcer", &mut self.disable_announcer)?;
        }
        if version >= 8 {
            v.value("creatorVersion_", &mut self.creator_version)?;
        }
        if version >= 6 {
            let e = &mut self.editor;
            v.value("LevelEditorCursorPosition", &mut e.cursor_position)?;
            v.value("LevelEditorCursorRotation", &mut e.cursor_rotation)?;
            v.value("LevelEditorCameraPosition", &mut e.camera_position)?;
            v.value("LevelEditorCameraRotation", &mut e.camera_rotation)?;
            v.value("LevelEditorCameraOrthographic", &mut e.camera_orthographic)?;
            v.value(
                "LevelEditorCameraTargetDistance",
                &mut e.camera_target_distance,
            )?;
        }
        Ok(())
    }
}
