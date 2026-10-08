//! A visitor mirroring the game's `IVisitor` for the binary format, so each hand-written component
//! serializer is translated once and drives both reading and writing.
//!
//! Most of the game's `Visit` overloads write the empty marker in place of a value equal to a
//! sentinel or to the prefab's, and check for it when reading. Those values are `Option`s here,
//! with `None` for the empty marker.

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::{Error, ErrorKind, Result};
use crate::io::{EMPTY_MARKER, Reader, Writer, mark};
use crate::string::DString;

/// A value with a fixed binary encoding ("POD" in the game's serializer).
pub(crate) trait Pod: Sized {
    /// Whether the game checks for the end of the body before reading the value.
    const BOUNDS_CHECKED: bool = true;

    fn read(r: &mut Reader) -> Result<Self>;
    fn write(&self, w: &mut Writer);

    /// A value for testing, distinct for each `n`.
    fn sample(n: u32) -> Self;
}

impl Pod for bool {
    fn sample(_n: u32) -> Self {
        true
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.bool()
    }
    fn write(&self, w: &mut Writer) {
        w.u8((*self).into());
    }
}

impl Pod for i32 {
    fn sample(n: u32) -> Self {
        n as i32 + 1
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.i32()
    }
    fn write(&self, w: &mut Writer) {
        w.i32(*self);
    }
}

impl Pod for u32 {
    fn sample(n: u32) -> Self {
        n + 1
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.u32()
    }
    fn write(&self, w: &mut Writer) {
        w.u32(*self);
    }
}

impl Pod for i16 {
    fn sample(n: u32) -> Self {
        n as i16 + 1
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.i16()
    }
    fn write(&self, w: &mut Writer) {
        w.i16(*self);
    }
}

impl Pod for u16 {
    fn sample(n: u32) -> Self {
        n as u16 + 1
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.u16()
    }
    fn write(&self, w: &mut Writer) {
        w.u16(*self);
    }
}

impl Pod for i64 {
    fn sample(n: u32) -> Self {
        i64::from(n) + 1
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.i64()
    }
    fn write(&self, w: &mut Writer) {
        w.i64(*self);
    }
}

impl Pod for f32 {
    fn sample(n: u32) -> Self {
        n as f32 + 0.5
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.f32()
    }
    fn write(&self, w: &mut Writer) {
        w.f32(*self);
    }
}

impl Pod for f64 {
    fn sample(n: u32) -> Self {
        f64::from(n) + 0.5
    }
    fn read(r: &mut Reader) -> Result<Self> {
        r.f64()
    }
    fn write(&self, w: &mut Writer) {
        w.f64(*self);
    }
}

impl Pod for DString {
    const BOUNDS_CHECKED: bool = false;

    fn sample(n: u32) -> Self {
        format!("s{n}").into()
    }

    fn read(r: &mut Reader) -> Result<Self> {
        r.string()
    }
    fn write(&self, w: &mut Writer) {
        w.string(self);
    }
}

/// Vector2, Vector3, Vector4, Quaternion and Color. The game reads each float with `Visit`,
/// which treats the empty marker as a missing value, so a float with those bits (a NaN) would
/// be misread and isn't accepted.
impl<const N: usize> Pod for [f32; N] {
    fn sample(n: u32) -> Self {
        std::array::from_fn(|i| n as f32 + 0.25 * (i + 1) as f32)
    }
    fn read(r: &mut Reader) -> Result<Self> {
        let mut out = [0.0; N];
        for v in &mut out {
            if r.peek_i32() == Some(EMPTY_MARKER) {
                return Err(r.error(ErrorKind::UnexpectedEmptyMarker));
            }
            *v = r.f32()?;
        }
        Ok(out)
    }
    fn write(&self, w: &mut Writer) {
        w.f32s(self);
    }
}

pub(crate) trait Visitor: Sized {
    /// A value the game reads with `Visit`: the empty marker or the value.
    fn value<T: Pod>(&mut self, name: &'static str, v: &mut Option<T>) -> Result<()>;

    /// A value the game reads without checking for the empty marker, such as an object
    /// reference.
    fn plain<T: Pod + Default + PartialEq>(&mut self, name: &'static str, v: &mut T) -> Result<()>;

    /// An array: `11111111`, the element count, then each element visited with `f`.
    fn array<T: Default>(
        &mut self,
        name: &'static str,
        items: &mut Vec<T>,
        f: impl FnMut(&mut Self, &mut T) -> Result<()>,
    ) -> Result<()> {
        self.sequence(mark::ARRAY, name, items, f)
    }

    /// A dictionary: `12121212`, the entry count, then each entry visited with `f`. Entries
    /// are kept in file order.
    fn dictionary<T: Default>(
        &mut self,
        name: &'static str,
        entries: &mut Vec<T>,
        f: impl FnMut(&mut Self, &mut T) -> Result<()>,
    ) -> Result<()> {
        self.sequence(mark::DICTIONARY, name, entries, f)
    }

    /// An array or dictionary with the given mark.
    fn sequence<T: Default>(
        &mut self,
        mark: i32,
        name: &'static str,
        items: &mut Vec<T>,
        f: impl FnMut(&mut Self, &mut T) -> Result<()>,
    ) -> Result<()>;

    /// A byte array: `11111111`, the length, then the bytes.
    fn bytes(&mut self, name: &'static str, v: &mut Vec<u8>) -> Result<()>;

    /// The length of a list of records that the game reads with `Visit`, ahead of the records
    /// (and often of their version). Returns the number of records: the count read, or `len`
    /// when writing. The game never writes the empty marker or a negative count here, and either
    /// fails or reads no records for them, so they aren't accepted.
    fn count(&mut self, name: &'static str, len: usize) -> Result<usize>;

    /// Visits `n` records with `f`, after a [`count`](Self::count). When reading, `items` starts
    /// empty and the records are added to it.
    fn records<T: Default>(
        &mut self,
        n: usize,
        items: &mut Vec<T>,
        mut f: impl FnMut(&mut Self, &mut T) -> Result<()>,
    ) -> Result<()> {
        for i in 0..n {
            if i == items.len() {
                items.push(T::default());
            }
            f(self, &mut items[i])?;
        }
        Ok(())
    }

    /// The number of bytes left in the body when reading, or `None` when writing. For the few
    /// serializers whose layout depends on state outside the file.
    fn remaining(&self) -> Option<usize>;

    /// A reference to another object by its GUID (`VisitReference`), with 0 for none.
    fn reference(&mut self, name: &'static str, v: &mut u32) -> Result<()> {
        self.plain(name, v)
    }

    /// An array of references (`VisitReferenceArray` and `VisitReferenceList`).
    fn references(&mut self, name: &'static str, refs: &mut Vec<u32>) -> Result<()> {
        self.array(name, refs, |v, r| v.reference(name, r))
    }
}

/// How a component body ends compared to the values its version's serializer reads. Files
/// written by older game builds can have fewer or more values under the same version.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyEnd {
    /// The body ends after the last value.
    #[default]
    Complete,
    /// The body ends after this many values. The game leaves the missing ones unchanged, as it
    /// does for empty ones, so they're read as `None` (and references as 0).
    Truncated(u32),
    /// Bytes after the last value, which the game skips.
    Trailing(#[serde(with = "crate::json::hex")] Vec<u8>),
}

pub(crate) struct ReadVisitor<'r, 'a> {
    r: &'r mut Reader<'a>,
    /// The number of values read.
    values: u32,
    /// Set to the number of values read when the body ended before the next one.
    cut: Option<u32>,
}

impl<'r, 'a> ReadVisitor<'r, 'a> {
    pub(crate) fn new(r: &'r mut Reader<'a>) -> Self {
        Self {
            r,
            values: 0,
            cut: None,
        }
    }

    /// Whether the body ended before a value of type `T`, which then keeps its default. The
    /// game checks for the end before reading each value, except strings, which it reads past it.
    fn at_end<T: Pod>(&mut self) -> Result<bool> {
        if self.cut.is_none() && self.r.remaining() > 0 {
            return Ok(false);
        }
        if !T::BOUNDS_CHECKED {
            return Err(self.r.error(ErrorKind::UnexpectedEnd { needed: 1 }));
        }
        self.cut.get_or_insert(self.values);
        Ok(true)
    }

    /// Reads the mark and element count that start an array or dictionary.
    fn sequence_start(&mut self, mark: i32) -> Result<usize> {
        // The game reads any other mark, including one cut off by the body's end, as -1 elements,
        // which fails for some arrays.
        if self.cut.is_some() {
            return Err(self.r.error(ErrorKind::UnexpectedEnd { needed: 4 }));
        }
        let mark_pos = self.r.pos();
        let found = self.r.i32()?;
        if found != mark {
            let expected: &'static [i32] = if mark == mark::ARRAY {
                &[mark::ARRAY]
            } else {
                &[mark::DICTIONARY]
            };
            let kind = ErrorKind::UnexpectedMark { expected, found };
            return Err(Error::new(mark_pos, kind));
        }
        self.r.count()
    }
}

impl Visitor for ReadVisitor<'_, '_> {
    fn remaining(&self) -> Option<usize> {
        Some(self.r.remaining())
    }

    fn value<T: Pod>(&mut self, _name: &'static str, v: &mut Option<T>) -> Result<()> {
        if self.at_end::<T>()? {
            *v = None;
            return Ok(());
        }
        *v = self.r.optional(T::read)?;
        self.values += 1;
        Ok(())
    }

    fn plain<T: Pod + Default + PartialEq>(
        &mut self,
        _name: &'static str,
        v: &mut T,
    ) -> Result<()> {
        if self.at_end::<T>()? {
            *v = T::default();
            return Ok(());
        }
        *v = T::read(self.r)?;
        self.values += 1;
        Ok(())
    }

    fn sequence<T: Default>(
        &mut self,
        mark: i32,
        _name: &'static str,
        items: &mut Vec<T>,
        mut f: impl FnMut(&mut Self, &mut T) -> Result<()>,
    ) -> Result<()> {
        let count = self.sequence_start(mark)?;
        items.clear();
        items.reserve(count.min(self.r.remaining()));
        for _ in 0..count {
            let mut item = T::default();
            f(self, &mut item)?;
            items.push(item);
        }
        Ok(())
    }

    fn bytes(&mut self, _name: &'static str, v: &mut Vec<u8>) -> Result<()> {
        let len = self.sequence_start(mark::ARRAY)?;
        // The game reads the empty marker here as a missing array, and then misreads what follows.
        if self.r.peek_i32() == Some(EMPTY_MARKER) {
            return Err(self.r.error(ErrorKind::UnexpectedEmptyMarker));
        }
        *v = self.r.byte_array(len)?.to_vec();
        Ok(())
    }

    fn count(&mut self, _name: &'static str, _len: usize) -> Result<usize> {
        if self.at_end::<i32>()? {
            return Ok(0);
        }
        if self.r.peek_i32() == Some(EMPTY_MARKER) {
            return Err(self.r.error(ErrorKind::UnexpectedEmptyMarker));
        }
        let n = self.r.count()?;
        self.values += 1;
        Ok(n)
    }
}

pub(crate) struct WriteVisitor<'w> {
    w: &'w mut Writer,
    /// The number of values written.
    values: u32,
    /// The number of values to keep, for a truncated body.
    cut: Option<u32>,
    /// Where the body is cut, while every value since then has been the one a truncated body is
    /// read with.
    cut_pos: Option<usize>,
}

impl<'w> WriteVisitor<'w> {
    pub(crate) fn new(w: &'w mut Writer, cut: Option<u32>) -> Self {
        Self {
            w,
            values: 0,
            cut,
            cut_pos: None,
        }
    }

    /// Called before each value, with whether it's the one a truncated body is read with.
    fn before_value(&mut self, missing: bool) {
        if let Some(cut) = self.cut {
            if self.values == cut {
                self.cut_pos = Some(self.w.pos());
            }
            if self.values >= cut && !missing {
                // A value after the cut is set, so the whole body is written.
                self.cut = None;
                self.cut_pos = None;
            }
        }
        self.values += 1;
    }

    fn sequence_start(&mut self, mark: i32, len: usize) {
        if self.cut.is_some_and(|cut| self.values >= cut) {
            // A truncated body can't end before an array or dictionary.
            self.cut = None;
            self.cut_pos = None;
        }
        self.w.i32(mark);
        self.w.count(len);
    }
}

impl Visitor for WriteVisitor<'_> {
    fn remaining(&self) -> Option<usize> {
        None
    }

    fn value<T: Pod>(&mut self, _name: &'static str, v: &mut Option<T>) -> Result<()> {
        self.before_value(v.is_none());
        self.w.optional(v.as_ref(), |w, v| v.write(w));
        Ok(())
    }

    fn plain<T: Pod + Default + PartialEq>(
        &mut self,
        _name: &'static str,
        v: &mut T,
    ) -> Result<()> {
        self.before_value(*v == T::default());
        v.write(self.w);
        Ok(())
    }

    fn sequence<T: Default>(
        &mut self,
        mark: i32,
        _name: &'static str,
        items: &mut Vec<T>,
        mut f: impl FnMut(&mut Self, &mut T) -> Result<()>,
    ) -> Result<()> {
        self.sequence_start(mark, items.len());
        for item in items {
            f(self, item)?;
        }
        Ok(())
    }

    fn bytes(&mut self, _name: &'static str, v: &mut Vec<u8>) -> Result<()> {
        self.sequence_start(mark::ARRAY, v.len());
        self.w.bytes(v);
        Ok(())
    }

    fn count(&mut self, _name: &'static str, len: usize) -> Result<usize> {
        self.before_value(len == 0);
        self.w.count(len);
        Ok(len)
    }
}

/// Sets every value to a distinct sample and gives every array and list one element, to test
/// serializers against the game on versions that no file has.
#[derive(Default)]
struct FillVisitor {
    values: u32,
}

impl FillVisitor {
    fn next(&mut self) -> u32 {
        self.values += 1;
        self.values
    }
}

impl Visitor for FillVisitor {
    fn remaining(&self) -> Option<usize> {
        None
    }

    fn value<T: Pod>(&mut self, _name: &'static str, v: &mut Option<T>) -> Result<()> {
        *v = Some(T::sample(self.next()));
        Ok(())
    }

    fn plain<T: Pod + Default + PartialEq>(
        &mut self,
        _name: &'static str,
        v: &mut T,
    ) -> Result<()> {
        *v = T::sample(self.next());
        Ok(())
    }

    fn sequence<T: Default>(
        &mut self,
        _mark: i32,
        _name: &'static str,
        items: &mut Vec<T>,
        mut f: impl FnMut(&mut Self, &mut T) -> Result<()>,
    ) -> Result<()> {
        if items.is_empty() {
            items.push(T::default());
        }
        for item in items {
            f(self, item)?;
        }
        Ok(())
    }

    fn bytes(&mut self, _name: &'static str, v: &mut Vec<u8>) -> Result<()> {
        *v = (0..3).map(|_| self.next() as u8).collect();
        Ok(())
    }

    fn count(&mut self, _name: &'static str, len: usize) -> Result<usize> {
        Ok(len.max(1))
    }
}

/// A component body with a hand-written serializer, translated from the game's `Visit` method.
pub(crate) trait Visit: Default + Clone {
    /// The component versions this type decodes. The game doesn't read versions above its
    /// current one, and some components use the generic field format for old versions.
    const VERSIONS: RangeInclusive<i32>;

    /// Visits the body as the game does for the given component version. Writing visits a copy
    /// of the data, and must not fail.
    fn visit(&mut self, v: &mut impl Visitor, version: i32) -> Result<()>;

    /// Reads a whole component body. Built on the heap, as some bodies have many fields.
    fn read_body(r: &mut Reader, version: i32) -> Result<(Box<Self>, BodyEnd)> {
        if !Self::VERSIONS.contains(&version) {
            return Err(r.error(ErrorKind::UnsupportedVersion(version)));
        }
        let mut data = Box::<Self>::default();
        let mut v = ReadVisitor::new(r);
        data.visit(&mut v, version)?;
        let end = match v.cut {
            Some(n) => BodyEnd::Truncated(n),
            None if r.remaining() > 0 => BodyEnd::Trailing(r.rest().to_vec()),
            None => BodyEnd::Complete,
        };
        Ok((data, end))
    }

    /// A body for the given version with every value set and one element in every list.
    fn sample(version: i32) -> Box<Self> {
        let mut data = Box::<Self>::default();
        data.visit(&mut FillVisitor::default(), version)
            .expect("the fill visitor doesn't fail");
        data
    }

    fn write_body(&self, w: &mut Writer, version: i32, end: &BodyEnd) {
        let cut = match end {
            BodyEnd::Truncated(n) => Some(*n),
            _ => None,
        };
        let mut v = WriteVisitor::new(w, cut);
        self.clone()
            .visit(&mut v, version)
            .expect("the write visitor doesn't fail");
        if let Some(pos) = v.cut_pos {
            w.truncate(pos);
        }
        if let BodyEnd::Trailing(bytes) = end {
            w.bytes(bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_id::ComponentId;
    use crate::components::ComponentData;
    use crate::generated::component_ids::NAMES;

    /// Every decoded version of every component reads back what it writes, with every value set
    /// and an element in every list.
    #[test]
    fn samples_round_trip() {
        let mut checked = 0;
        for &(id, _) in NAMES {
            let id = ComponentId(id);
            for version in 0..=64 {
                let Some(data) = ComponentData::sample(id, version) else {
                    continue;
                };
                let mut w = Writer::new();
                data.write(&mut w, version, &BodyEnd::Complete);
                let bytes = w.finish();
                let read = ComponentData::read(&mut Reader::new(&bytes), id, version);
                let read = read.unwrap_or_else(|e| panic!("{id} v{version}: {e}"));
                assert_eq!(read, (data, BodyEnd::Complete), "{id} v{version}");
                checked += 1;
            }
        }
        assert!(checked > 300, "only {checked} versions decoded");
    }

    #[test]
    fn legacy_settings_samples_round_trip() {
        use crate::legacy_settings::LegacyLevelSettings;

        for version in 0..=8 {
            let data = LegacyLevelSettings::sample(version);
            let mut w = Writer::new();
            data.write_body(&mut w, version, &BodyEnd::Complete);
            let bytes = w.finish();
            let read = LegacyLevelSettings::read_body(&mut Reader::new(&bytes), version);
            let (read, end) = read.unwrap_or_else(|e| panic!("v{version}: {e}"));
            assert_eq!((*read, end), (data, BodyEnd::Complete), "v{version}");
        }
    }
}
