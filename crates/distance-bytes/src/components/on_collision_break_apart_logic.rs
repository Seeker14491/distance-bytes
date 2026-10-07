use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `OnCollisionBreakApartLogic`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OnCollisionBreakApartLogic {
    pub bank_id: Option<i32>,
}

impl Visit for OnCollisionBreakApartLogic {
    const VERSIONS: RangeInclusive<i32> = 0..=0;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version >= 0 {
            v.value("bankID_", &mut self.bank_id)?;
        }
        Ok(())
    }
}
