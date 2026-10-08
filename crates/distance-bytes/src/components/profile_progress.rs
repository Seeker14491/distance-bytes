use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `ProfileProgress`: a profile's progress through levels and the campaign.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProfileProgress {
    /// The version of the levels' progress format. The game reads them as
    /// [`LevelProgress::VERSION`] when this is missing.
    pub level_progress_version: Option<i32>,
    /// The progress in each level, by the level's relative path, in file order.
    pub level_progress: Vec<(Option<DString>, LevelProgress)>,
    /// Version 1 and later.
    pub unlocked_levels: Vec<Option<DString>>,
    /// A 32-bit integer before version 2, which keeps only the low 32 bits when writing.
    pub total_medal_count: Option<i64>,
    pub adventure_index: Option<i32>,
    /// Version 1 and later. A bool in version 1, which is written as true for any value but 0.
    pub finished_adventure_count: Option<i64>,
    /// Version 2 and later.
    pub total_levels_attempted_count: Option<i64>,
    /// Version 2 and later.
    pub total_levels_finished_count: Option<i64>,
    /// Version 3 and later.
    pub completed_tricks: Vec<Option<DString>>,
    /// Version 4 and later.
    pub unlocked_adventure_levels: Vec<Option<DString>>,
    /// Version 4 and later.
    pub finished_lost_to_echoes_count: Option<i64>,
    /// Version 5 and later.
    pub unlocked_campaign_plus: Option<bool>,
    /// Version 5 and later.
    pub unlocked_lost_to_echoes: Option<bool>,
    /// The relative paths of levels not yet seen in the level select. Version 6 and later.
    pub unseen_levels: Vec<Option<DString>>,
    /// Version 6 and later, ignored by the game.
    pub show_campaign_plus_dot: Option<bool>,
    /// Version 7 and later, ignored by the game.
    pub show_campaign_dot: Option<bool>,
    /// Version 7 and later, ignored by the game.
    pub show_echoes_dot: Option<bool>,
    /// Version 8 and later.
    pub interceptor_piece_flags: Option<u32>,
    /// `storedUnseenCarFlags_`. Version 9 and later.
    pub unseen_car_flags: Option<u32>,
    /// `crabFlags_`. Version 11 and later.
    pub crab_flags: Option<u32>,
}

/// `ProfileProgress.LevelProgress`: a profile's progress in a level.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LevelProgress {
    /// The level's version (its `DateTime.ToBinary`) when last played, as a decimal string.
    pub last_played_level_version: Option<DString>,
    pub prompted_to_vote_on_level: Option<bool>,
    /// The `MedalStatus` in each game mode, indexed by `GameModeID`.
    pub medals: Vec<Option<i32>>,
    /// The best time or score in each game mode, indexed by `GameModeID`, -1 for none.
    pub top_results: Vec<Option<i32>>,
    /// As .NET's `DateTime.ToBinary`. Version 1 and later.
    pub time_last_played: Option<i64>,
}

impl LevelProgress {
    /// The format version the game writes.
    pub const VERSION: i32 = 1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value(
                "LastPlayedLevelVersion",
                &mut self.last_played_level_version,
            )?;
            v.value("PromptedToVoteOnLevel", &mut self.prompted_to_vote_on_level)?;
            v.array("Medals", &mut self.medals, |v, m| v.value("Medals", m))?;
            v.array("TopResults", &mut self.top_results, |v, r| {
                v.value("TopResults", r)
            })?;
        }
        if version >= 1 {
            v.value("TimeLastPlayed", &mut self.time_last_played)?;
        }
        Ok(())
    }
}

impl ProfileProgress {
    fn visit_level_progress(&mut self, v: &mut impl Visitor) -> Result<()> {
        let n = v.count("LevelProgressCount", self.level_progress.len())?;
        v.value("ProgressVersion", &mut self.level_progress_version)?;
        let progress_version = self
            .level_progress_version
            .unwrap_or(LevelProgress::VERSION);
        v.records(n, &mut self.level_progress, |v, (path, progress)| {
            v.value("RelativeLevelPath", path)?;
            progress.visit(v, progress_version)
        })
    }

    fn visit_unlocked_levels(&mut self, v: &mut impl Visitor) -> Result<()> {
        v.array("UnlockedLevels", &mut self.unlocked_levels, |v, l| {
            v.value("UnlockedLevels", l)
        })
    }
}

impl Visit for ProfileProgress {
    const VERSIONS: RangeInclusive<i32> = 0..=11;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version < 2 {
            self.visit_level_progress(v)?;
            let mut count = self.total_medal_count.map(|c| c as i32);
            v.value("TotalMedalCount", &mut count)?;
            self.total_medal_count = count.map(i64::from);
            v.value("AdventureIndex", &mut self.adventure_index)?;
            if version == 1 {
                let mut finished = self.finished_adventure_count.map(|c| c != 0);
                v.value("FinishedAdventureCount", &mut finished)?;
                self.finished_adventure_count = finished.map(i64::from);
                self.visit_unlocked_levels(v)?;
            }
        }
        if version >= 2 {
            self.visit_level_progress(v)?;
            self.visit_unlocked_levels(v)?;
            v.value("TotalMedalCount", &mut self.total_medal_count)?;
            v.value("AdventureIndex", &mut self.adventure_index)?;
            v.value("FinishedAdventureCount", &mut self.finished_adventure_count)?;
            v.value(
                "TotalLevelsAttemptedCount",
                &mut self.total_levels_attempted_count,
            )?;
            v.value(
                "TotalLevelsFinishedCount",
                &mut self.total_levels_finished_count,
            )?;
        }
        if version >= 3 {
            v.array("CompletedTricks", &mut self.completed_tricks, |v, t| {
                v.value("CompletedTricks", t)
            })?;
        }
        if version >= 4 {
            v.array(
                "UnlockedAdventureLevels",
                &mut self.unlocked_adventure_levels,
                |v, l| v.value("UnlockedAdventureLevels", l),
            )?;
            v.value(
                "FinishedLostToEchoesCount",
                &mut self.finished_lost_to_echoes_count,
            )?;
        }
        if version >= 5 {
            v.value("UnlockedCampaignPlus", &mut self.unlocked_campaign_plus)?;
            v.value("UnlockedLostToEchoes", &mut self.unlocked_lost_to_echoes)?;
        }
        if version >= 6 {
            v.array("UnseenLevels", &mut self.unseen_levels, |v, l| {
                v.value("UnseenLevels", l)
            })?;
            v.value("ShowCampaignPlusDot", &mut self.show_campaign_plus_dot)?;
        }
        if version >= 7 {
            v.value("ShowCampaignDot", &mut self.show_campaign_dot)?;
            v.value("ShowEchoesDot", &mut self.show_echoes_dot)?;
        }
        if version >= 8 {
            v.value("InterceptorPieceFlags", &mut self.interceptor_piece_flags)?;
        }
        if version >= 9 {
            v.value("storedUnseenCarFlags_", &mut self.unseen_car_flags)?;
        }
        if version >= 11 {
            v.value("crabFlags_", &mut self.crab_flags)?;
        }
        Ok(())
    }
}
