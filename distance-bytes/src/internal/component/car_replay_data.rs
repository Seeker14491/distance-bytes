use crate::internal::{Serializable, Visitor};
use crate::CarData;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CarReplayData {
    pub name: Option<String>,
    pub steam_id: i64,
    pub steam_name: Option<String>,
    pub finish_value: i32,
    pub replay_length_ms: i32,
    pub car_data: CarData,
    pub player_event_versions: Vec<i32>,
    pub event_buffer: Vec<u8>,
    pub transform_buffer: Vec<u8>,
    pub directive_buffer: Vec<u8>,
    pub state_buffer: Vec<u8>,
    pub data_was_truncated: bool,
}

impl Serializable for CarReplayData {
    const VERSION: i32 = 7;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        visitor.visit_string("Name", &mut self.name)?;

        if version >= 2 {
            visitor.visit_i64("SteamID", &mut self.steam_id)?;
            visitor.visit_string("SteamName", &mut self.steam_name)?;

            if version >= 3 {
                visitor.visit_i32("finishValue_", &mut self.finish_value)?;
                visitor.visit_i32("replayLengthMS_", &mut self.replay_length_ms)?;
            }
        }

        self.car_data.accept(&mut visitor, 0)?;
        visitor.visit_array(
            "PlayerEventVersions",
            &mut self.player_event_versions,
            |visitor, element| visitor.visit_i32("PlayerEventVersions", element),
        )?;
        visitor.visit_array("EventBuffer", &mut self.event_buffer, |visitor, elem| {
            visitor.visit_u8("EventBuffer", elem)
        })?;

        if version >= 5 {
            visitor.visit_array(
                "transformBuffer",
                &mut self.transform_buffer,
                |visitor, elem| visitor.visit_u8("transformBuffer", elem),
            )?;
            visitor.visit_array(
                "directiveBuffer",
                &mut self.directive_buffer,
                |visitor, elem| visitor.visit_u8("directiveBuffer", elem),
            )?;
        } else {
            visitor.visit_array("StateBuffer", &mut self.state_buffer, |visitor, elem| {
                visitor.visit_u8("StateBuffer", elem)
            })?;
        }

        if (1..=4).contains(&version) {
            visitor.visit_array("EyeStateBuffer", &mut Vec::new(), |visitor, elem| {
                visitor.visit_u8("EyeStateBuffer", elem)
            })?;
        }

        if version >= 7 {
            visitor.visit_bool("dataWasTruncated_", &mut self.data_was_truncated)?;
        }

        Ok(())
    }
}
