use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `IgnoreInputTrigger`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IgnoreInputTrigger {
    pub ignore_input: Option<bool>,
}

impl Visit for IgnoreInputTrigger {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
            v.value("ignoreInput_", &mut self.ignore_input)?;
        }
        Ok(())
    }
}
