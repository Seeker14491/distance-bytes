use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `VirusDropperDroneLogic`: Enemy that tries to drop in front of the car.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct VirusDropperDroneLogic {
    pub bank_id: Option<i32>,
    /// Version 1 only.
    pub trigger_distance: Option<f32>,
    /// Version 1 only.
    pub chase_track_ahead_distance: Option<f32>,
    /// Version 1 only.
    pub stomp_track_ahead_distance: Option<f32>,
    /// Version 1 only.
    pub chase_time: Option<f32>,
    /// Version 1 only.
    pub chase_height: Option<f32>,
    /// Version 1 only.
    pub stomp_height: Option<f32>,
    /// Version 1 only.
    pub stomp_speed: Option<f32>,
    /// Version 1 only.
    pub stomp_animation_speed: Option<f32>,
    /// Version 1 only.
    pub distance_before_letting_player_go: Option<f32>,
}

impl Visit for VirusDropperDroneLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("bankID_", &mut self.bank_id)?;
        }
        if version >= 1 {
            v.value("triggerDistance_", &mut self.trigger_distance)?;
            v.value(
                "chaseTrackAheadDistance_",
                &mut self.chase_track_ahead_distance,
            )?;
            v.value(
                "stompTrackAheadDistance_",
                &mut self.stomp_track_ahead_distance,
            )?;
            v.value("chaseTime_", &mut self.chase_time)?;
            v.value("chaseHeight_", &mut self.chase_height)?;
            v.value("stompHeight_", &mut self.stomp_height)?;
            v.value("stompSpeed_", &mut self.stomp_speed)?;
            v.value("stompAnimationSpeed_", &mut self.stomp_animation_speed)?;
            v.value(
                "distanceBeforeLettingPlayerGo_",
                &mut self.distance_before_letting_player_go,
            )?;
        }
        Ok(())
    }
}
