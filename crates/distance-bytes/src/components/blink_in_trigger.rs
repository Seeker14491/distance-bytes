use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `BlinkInTrigger`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlinkInTrigger {
    /// Version 1 only.
    pub total_time: Option<f32>,
    /// Version 1 only.
    pub blink_anim_scale: Option<f32>,
    /// Version 1 only.
    pub use_vignette: Option<bool>,
    /// Version 1 only.
    pub vignette_anim_scale: Option<f32>,
    /// Version 1 only.
    pub vignette_blur: Option<f32>,
    /// Version 1 only.
    pub vignette_blur_spread: Option<f32>,
    /// Version 1 only.
    pub vignette_blur_distance: Option<f32>,
    /// Version 1 only.
    pub vignette_intensity: Option<f32>,
    /// Version 1 only.
    pub use_motion_blur: Option<bool>,
    /// Version 1 only.
    pub motion_blur_anim_scale: Option<f32>,
    /// Version 1 only.
    pub motion_blur_amount: Option<f32>,
    /// Version 1 only.
    pub use_saturation: Option<bool>,
    /// Version 1 only.
    pub saturation_anim_scale: Option<f32>,
    /// Version 1 only.
    pub saturation_amount: Option<f32>,
    /// Version 1 only.
    pub use_noise_and_grain: Option<bool>,
    /// Version 1 only.
    pub noise_and_grain_anim_scale: Option<f32>,
    /// Version 1 only.
    pub noise_and_grain_amount: Option<f32>,
}

impl Visit for BlinkInTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version > 0 {
            v.value("totalTime_", &mut self.total_time)?;
            v.value("blinkAnimScale_", &mut self.blink_anim_scale)?;
            v.value("useVignette_", &mut self.use_vignette)?;
            v.value("vignetteAnimScale_", &mut self.vignette_anim_scale)?;
            v.value("vignetteBlur_", &mut self.vignette_blur)?;
            v.value("vignetteBlurSpread_", &mut self.vignette_blur_spread)?;
            v.value("vignetteBlurDistance_", &mut self.vignette_blur_distance)?;
            v.value("vignetteIntensity_", &mut self.vignette_intensity)?;
            v.value("useMotionBlur_", &mut self.use_motion_blur)?;
            v.value("motionBlurAnimScale_", &mut self.motion_blur_anim_scale)?;
            v.value("motionBlurAmount_", &mut self.motion_blur_amount)?;
            v.value("useSaturation_", &mut self.use_saturation)?;
            v.value("saturationAnimScale_", &mut self.saturation_anim_scale)?;
            v.value("saturationAmount_", &mut self.saturation_amount)?;
            v.value("useNoiseAndGrain_", &mut self.use_noise_and_grain)?;
            v.value("noiseAndGrainAnimScale_", &mut self.noise_and_grain_anim_scale)?;
            v.value("noiseAndGrainAmount_", &mut self.noise_and_grain_amount)?;
        }
        Ok(())
    }
}
