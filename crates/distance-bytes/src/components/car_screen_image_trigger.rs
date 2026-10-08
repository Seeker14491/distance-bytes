use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `CarScreenImageTrigger`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CarScreenImageTrigger {
    /// `CarScreenImageTrigger.CarScreenSpecialObject`.
    pub image: Option<i32>,
    /// Version 1 only.
    pub show_when_inside_trigger: Option<bool>,
    pub time_visible: Option<f32>,
    pub delay: Option<f32>,
    pub one_shot: Option<bool>,
}

impl Visit for CarScreenImageTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("Image", &mut self.image)?;
            if version >= 1 {
                v.value(
                    "Show When Inside Trigger",
                    &mut self.show_when_inside_trigger,
                )?;
            }
            v.value("Time Visible", &mut self.time_visible)?;
            v.value("Delay", &mut self.delay)?;
            v.value("One Shot", &mut self.one_shot)?;
        }
        Ok(())
    }
}
