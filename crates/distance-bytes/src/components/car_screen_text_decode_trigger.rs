use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `CarScreenTextDecodeTrigger`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CarScreenTextDecodeTrigger {
    pub text: Option<DString>,
    pub per_char_speed: Option<f32>,
    pub clear_on_finish: Option<bool>,
    pub clear_on_trigger_exit: Option<bool>,
    pub destroy_on_trigger_exit: Option<bool>,
    pub time_text: Option<DString>,
    pub static_time_text: Option<bool>,
    pub delay: Option<f32>,
    /// Version 3 and earlier.
    pub announcer_action: Option<i32>,
    /// Version 2 and later.
    pub font_size: Option<i32>,
    /// Version 3 and later.
    pub play_click_sound: Option<bool>,
    /// Version 5 only.
    pub ignore_in_arcade: Option<bool>,
}

impl Visit for CarScreenTextDecodeTrigger {
    const VERSIONS: RangeInclusive<i32> = 1..=5;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version < 4 {
            if version == 0 {
            } else if version > 0 {
                v.value("Text", &mut self.text)?;
                v.value("PerCharSpeed", &mut self.per_char_speed)?;
                v.value("ClearOnFinish", &mut self.clear_on_finish)?;
                v.value("ClearOnTriggerExit", &mut self.clear_on_trigger_exit)?;
                v.value("DestroyOnTriggerExit", &mut self.destroy_on_trigger_exit)?;
                v.value("TimeText", &mut self.time_text)?;
                v.value("StaticTimeText", &mut self.static_time_text)?;
                v.value("Delay", &mut self.delay)?;
                v.value("AnnouncerAction", &mut self.announcer_action)?;
            }
            if version >= 2 {
                v.value("FontSize", &mut self.font_size)?;
            }
            if version >= 3 {
                v.value("PlayClickSound", &mut self.play_click_sound)?;
            }
        }
        if version >= 4 {
            v.value("Text", &mut self.text)?;
            v.value("PerCharSpeed", &mut self.per_char_speed)?;
            v.value("ClearOnFinish", &mut self.clear_on_finish)?;
            v.value("ClearOnTriggerExit", &mut self.clear_on_trigger_exit)?;
            v.value("DestroyOnTriggerExit", &mut self.destroy_on_trigger_exit)?;
            v.value("TimeText", &mut self.time_text)?;
            v.value("StaticTimeText", &mut self.static_time_text)?;
            v.value("Delay", &mut self.delay)?;
            v.value("FontSize", &mut self.font_size)?;
            v.value("PlayClickSound", &mut self.play_click_sound)?;
        }
        if version >= 5 {
            v.value("Ignore in Arcade", &mut self.ignore_in_arcade)?;
        }
        Ok(())
    }
}
