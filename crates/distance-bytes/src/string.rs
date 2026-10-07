use std::borrow::Cow;
use std::fmt;

/// A string as stored in a `.bytes` file (UTF-16LE).
///
/// Strings that aren't valid UTF-16 are kept as their raw bytes so they can be written back
/// unchanged.
#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct DString(Repr);

#[derive(Clone, PartialEq, Eq, Hash)]
enum Repr {
    Text(String),
    Raw(Vec<u8>),
}

impl Default for Repr {
    fn default() -> Self {
        Repr::Text(String::new())
    }
}

impl DString {
    pub fn from_utf16le_bytes(bytes: &[u8]) -> Self {
        let (pairs, odd) = bytes.as_chunks::<2>();
        if odd.is_empty() {
            let units = pairs.iter().map(|&c| u16::from_le_bytes(c));
            if let Ok(s) = char::decode_utf16(units).collect::<Result<String, _>>() {
                return DString(Repr::Text(s));
            }
        }
        DString(Repr::Raw(bytes.to_vec()))
    }

    pub fn to_utf16le_bytes(&self) -> Cow<'_, [u8]> {
        match &self.0 {
            Repr::Text(s) => s.encode_utf16().flat_map(u16::to_le_bytes).collect(),
            Repr::Raw(b) => Cow::Borrowed(b),
        }
    }

    /// Returns the string, or `None` if it isn't valid UTF-16.
    pub fn as_str(&self) -> Option<&str> {
        match &self.0 {
            Repr::Text(s) => Some(s),
            Repr::Raw(_) => None,
        }
    }

    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        match &self.0 {
            Repr::Text(s) => Cow::Borrowed(s),
            Repr::Raw(b) => {
                let units = b
                    .chunks(2)
                    .map(|c| u16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)]));
                Cow::Owned(
                    char::decode_utf16(units)
                        .map(|c| c.unwrap_or('\u{FFFD}'))
                        .collect(),
                )
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Repr::Text(s) => s.is_empty(),
            Repr::Raw(b) => b.is_empty(),
        }
    }
}

impl From<String> for DString {
    fn from(s: String) -> Self {
        DString(Repr::Text(s))
    }
}

impl From<&str> for DString {
    fn from(s: &str) -> Self {
        DString(Repr::Text(s.to_owned()))
    }
}

impl PartialEq<str> for DString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}

impl PartialEq<&str> for DString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == Some(*other)
    }
}

impl fmt::Display for DString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string_lossy())
    }
}

impl fmt::Debug for DString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Repr::Text(s) => fmt::Debug::fmt(s, f),
            Repr::Raw(b) => write!(f, "DString::Raw({b:02x?})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_utf16_round_trips() {
        let s = DString::from("Größe 🚗");
        let bytes = s.to_utf16le_bytes();
        assert_eq!(DString::from_utf16le_bytes(&bytes), s);
    }

    #[test]
    fn invalid_utf16_is_preserved() {
        // Unpaired high surrogate, then an odd trailing byte.
        let bytes = [0x00, 0xD8, 0x41];
        let s = DString::from_utf16le_bytes(&bytes);
        assert_eq!(s.as_str(), None);
        assert_eq!(&*s.to_utf16le_bytes(), &bytes);
    }
}
