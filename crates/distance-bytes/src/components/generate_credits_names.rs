use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `GenerateCreditsNames`, the game's `GenerateCreditsNameOrbs`: places the orbs with the
/// backers' names in the credits.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct GenerateCreditsNames {
    pub cast_from_radius: Option<f32>,
    /// Version 2 and later.
    pub min_radius: Option<f32>,
    /// Version 2 and later.
    pub max_radius: Option<f32>,
    pub name_pair_offset: Option<f32>,
    /// Version 2 and later.
    pub collide_iterations: Option<i32>,
    /// Version 3 and later.
    pub random_seed: Option<i32>,
    /// Version 3 and later.
    pub obstacle_ray_count: Option<i32>,
    /// Version 3 and later.
    pub obstacle_angle: Option<f32>,
    /// The level editor's search for orbs by name.
    pub search: Option<DString>,
}

impl Visit for GenerateCreditsNames {
    const VERSIONS: RangeInclusive<i32> = 1..=3;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("CastFromRadius", &mut self.cast_from_radius)?;
        if version >= 2 {
            v.value("minRadius_", &mut self.min_radius)?;
            v.value("maxRadius_", &mut self.max_radius)?;
        }
        v.value("NamePairOffset", &mut self.name_pair_offset)?;
        if version >= 2 {
            v.value("collideIterations_", &mut self.collide_iterations)?;
        }
        if version >= 3 {
            v.value("randomSeed_", &mut self.random_seed)?;
            v.value("obstacleRayCount_", &mut self.obstacle_ray_count)?;
            v.value("obstacleAngle_", &mut self.obstacle_angle)?;
        }
        v.value("Search", &mut self.search)
    }
}
