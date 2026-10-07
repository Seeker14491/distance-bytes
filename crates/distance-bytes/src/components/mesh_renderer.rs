use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `MeshRenderer` (the game's `MeshRendererWrapper` and `SerialRenderer`): the colors of the
/// renderer's materials.
///
/// The game writes every material and color, not only those that differ from the prefab. An
/// empty list keeps the prefab's materials.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeshRenderer {
    /// Version 0 only, ignored by the game.
    pub cast_shadows: Option<bool>,
    /// Version 0 only, ignored by the game.
    pub receive_shadows: Option<bool>,
    pub materials: Vec<MaterialColors>,
    /// Version 0 only, ignored by the game.
    pub lightmap_index: Option<i32>,
    /// Version 0 only, ignored by the game. The game wrote a `Vector4` but reads three floats.
    pub lightmap_tiling_offset: Option<[f32; 3]>,
    /// Version 0 only, ignored by the game.
    pub use_light_probes: Option<bool>,
}

/// The colors of one material of a renderer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MaterialColors {
    /// The material's name.
    pub material: Option<DString>,
    pub colors: Vec<MaterialColor>,
}

/// A color property of a material, such as `_Color` or `_EmitColor`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MaterialColor {
    pub name: Option<DString>,
    /// `[r, g, b, a]`.
    pub color: Option<[f32; 4]>,
}

impl Visit for MeshRenderer {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        // SerialRenderer.Visit
        if version < 1 {
            v.value("CastShadows", &mut self.cast_shadows)?;
            v.value("ReceiveShadows", &mut self.receive_shadows)?;
            visit_materials(v, &mut self.materials)?;
            v.value("LightmapIndex", &mut self.lightmap_index)?;
            v.value("LightmapTilingOffset", &mut self.lightmap_tiling_offset)?;
            v.value("UseLightProbes", &mut self.use_light_probes)
        } else {
            visit_materials(v, &mut self.materials)
        }
    }
}

// ObjectMaterialColorVisitor.Visit
fn visit_materials(v: &mut impl Visitor, materials: &mut Vec<MaterialColors>) -> Result<()> {
    v.array("MaterialsColor", materials, |v, m| {
        v.value("MatName", &mut m.material)?;
        v.array("UniformColors", &mut m.colors, |v, c| {
            v.value("Name", &mut c.name)?;
            v.value("Color", &mut c.color)
        })
    })
}
