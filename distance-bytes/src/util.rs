use std::{fmt, fmt::Formatter};

pub(crate) fn f32_max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

pub(crate) fn write_pretty_input(
    f: &mut Formatter<'_>,
    input: &[u8],
    num_bytes_to_print: usize,
) -> fmt::Result {
    write!(f, "(hex) [")?;
    for (i, byte) in input.iter().take(num_bytes_to_print).enumerate() {
        write!(f, "{:02X}", byte)?;
        if i != num_bytes_to_print - 1 {
            write!(f, " ")?;
        }
    }

    if input.len() > num_bytes_to_print {
        write!(f, " ...")?;
    }

    write!(f, "]")
}
