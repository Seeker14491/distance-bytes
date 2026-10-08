use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `Projector`: Unity's projector, through the game's `ProjectorWrapper`.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default, deny_unknown_fields)]
pub struct Projector {
    pub near_clip_plane: Option<f32>,
    pub far_clip_plane: Option<f32>,
    pub field_of_view: Option<f32>,
    pub aspect_ratio: Option<f32>,
    /// A `BoolState`. Version 0 only, ignored by the game.
    pub orthographic: Option<bool>,
    /// Version 0 only, ignored by the game.
    pub orthographic_size: Option<f32>,
    /// Version 0 only, ignored by the game.
    pub ignore_layers: Option<i32>,
    /// Version 0 only, ignored by the game.
    pub material_name: Option<DString>,
}

impl Visit for Projector {
    const VERSIONS: RangeInclusive<i32> = 0..=1;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("NearClipPlane", &mut self.near_clip_plane)?;
        v.value("FarClipPlane", &mut self.far_clip_plane)?;
        v.value("FieldOfView", &mut self.field_of_view)?;
        v.value("AspectRatio", &mut self.aspect_ratio)?;
        if version == 0 {
            v.value("Orthographic", &mut self.orthographic)?;
            v.value("OrthographicSize", &mut self.orthographic_size)?;
            v.value("IgnoreLayers", &mut self.ignore_layers)?;
            v.value("MaterialName", &mut self.material_name)?;
        }
        Ok(())
    }
}
