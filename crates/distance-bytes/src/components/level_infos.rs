use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `LevelInfos`: the game's cache of level metadata (`Settings/LevelInfos.bytes`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelInfos {
    /// The version of the infos' format. The game reads them as [`LevelInfo::VERSION`] when this
    /// is missing, and discards them when it isn't the current version.
    pub level_info_version: Option<i32>,
    pub level_infos: Vec<LevelInfo>,
}

/// `LevelInfo`: a level's metadata in [`LevelInfos`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelInfo {
    pub level_name: Option<DString>,
    pub relative_path: Option<DString>,
    pub file_name_without_extension: Option<DString>,
    /// As .NET's `DateTime.ToBinary`.
    pub level_version_date_time: Option<i64>,
    /// As .NET's `DateTime.ToBinary`.
    pub file_last_write_date_time: Option<i64>,
    /// The game modes the level supports, as `GameModeID` and whether it's enabled, in file
    /// order.
    pub modes: Vec<(Option<i32>, Option<bool>)>,
    pub bronze_time: Option<f32>,
    pub bronze_points: Option<i32>,
    pub silver_time: Option<f32>,
    pub silver_points: Option<i32>,
    pub gold_time: Option<f32>,
    pub gold_points: Option<i32>,
    pub diamond_time: Option<f32>,
    pub diamond_points: Option<i32>,
    pub infinite_cooldown: Option<bool>,
    pub disable_flying: Option<bool>,
    pub disable_jumping: Option<bool>,
    pub disable_boosting: Option<bool>,
    pub disable_jet_rotating: Option<bool>,
    /// `LevelDifficulty`.
    pub difficulty: Option<i32>,
    /// `LevelType`.
    pub level_type: Option<i32>,
    /// The Steam ID of a workshop level's creator, an unsigned value stored as signed.
    pub workshop_creator_id: Option<i64>,
    /// `MusicCueID`.
    pub music_cue_id: Option<i32>,
    /// Version 1 and later.
    pub description: Option<DString>,
    /// Version 2 and later.
    pub creator_name: Option<DString>,
}

impl LevelInfo {
    /// The format version the game writes.
    pub const VERSION: i32 = 2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("LevelName", &mut self.level_name)?;
            v.value("RelativePath", &mut self.relative_path)?;
            v.value("FileNameWithoutExtension", &mut self.file_name_without_extension)?;
            v.value("LevelVersionDateTime", &mut self.level_version_date_time)?;
            v.value("FileLastWriteDateTime", &mut self.file_last_write_date_time)?;
            v.dictionary("Modes", &mut self.modes, |v, (mode, enabled)| {
                v.value("Key", mode)?;
                v.value("Val", enabled)
            })?;
            v.value("BronzeTime", &mut self.bronze_time)?;
            v.value("BronzePoints", &mut self.bronze_points)?;
            v.value("SilverTime", &mut self.silver_time)?;
            v.value("SilverPoints", &mut self.silver_points)?;
            v.value("GoldTime", &mut self.gold_time)?;
            v.value("GoldPoints", &mut self.gold_points)?;
            v.value("DiamondTime", &mut self.diamond_time)?;
            v.value("DiamondPoints", &mut self.diamond_points)?;
            v.value("InfiniteCooldown", &mut self.infinite_cooldown)?;
            v.value("DisableFlying", &mut self.disable_flying)?;
            v.value("DisableJumping", &mut self.disable_jumping)?;
            v.value("DisableBoosting", &mut self.disable_boosting)?;
            v.value("DisableJetRotating", &mut self.disable_jet_rotating)?;
            v.value("Difficulty", &mut self.difficulty)?;
            v.value("LevelType", &mut self.level_type)?;
            v.value("WorkshopCreatorID", &mut self.workshop_creator_id)?;
            v.value("MusicCueID", &mut self.music_cue_id)?;
        }
        if version >= 1 {
            v.value("Description", &mut self.description)?;
        }
        if version >= 2 {
            v.value("CreatorName", &mut self.creator_name)?;
        }
        Ok(())
    }
}

impl Visit for LevelInfos {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        let n = v.count("NumberOfLevelInfos", self.level_infos.len())?;
        v.value("LevelInfoVersion", &mut self.level_info_version)?;
        let info_version = self.level_info_version.unwrap_or(LevelInfo::VERSION);
        v.records(n, &mut self.level_infos, |v, info| info.visit(v, info_version))
    }
}
