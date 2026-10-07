use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `TrackAttachment`: This object is attached to a track, and will move along with it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TrackAttachment {
    /// A reference to an object.
    pub attached_to: u32,
    pub copy_rotation: Option<bool>,
    pub copy_scale: Option<bool>,
    pub subsegment_index: Option<i32>,
}

impl Visit for TrackAttachment {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.reference("Attached To", &mut self.attached_to)?;
            v.value("Copy Rotation", &mut self.copy_rotation)?;
            v.value("Copy Scale", &mut self.copy_scale)?;
            if true {
                v.value("Subsegment Index", &mut self.subsegment_index)?;
            }
        }
        Ok(())
    }
}
