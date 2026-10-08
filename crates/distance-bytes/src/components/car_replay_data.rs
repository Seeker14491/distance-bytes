use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `CarReplayData`: a recorded run of a car, as saved for local leaderboards and ghosts.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CarReplayData {
    /// The player's name. From version 2 the game truncates it to 64 characters when writing.
    pub name: Option<DString>,
    /// Version 2 and later.
    pub steam_id: Option<i64>,
    /// Version 2 and later.
    pub steam_name: Option<DString>,
    /// The run's result (`finishValue_`), -1 when unknown. Version 3 and later.
    pub finish_value: Option<i32>,
    /// `replayLengthMS_`, -1 when unknown. Version 3 and later.
    pub replay_length_ms: Option<i32>,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub car: CarData,
    /// The game's version of each type of player event, as of the recording.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub player_event_versions: Vec<Option<i32>>,
    #[serde(with = "crate::json::hex", skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(with = "crate::json::hex::Hex<'static>"))]
    pub event_buffer: Vec<u8>,
    /// The car's states, uncompressed. Version 4 and earlier.
    #[serde(with = "crate::json::hex", skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(with = "crate::json::hex::Hex<'static>"))]
    pub state_buffer: Vec<u8>,
    /// The car's states, compressed in two parts. Version 5 and later; version 5 uses a different
    /// encoding than later versions.
    #[serde(with = "crate::json::hex", skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(with = "crate::json::hex::Hex<'static>"))]
    pub transform_buffer: Vec<u8>,
    /// Version 5 and later.
    #[serde(with = "crate::json::hex", skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(with = "crate::json::hex::Hex<'static>"))]
    pub directive_buffer: Vec<u8>,
    /// Versions 1 to 4, ignored by the game.
    #[serde(with = "crate::json::hex", skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(with = "crate::json::hex::Hex<'static>"))]
    pub eye_state_buffer: Vec<u8>,
    /// Whether the states were cut short when compressed. Version 7 and later.
    pub data_was_truncated: Option<bool>,
}

/// `CarReplayData.CarData`: the car's look.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CarData {
    /// Always 0.
    pub version: Option<i32>,
    /// The car's name, which the game truncates to 32 characters when writing.
    pub name: Option<DString>,
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub colors: CarColors,
}

/// `CarColors`, each `[r, g, b, a]`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CarColors {
    pub primary: Option<[f32; 4]>,
    pub secondary: Option<[f32; 4]>,
    pub glow: Option<[f32; 4]>,
    pub sparkle: Option<[f32; 4]>,
}

impl Visit for CarReplayData {
    const VERSIONS: RangeInclusive<i32> = 0..=7;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Name", &mut self.name)?;
        if version >= 2 {
            v.value("SteamID", &mut self.steam_id)?;
            v.value("SteamName", &mut self.steam_name)?;
        }
        if version >= 3 {
            v.value("finishValue_", &mut self.finish_value)?;
            v.value("replayLengthMS_", &mut self.replay_length_ms)?;
        }
        let car = &mut self.car;
        v.value("CarDataVersion", &mut car.version)?;
        v.value("name_", &mut car.name)?;
        v.value("Primary", &mut car.colors.primary)?;
        v.value("Secondary", &mut car.colors.secondary)?;
        v.value("Glow", &mut car.colors.glow)?;
        v.value("Sparkle", &mut car.colors.sparkle)?;

        v.array(
            "PlayerEventVersions",
            &mut self.player_event_versions,
            |v, x| v.value("PlayerEventVersions", x),
        )?;
        v.bytes("EventBuffer", &mut self.event_buffer)?;
        if version >= 5 {
            v.bytes("transformBuffer", &mut self.transform_buffer)?;
            v.bytes("directiveBuffer", &mut self.directive_buffer)?;
        } else {
            v.bytes("StateBuffer", &mut self.state_buffer)?;
        }
        if (1..5).contains(&version) {
            v.bytes("EyeStateBuffer", &mut self.eye_state_buffer)?;
        }
        if version >= 7 {
            v.value("dataWasTruncated_", &mut self.data_was_truncated)?;
        }
        Ok(())
    }
}
