use crate::internal::{Serializable, Visitor};
use crate::{DistanceDateTime, LevelDifficulty, LevelType, MusicCueId};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelInfo {
    level_name: Option<String>,
    relative_path: Option<String>,
    file_name_without_extension: Option<String>,
    level_version_date_time: DistanceDateTime,
    file_last_write_date_time: DistanceDateTime,
    modes: Option<HashMap<i32, bool>>,
    bronze_time: f32,
    bronze_points: i32,
    silver_time: f32,
    silver_points: i32,
    gold_time: f32,
    gold_points: i32,
    diamond_time: f32,
    diamond_points: i32,
    infinite_cooldown: bool,
    disable_flying: bool,
    disable_jumping: bool,
    disable_boosting: bool,
    disable_jet_rotating: bool,
    difficulty: LevelDifficulty,
    level_type: LevelType,
    workshop_creator_id: u64,
    music_cue_id: MusicCueId,
    description: Option<String>,
    creator_name: Option<String>,
}

impl Serializable for LevelInfo {
    const VERSION: i32 = 2;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        if version >= 0 {
            visitor.visit_string("LevelName", &mut self.level_name)?;
            visitor.visit_string("RelativePath", &mut self.relative_path)?;
            visitor.visit_string(
                "FileNameWithoutExtension",
                &mut self.file_name_without_extension,
            )?;

            visitor.visit_datetime("LevelVersionDateTime", &mut self.level_version_date_time)?;
            visitor.visit_datetime("FileLastWriteDateTime", &mut self.file_last_write_date_time)?;

            visitor.visit_dictionary_i32_to_bool("Modes", &mut self.modes)?;

            visitor.visit_f32("BronzeTime", &mut self.bronze_time)?;
            visitor.visit_i32("BronzePoints", &mut self.bronze_points)?;
            visitor.visit_f32("SilverTime", &mut self.silver_time)?;
            visitor.visit_i32("SilverPoints", &mut self.silver_points)?;
            visitor.visit_f32("GoldTime", &mut self.gold_time)?;
            visitor.visit_i32("GoldPoints", &mut self.gold_points)?;
            visitor.visit_f32("DiamondTime", &mut self.diamond_time)?;
            visitor.visit_i32("DiamondPoints", &mut self.diamond_points)?;

            visitor.visit_bool("InfiniteCooldown", &mut self.infinite_cooldown)?;
            visitor.visit_bool("DisableFlying", &mut self.disable_flying)?;
            visitor.visit_bool("DisableJumping", &mut self.disable_jumping)?;
            visitor.visit_bool("DisableBoosting", &mut self.disable_boosting)?;
            visitor.visit_bool("DisableJetRotating", &mut self.disable_jet_rotating)?;

            visitor.visit_enum("Difficulty", &mut self.difficulty)?;
            visitor.visit_enum("LevelType", &mut self.level_type)?;

            {
                let mut value = self.workshop_creator_id as i64;
                visitor.visit_i64("WorkshopCreatorID", &mut value)?;
                self.workshop_creator_id = value as u64;
            }

            visitor.visit_enum("MusicCueID", &mut self.music_cue_id)?;
        }

        if version >= 1 {
            visitor.visit_string("Description", &mut self.description)?;
        }

        if version >= 2 {
            visitor.visit_string("CreatorName", &mut self.creator_name)?;
        }

        Ok(())
    }
}
