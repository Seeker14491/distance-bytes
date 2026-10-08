use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `SplineSegment`: a segment of a spline track. Visited as a `TrackSegment` of version 2
/// (nothing) for version 0, and of version 3 for version 1.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SplineSegment {
    /// Version 1 and later.
    pub include_in_track_calculation: Option<bool>,
}

impl Visit for SplineSegment {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 1 {
            v.value(
                "Include In Track Calculation",
                &mut self.include_in_track_calculation,
            )?;
        }
        Ok(())
    }
}
