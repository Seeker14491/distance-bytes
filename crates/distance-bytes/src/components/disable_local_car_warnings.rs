use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `DisableLocalCarWarnings`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisableLocalCarWarnings {
    pub disable_press_back_to_reset: Option<bool>,
    pub disable_wrong_way: Option<bool>,
    pub disable_car_track_arrow: Option<bool>,
}

impl Visit for DisableLocalCarWarnings {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value(
                "disablePressBackToReset_",
                &mut self.disable_press_back_to_reset,
            )?;
            v.value("disableWrongWay_", &mut self.disable_wrong_way)?;
            v.value("disableCarTrackArrow_", &mut self.disable_car_track_arrow)?;
        }
        Ok(())
    }
}
