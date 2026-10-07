use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `FadeOut`: Fades out lights and emissive colors of this object when a car comes within the
/// trigger distance.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FadeOut {
    pub one_shot: Option<bool>,
    pub trigger_distance: Option<f32>,
    pub delay_time: Option<f32>,
    pub fade_time: Option<f32>,
    /// `AnimatorBase.CurveType`.
    pub curve_type: Option<i32>,
    pub set_rgb: Option<bool>,
    pub affect_color_property: Option<bool>,
}

impl Visit for FadeOut {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("One Shot", &mut self.one_shot)?;
            v.value("TriggerDistance", &mut self.trigger_distance)?;
            v.value("Delay Time", &mut self.delay_time)?;
            v.value("Fade Time", &mut self.fade_time)?;
            v.value("curveType_", &mut self.curve_type)?;
            v.value("Set RGB", &mut self.set_rgb)?;
            v.value("AffectColorProperty", &mut self.affect_color_property)?;
        }
        Ok(())
    }
}
