use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `ZEventListener`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct ZEventListener {
    /// Version 0 only.
    pub event_id: Option<i32>,
    /// Version 1 and later.
    pub event_name: Option<DString>,
    /// Version 2 only.
    pub delay: Option<f32>,
}

impl Visit for ZEventListener {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.value("Event ID", &mut self.event_id)?;
        } else if version >= 1 {
            v.value("eventName_", &mut self.event_name)?;
            if version >= 2 {
                v.value("delay_", &mut self.delay)?;
            }
        }
        Ok(())
    }
}
