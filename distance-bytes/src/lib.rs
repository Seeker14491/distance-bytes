#![warn(
    rust_2018_idioms,
    deprecated_in_future,
    macro_use_extern_crate,
    missing_debug_implementations,
    unused_qualifications
)]
#![allow(dead_code)] // FIXME: remove

mod domain;
mod serialization;
mod util;

#[cfg(test)]
mod test_util;

pub use crate::domain::{
    component::{Component, ComponentData, RawComponentData},
    GameObject,
};

use crate::serialization::{Deserializer, Serializer};
use anyhow::Error;
use std::io::{Read, Seek, Write};

pub fn read_game_object(reader: impl Read + Seek) -> Result<GameObject, Error> {
    let mut deserializer = Deserializer::new(reader);

    deserializer.read_game_object()
}

pub fn write_game_object(
    writer: impl Write + Seek,
    game_object: &mut GameObject,
) -> Result<(), Error> {
    let mut serializer = Serializer::new(writer);

    serializer.write_game_object(game_object)
}
