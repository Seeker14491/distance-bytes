use crate::internal::{Serializable, Vector3, Visitor, ONES_VECTOR_3, ZEROS_VECTOR_3};
use anyhow::Error;
use enum_primitive_derive::Primitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GoldenSimples {
    pub image_index: i32,
    pub emit_index: i32,
    pub preset: GoldenSimplesPresets,
    pub texture_scale: Vector3,
    pub texture_offset: Vector3,
    pub flip_texture_uv: bool,
    pub world_mapped: bool,
    pub disable_diffuse: bool,
    pub disable_bump: bool,
    pub bump_strength: f32,
    pub disable_reflect: bool,
    pub disable_collision: bool,
    pub additive_transparency: bool,
    pub multiplicative_transparency: bool,
    pub invert_emit: bool,
}

impl Default for GoldenSimples {
    fn default() -> Self {
        GoldenSimples {
            image_index: 0,
            emit_index: 0,
            preset: Default::default(),
            texture_scale: ONES_VECTOR_3,
            texture_offset: ZEROS_VECTOR_3,
            flip_texture_uv: false,
            world_mapped: false,
            disable_diffuse: false,
            disable_bump: false,
            bump_strength: 1.0,
            disable_reflect: true,
            disable_collision: false,
            additive_transparency: false,
            multiplicative_transparency: false,
            invert_emit: false,
        }
    }
}

impl Serializable for GoldenSimples {
    const VERSION: i32 = 4;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<(), Error> {
        const TEXTURE_COUNT: i32 = 72;
        const TEXTURE_COUNT_VERSION_1: i32 = 35;
        const TEXTURE_COUNT_VERSION_2: i32 = 46;
        const TEXTURE_COUNT_VERSION_3: i32 = 55;

        visitor.visit_i32("ImageIndex", &mut self.image_index)?;

        match version {
            n if n < 1 => {
                self.emit_index = self.image_index;
            }
            1 => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
                if self.image_index >= TEXTURE_COUNT_VERSION_1 {
                    self.image_index += TEXTURE_COUNT - TEXTURE_COUNT_VERSION_1;
                }
            }
            2 => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
                if self.image_index >= TEXTURE_COUNT_VERSION_2 {
                    self.image_index += TEXTURE_COUNT - TEXTURE_COUNT_VERSION_2;
                }
            }
            3 => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
                if self.image_index >= TEXTURE_COUNT_VERSION_3 {
                    self.image_index += TEXTURE_COUNT - TEXTURE_COUNT_VERSION_3;
                }
            }
            _ => {
                visitor.visit_i32("EmitIndex", &mut self.emit_index)?;
            }
        }

        if version >= 1 {
            visitor.visit_enum("Preset", &mut self.preset)?;
        } else {
            self.preset = GoldenSimplesPresets::Custom;
        }

        visitor.visit_vector_3("textureScale", &mut self.texture_scale)?;
        visitor.visit_vector_3("textureOffset", &mut self.texture_offset)?;
        visitor.visit_bool("flipTextureUV", &mut self.flip_texture_uv)?;
        visitor.visit_bool("worldMapped", &mut self.world_mapped)?;
        visitor.visit_bool("disableDiffuse", &mut self.disable_diffuse)?;
        visitor.visit_bool("disableBump", &mut self.disable_bump)?;
        if version >= 3 {
            visitor.visit_f32("bumpStrength", &mut self.bump_strength)?;
        }
        visitor.visit_bool("disableReflect", &mut self.disable_reflect)?;
        if version >= 1 {
            visitor.visit_bool("disableCollision", &mut self.disable_collision)?;
            visitor.visit_bool("additiveTransparency", &mut self.additive_transparency)?;
        }
        if version >= 2 {
            visitor.visit_bool(
                "multiplicativeTransparency",
                &mut self.multiplicative_transparency,
            )?;
            visitor.visit_bool("invertEmit", &mut self.invert_emit)?;
        }

        Ok(())
    }
}

#[derive(
    Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Primitive, Serialize, Deserialize,
)]
pub enum GoldenSimplesPresets {
    Custom = 0,
    Solid = 1,
    Shadeless = 2,
    Emissive = 3,
    Rock = 4,
    Rock2 = 5,
    WetRock = 6,
    Water = 7,
    ReflectiveWater = 8,
    Lava = 9,
    Asphalt = 10,
    RustyMetal = 11,
    Sand = 12,
    Ice = 13,
    Snow = 14,
    Turf = 15,
    Dirt = 16,
    Glass = 17,
    Panel1 = 18,
    Panel2 = 19,
    Panel3 = 20,
    Empire1 = 21,
    Empire2 = 22,
    Empire3 = 23,
    Empire4 = 24,
    Infected = 25,
    Ancient1 = 26,
    Ancient2 = 27,
    Ancient3 = 28,
    Alien = 29,
}

impl Default for GoldenSimplesPresets {
    fn default() -> Self {
        GoldenSimplesPresets::Custom
    }
}
