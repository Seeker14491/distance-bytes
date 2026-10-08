use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `AnimatorCameraShake`: Shake the camera during certain events of an Animator.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct AnimatorCameraShake {
    pub play_intensity: Option<f32>,
    pub stop_intensity: Option<f32>,
    pub pong_play_intensity: Option<f32>,
    pub pong_stop_intensity: Option<f32>,
    pub effective_near: Option<f32>,
    pub falloff_distance: Option<f32>,
    /// Version 1 only.
    pub center_point: Option<[f32; 3]>,
}

impl Visit for AnimatorCameraShake {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("Play Intensity", &mut self.play_intensity)?;
            v.value("Stop Intensity", &mut self.stop_intensity)?;
            v.value("Pong Play Intensity", &mut self.pong_play_intensity)?;
            v.value("Pong Stop Intensity", &mut self.pong_stop_intensity)?;
            v.value("Effective Near", &mut self.effective_near)?;
            v.value("Falloff Distance", &mut self.falloff_distance)?;
        }
        if version >= 1 {
            v.value("Center Point", &mut self.center_point)?;
        }
        Ok(())
    }
}
