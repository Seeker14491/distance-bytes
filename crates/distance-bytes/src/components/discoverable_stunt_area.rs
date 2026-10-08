use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `DiscoverableStuntArea`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DiscoverableStuntArea {
    pub points: Option<i32>,
    pub area_name: Option<DString>,
}

impl Visit for DiscoverableStuntArea {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else if version >= 1 {
            v.value("Points", &mut self.points)?;
            v.value("Area Name", &mut self.area_name)?;
        }
        Ok(())
    }
}
