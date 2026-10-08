use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `Group`: Contains one or more individual objects.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Group {
    /// References to the linked objects.
    pub links: Vec<u32>,
    /// `Group.InspectChildrenType`. Version 1 only.
    pub inspect_children: Option<i32>,
}

impl Visit for Group {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.references("Links", &mut self.links)?;
        if version >= 1 {
            v.value("inspectChildren_", &mut self.inspect_children)?;
        }
        Ok(())
    }
}
