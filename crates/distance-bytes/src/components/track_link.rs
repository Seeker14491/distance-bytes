use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TrackLink`: one end of a track segment, which can connect to another segment's end.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct TrackLink {
    /// A reference to the segment this link belongs to. Version 1 and later.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub seg_ref: u32,
    /// A reference to the link this one connects to. Version 1 and later.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub link_ref: u32,
    /// A reference to the track node manipulating this link. Version 1 only.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub track_node_ref: u32,
    /// The GUID of the track node manipulating this link. Version 2 and later.
    pub manipulator_node_id: Option<u32>,
    /// Version 2 and later.
    pub manipulator_node_should_be_created: Option<bool>,
}

impl Visit for TrackLink {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        match version {
            1 => {
                v.reference("SegRef", &mut self.seg_ref)?;
                v.reference("LinkRef", &mut self.link_ref)?;
                v.reference("TrackNodeRef", &mut self.track_node_ref)?;
            }
            2 => {
                v.reference("SegRef", &mut self.seg_ref)?;
                v.reference("LinkRef", &mut self.link_ref)?;
                v.value("ManipulatorNodeID", &mut self.manipulator_node_id)?;
                v.value(
                    "ManipulatorNodeShouldBeCreated",
                    &mut self.manipulator_node_should_be_created,
                )?;
            }
            _ => {}
        }
        Ok(())
    }
}
