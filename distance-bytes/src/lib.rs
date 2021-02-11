#![warn(
    rust_2018_idioms,
    deprecated_in_future,
    macro_use_extern_crate,
    missing_debug_implementations,
    unused_qualifications
)]
#![allow(dead_code)]

mod domain;
mod serialization;
mod util;

pub use crate::domain::{
    component::{Component, ComponentData, RawComponentData},
    GameObject,
};

use crate::serialization::Deserializer;
use anyhow::Error;
use std::io::{Read, Seek};

pub fn read_game_object(reader: impl Read + Seek) -> Result<GameObject, Error> {
    let mut deserializer = Deserializer::new(reader);

    deserializer.read_game_object()
}
