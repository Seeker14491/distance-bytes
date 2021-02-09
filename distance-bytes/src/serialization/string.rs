use crate::serialization::{Error, Parser};
use combine::{
    error::StreamError,
    parser::range::{length_prefix, take_while},
    Parser as _,
};
use std::convert::TryInto;
use widestring::U16String;

pub(crate) fn string<'a>() -> impl Parser<'a, String> {
    length_prefix(encoded_string_len()).and_then(|data: &[u8]| {
        let mut code_units = Vec::with_capacity(data.len() as usize);
        for slice in data.chunks_exact(2) {
            let code_unit = u16::from_le_bytes(slice.try_into().unwrap());
            code_units.push(code_unit);
        }

        U16String::from_vec(code_units).to_string()
    })
}

fn encoded_string_len<'a>() -> impl Parser<'a, i32> {
    let mut is_last_byte = false;
    let mut byte_idx = 0;
    let len_range = take_while(move |byte| {
        let take_this_byte = !is_last_byte;
        if byte_idx == 4 || byte & 0b1000_0000 == 0 {
            is_last_byte = true;
        }

        byte_idx += 1;
        take_this_byte
    });

    len_range.map(decode_len).and_then(|len| {
        if len % 2 == 0 {
            Ok(len)
        } else {
            Err(Error::unexpected_format(
                "odd number of bytes for UTF-16 string",
            ))
        }
    })
}

fn decode_len(input: &[u8]) -> i32 {
    assert!(!input.is_empty() && input.len() <= 5);

    let mut decoded = 0;
    let mut insert_idx = 0;
    for byte in input.iter() {
        let contribution = (byte & 0b0111_1111) as i32;
        decoded |= contribution << insert_idx;
        insert_idx += 7;
    }

    decoded
}
