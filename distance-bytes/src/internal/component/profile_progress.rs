use crate::internal::{Serializable, VisitDirection, Visitor};
use crate::{Enum, MedalStatus};
use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProfileProgress {
    pub relative_level_paths_to_progress: HashMap<String, LevelProgress>,
    pub total_medal_count: i64,
    pub adventure_index: i32,
    pub finished_adventure_count: i64,
    pub unlocked_levels: Vec<Option<String>>,
    pub total_levels_attempted_count: i64,
    pub total_levels_finished_count: i64,
    pub completed_tricks: Vec<Option<String>>,
    pub unlocked_adventure_levels: Vec<Option<String>>,
    pub finished_lost_to_echoes_count: i64,
    pub unlocked_campaign_plus: bool,
    pub unlocked_lost_to_echoes: bool,
    pub unseen_levels: Vec<Option<String>>,
    pub interceptor_piece_flags: u32,
    pub stored_unseen_car_flags: u32,
    pub crab_flags: u32,
}

impl ProfileProgress {
    fn accept_level_progress<V: Visitor>(&mut self, mut visitor: V) -> Result<()> {
        let mut level_progress_count: i32 =
            self.relative_level_paths_to_progress.len().try_into()?;
        visitor.visit_i32("LevelProgressCount", &mut level_progress_count)?;

        let mut progress_version = LevelProgress::VERSION;
        visitor.visit_i32("ProgressVersion", &mut progress_version)?;

        match V::VISIT_DIRECTION {
            VisitDirection::Out => {
                for (relative_level_path, level_progress) in
                    &mut self.relative_level_paths_to_progress
                {
                    let mut key = Some(relative_level_path.clone());
                    visitor.visit_string("RelativeLevelPath", &mut key)?;

                    level_progress.accept(&mut visitor, progress_version)?;
                }
            }
            VisitDirection::In => {
                for _ in 0..level_progress_count {
                    let mut relative_level_path = None;
                    visitor.visit_string("RelativeLevelPath", &mut relative_level_path)?;
                    let relative_level_path = relative_level_path
                        .ok_or_else(|| format_err!("relative level path was null"))?;

                    let mut level_progress = LevelProgress::default();
                    level_progress.accept(&mut visitor, progress_version)?;
                    self.relative_level_paths_to_progress
                        .insert(relative_level_path, level_progress);
                }
            }
        }

        Ok(())
    }
}

impl Serializable for ProfileProgress {
    const VERSION: i32 = 11;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        self.accept_level_progress(&mut visitor)?;

        if version < 2 {
            let mut total_medal_count = 0;
            visitor.visit_i32("TotalMedalCount", &mut total_medal_count)?;
            self.total_medal_count = total_medal_count.into();

            visitor.visit_i32("AdventureIndex", &mut self.adventure_index)?;
            if version == 1 {
                let mut finished_adventure_count = false;
                visitor.visit_bool("FinishedAdventureCount", &mut finished_adventure_count)?;
                self.finished_adventure_count = finished_adventure_count.into();

                visitor.visit_array(
                    "UnlockedLevels",
                    &mut self.unlocked_levels,
                    |visitor, item| visitor.visit_string("UnlockedLevels", item),
                )?;
            }
        } else {
            visitor.visit_array(
                "UnlockedLevels",
                &mut self.unlocked_levels,
                |visitor, item| visitor.visit_string("UnlockedLevels", item),
            )?;
            visitor.visit_i64("TotalMedalCount", &mut self.total_medal_count)?;
            visitor.visit_i32("AdventureIndex", &mut self.adventure_index)?;
            visitor.visit_i64("FinishedAdventureCount", &mut self.finished_adventure_count)?;
            visitor.visit_i64(
                "TotalLevelsAttemptedCount",
                &mut self.total_levels_attempted_count,
            )?;
            visitor.visit_i64(
                "TotalLevelsFinishedCount",
                &mut self.total_levels_finished_count,
            )?;
        }

        if version >= 3 {
            visitor.visit_array(
                "CompletedTricks",
                &mut self.completed_tricks,
                |visitor, item| visitor.visit_string("CompletedTricks", item),
            )?;
        }
        if version >= 4 {
            visitor.visit_array(
                "UnlockedAdventureLevels",
                &mut self.unlocked_adventure_levels,
                |visitor, item| visitor.visit_string("UnlockedAdventureLevels", item),
            )?;
            visitor.visit_i64(
                "FinishedLostToEchoesCount",
                &mut self.finished_lost_to_echoes_count,
            )?;
        }
        if version >= 5 {
            visitor.visit_bool("UnlockedCampaignPlus", &mut self.unlocked_campaign_plus)?;
            visitor.visit_bool("UnlockedLostToEchoes", &mut self.unlocked_lost_to_echoes)?;
        }
        if version >= 6 {
            visitor.visit_array("UnseenLevels", &mut self.unseen_levels, |visitor, item| {
                visitor.visit_string("UnseenLevels", item)
            })?;
            visitor.visit_bool("ShowCampaignPlusDot", &mut false)?;
        }
        if version >= 7 {
            visitor.visit_bool("ShowCampaignDot", &mut false)?;
            visitor.visit_bool("ShowEchoesDot", &mut false)?;
        }
        if version >= 8 {
            visitor.visit_u32("InterceptorPieceFlags", &mut self.interceptor_piece_flags)?;
        }
        if version >= 9 {
            visitor.visit_u32("storedUnseenCarFlags_", &mut self.stored_unseen_car_flags)?;
        }
        if version >= 11 {
            visitor.visit_u32("crabFlags_", &mut self.crab_flags)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct LevelProgress {
    pub last_played_level_version: Option<String>,
    pub prompted_to_vote_on_level: bool,
    pub medals: Vec<Enum<MedalStatus>>,
    pub top_results: Vec<i32>,
    pub time_last_played: i64,
}

impl LevelProgress {
    const VERSION: i32 = 1;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        if version >= 0 {
            visitor.visit_string(
                "LastPlayedLevelVersion",
                &mut self.last_played_level_version,
            )?;
            visitor.visit_bool("PromptedToVoteOnLevel", &mut self.prompted_to_vote_on_level)?;
            visitor.visit_array("Medals", &mut self.medals, |visitor, item| {
                visitor.visit_enum("Medals", item)
            })?;
            visitor.visit_array("TopResults", &mut self.top_results, |visitor, item| {
                visitor.visit_i32("TopResults", item)
            })?;
        }

        if version >= 1 {
            visitor.visit_i64("TimeLastPlayed", &mut self.time_last_played)?;
        }

        if V::VISIT_DIRECTION == VisitDirection::In {
            self.medals.resize(17, MedalStatus::None.into());
            self.top_results.resize(17, -1);
        }

        Ok(())
    }
}
