use anyhow::{ensure, Result};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Read, Write};
use widestring::U16String;

pub(crate) fn read(mut reader: impl Read) -> Result<String> {
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

pub(crate) fn write(mut writer: impl Write, s: &str) -> Result<()> {
    let s = U16String::from_str(s);
    let s = s.as_slice();
    let string_len_in_bytes = 2 * s.len();

    let mut n = string_len_in_bytes;
    loop {
        let mut byte_to_write = (n & 0b0111_1111) as u8;
        n >>= 7;

        let done = n == 0;
        if !done {
            byte_to_write |= 0b1000_0000;
        }

        writer.write_u8(byte_to_write)?;

        if done {
            break;
        }
    }

    for &code_unit in s {
        writer.write_u16::<LE>(code_unit)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::test_util::hex_dump;
    use insta::assert_snapshot;
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_write() {
        let s = "Nitronic";
        let mut buf = Cursor::new(Vec::new());

        write(&mut buf, s).unwrap();

        assert_snapshot!(hex_dump(buf.get_ref()), @r###"
        Length: 17 (0x11) bytes
        0000:   10 4e 00 69  00 74 00 72  00 6f 00 6e  00 69 00 63   .N.i.t.r.o.n.i.c
        0010:   00                                                   .
        "###);
    }

    #[test]
    fn test_writing_then_reading_yields_original() {
        let original = "Rush";

        let mut buf = Cursor::new(Vec::new());
        write(&mut buf, original).unwrap();

        buf.seek(SeekFrom::Start(0)).unwrap();
        let deserialized = read(&mut buf).unwrap();
        assert_eq!(original, &deserialized);
    }
}
