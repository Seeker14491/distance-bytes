use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TrackSegment`: This object can connect to other Track segments.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct TrackSegment {
    /// References to the linked objects. Version 0 only.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub links: Vec<u32>,
    /// Version 1 and earlier.
    pub update_all_links: Option<bool>,
    /// Version 3 only.
    pub include_in_track_calculation: Option<bool>,
}

impl Visit for TrackSegment {
    const VERSIONS: RangeInclusive<i32> = 0..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        match version {
            0 => {
                v.references("Links", &mut self.links)?;
                v.value("UpdateAllLinks", &mut self.update_all_links)?;
            }
            1 => {
                v.value("UpdateAllLinks", &mut self.update_all_links)?;
            }
            3 => {
                v.value(
                    "Include In Track Calculation",
                    &mut self.include_in_track_calculation,
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
