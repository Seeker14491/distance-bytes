//! The level settings of level versions 0 and 1, which have a format of their own instead of a
//! game object.

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// Level settings in the format of level versions 0 and 1, read by the game's
/// `LevelSettings.OldVisit`. Colors are `[r, g, b, a]`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LegacyLevelSettings {
    /// Ignored by the game.
    pub skybox_material: Option<DString>,
    pub skybox_color: Option<[f32; 4]>,
    /// Ignored by the game.
    pub ambient_light_color: Option<[f32; 4]>,
    pub fog_enabled: Option<bool>,
    pub fog_color: Option<[f32; 4]>,
    /// Ignored by the game.
    pub fog_mode: Option<i32>,
    /// Ignored by the game.
    pub fog_density: Option<f32>,
    pub fog_near: Option<f32>,
    pub fog_far: Option<f32>,
    /// Ignored by the game.
    pub light_halo_strength: Option<f32>,
    /// Ignored by the game.
    pub flare_strength: Option<f32>,
    /// Version 7 and earlier.
    pub car_spawn_position: Option<[f32; 3]>,
    /// Quaternion as `[x, y, z, w]`. Version 7 and earlier.
    pub car_spawn_rotation: Option<[f32; 4]>,
    /// The game modes the level supports, by name, and whether each is enabled, in file order.
    /// Versions 1 to 3.
    pub mode_names: Vec<(Option<DString>, Option<bool>)>,
    /// The game modes the level supports, as `GameModeID` and whether it's enabled, in file
    /// order. Version 4 and later.
    pub modes: Vec<(Option<i32>, Option<bool>)>,
    /// When the level was saved, as .NET's `DateTime.ToBinary`. Version 2 and later.
    pub date_time: Option<i64>,
    /// Version 4 and later, ignored by the game.
    pub is_read_only: Option<bool>,
    /// Version 5 and later.
    pub level_name: Option<DString>,
    /// `MusicCueID`. Version 6 and later.
    pub music_cue_id: Option<i32>,
    /// Version 7 and later.
    pub diamond_time: Option<f32>,
    /// Version 7 and later.
    pub diamond_points: Option<i32>,
    /// Version 7 and later.
    pub gold_time: Option<f32>,
    /// Version 7 and later.
    pub gold_points: Option<i32>,
    /// Version 7 and later.
    pub silver_time: Option<f32>,
    /// Version 7 and later.
    pub silver_points: Option<i32>,
    /// Version 7 and later.
    pub bronze_time: Option<f32>,
    /// Version 7 and later.
    pub bronze_points: Option<i32>,
}

impl LegacyLevelSettings {
    /// Settings of the given version with every value set to a distinct sample and an entry in
    /// every dictionary. For testing against the game on versions that no file has.
    #[doc(hidden)]
    pub fn sample(version: i32) -> Self {
        *<Self as Visit>::sample(version)
    }
}

impl Visit for LegacyLevelSettings {
    /// The game reads any version from 0 the same as the newest it knows, 8. It reads nothing for
    /// negative versions.
    const VERSIONS: RangeInclusive<i32> = 0..=i32::MAX;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("SkyboxMaterial", &mut self.skybox_material)?;
        v.value("SkyboxColor", &mut self.skybox_color)?;
        v.value("AmbientLightColor", &mut self.ambient_light_color)?;
        v.value("FogEnabled", &mut self.fog_enabled)?;
        v.value("FogColor", &mut self.fog_color)?;
        v.value("FogMode", &mut self.fog_mode)?;
        v.value("FogDensity", &mut self.fog_density)?;
        v.value("FogNear", &mut self.fog_near)?;
        v.value("FogFar", &mut self.fog_far)?;
        v.value("LightHaloStrength", &mut self.light_halo_strength)?;
        v.value("FlareStrength", &mut self.flare_strength)?;
        if version < 8 {
            v.value("CarSpawnPos", &mut self.car_spawn_position)?;
            v.value("CarSpawnOrient", &mut self.car_spawn_rotation)?;
        }
        if (1..4).contains(&version) {
            v.dictionary("Modes", &mut self.mode_names, |v, (mode, enabled)| {
                v.value("Key", mode)?;
                v.value("Val", enabled)
            })?;
        } else if version >= 4 {
            v.dictionary("Modes", &mut self.modes, |v, (mode, enabled)| {
                v.value("Key", mode)?;
                v.value("Val", enabled)
            })?;
        }
        if version >= 2 {
            v.value("DateTime", &mut self.date_time)?;
        }
        // The game checks for version 3 and 4, which skips version 3.
        if version >= 4 {
            v.value("IsReadOnly", &mut self.is_read_only)?;
        }
        if version >= 5 {
            v.value("LevelName", &mut self.level_name)?;
        }
        if version >= 6 {
            v.value("MusicCueID", &mut self.music_cue_id)?;
        }
        if version >= 7 {
            v.value("DiamondTime", &mut self.diamond_time)?;
            v.value("DiamondPoints", &mut self.diamond_points)?;
            v.value("GoldTime", &mut self.gold_time)?;
            v.value("GoldPoints", &mut self.gold_points)?;
            v.value("SilverTime", &mut self.silver_time)?;
            v.value("SilverPoints", &mut self.silver_points)?;
            v.value("BronzeTime", &mut self.bronze_time)?;
            v.value("BronzePoints", &mut self.bronze_points)?;
        }
        Ok(())
    }
}
