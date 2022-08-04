use crate::internal::{Serializable, Visitor};
use crate::CarColors;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CarData {
    pub version: i32,
    pub name: Option<String>,
    pub colors: CarColors,
}

impl Serializable for CarData {
    const VERSION: i32 = 0;

    fn accept<V: Visitor>(&mut self, mut visitor: V, _version: i32) -> Result<()> {
        let mut version = 0;
        visitor.visit_i32("CarDataVersion", &mut version)?;

        visitor.visit_string("name_", &mut self.name)?;

        self.colors.accept(visitor, 0)
    }
}
