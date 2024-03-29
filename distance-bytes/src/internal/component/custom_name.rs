use crate::internal::{Serializable, Visitor};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct CustomName {
    pub custom_name: Option<String>,
}

impl Serializable for CustomName {
    const VERSION: i32 = 0;

    fn accept<V: Visitor>(&mut self, mut visitor: V, _version: i32) -> Result<()> {
        visitor.visit_string("CustomName", &mut self.custom_name)
    }
}
