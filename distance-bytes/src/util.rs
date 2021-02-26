use crate::{
    domain::{Quaternion, Vector3},
    serialization::EMPTY_MARK,
};
use std::{
    fmt::Write,
    io,
    io::{Seek, SeekFrom},
};

pub(crate) fn f32_max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

// Implementation of Unity's `Mathf.Approximately()` function
pub(crate) fn f32_approx_equal(a: f32, b: f32) -> bool {
    f32::abs(b - a) < f32_max(1E-6 * f32_max(a.abs(), b.abs()), f32::EPSILON * 8.0)
}

pub(crate) fn vector3_approx_equals(a: Vector3, b: Vector3) -> bool {
    f32_approx_equal(a.x, b.x) && f32_approx_equal(a.y, b.y) && f32_approx_equal(a.z, b.z)
}

pub(crate) fn quaternion_approx_equals(a: Quaternion, b: Quaternion) -> bool {
    f32_approx_equal(a.v.x, b.v.x)
        && f32_approx_equal(a.v.y, b.v.y)
        && f32_approx_equal(a.v.z, b.v.z)
        && f32_approx_equal(a.s, b.s)
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

// Modified from the unstable `stream_len()` function in `std::io::Seek`.
pub(crate) fn stream_len(mut stream: impl Seek) -> io::Result<u64> {
    let old_pos = stream.stream_position()?;
    let len = stream.seek(SeekFrom::End(0))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        stream.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len)
}

pub(crate) fn scope_mark_string(scope_mark: i32) -> &'static str {
    match scope_mark {
        11111111 => "Array",
        12121212 => "Dictionary",
        22222222 => "SerialComponent",
        23232323 => "UnknownComponent",
        33333333 => "BuiltInComponent",
        44444444 => "General",
        55555555 => "Children",
        66666666 => "GameObject",
        88888888 => "LevelSettings",
        99999999 => "Level",
        n if n == EMPTY_MARK => "Empty",
        _ => "INVALID",
    }
}
