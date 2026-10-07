use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `LostToEchoesIntroCutscene`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LostToEchoesIntroCutscene {
    pub delay_before_fade_in: Option<f32>,
    pub fade_in_time: Option<f32>,
    /// Version 2 only.
    pub fade_out_time: Option<f32>,
    pub shake_start_time: Option<f32>,
    pub shake_end_time: Option<f32>,
    pub shake_amount: Option<f32>,
    pub radial_blur_strength: Option<f32>,
    pub aspect_ratio_21_9: Option<bool>,
    /// `[r, g, b, a]`.
    pub final_core_color: Option<[f32; 4]>,
    /// Version 1 and later.
    pub time_to_switch_to_next_level: Option<f32>,
}

impl Visit for LostToEchoesIntroCutscene {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("Delay Before Fade In", &mut self.delay_before_fade_in)?;
            v.value("Fade In Time", &mut self.fade_in_time)?;
            if version >= 2 {
                v.value("Fade Out Time", &mut self.fade_out_time)?;
            }
            v.value("Shake Start Time", &mut self.shake_start_time)?;
            v.value("Shake End Time", &mut self.shake_end_time)?;
            v.value("Shake Amount", &mut self.shake_amount)?;
            v.value("Radial Blur Strength_", &mut self.radial_blur_strength)?;
            v.value("21:9 Aspect Ratio", &mut self.aspect_ratio_21_9)?;
            v.value("Final Core Color", &mut self.final_core_color)?;
        }
        if version >= 1 {
            v.value("TimeToSwitchToNextLevel", &mut self.time_to_switch_to_next_level)?;
        }
        Ok(())
    }
}
