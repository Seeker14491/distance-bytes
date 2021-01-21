use crate::serialization::{
    error::{failure, BytesDeserializeErrorKind},
    BytesParseResult, Input,
};
use nom::number::complete::{le_u16, u8};
use widestring::U16String;

pub(crate) fn string(input: Input<'_>) -> BytesParseResult<'_, String> {
    let err_input = input;
    let (mut input, len_in_bytes) = encoded_len_in_bytes(input)?;
    if len_in_bytes < 0 || len_in_bytes % 2 != 0 {
        return Err(failure(err_input, BytesDeserializeErrorKind::String));
    }

    let len = len_in_bytes / 2;
    let mut code_units = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let (input_2, code_unit) = le_u16(input)?;
        input = input_2;
        code_units.push(code_unit);
    }

    let s = U16String::from_vec(code_units)
        .to_string()
        .map_err(|_| failure(err_input, BytesDeserializeErrorKind::String))?;

    Ok((input, s))
}

fn encoded_len_in_bytes(mut input: Input<'_>) -> BytesParseResult<'_, i32> {
    let mut decoded_len = 0;
    let mut insert_index = 0;
    for _byte_index in 0..5 {
        let (input_2, byte) = u8(input)?;
        input = input_2;

        let contribution = (byte & 0b0111_1111) as i32;
        let done = byte & 0b1000_0000 == 0;

        decoded_len |= contribution << insert_index;

        if done {
            break;
        }

        insert_index += 7;
    }

    Ok((input, decoded_len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let input = b"\x10S\0p\0e\0c\0t\0r\0u\0m\0";

        let (input, s) = string(input[..].into()).unwrap();

        assert_eq!(input.as_ref(), &[]);
        assert_eq!(&s, "Spectrum");
    }

    #[test]
    fn test_empty_string() {
        let input = b"\0";

        let (input, s) = string(input[..].into()).unwrap();

        assert_eq!(input.as_ref(), &[]);
        assert_eq!(&s, "");
    }

    #[test]
    fn test_encoded_len() {
        let input = [0b1000_0111, 0b0000_0011];

        let (_input, len) = encoded_len_in_bytes(input[..].into()).unwrap();

        assert_eq!(len, 0b1_1000_0111);
    }
}
