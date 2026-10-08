use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `SetAbilitiesTrigger`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct SetAbilitiesTrigger {
    pub enable_flying: Option<bool>,
    pub enable_jumping: Option<bool>,
    pub enable_boosting: Option<bool>,
    pub enable_jet_rotating: Option<bool>,
    /// Version 4 and later.
    pub infinite_cooldown: Option<bool>,
    /// Version 3 and later.
    pub delay: Option<f32>,
    pub show_ability_alert: Option<bool>,
    pub bloom_out: Option<bool>,
    pub play_sound: Option<bool>,
    /// Version 1 and later.
    pub show_car_screen_image: Option<bool>,
    /// Version 2 and later.
    pub timer_text: Option<DString>,
    /// Version 5 and later.
    pub ignore_in_arcade: Option<bool>,
    /// Version 6 and later.
    pub use_slow_mo: Option<bool>,
    /// Version 6 and later.
    pub delay_before_slow_mo: Option<f32>,
    /// Version 6 and later.
    pub slow_mo_time_scale: Option<f32>,
    /// Version 6 and later.
    pub slow_mo_duration: Option<f32>,
    /// Version 6 and later.
    pub glitch_duration_after: Option<f32>,
    /// Version 6 and later.
    pub play_slow_mo_audio: Option<bool>,
    /// Version 7 and later.
    pub visuals_only: Option<bool>,
    /// Version 8 only.
    pub one_shot: Option<bool>,
    /// Version 8 only.
    pub restore_abilities_on_exit: Option<bool>,
    /// Version 8 only.
    pub restore_abilities_on_death: Option<bool>,
    /// Version 8 only.
    pub delay_before_restore: Option<f32>,
}

impl Visit for SetAbilitiesTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=8;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("EnableFlying", &mut self.enable_flying)?;
            v.value("EnableJumping", &mut self.enable_jumping)?;
            v.value("EnableBoosting", &mut self.enable_boosting)?;
            v.value("EnableJetRotating", &mut self.enable_jet_rotating)?;
            if version >= 4 {
                v.value("InfiniteCooldown", &mut self.infinite_cooldown)?;
            }
            if version >= 3 {
                v.value("Delay", &mut self.delay)?;
            }
            v.value("ShowAbilityAlert", &mut self.show_ability_alert)?;
            v.value("BloomOut", &mut self.bloom_out)?;
            v.value("PlaySound", &mut self.play_sound)?;
        }
        if version >= 1 {
            v.value("ShowCarScreenImage", &mut self.show_car_screen_image)?;
        }
        if version >= 2 {
            v.value("Timer Text", &mut self.timer_text)?;
        }
        if version >= 5 {
            v.value("ignoreInArcade_", &mut self.ignore_in_arcade)?;
        }
        if version >= 6 {
            v.value("useSlowMo_", &mut self.use_slow_mo)?;
            v.value("delayBeforeSlowMo_", &mut self.delay_before_slow_mo)?;
            v.value("slowMoTimeScale_", &mut self.slow_mo_time_scale)?;
            v.value("slowMoDuration_", &mut self.slow_mo_duration)?;
            v.value("glitchDurationAfter_", &mut self.glitch_duration_after)?;
            v.value("playSlowMoAudio_", &mut self.play_slow_mo_audio)?;
        }
        if version >= 7 {
            v.value("visualsOnly_", &mut self.visuals_only)?;
        }
        if version >= 8 {
            v.value("oneShot_", &mut self.one_shot)?;
            v.value(
                "restoreAbilitiesOnExit_",
                &mut self.restore_abilities_on_exit,
            )?;
            v.value(
                "restoreAbilitiesOnDeath_",
                &mut self.restore_abilities_on_death,
            )?;
            v.value("delayBeforeRestore_", &mut self.delay_before_restore)?;
        }
        Ok(())
    }
}
