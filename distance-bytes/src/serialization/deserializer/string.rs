use anyhow::{ensure, Error};
use byteorder::{ReadBytesExt, LE};
use std::io::Read;
use widestring::U16String;

pub(crate) fn read(mut reader: impl Read) -> Result<String, Error> {
    let mut string_len_in_bytes: usize = 0;
    for i in 0.. {
        ensure!(i <= 4, "Too many bytes in encoded string length");

        let byte = reader.read_u8()?;
        let contribution = (byte & 0b0111_1111) as usize;
        string_len_in_bytes |= contribution << (i * 7);

        let done = byte & 0b1000_0000 == 0;
        if done {
            break;
        }
    }

    ensure!(
        string_len_in_bytes % 2 == 0,
        "Odd number of bytes in a UTF-16 string ({} bytes)",
        string_len_in_bytes
    );

    let mut code_units = vec![0; string_len_in_bytes / 2];
    reader.read_u16_into::<LE>(&mut code_units)?;
    let string = U16String::from_vec(code_units).to_string()?;

    Ok(string)
}
