use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `PopupBlockerLogic`: Pops up out of the track in front of the car.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PopupBlockerLogic {
    pub holo_distance: Option<f32>,
    pub holo_color_speed: Option<f32>,
    /// `[r, g, b, a]`.
    pub holo_color: Option<[f32; 4]>,
    pub holo_brightness: Option<f32>,
    pub holo_light_intensity: Option<f32>,
    pub holo_light_range: Option<f32>,
    pub pop_up_distance: Option<f32>,
    pub pop_up_speed: Option<f32>,
    /// Version 2 only.
    pub blocker_height_multiplier: Option<f32>,
}

impl Visit for PopupBlockerLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("HoloDistance", &mut self.holo_distance)?;
            v.value("HoloColorSpeed", &mut self.holo_color_speed)?;
            v.value("HoloColor", &mut self.holo_color)?;
            v.value("HoloBrightness", &mut self.holo_brightness)?;
            v.value("HoloLightIntensity", &mut self.holo_light_intensity)?;
            v.value("HoloLightRange", &mut self.holo_light_range)?;
            v.value("PopUpDistance", &mut self.pop_up_distance)?;
            v.value("PopUpSpeed", &mut self.pop_up_speed)?;
        }
        if version >= 2 {
            v.value(
                "BlockerHeightMultiplier",
                &mut self.blocker_height_multiplier,
            )?;
        }
        Ok(())
    }
}
