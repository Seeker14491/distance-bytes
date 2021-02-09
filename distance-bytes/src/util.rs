use std::fmt::Write;

pub(crate) fn f32_max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

pub(crate) fn format_byte_slice(slice: &[u8], max_bytes_to_print: usize) -> String {
    let mut s = String::new();

    write!(&mut s, "(hex) [").unwrap();
    for (i, byte) in slice.iter().take(max_bytes_to_print).enumerate() {
        write!(&mut s, "{:02X}", byte).unwrap();
        if i != max_bytes_to_print - 1 {
            write!(&mut s, " ").unwrap();
        }
    }

    if slice.len() > max_bytes_to_print {
        write!(&mut s, " ...").unwrap();
    }

    write!(&mut s, "]").unwrap();

    s
}
