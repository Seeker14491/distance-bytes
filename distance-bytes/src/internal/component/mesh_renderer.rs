use crate::internal::{MaterialInfo, Serializable, Visitor, ZEROS_VECTOR_3};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MeshRenderer {
    pub material_infos: Vec<MaterialInfo>,
}

impl Serializable for MeshRenderer {
    const VERSION: i32 = 2;

    fn accept<V: Visitor>(&mut self, mut visitor: V, version: i32) -> Result<()> {
        if version < 1 {
            visitor.visit_bool("CastShadows", &mut false)?;
            visitor.visit_bool("ReceiveShadows", &mut false)?;

            visitor.visit_array(
                "MaterialsColor",
                &mut self.material_infos,
                |visitor, element| visitor.visit_material_info("MaterialsColor", element),
            )?;

            visitor.visit_i32("LightmapIndex", &mut 0)?;
            visitor.visit_vector_3("LightmapTilingOffset", &mut { ZEROS_VECTOR_3 })?;
            visitor.visit_bool("UseLightProbes", &mut false)?;
        } else {
            visitor.visit_array(
                "MaterialsColor",
                &mut self.material_infos,
                |visitor, element| visitor.visit_material_info("MaterialsColor", element),
            )?;
        }

        Ok(())
    }
}
