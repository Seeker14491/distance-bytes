//! Conversion to and from JSON, losslessly: a file converted to JSON and back is byte-identical.
//!
//! ```no_run
//! let data = std::fs::read("level.bytes").unwrap();
//! let file = distance_bytes::BytesFile::read(&data).unwrap();
//! let json = distance_bytes::json::to_string_pretty(&file).unwrap();
//! let back: distance_bytes::BytesFile = distance_bytes::json::from_str(&json).unwrap();
//! assert_eq!(back.write(), data);
//! ```
//!
//! Use these functions rather than serde_json's directly: JSON has no NaN or infinities, which
//! these write as the strings `"NaN"`, `"Infinity"` and `"-Infinity"` (a NaN other than the
//! canonical one is written with its bits, like `"NaN:0xffc00000"`), and objects can nest
//! deeper than serde_json allows by default. Integers too large for a JavaScript number to hold
//! exactly, beyond ±(2⁵³ − 1), are written as strings of digits, like a level's `date_time`
//! `"5247589795433381000"`, so that the JSON can be read in a browser without losing them; 64-bit
//! integers are read from either form.
//!
//! # Shape
//!
//! - A file is `{"level": {...}}` or, for non-level files, `{"game_object": {...}}`.
//! - Struct fields use their Rust names. A `None` value is the empty marker, "the same as the
//!   prefab's", and is left out; `null` reads the same. The few fields that aren't optional
//!   (lists, references and flags) are left out when empty, 0 or false.
//! - A component is `{"type": "TeleporterEntrance", "version": 4, "guid": 12, ...}` with at
//!   most one body:
//!   - `"data"`: a typed body ([`ComponentData`](crate::ComponentData) or, for a `Transform`,
//!     [`Transform`](crate::Transform));
//!   - `"fields"`: a body in the game's generic format, as an object of the game's field names.
//!     Values are typed by the field's type in the game; a value that doesn't decode is
//!     `{"raw": "<hex>"}`;
//!   - `"raw"`: an undecoded body as hex.
//!
//!   Without a body, the component's body is empty. `"mark"` (`"built_in"`, `"serial"` or
//!   `"added"`) appears only when it isn't the one the game writes for the type, and `"end"`
//!   only when a typed body is truncated or has trailing bytes. A component of a type the game
//!   doesn't know is `{"unknown_type": "<name>", "guid": ..., "raw": ...}`.
//! - Byte arrays are hex strings. A string that isn't valid UTF-16 is `{"utf16le": "<hex>"}`.
//!
//! When reading, `"type"` must come before `"data"` and `"fields"`, which are read according to
//! it. A missing `"version"` is the version the game writes for the type, and a missing
//! `"guid"` is 0, which [`Level::assign_guids`](crate::Level::assign_guids) replaces with a new
//! one.

mod adapter;
mod deep;
mod format;
mod impls;
#[cfg(feature = "schema")]
mod schema;

use std::fmt;
use std::io;

use serde::{Deserialize, Serialize};

pub(crate) use deep::children;

/// An error converting to or from JSON, with where it happened.
#[derive(Debug)]
pub struct Error {
    path: String,
    /// The line and column, when they aren't the inner error's: for an error in a part of the
    /// document read on its own.
    position: Option<(usize, usize)>,
    inner: serde_json::Error,
}

impl Error {
    fn new(e: serde_path_to_error::Error<serde_json::Error>) -> Self {
        let path = e.path().to_string();
        Error {
            path,
            position: None,
            inner: e.into_inner(),
        }
    }

    /// The error of the part of `text` that starts at `offset`, which was read on its own.
    fn within(self, text: &[u8], offset: usize) -> Self {
        let offset = offset.min(text.len());
        let prefix = deep::path_at(text, offset);
        let path = match self.path.as_str() {
            "" | "." => prefix,
            path if path.starts_with('[') => prefix + path,
            path => format!("{prefix}.{path}"),
        };
        let position = (self.line() != 0).then(|| {
            let start_of_line = text[..offset]
                .iter()
                .rposition(|&b| b == b'\n')
                .map_or(0, |i| i + 1);
            let line = 1 + text[..start_of_line]
                .iter()
                .filter(|&&b| b == b'\n')
                .count();
            match self.line() {
                1 => (line, offset - start_of_line + self.column()),
                n => (line + n - 1, self.column()),
            }
        });
        Error {
            path,
            position,
            inner: self.inner,
        }
    }

    /// The path of the value in the JSON document, like `level.layers[0].objects[3].guid`.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The line of the error when reading, starting at 1, or 0 when writing.
    pub fn line(&self) -> usize {
        self.position.map_or(self.inner.line(), |(line, _)| line)
    }

    /// The column of the error when reading, starting at 1, or 0 when writing.
    pub fn column(&self) -> usize {
        self.position
            .map_or(self.inner.column(), |(_, column)| column)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.path.is_empty() && self.path != "." {
            write!(f, "at {}: ", self.path)?;
        }
        let Some((line, column)) = self.position else {
            return write!(f, "{}", self.inner);
        };
        // The inner error's message ends with its own position.
        let message = self.inner.to_string();
        let suffix = format!(
            " at line {} column {}",
            self.inner.line(),
            self.inner.column()
        );
        let message = message.strip_suffix(&suffix).unwrap_or(&message);
        write!(f, "{message} at line {line} column {column}")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

pub fn to_writer<W: io::Write, T: Serialize + ?Sized>(writer: W, value: &T) -> Result<(), Error> {
    let mut ser = serde_json::Serializer::new(writer);
    serde_path_to_error::serialize(value, adapter::Ser(&mut ser)).map_err(Error::new)
}

/// Like [`to_writer`], indented, with arrays of plain values (such as vectors) on one line.
pub fn to_writer_pretty<W: io::Write, T: Serialize + ?Sized>(
    writer: W,
    value: &T,
) -> Result<(), Error> {
    let mut ser = serde_json::Serializer::with_formatter(writer, format::Pretty::default());
    serde_path_to_error::serialize(value, adapter::Ser(&mut ser)).map_err(Error::new)
}

pub fn to_vec<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, Error> {
    let mut out = Vec::new();
    to_writer(&mut out, value)?;
    Ok(out)
}

pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    // serde_json writes only valid UTF-8.
    Ok(String::from_utf8(to_vec(value)?).unwrap())
}

pub fn to_string_pretty<T: Serialize + ?Sized>(value: &T) -> Result<String, Error> {
    let mut out = Vec::new();
    to_writer_pretty(&mut out, value)?;
    Ok(String::from_utf8(out).unwrap())
}

pub fn from_str<'a, T: Deserialize<'a>>(s: &'a str) -> Result<T, Error> {
    from_json(serde_json::Deserializer::from_str(s), s.as_bytes())
}

pub fn from_slice<'a, T: Deserialize<'a>>(v: &'a [u8]) -> Result<T, Error> {
    from_json(serde_json::Deserializer::from_slice(v), v)
}

/// Reads `text` with `de`, which reads from it.
fn from_json<'de, R: serde_json::de::Read<'de>, T: Deserialize<'de>>(
    mut de: serde_json::Deserializer<R>,
    text: &[u8],
) -> Result<T, Error> {
    // The adapter grows the stack instead, and deep children are read separately.
    de.disable_recursion_limit();
    deep::reading(text, || {
        let value = serde_path_to_error::deserialize(adapter::De(&mut de)).map_err(Error::new)?;
        de.end().map_err(|e| Error {
            path: String::new(),
            position: None,
            inner: e,
        })?;
        Ok(value)
    })
}

#[cfg(feature = "schema")]
pub use schema::{SchemaOptions, schema};

/// For `#[serde(skip_serializing_if = "crate::json::is_default")]` on fields that aren't
/// optional, which read as their default when left out.
pub(crate) fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

/// Byte arrays as lowercase hex strings, for `#[serde(with = "crate::json::hex")]`.
pub(crate) mod hex {
    use std::fmt;

    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serialize, Serializer};

    pub(crate) fn encode(bytes: &[u8]) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(DIGITS[usize::from(b >> 4)].into());
            s.push(DIGITS[usize::from(b & 0xf)].into());
        }
        s
    }

    pub(crate) fn decode(s: &str) -> Option<Vec<u8>> {
        fn digit(c: u8) -> Option<u8> {
            (c as char).to_digit(16).map(|d| d as u8)
        }
        let (pairs, odd) = s.as_bytes().as_chunks::<2>();
        if !odd.is_empty() {
            return None;
        }
        pairs
            .iter()
            .map(|&[hi, lo]| Some(digit(hi)? << 4 | digit(lo)?))
            .collect()
    }

    /// Bytes serialized as hex.
    pub(crate) struct Hex<'a>(pub &'a [u8]);

    #[cfg(feature = "schema")]
    impl schemars::JsonSchema for Hex<'_> {
        fn schema_name() -> std::borrow::Cow<'static, str> {
            "Hex".into()
        }

        fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
            schemars::json_schema!({
                "type": "string",
                "pattern": "^([0-9a-fA-F]{2})*$",
                "description": "Bytes as hexadecimal digits."
            })
        }
    }

    impl Serialize for Hex<'_> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            s.serialize_str(&encode(self.0))
        }
    }

    pub(crate) fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        Hex(bytes).serialize(s)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        d.deserialize_str(HexVisitor)
    }

    pub(crate) struct HexVisitor;

    impl Visitor<'_> for HexVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a hex string")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<u8>, E> {
            decode(v).ok_or_else(|| E::invalid_value(de::Unexpected::Str(v), &self))
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn round_trip() {
            let bytes = [0x00, 0x7f, 0x80, 0xff, 0x1a];
            assert_eq!(super::encode(&bytes), "007f80ff1a");
            assert_eq!(super::decode("007f80FF1a").unwrap(), bytes);
            assert_eq!(super::decode("abc"), None);
            assert_eq!(super::decode("zz"), None);
        }
    }
}

#[cfg(test)]
mod tests;
