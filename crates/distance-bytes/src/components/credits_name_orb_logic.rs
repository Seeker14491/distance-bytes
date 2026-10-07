use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `CreditsNameOrbLogic`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CreditsNameOrbLogic {
    pub name: Option<DString>,
    pub key: Option<DString>,
    pub linked_key: Option<DString>,
}

impl Visit for CreditsNameOrbLogic {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else {
            v.value("name_", &mut self.name)?;
            if true {
                v.value("key_", &mut self.key)?;
                v.value("linkedKey_", &mut self.linked_key)?;
            }
        }
        Ok(())
    }
}
