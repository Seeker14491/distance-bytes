use crate::internal::{Serializable, Visitor};
use crate::Color;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CarColors {
    pub primary: Color,
    pub secondary: Color,
    pub glow: Color,
    pub sparkle: Color,
}

impl Serializable for CarColors {
    const VERSION: i32 = 0;

    fn accept<V: Visitor>(&mut self, mut visitor: V, _version: i32) -> Result<()> {
        visitor.visit_color("Primary", &mut self.primary)?;
        visitor.visit_color("Secondary", &mut self.secondary)?;
        visitor.visit_color("Glow", &mut self.glow)?;
        visitor.visit_color("Sparkle", &mut self.sparkle)?;

        Ok(())
    }
}
