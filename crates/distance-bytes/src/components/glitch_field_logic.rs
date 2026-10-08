use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `GlitchFieldLogic`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct GlitchFieldLogic {
    /// Version 2 and later.
    pub id: Option<i32>,
    /// `GlitchFieldLogic.Type`.
    pub r#type: Option<i32>,
    pub gradual_intensity: Option<bool>,
    pub gradual_low_pass: Option<bool>,
    pub glitch_car_screen: Option<bool>,
    pub shake_camera: Option<bool>,
    pub bloom_camera: Option<bool>,
    pub one_time_use: Option<bool>,
    /// Version 3 and later.
    pub use_vhs: Option<bool>,
    /// Version 4 and later.
    pub use_random_seed: Option<bool>,
    /// Version 4 and later.
    pub seed: Option<i32>,
    /// Version 5 only.
    pub ignore_in_arcade: Option<bool>,
    pub glitch_intensity_min: Option<f32>,
    pub glitch_intensity_max: Option<f32>,
    pub car_screen_glitch_intensity: Option<f32>,
    pub car_screen_noise_intensity: Option<f32>,
    pub bloom_max: Option<f32>,
    pub bloom_decay: Option<f32>,
    pub filter_colors: Option<bool>,
    pub glitch_colors_up: Option<bool>,
    pub glitch_colors_down: Option<bool>,
    pub scale_glitch: Option<bool>,
    pub displace: Option<bool>,
    pub min_glitch_time_low: Option<f32>,
    pub max_glitch_time_low: Option<f32>,
    pub min_glitch_time_high: Option<f32>,
    pub max_glitch_time_high: Option<f32>,
    pub min_delay_time_low: Option<f32>,
    pub max_delay_time_low: Option<f32>,
    pub min_delay_time_high: Option<f32>,
    pub max_delay_time_high: Option<f32>,
}

impl Visit for GlitchFieldLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=5;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 2 {
            v.value("ID", &mut self.id)?;
        }
        if version >= 1 {
            v.value("Type", &mut self.r#type)?;
            v.value("Gradual Intensity", &mut self.gradual_intensity)?;
            v.value("Gradual Low Pass", &mut self.gradual_low_pass)?;
            v.value("Glitch Car Screen", &mut self.glitch_car_screen)?;
            v.value("Shake Camera", &mut self.shake_camera)?;
            v.value("Bloom Camera", &mut self.bloom_camera)?;
            v.value("One Time Use", &mut self.one_time_use)?;
            if version >= 3 {
                v.value("Use VHS", &mut self.use_vhs)?;
            }
            if version >= 4 {
                v.value("Use Random Seed", &mut self.use_random_seed)?;
                v.value("Seed", &mut self.seed)?;
            }
            if version >= 5 {
                v.value("Ignore in Arcade", &mut self.ignore_in_arcade)?;
            }
            v.value("Glitch Intensity Min", &mut self.glitch_intensity_min)?;
            v.value("Glitch Intensity Max", &mut self.glitch_intensity_max)?;
            v.value(
                "Car Screen Glitch Intensity",
                &mut self.car_screen_glitch_intensity,
            )?;
            v.value(
                "Car Screen Noise Intensity",
                &mut self.car_screen_noise_intensity,
            )?;
            v.value("Bloom Max", &mut self.bloom_max)?;
            v.value("Bloom Decay", &mut self.bloom_decay)?;
            v.value("Filter Colors", &mut self.filter_colors)?;
            v.value("Glitch Colors Up", &mut self.glitch_colors_up)?;
            v.value("Glitch Colors Down", &mut self.glitch_colors_down)?;
            v.value("Scale Glitch", &mut self.scale_glitch)?;
            v.value("Displace", &mut self.displace)?;
            v.value("Min Glitch Time Low_", &mut self.min_glitch_time_low)?;
            v.value("Max Glitch Time Low_", &mut self.max_glitch_time_low)?;
            v.value("Min Glitch Time High_", &mut self.min_glitch_time_high)?;
            v.value("Max Glitch Time High_", &mut self.max_glitch_time_high)?;
            v.value("Min Delay Time Low_", &mut self.min_delay_time_low)?;
            v.value("Max Delay Time Low_", &mut self.max_delay_time_low)?;
            v.value("Min Delay Time High_", &mut self.min_delay_time_high)?;
            v.value("Max Delay Time High_", &mut self.max_delay_time_high)?;
        }
        Ok(())
    }
}
