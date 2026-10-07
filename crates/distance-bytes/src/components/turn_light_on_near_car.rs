use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TurnLightOnNearCar`: Turns on lights and emissive colors of this object when a car comes within
/// the trigger distance.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TurnLightOnNearCar {
    pub trigger_distance: Option<f32>,
    /// Version 5 and earlier.
    pub add_delay: Option<bool>,
    pub delay_range_low: Option<f32>,
    pub delay_range_high: Option<f32>,
    pub set_rgb: Option<bool>,
    /// Version 2 and later.
    pub affect_color_property: Option<bool>,
    /// Version 6 only.
    pub use_random_delay: Option<bool>,
    /// Version 6 only.
    pub delay_time: Option<f32>,
    /// Version 4 and later.
    pub play_sound: Option<bool>,
    /// Version 5 and later.
    pub turn_off_on_trigger_exit: Option<bool>,
}

impl Visit for TurnLightOnNearCar {
    const VERSIONS: RangeInclusive<i32> = 1..=6;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if (1..3).contains(&version) {
            v.value("TriggerDistance", &mut self.trigger_distance)?;
            v.value("AddDelay", &mut self.add_delay)?;
            v.value("DelayRangeLow", &mut self.delay_range_low)?;
            v.value("DelayRangeHigh", &mut self.delay_range_high)?;
            v.value("SetRGB", &mut self.set_rgb)?;
        }
        if (2..3).contains(&version) {
            v.value("AffectColorProperty", &mut self.affect_color_property)?;
        }
        if version >= 3 {
            v.value("TriggerDistance", &mut self.trigger_distance)?;
            if version < 6 {
                v.value("AddDelay", &mut self.add_delay)?;
                v.value("DelayRangeLow", &mut self.delay_range_low)?;
                v.value("DelayRangeHigh", &mut self.delay_range_high)?;
            } else {
                v.value("Use Random Delay", &mut self.use_random_delay)?;
                v.value("Delay Time", &mut self.delay_time)?;
                v.value("DelayRangeLow", &mut self.delay_range_low)?;
                v.value("DelayRangeHigh", &mut self.delay_range_high)?;
            }
            v.value("SetRGB", &mut self.set_rgb)?;
            v.value("AffectColorProperty", &mut self.affect_color_property)?;
        }
        if version >= 4 {
            v.value("Play Sound", &mut self.play_sound)?;
        }
        if version >= 5 {
            v.value(
                "Turn off on Trigger Exit",
                &mut self.turn_off_on_trigger_exit,
            )?;
        }
        Ok(())
    }
}
