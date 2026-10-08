use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `PowerPosterLogic`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct PowerPosterLogic {
    /// `PowerPosterLogic.PowerPosterDisplayOptions`.
    pub display_options: Option<i32>,
}

impl Visit for PowerPosterLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.value("displayOptions_", &mut self.display_options)?;
        }
        Ok(())
    }
}
