//! Low-level primitives matching .NET's `BinaryReader`/`BinaryWriter` as used by the game.

use std::cell::RefCell;

use crate::error::{Error, ErrorKind, Result};
use crate::string::DString;

/// Scope marks written by the game's `BinarySerializer`.
pub mod mark {
    pub const LEVEL: i32 = 99999999;
    /// Level settings in level versions 0 and 1.
    pub const LEGACY_LEVEL_SETTINGS: i32 = 88888888;
    pub const LAYER: i32 = 77777777;
    pub const GAME_OBJECT: i32 = 66666666;
    pub const CHILDREN: i32 = 55555555;
    pub const SPLINE_TRACK_TRANSFORM: i32 = 44444444;
    /// A component that isn't a `SerialComponent` (mostly wrappers of Unity components).
    pub const SERIALIZABLE: i32 = 33333333;
    /// A component deriving from the game's `SerialComponent`.
    pub const SERIAL_COMPONENT: i32 = 22222222;
    /// Read by the game like [`SERIALIZABLE`]. The current game never writes it, but some Workshop
    /// levels contain it.
    pub const SERIALIZABLE_ALT: i32 = 32323232;
    pub const UNKNOWN_COMPONENT: i32 = 23232323;
    /// Not a scope: marks the start of an array or list, followed by an `i32` count.
    pub const ARRAY: i32 = 11111111;
    /// Not a scope: marks the start of a dictionary, followed by an `i32` count.
    pub const DICTIONARY: i32 = 12121212;
}

/// Written in place of a value that equals the prefab's (or is otherwise absent).
pub const EMPTY_MARKER: i32 = 0x7FFF_FFFD;

/// A cursor over a byte range of a file. Positions are absolute offsets into the whole file.
///
/// If created with [`Reader::with_trace`], every value read is recorded as a [`TraceEvent`].
#[derive(Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    end: usize,
    trace: Option<&'a RefCell<Vec<TraceEvent>>>,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0, end: data.len(), trace: None }
    }

    pub fn with_trace(data: &'a [u8], trace: &'a RefCell<Vec<TraceEvent>>) -> Self {
        Reader { trace: Some(trace), ..Reader::new(data) }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn remaining(&self) -> usize {
        self.end - self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.pos == self.end
    }

    pub fn error(&self, kind: ErrorKind) -> Error {
        Error::new(self.pos, kind)
    }

    /// Fails if any bytes are left in this reader's range.
    pub fn expect_end(&self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.error(ErrorKind::TrailingBytes { len: self.remaining() }))
        }
    }

    fn record(&self, offset: usize, value: impl FnOnce() -> TraceValue) {
        if let Some(trace) = self.trace {
            trace.borrow_mut().push(TraceEvent { offset, len: self.pos - offset, value: value() });
        }
    }

    /// Runs `f` on a copy of this reader. On success, this reader advances to where the copy
    /// ended; on failure, this reader and the trace are left unchanged.
    pub fn attempt<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        let trace_len = self.trace.map(|t| t.borrow().len());
        let mut copy = self.clone();
        match f(&mut copy) {
            Ok(v) => {
                *self = copy;
                Ok(v)
            }
            Err(e) => {
                if let (Some(trace), Some(len)) = (self.trace, trace_len) {
                    trace.borrow_mut().truncate(len);
                }
                Err(e)
            }
        }
    }

    /// Reads `n` bytes without recording them in the trace.
    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.remaining() < n {
            return Err(self.error(ErrorKind::UnexpectedEnd { needed: n }));
        }
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    /// Reads the rest of this reader's range without recording it in the trace.
    pub fn rest(&mut self) -> &'a [u8] {
        let out = &self.data[self.pos..self.end];
        self.pos = self.end;
        out
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.bytes(N)?.try_into().unwrap())
    }

    /// Reads a .NET `bool`. Only 0 and 1 are accepted so that reading and writing is lossless
    /// (the game reads any non-zero byte as true, but only writes 0 or 1).
    pub fn bool(&mut self) -> Result<bool> {
        let start = self.pos;
        let v = match self.array::<1>()?[0] {
            0 => false,
            1 => true,
            b => return Err(Error::new(start, ErrorKind::InvalidBool(b))),
        };
        self.record(start, || TraceValue::Bool(v));
        Ok(v)
    }

    pub fn i32(&mut self) -> Result<i32> {
        let start = self.pos;
        let v = i32::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::Int(v));
        Ok(v)
    }

    pub fn u32(&mut self) -> Result<u32> {
        let start = self.pos;
        let v = u32::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::UInt(v));
        Ok(v)
    }

    pub fn i16(&mut self) -> Result<i16> {
        let start = self.pos;
        let v = i16::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::Short(v));
        Ok(v)
    }

    pub fn u16(&mut self) -> Result<u16> {
        let start = self.pos;
        let v = u16::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::UShort(v));
        Ok(v)
    }

    /// Reads a .NET `char` as written with UTF-16: one code unit.
    pub fn char(&mut self) -> Result<u16> {
        let start = self.pos;
        let v = u16::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::Char(v));
        Ok(v)
    }

    pub fn i64(&mut self) -> Result<i64> {
        let start = self.pos;
        let v = i64::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::Long(v));
        Ok(v)
    }

    pub fn f32(&mut self) -> Result<f32> {
        let start = self.pos;
        let v = f32::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::Float(v.to_bits()));
        Ok(v)
    }

    pub fn f64(&mut self) -> Result<f64> {
        let start = self.pos;
        let v = f64::from_le_bytes(self.array()?);
        self.record(start, || TraceValue::Double(v.to_bits()));
        Ok(v)
    }

    pub fn f32s<const N: usize>(&mut self) -> Result<[f32; N]> {
        let mut out = [0.0; N];
        for v in &mut out {
            *v = self.f32()?;
        }
        Ok(out)
    }

    pub fn peek_i32(&self) -> Option<i32> {
        let mut copy = Reader { trace: None, ..self.clone() };
        copy.i32().ok()
    }

    /// Reads a non-negative `i32` element count.
    pub fn count(&mut self) -> Result<usize> {
        let start = self.pos;
        let n = self.i32()?;
        usize::try_from(n).map_err(|_| Error::new(start, ErrorKind::NegativeCount(n)))
    }

    /// Reads .NET's 7-bit encoded `Int32`, used as the byte-length prefix of strings. Only the
    /// shortest encoding of a non-negative value is accepted, so that reading and writing is
    /// lossless.
    fn len_7bit(&mut self) -> Result<usize> {
        let start = self.pos;
        let mut value: u32 = 0;
        for i in 0..5 {
            let b = self.array::<1>()?[0];
            let overlong = i > 0 && b == 0;
            if overlong || (i == 4 && b > 0x07) {
                break;
            }
            value |= u32::from(b & 0x7F) << (7 * i);
            if b & 0x80 == 0 {
                return Ok(value as usize);
            }
        }
        Err(Error::new(start, ErrorKind::InvalidLength))
    }

    pub fn string(&mut self) -> Result<DString> {
        let start = self.pos;
        let len = self.len_7bit()?;
        let s = DString::from_utf16le_bytes(self.bytes(len)?);
        self.record(start, || TraceValue::String(s.to_string_lossy().into_owned()));
        Ok(s)
    }

    /// If the next value is the empty marker, consumes it and returns true.
    pub fn empty_marker(&mut self) -> bool {
        if self.peek_i32() == Some(EMPTY_MARKER) {
            let start = self.pos;
            self.pos += 4;
            self.record(start, || TraceValue::Empty);
            true
        } else {
            false
        }
    }

    /// If the next value is the empty marker, consumes it and returns `None`; otherwise reads
    /// the value with `f`.
    pub fn optional<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<Option<T>> {
        if self.empty_marker() { Ok(None) } else { f(self).map(Some) }
    }

    /// Returns a reader over the next `len` bytes, and advances this reader past them.
    pub fn sub(&mut self, len: usize) -> Result<Reader<'a>> {
        if self.remaining() < len {
            return Err(self.error(ErrorKind::UnexpectedEnd { needed: len }));
        }
        let inner = Reader { end: self.pos + len, ..self.clone() };
        self.pos += len;
        Ok(inner)
    }

    /// Reads a scope header and returns its mark and a reader over its contents. This reader is
    /// advanced past the whole scope.
    pub fn scope(&mut self, expected: &'static [i32]) -> Result<(i32, Reader<'a>)> {
        let start = self.pos;
        let mark = i32::from_le_bytes(self.array()?);
        if !expected.is_empty() && !expected.contains(&mark) {
            return Err(Error::new(start, ErrorKind::UnexpectedMark { expected, found: mark }));
        }
        let len_pos = self.pos;
        let len = i64::from_le_bytes(self.array()?);
        let len_usize = usize::try_from(len)
            .ok()
            .filter(|&l| l <= self.remaining())
            .ok_or(Error::new(len_pos, ErrorKind::InvalidScopeLength(len)))?;
        self.record(start, || TraceValue::Scope { mark, len });
        let inner = Reader { pos: self.pos, end: self.pos + len_usize, ..self.clone() };
        self.pos += len_usize;
        Ok((mark, inner))
    }
}

/// A value read from a file, at an absolute offset.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceEvent {
    pub offset: usize,
    pub len: usize,
    pub value: TraceValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceValue {
    /// A scope header (mark and length).
    Scope { mark: i32, len: i64 },
    Int(i32),
    UInt(u32),
    Short(i16),
    UShort(u16),
    Long(i64),
    /// The bits of an `f32`.
    Float(u32),
    /// The bits of an `f64`.
    Double(u64),
    Char(u16),
    Bool(bool),
    String(String),
    /// The empty marker.
    Empty,
}

/// Output buffer with support for scopes whose lengths are filled in when they end.
#[derive(Default)]
pub struct Writer {
    buf: Vec<u8>,
    scopes: Vec<usize>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pos(&self) -> usize {
        self.buf.len()
    }

    pub fn finish(self) -> Vec<u8> {
        assert!(self.scopes.is_empty(), "unclosed scope");
        self.buf
    }

    pub fn bytes(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn i32(&mut self, v: i32) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn i16(&mut self, v: i16) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn u16(&mut self, v: u16) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn i64(&mut self, v: i64) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn f32(&mut self, v: f32) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn f64(&mut self, v: f64) {
        self.bytes(&v.to_le_bytes());
    }

    pub fn empty_marker(&mut self) {
        self.i32(EMPTY_MARKER);
    }

    pub fn f32s(&mut self, v: &[f32]) {
        for &x in v {
            self.f32(x);
        }
    }

    pub fn count(&mut self, n: usize) {
        self.i32(i32::try_from(n).expect("count exceeds i32::MAX"));
    }

    pub fn len_7bit(&mut self, n: usize) {
        let mut v = u32::try_from(n).expect("length exceeds u32::MAX");
        while v >= 0x80 {
            self.u8((v as u8) | 0x80);
            v >>= 7;
        }
        self.u8(v as u8);
    }

    pub fn string(&mut self, s: &DString) {
        let bytes = s.to_utf16le_bytes();
        self.len_7bit(bytes.len());
        self.bytes(&bytes);
    }

    pub fn optional<T>(&mut self, v: Option<&T>, f: impl FnOnce(&mut Self, &T)) {
        match v {
            Some(v) => f(self, v),
            None => self.empty_marker(),
        }
    }

    pub fn begin_scope(&mut self, mark: i32) {
        self.i32(mark);
        self.scopes.push(self.buf.len());
        self.i64(0);
    }

    pub fn end_scope(&mut self) {
        let len_pos = self.scopes.pop().expect("end_scope without begin_scope");
        let len = (self.buf.len() - len_pos - 8) as i64;
        self.buf[len_pos..len_pos + 8].copy_from_slice(&len.to_le_bytes());
    }

    pub fn scope(&mut self, mark: i32, f: impl FnOnce(&mut Self)) {
        self.begin_scope(mark);
        f(self);
        self.end_scope();
    }

    /// Writes an `i64` holding the absolute position of the end of what `f` writes, as the game
    /// does before each generic component field. Positions are from the start of the file, so a
    /// writer must always start at the beginning of a file.
    pub fn end_offset(&mut self, f: impl FnOnce(&mut Self)) {
        let slot = self.buf.len();
        self.i64(0);
        f(self);
        let end = self.buf.len() as i64;
        self.buf[slot..slot + 8].copy_from_slice(&end.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_7bit_round_trip() {
        for n in [0, 1, 0x7F, 0x80, 0x3FFF, 0x4000, 1 << 21, i32::MAX as usize] {
            let mut w = Writer::new();
            w.len_7bit(n);
            let buf = w.finish();
            let mut r = Reader::new(&buf);
            assert_eq!(r.len_7bit().unwrap(), n);
            assert!(r.is_empty());
        }
    }

    #[test]
    fn nested_scopes() {
        let mut w = Writer::new();
        w.scope(mark::LEVEL, |w| {
            w.i32(7);
            w.scope(mark::LAYER, |w| w.u8(1));
        });
        let buf = w.finish();
        let mut r = Reader::new(&buf);
        let (m, mut inner) = r.scope(&[mark::LEVEL]).unwrap();
        assert_eq!(m, mark::LEVEL);
        assert!(r.is_empty());
        assert_eq!(inner.i32().unwrap(), 7);
        let (_, mut layer) = inner.scope(&[mark::LAYER]).unwrap();
        assert!(layer.bool().unwrap());
        inner.expect_end().unwrap();
    }

    #[test]
    fn len_7bit_rejects_overlong() {
        for bytes in [&[0x80, 0x00][..], &[0xFF, 0xFF, 0xFF, 0xFF, 0x08], &[0x80, 0x80, 0x80, 0x80, 0x80]] {
            assert_eq!(Reader::new(bytes).len_7bit().unwrap_err().kind, ErrorKind::InvalidLength);
        }
    }

    #[test]
    fn end_offset_is_absolute() {
        let mut w = Writer::new();
        w.u8(0);
        w.end_offset(|w| w.i32(5));
        let buf = w.finish();
        assert_eq!(i64::from_le_bytes(buf[1..9].try_into().unwrap()), 13);
    }

    #[test]
    fn scope_length_is_bounded_by_parent() {
        let mut w = Writer::new();
        w.i32(mark::LEVEL);
        w.i64(100);
        let buf = w.finish();
        let err = Reader::new(&buf).scope(&[]).err().unwrap();
        assert_eq!(err.kind, ErrorKind::InvalidScopeLength(100));
    }
}
