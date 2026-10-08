use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `CheckpointLogic`: Restores car damage / heat, and sets new respawn point when entering.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CheckpointLogic {
    /// `[r, g, b, a]`.
    pub start_color: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub finish_color: Option<[f32; 4]>,
    /// Version 2 and later.
    pub silent: Option<bool>,
    /// Version 3 only.
    pub bank_id: Option<i32>,
}

impl Visit for CheckpointLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 1 {
            v.value("StartColor", &mut self.start_color)?;
            v.value("FinishColor", &mut self.finish_color)?;
        }
        if version >= 2 {
            v.value("Silent", &mut self.silent)?;
        }
        if version >= 3 {
            v.value("bankID_", &mut self.bank_id)?;
        }
        Ok(())
    }
}
