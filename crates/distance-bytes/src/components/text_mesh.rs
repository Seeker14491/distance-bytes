use std::ops::RangeInclusive;

use crate::error::Result;
use crate::string::DString;
use crate::visit::{Visit, Visitor};

/// `TextMesh` (the game's `TextMeshWrapper`): settings for the 3D text.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextMesh {
    pub text: Option<DString>,
    /// Unity's `FontStyle`: 0 normal, 1 bold, 2 italic, 3 bold and italic.
    pub font_style: Option<i32>,
    /// `TextMeshWrapper.FontName`: 0 Petita, 1 Dense. Version 2 and later.
    pub font: Option<i32>,
    /// `[r, g, b, a]`. Version 2 and later.
    pub color: Option<[f32; 4]>,
}

impl Visit for TextMesh {
    const VERSIONS: RangeInclusive<i32> = 0..=2;

    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()> {
        v.value("Text", &mut self.text)?;
        v.value("FontStyle", &mut self.font_style)?;
        if version >= 2 {
            v.value("Font", &mut self.font)?;
            v.value("Color", &mut self.color)?;
        }
        Ok(())
    }
}
