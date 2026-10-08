use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `BoxCollider` (the game's `BoxColliderWrapper`): a box-shaped zone which detects when the
/// player's car enters or leaves it.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct BoxCollider {
    pub center: Option<[f32; 3]>,
    /// Version 0 only, ignored by the game.
    pub extents: Option<[f32; 3]>,
    pub size: Option<[f32; 3]>,
    /// Version 1 and earlier, ignored by the game.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub collider: SerialCollider,
}

impl Visit for BoxCollider {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Center", &mut self.center)?;
        if version == 0 {
            v.value("Extents", &mut self.extents)?;
        }
        v.value("Size", &mut self.size)?;
        if version < 2 {
            self.collider.visit(v)?;
        }
        Ok(())
    }
}

/// `SphereCollider` (the game's `SphereColliderWrapper`): a spherical zone which detects when
/// the player's car enters or leaves it.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct SphereCollider {
    pub center: Option<[f32; 3]>,
    pub radius: Option<f32>,
    /// Version 0 only, ignored by the game.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub base_collider: SerialCollider,
}

impl Visit for SphereCollider {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Center", &mut self.center)?;
        v.value("Radius", &mut self.radius)?;
        if version < 1 {
            self.base_collider.visit(v)?;
        }
        Ok(())
    }
}

/// `CapsuleCollider` (the game's `CapsuleColliderWrapper`): a capsule zone which detects when
/// the player's car enters or leaves it.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct CapsuleCollider {
    pub center: Option<[f32; 3]>,
    pub radius: Option<f32>,
    pub height: Option<f32>,
    /// The axis: 0 X, 1 Y, 2 Z.
    pub direction: Option<i32>,
    /// Version 0 only, ignored by the game.
    #[serde(skip_serializing_if = "crate::json::is_default")]
    pub base_collider: SerialCollider,
}

impl Visit for CapsuleCollider {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Center", &mut self.center)?;
        v.value("Radius", &mut self.radius)?;
        v.value("Height", &mut self.height)?;
        v.value("Direction", &mut self.direction)?;
        if version < 1 {
            self.base_collider.visit(v)?;
        }
        Ok(())
    }
}

/// `SerialCollider`: collider settings old versions of the collider components stored.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct SerialCollider {
    /// A `BoolState`.
    pub is_trigger: Option<bool>,
    pub physic_material_name: Option<DString>,
}

impl SerialCollider {
    fn visit(&mut self, v: &mut impl Visitor) -> Result<()> {
        v.value("IsTrigger", &mut self.is_trigger)?;
        v.value("PhysicMaterialName", &mut self.physic_material_name)
    }
}
