use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `LocalLeaderboard`: the results of a level's runs on this computer.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct LocalLeaderboard {
    /// The version of the results' format. The game reads them as [`ResultInfo::VERSION`] when
    /// this is missing.
    pub results_version: Option<i32>,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub results: Vec<ResultInfo>,
    /// Version 1 and later.
    pub creator_version: Option<i32>,
}

/// `ResultInfo`: a result in a [`LocalLeaderboard`].
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct ResultInfo {
    /// Version 2 and later.
    pub profile_id: Option<i32>,
    pub profile_name: Option<DString>,
    /// The time or score. Version 2 and later.
    pub value: Option<i32>,
    /// Version 1 and earlier; the game uses it as the value unless it's negative.
    pub time: Option<i32>,
    /// Version 1 and earlier; the game uses it as the value when there's no time.
    pub points: Option<i32>,
    /// The GUID of the run's replay, -1 for none. Version 1 and later.
    pub replay_guid: Option<i64>,
    /// As .NET's `DateTime.ToBinary`. Version 3 and later.
    pub time_of_recording: Option<i64>,
}

impl ResultInfo {
    /// The format version the game writes.
    pub const VERSION: i32 = 3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version <= 1 {
            v.value("ProfileName", &mut self.profile_name)?;
            v.value("Time", &mut self.time)?;
            v.value("Points", &mut self.points)?;
            if version == 1 {
                v.value("ReplayGuid", &mut self.replay_guid)?;
            }
        }
        if version >= 2 {
            v.value("ProfileID", &mut self.profile_id)?;
            v.value("ProfileName", &mut self.profile_name)?;
            v.value("Value", &mut self.value)?;
            v.value("ReplayGuid", &mut self.replay_guid)?;
        }
        if version >= 3 {
            v.value("TimeOfRecording", &mut self.time_of_recording)?;
        }
        Ok(())
    }
}

impl Visit for LocalLeaderboard {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        let n = v.count("NumberOfResults", self.results.len())?;
        v.value("ResultsVersion", &mut self.results_version)?;
        let results_version = self.results_version.unwrap_or(ResultInfo::VERSION);
        v.records(n, &mut self.results, |v, r| r.visit(v, results_version))?;
        if version >= 1 {
            v.value("CreatorVersion", &mut self.creator_version)?;
        }
        Ok(())
    }
}
