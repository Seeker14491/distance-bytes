use crate::internal::{Serializable, VisitDirection, Visitor};
use crate::LevelInfo;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelInfos(Vec<LevelInfo>);

impl Serializable for LevelInfos {
    const VERSION: i32 = 0;

    fn accept<V: Visitor>(&mut self, mut visitor: V, _version: i32) -> Result<()> {
        let mut number_of_level_infos = self.0.len().try_into().context("too many LevelInfos")?;
        visitor.visit_i32("NumberOfLevelInfos", &mut number_of_level_infos)?;

        let mut level_info_version = Self::VERSION;
        visitor.visit_i32("LevelInfoVersion", &mut level_info_version)?;

        match V::VISIT_DIRECTION {
            VisitDirection::Out => {
                for level_info in &mut self.0 {
                    level_info.accept(&mut visitor, level_info_version)?;
                }
            }
            VisitDirection::In => {
                let num_level_infos: usize = number_of_level_infos
                    .try_into()
                    .context("the serialized NumberOfLevelInfos field was negative")?;
                let mut level_infos = Vec::with_capacity(num_level_infos);
                for _ in 0..num_level_infos {
                    let mut level_info = LevelInfo::default();
                    level_info.accept(&mut visitor, level_info_version)?;
                    level_infos.push(level_info);
                }

                self.0 = level_infos;
            }
        }

        Ok(())
    }
}
