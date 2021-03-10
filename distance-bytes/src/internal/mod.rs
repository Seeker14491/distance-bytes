pub use component::ComponentId;

use anyhow::Error;
use auto_impl::auto_impl;
use component::Component;
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;

pub(crate) mod component;
pub(crate) mod deserializer;
pub(crate) mod serializer;

mod string;
mod util;

#[cfg(test)]
mod test_util;

pub(crate) const ZEROS_VECTOR_3: Vector3 = Vector3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

pub(crate) const ONES_VECTOR_3: Vector3 = Vector3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

pub(crate) const DEFAULT_QUATERNION: Quaternion = Quaternion {
    v: Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    s: 1.0,
};

pub(crate) const INVALID_INT: i32 = -127;
pub(crate) const INVALID_FLOAT: f32 = -10_000.0;

pub(crate) const INVALID_VECTOR_3: Vector3 = Vector3 {
    x: INVALID_FLOAT,
    y: INVALID_FLOAT,
    z: INVALID_FLOAT,
};

pub(crate) const INVALID_QUATERNION: Quaternion = Quaternion {
    v: INVALID_VECTOR_3,
    s: INVALID_FLOAT,
};

pub(crate) const EMPTY_MARK: i32 = 0x7FFF_FFFD;

pub type Vector3 = mint::Vector3<f32>;
pub type Quaternion = mint::Quaternion<f32>;

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GameObject {
    pub name: String,
    pub guid: u32,
    pub components: Vec<Component>,
}

impl GameObject {
    pub fn read_from_reader(reader: impl Read + Seek) -> Result<GameObject, Error> {
        deserializer::read_game_object(reader)
    }

    pub fn read_from_file(path: impl AsRef<Path>) -> Result<GameObject, Error> {
        let mut file = BufReader::new(File::open(path.as_ref())?);
        deserializer::read_game_object(&mut file)
    }

    pub fn write_to_reader(&mut self, writer: impl Write + Seek) -> Result<(), Error> {
        serializer::write_game_object(writer, self)
    }

    pub fn write_to_file(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let mut file = BufWriter::new(File::create(path.as_ref())?);
        serializer::write_game_object(&mut file, self)
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MaterialInfo {
    pub mat_name: Option<String>,
    pub colors: Vec<MaterialColorInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MaterialColorInfo {
    pub name: Option<String>,
    pub color: Color,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum VisitDirection {
    In,
    Out,
}

#[auto_impl(&mut)]
pub(crate) trait Visitor
where
    Self: Sized,
{
    // Workaround for a problem with referencing `Self` while using the `auto_impl` macro
    type Self_: Visitor;

    const VISIT_DIRECTION: VisitDirection;

    fn visit_bool(&mut self, name: &str, value: &mut bool) -> Result<(), Error>;
    fn visit_i32(&mut self, name: &str, value: &mut i32) -> Result<(), Error>;
    fn visit_u32(&mut self, name: &str, value: &mut u32) -> Result<(), Error>;
    fn visit_i64(&mut self, name: &str, value: &mut i64) -> Result<(), Error>;
    fn visit_f32(&mut self, name: &str, value: &mut f32) -> Result<(), Error>;
    fn visit_string(&mut self, name: &str, value: &mut Option<String>) -> Result<(), Error>;
    fn visit_vector_3(&mut self, name: &str, value: &mut Vector3) -> Result<(), Error>;
    fn visit_quaternion(&mut self, name: &str, value: &mut Quaternion) -> Result<(), Error>;
    fn visit_reference(&mut self, name: &str, value: &mut u32) -> Result<(), Error>;
    fn visit_reference_array(
        &mut self,
        array_name: &str,
        element_name: &str,
        value: &mut Vec<u32>,
    ) -> Result<(), Error>;

    fn visit_material_info(&mut self, _name: &str, value: &mut MaterialInfo) -> Result<(), Error> {
        self.visit_string("MatName", &mut value.mat_name)?;
        self.visit_array("", &mut value.colors, |visitor, element| {
            visitor.visit_material_color_info("", element)
        })?;

        Ok(())
    }

    fn visit_material_color_info(
        &mut self,
        _name: &str,
        value: &mut MaterialColorInfo,
    ) -> Result<(), Error> {
        self.visit_string("Name", &mut value.name)?;

        value.color = Color::new(1.0, 1.0, 1.0, 1.0);
        self.visit_color("Color", &mut value.color)?;

        Ok(())
    }

    fn visit_color(&mut self, _name: &str, value: &mut Color) -> Result<(), Error> {
        self.visit_f32("r", &mut value.r)?;
        self.visit_f32("g", &mut value.g)?;
        self.visit_f32("b", &mut value.b)?;
        self.visit_f32("a", &mut value.a)?;

        Ok(())
    }

    fn visit_array<T, F>(
        &mut self,
        element_name: &str,
        array: &mut Vec<T>,
        visit_t_fn: F,
    ) -> Result<(), Error>
    where
        T: Default,
        F: FnMut(&mut Self::Self_, &mut T) -> Result<(), Error>;

    fn visit_children(&mut self, value: &mut Vec<GameObject>) -> Result<(), Error>;

    fn visit_enum<T: FromPrimitive + ToPrimitive>(
        &mut self,
        name: &str,
        value: &mut T,
    ) -> Result<(), Error> {
        let mut n = value.to_i32().unwrap();
        self.visit_i32(name, &mut n)?;

        if Self::VISIT_DIRECTION == VisitDirection::In {
            *value = T::from_i32(n).unwrap();
        }

        Ok(())
    }

    fn visit_serial_collider_deprecated(&mut self, _name: &str) -> Result<(), Error> {
        self.visit_bool("IsTrigger", &mut false)?;
        self.visit_string("PhysicMaterialName", &mut None)?;

        Ok(())
    }
}

#[auto_impl(&mut)]
pub(crate) trait Serializable: Default {
    const VERSION: i32;

    fn accept<V: Visitor>(&mut self, visitor: V, version: i32) -> Result<(), Error>;
}
