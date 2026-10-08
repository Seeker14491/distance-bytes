use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `ShardCluster`: Drops a collection of sharp objects in front of the car.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShardCluster {
    pub trigger_distance: Option<f32>,
    pub animation_duration: Option<f32>,
    pub time_between_each_shard: Option<f32>,
    pub ahead_distance: Option<f32>,
    pub shard_speed_multiplier: Option<f32>,
    /// Version 1 only.
    pub bank_id: Option<i32>,
}

impl Visit for ShardCluster {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Trigger Distance", &mut self.trigger_distance)?;
        v.value("Animation Duration_", &mut self.animation_duration)?;
        v.value("Time Between Each Shard", &mut self.time_between_each_shard)?;
        v.value("Ahead Distance", &mut self.ahead_distance)?;
        v.value("Shard Speed Multiplier", &mut self.shard_speed_multiplier)?;
        if version >= 1 {
            v.value("bankID_", &mut self.bank_id)?;
        }
        Ok(())
    }
}
