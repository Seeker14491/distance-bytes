use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `GravityToggle`.
///
/// Version 0 uses the generic field format.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GravityToggle {
    pub disable_gravity: Option<bool>,
    pub drag_scale: Option<f32>,
    pub angular_drag_scale: Option<f32>,
}

impl Visit for GravityToggle {
    const VERSIONS: RangeInclusive<i32> = 1..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        if version == 0 {
        } else if version > 0 {
            v.value("DisableGravity", &mut self.disable_gravity)?;
            v.value("DragScale", &mut self.drag_scale)?;
            v.value("AngularDragScale", &mut self.angular_drag_scale)?;
        }
        Ok(())
    }
}
