use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `InfoDisplayLogic`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InfoDisplayLogic {
    pub fade_out_time: Option<f32>,
    pub delay0: Option<f32>,
    pub info_text0: Option<DString>,
    pub delay1: Option<f32>,
    pub info_text1: Option<DString>,
    pub delay2: Option<f32>,
    pub info_text2: Option<DString>,
    pub delay3: Option<f32>,
    pub info_text3: Option<DString>,
    pub delay4: Option<f32>,
    pub info_text4: Option<DString>,
    pub random_char_count: Option<i32>,
    pub per_char_speed: Option<f32>,
    pub destroy_on_trigger_exit: Option<bool>,
    /// Version 2 and later.
    pub display_in_arcade: Option<bool>,
    /// Version 3 only.
    pub to_upper: Option<bool>,
}

impl Visit for InfoDisplayLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("FadeOutTime", &mut self.fade_out_time)?;
            v.value("Delay0", &mut self.delay0)?;
            v.value("InfoText0", &mut self.info_text0)?;
            v.value("Delay1", &mut self.delay1)?;
            v.value("InfoText1", &mut self.info_text1)?;
            v.value("Delay2", &mut self.delay2)?;
            v.value("InfoText2", &mut self.info_text2)?;
            v.value("Delay3", &mut self.delay3)?;
            v.value("InfoText3", &mut self.info_text3)?;
            v.value("Delay4", &mut self.delay4)?;
            v.value("InfoText4", &mut self.info_text4)?;
            v.value("RandomCharCount", &mut self.random_char_count)?;
            v.value("PerCharSpeed", &mut self.per_char_speed)?;
            v.value("DestroyOnTriggerExit", &mut self.destroy_on_trigger_exit)?;
        }
        if version >= 2 {
            v.value("DisplayInArcade", &mut self.display_in_arcade)?;
        }
        if version < 3 {
        } else {
            v.value("toUpper_", &mut self.to_upper)?;
        }
        Ok(())
    }
}
