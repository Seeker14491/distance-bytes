use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TeleporterExitCheckpoint`: Car can exit from here when entering a Teleporter Entrance. It
/// can also trigger a checkpoint here.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct TeleporterExitCheckpoint {
    pub trigger_checkpoint: Option<bool>,
    /// Version 1 and later.
    pub silent: Option<bool>,
    /// Version 2 only.
    pub bank_id: Option<i32>,
}

impl Visit for TeleporterExitCheckpoint {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("TriggerCheckpoint", &mut self.trigger_checkpoint)?;
        }
        if version >= 1 {
            v.value("Silent", &mut self.silent)?;
        }
        if version >= 2 {
            v.value("bankID_", &mut self.bank_id)?;
        }
        Ok(())
    }
}
