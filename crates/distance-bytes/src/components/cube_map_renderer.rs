use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `CubeMapRenderer`, the game's `LevelCubeMapRenderer`: captures the level from its position
/// for nearby reflective objects.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CubeMapRenderer {
    pub previous_level_cubemap: Option<bool>,
}

impl Visit for CubeMapRenderer {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, _version: i32) -> Result<()> {
        v.value("PreviousLevelCubemap", &mut self.previous_level_cubemap)
    }
}
