use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `LensFlare` (the game's `LensFlareWrapper`): simulated camera lens flares.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct LensFlare {
    /// Version 0 only, ignored by the game.
    pub flare_name: Option<DString>,
    pub brightness: Option<f32>,
    /// `[r, g, b, a]`.
    pub color: Option<[f32; 4]>,
}

impl Visit for LensFlare {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.value("FlareName", &mut self.flare_name)?;
        }
        v.value("Brightness", &mut self.brightness)?;
        v.value("Color", &mut self.color)?;
        Ok(())
    }
}
