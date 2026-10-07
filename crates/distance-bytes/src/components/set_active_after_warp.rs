use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `SetActiveAfterWarp`: Set this object active or inactive after a warp segment has completed.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetActiveAfterWarp {
    /// Named `Index` in version 0 only.
    pub warp_anchor_id: Option<i32>,
    /// Version 0 only.
    pub set_active: Option<bool>,
    /// `SetActiveAfterWarp.ActiveState`. Version 1 only.
    pub state_after_warp: Option<i32>,
    /// `SetActiveAfterWarp.WarpEvent`. Version 1 only.
    pub warp_anchor_event: Option<i32>,
}

impl Visit for SetActiveAfterWarp {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.value("Index", &mut self.warp_anchor_id)?;
            v.value("SetActive", &mut self.set_active)?;
        } else if version >= 1 {
            v.value("Warp Anchor ID", &mut self.warp_anchor_id)?;
            v.value("State After Warp", &mut self.state_after_warp)?;
            v.value("Warp Anchor Event", &mut self.warp_anchor_event)?;
        }
        Ok(())
    }
}
