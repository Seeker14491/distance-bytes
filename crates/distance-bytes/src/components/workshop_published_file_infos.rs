use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `WorkshopPublishedFileInfos`, the game's `WorkshopLevelInfos`: its cache of the Workshop
/// levels the player is subscribed to (`Levels/WorkshopLevels/WorkshopLevelInfos.bytes`).
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct WorkshopPublishedFileInfos {
    /// The version of the infos' format, which the game ignores.
    pub workshop_level_data_version: Option<i32>,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub workshop_level_infos: Vec<WorkshopLevelInfo>,
}

/// `WorkshopLevelInfo`: a Workshop level's metadata in [`WorkshopPublishedFileInfos`].
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct WorkshopLevelInfo {
    /// The Steam ID of the published file, an unsigned value stored as signed.
    pub level_id: Option<i64>,
    pub title: Option<DString>,
    pub description: Option<DString>,
    /// In Unix time.
    pub time_updated: Option<u32>,
    /// In Unix time.
    pub time_created: Option<u32>,
    pub tags: Option<DString>,
    /// The Steam ID of the level's creator, an unsigned value stored as signed.
    pub steam_owner_id: Option<i64>,
    pub steam_owner_name: Option<DString>,
    pub relative_path: Option<DString>,
    /// `WorkshopType`.
    pub workshop_type: Option<i32>,
    pub vote_reports: Option<i32>,
    pub votes_for: Option<i32>,
    pub votes_against: Option<i32>,
    pub vote_score: Option<f32>,
    /// Steam's `EWorkshopVote`.
    pub workshop_vote: Option<i32>,
}

impl WorkshopLevelInfo {
    fn visit(&mut self, v: &mut impl Visitor) -> Result<()> {
        v.value("LevelID", &mut self.level_id)?;
        v.value("Title", &mut self.title)?;
        v.value("Description", &mut self.description)?;
        v.value("Time Updated", &mut self.time_updated)?;
        v.value("Time Created", &mut self.time_created)?;
        v.value("Tags", &mut self.tags)?;
        v.value("SteamOwnerID", &mut self.steam_owner_id)?;
        v.value("SteamOwnerName", &mut self.steam_owner_name)?;
        v.value("RelativePath", &mut self.relative_path)?;
        v.value("WorkshopType", &mut self.workshop_type)?;
        v.value("VoteReports", &mut self.vote_reports)?;
        v.value("VotesFor", &mut self.votes_for)?;
        v.value("VotesAgainst", &mut self.votes_against)?;
        v.value("VoteScore", &mut self.vote_score)?;
        v.value("WorkshopVote", &mut self.workshop_vote)
    }
}

impl Visit for WorkshopPublishedFileInfos {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        let n = v.count("WorkshopLevelDataCount", self.workshop_level_infos.len())?;
        v.value(
            "WorkshopLevelDataVersion",
            &mut self.workshop_level_data_version,
        )?;
        v.records(n, &mut self.workshop_level_infos, |v, info| info.visit(v))
    }
}
