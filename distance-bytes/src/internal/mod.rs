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

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
enum VisitDirection {
    In,
    Out,
}

#[auto_impl(&mut)]
trait Visitor {
    const VISIT_DIRECTION: VisitDirection;

    fn visit_bool(&mut self, name: &str, value: &mut bool) -> Result<(), Error>;
    fn visit_i32(&mut self, name: &str, value: &mut i32) -> Result<(), Error>;
    fn visit_u32(&mut self, name: &str, value: &mut u32) -> Result<(), Error>;
    fn visit_i64(&mut self, name: &str, value: &mut i64) -> Result<(), Error>;
    fn visit_f32(&mut self, name: &str, value: &mut f32) -> Result<(), Error>;
    fn visit_vector_3(&mut self, name: &str, value: &mut Vector3) -> Result<(), Error>;
    fn visit_quaternion(&mut self, name: &str, value: &mut Quaternion) -> Result<(), Error>;

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
}

#[auto_impl(&mut)]
trait Serializable: Default {
    const VERSION: i32;

    fn accept<V: Visitor>(&mut self, visitor: V, version: i32) -> Result<(), Error>;
}
