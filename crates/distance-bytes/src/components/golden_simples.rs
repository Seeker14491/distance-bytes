use std::ops::RangeInclusive;

use crate::error::Result;
use crate::visit::{Visit, Visitor};

/// `GoldenSimples`: the material settings of the simple shapes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GoldenSimples {
    pub image_index: Option<i32>,
    /// Version 1 and later. In versions 1 to 3, indexes past that version's texture count are
    /// shifted by the game to make room for textures added later.
    pub emit_index: Option<i32>,
    /// `GoldenSimples.Presets`. Version 1 and later.
    pub preset: Option<i32>,
    pub texture_scale: Option<[f32; 3]>,
    pub texture_offset: Option<[f32; 3]>,
    pub flip_texture_uv: Option<bool>,
    pub world_mapped: Option<bool>,
    pub disable_diffuse: Option<bool>,
    pub disable_bump: Option<bool>,
    /// Version 3 and later.
    pub bump_strength: Option<f32>,
    pub disable_reflect: Option<bool>,
    /// Version 1 and later.
    pub disable_collision: Option<bool>,
    /// Version 1 and later.
    pub additive_transparency: Option<bool>,
    /// Version 2 and later.
    pub multiplicative_transparency: Option<bool>,
    /// Version 2 and later.
    pub invert_emit: Option<bool>,
}

impl Visit for GoldenSimples {
    const VERSIONS: RangeInclusive<i32> = 0..=4;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("ImageIndex", &mut self.image_index)?;
        if version >= 1 {
            v.value("EmitIndex", &mut self.emit_index)?;
            v.value("Preset", &mut self.preset)?;
        }
        v.value("textureScale", &mut self.texture_scale)?;
        v.value("textureOffset", &mut self.texture_offset)?;
        v.value("flipTextureUV", &mut self.flip_texture_uv)?;
        v.value("worldMapped", &mut self.world_mapped)?;
        v.value("disableDiffuse", &mut self.disable_diffuse)?;
        v.value("disableBump", &mut self.disable_bump)?;
        if version >= 3 {
            v.value("bumpStrength", &mut self.bump_strength)?;
        }
        v.value("disableReflect", &mut self.disable_reflect)?;
        if version >= 1 {
            v.value("disableCollision", &mut self.disable_collision)?;
            v.value("additiveTransparency", &mut self.additive_transparency)?;
        }
        if version >= 2 {
            v.value("multiplicativeTransparency", &mut self.multiplicative_transparency)?;
            v.value("invertEmit", &mut self.invert_emit)?;
        }
        Ok(())
    }
}
