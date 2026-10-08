use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `AnimatorAudio`: play an audio event (sound) during certain events of an Animator.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AnimatorAudio {
    pub play: Option<DString>,
    pub stop: Option<DString>,
    pub pong_play: Option<DString>,
    pub pong_stop: Option<DString>,
    /// Version 1 only.
    pub center_point: Option<[f32; 3]>,
}

impl Visit for AnimatorAudio {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("Play", &mut self.play)?;
            v.value("Stop", &mut self.stop)?;
            v.value("Pong Play", &mut self.pong_play)?;
            v.value("Pong Stop", &mut self.pong_stop)?;
        }
        if version >= 1 {
            v.value("Center Point", &mut self.center_point)?;
        }
        Ok(())
    }
}
