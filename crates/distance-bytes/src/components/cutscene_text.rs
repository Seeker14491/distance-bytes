use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `CutsceneText`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CutsceneText {
    /// `CutsceneText.Type`.
    pub r#type: Option<i32>,
    pub text: Option<DString>,
    pub typing_speed: Option<f32>,
    pub start_time: Option<f32>,
    pub font_size: Option<i32>,
    /// `[r, g, b, a]`.
    pub font_color: Option<[f32; 4]>,
    /// `UILabel.Effect`.
    pub effect_style: Option<i32>,
    pub effect_distance: Option<[f32; 2]>,
}

impl Visit for CutsceneText {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("Type", &mut self.r#type)?;
            v.value("Text", &mut self.text)?;
            v.value("Typing Speed", &mut self.typing_speed)?;
            v.value("StartTime", &mut self.start_time)?;
            v.value("Font Size", &mut self.font_size)?;
            v.value("Font Color", &mut self.font_color)?;
            v.value("Effect Style", &mut self.effect_style)?;
            v.value("Effect Distance", &mut self.effect_distance)?;
        }
        Ok(())
    }
}
