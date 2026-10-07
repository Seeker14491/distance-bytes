use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TrackSegment`: This object can connect to other Track segments.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackSegment {
    /// References to the linked objects. Version 0 only.
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
