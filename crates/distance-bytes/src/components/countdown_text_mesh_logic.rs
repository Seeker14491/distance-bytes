use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `CountdownTextMeshLogic`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CountdownTextMeshLogic {
    /// Version 1 and later.
    pub countdown_time: Option<f32>,
    /// Version 1 and later.
    pub warp_id: Option<i32>,
    /// `WarpAnchorEvent.State`. Version 1 and later.
    pub warp_state: Option<i32>,
    /// Version 2 only.
    pub use_mode_time: Option<bool>,
}

impl Visit for CountdownTextMeshLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("CountdownTime", &mut self.countdown_time)?;
            v.value("WarpID", &mut self.warp_id)?;
            v.value("WarpState", &mut self.warp_state)?;
        }
        if version >= 2 {
            v.value("UseModeTime", &mut self.use_mode_time)?;
        }
        Ok(())
    }
}
