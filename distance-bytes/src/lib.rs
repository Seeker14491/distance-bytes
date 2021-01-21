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

pub use crate::{domain::GameObject, serialization::error::BytesDeserializeError};

use nom::Finish;

pub fn deserialize_game_object(data: &[u8]) -> Result<GameObject, BytesDeserializeError> {
    serialization::read_game_object(data.into())
        .finish()
        .map(|(_, game_object)| game_object)
}

pub fn serialize_game_object(_game_object: GameObject) -> Vec<u8> {
    todo!()
}
