use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `InterpolateRTPCLogic`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InterpolateRTPCLogic {
    pub rtpc: Option<DString>,
    pub low: Option<f32>,
    pub high: Option<f32>,
}

impl Visit for InterpolateRTPCLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("RTPC", &mut self.rtpc)?;
            v.value("Low", &mut self.low)?;
            v.value("High", &mut self.high)?;
        }
        Ok(())
    }
}
