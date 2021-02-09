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

pub use crate::domain::GameObject;
use combine::EasyParser;

pub fn deserialize_game_object(
    data: &[u8],
) -> Result<GameObject, combine::easy::Errors<u8, String, usize>> {
    serialization::game_object()
        .easy_parse(data)
        .map(|x| x.0)
        .map_err(|e| serialization::finalize_errors(data, e))
}

pub fn serialize_game_object(_game_object: GameObject) -> Vec<u8> {
    todo!()
}
