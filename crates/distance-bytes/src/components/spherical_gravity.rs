use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `SphericalGravity`.
///
/// Version 0 uses the generic field format.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SphericalGravity {
    pub gravity: Option<f32>,
    pub exclusive_spherical_gravity: Option<bool>,
}

impl Visit for SphericalGravity {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else {
            v.value("gravity_", &mut self.gravity)?;
            v.value("exclusiveSphericalGravity_", &mut self.exclusive_spherical_gravity)?;
        }
        Ok(())
    }
}
