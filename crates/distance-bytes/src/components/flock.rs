use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `Flock`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Flock {
    pub count: Option<i32>,
    pub cohesion_scalar: Option<f32>,
    pub alignment_scalar: Option<f32>,
    pub seperation_scalar: Option<f32>,
    pub max_neighbor_distance: Option<f32>,
    pub desired_seperation_distance: Option<f32>,
    pub max_speed: Option<f32>,
    pub max_acceleration: Option<f32>,
    /// `[r, g, b, a]`.
    pub color: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub spec_color: Option<[f32; 4]>,
    /// `[r, g, b, a]`.
    pub emit_color: Option<[f32; 4]>,
}

impl Visit for Flock {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else if version >= 1 {
            v.value("Count", &mut self.count)?;
            v.value("CohesionScalar", &mut self.cohesion_scalar)?;
            v.value("AlignmentScalar", &mut self.alignment_scalar)?;
            v.value("SeperationScalar", &mut self.seperation_scalar)?;
            v.value("MaxNeighborDistance", &mut self.max_neighbor_distance)?;
            v.value(
                "DesiredSeperationDistance",
                &mut self.desired_seperation_distance,
            )?;
            v.value("MaxSpeed", &mut self.max_speed)?;
            v.value("MaxAcceleration", &mut self.max_acceleration)?;
            v.value("Color", &mut self.color)?;
            v.value("SpecColor", &mut self.spec_color)?;
            v.value("EmitColor", &mut self.emit_color)?;
        }
        Ok(())
    }
}
