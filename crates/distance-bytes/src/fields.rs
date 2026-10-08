//! The game's generic format for the fields of `SerialComponent`s, written by
//! `BinarySerializer.VisitSerialComponentData` for components without a hand-written serializer
//! (and for version 0 of most that have one).
//!
//! The body is a field count, then for each field its name, the absolute file offset of the end
//! of its value (except in level version 0), and the value or the empty marker.

use crate::component_id::ComponentId;
use crate::error::{Error, ErrorKind, Result};
use crate::generated::fields::FIELDS;
use crate::io::{Reader, Writer, mark};
use crate::string::DString;
use crate::visit::Pod;

/// How generic fields are stored, which depends on the level version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldFormat {
    /// Level version 0: no end offsets, so a value can only be skipped by knowing its type.
    NoOffsets,
    /// Each field stores the absolute file offset of its end. Used by all other levels and by
    /// non-level files.
    EndOffsets,
}

impl FieldFormat {
    pub(crate) fn for_level(version: i32) -> Self {
        if version == 0 {
            FieldFormat::NoOffsets
        } else {
            FieldFormat::EndOffsets
        }
    }
}

/// A field of a component in the game's generic format.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The field's serialized name: the C# field name without a trailing `_`, capitalized.
    pub name: DString,
    pub value: FieldValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// The same as the prefab's.
    Empty,
    Value(Value),
    /// Bytes that don't decode as the field's type, or a field whose type isn't known. The game
    /// skips these (or misreads them).
    ///
    /// The game writes nothing for values of types it can't serialize, such as references to
    /// objects; those are empty `Raw` values.
    Raw(Vec<u8>),
}

/// A field value. Its variant determines how it is written.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    /// Also used for enums.
    Int(i32),
    UInt(u32),
    Short(i16),
    UShort(u16),
    Long(i64),
    Float(f32),
    Double(f64),
    /// A UTF-16 code unit.
    Char(u16),
    String(DString),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    /// `[x, y, z, w]`.
    Quaternion([f32; 4]),
    /// `[r, g, b, a]`.
    Color([f32; 4]),
    Bounds {
        center: [f32; 3],
        extents: [f32; 3],
    },
    /// The ID of a referenced game object or component.
    Reference(u32),
    Array(Vec<Value>),
}

/// How a field's value is encoded, determined by the field's C# type as the game reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Bool,
    Int,
    UInt,
    Short,
    UShort,
    Long,
    Float,
    Double,
    Char,
    /// Stored as an `Int`.
    Enum,
    String,
    Vector2,
    Vector3,
    Vector4,
    Quaternion,
    Color,
    Bounds,
    /// Any other type is read as the ID of a referenced object.
    Reference,
    Array(&'static FieldType),
}

/// `(serialized name, C# type, encoding)` of a `SerialComponent` field. The encoding is `None`
/// for types the game can't read.
pub(crate) type FieldInfo = (&'static str, &'static str, Option<FieldType>);

/// The type of a component's field in the generic format, if the game has a field of that name.
pub fn field_type(id: ComponentId, name: &str) -> Option<FieldType> {
    fields_of(id)?
        .iter()
        .find(|&&(n, _, _)| n == name)
        .and_then(|&(_, _, ty)| ty)
}

/// The fields of a component type in the generic format, if it is a `SerialComponent`.
pub(crate) fn fields_of(id: ComponentId) -> Option<&'static [FieldInfo]> {
    let i = FIELDS.binary_search_by_key(&id.0, |&(id, _)| id).ok()?;
    Some(FIELDS[i].1)
}

/// Reads a whole component body in the generic format.
pub(crate) fn read_fields(
    r: &mut Reader,
    id: ComponentId,
    format: FieldFormat,
) -> Result<Vec<Field>> {
    let count_pos = r.pos();
    let count = r.count()?;
    // A body holding only a zero count could just as well be hand-written data, so don't claim
    // it.
    if count == 0 {
        return Err(Error::new(count_pos, ErrorKind::NoFields));
    }
    let mut fields = Vec::with_capacity(count.min(r.remaining()));
    for _ in 0..count {
        let name = r.string()?;
        let ty = name.as_str().and_then(|n| field_type(id, n));
        let value = match format {
            FieldFormat::EndOffsets => {
                let end_pos = r.pos();
                let end = r.i64()?;
                let len = usize::try_from(end)
                    .ok()
                    .and_then(|e| e.checked_sub(r.pos()))
                    .filter(|&len| len <= r.remaining())
                    .ok_or(Error::new(end_pos, ErrorKind::InvalidFieldEnd(end)))?;
                read_bounded_value(&mut r.sub(len)?, ty)
            }
            FieldFormat::NoOffsets => {
                if r.empty_marker() {
                    FieldValue::Empty
                } else {
                    // The game stops reading at a field it doesn't know.
                    let ty = ty.ok_or(r.error(ErrorKind::UnknownField))?;
                    FieldValue::Value(read_value(r, ty)?)
                }
            }
        };
        fields.push(Field { name, value });
    }
    r.expect_end()?;
    Ok(fields)
}

/// Reads a value that extends to the end of `r`.
fn read_bounded_value(r: &mut Reader, ty: Option<FieldType>) -> FieldValue {
    // The game checks for the empty marker first, and doesn't seek to the end offset after
    // finding it, so only a value of exactly the marker is empty.
    if r.remaining() == 4 && r.empty_marker() {
        return FieldValue::Empty;
    }
    if let Some(ty) = ty {
        let value = r.attempt(|r| {
            let v = read_value(r, ty)?;
            r.expect_end()?;
            Ok(v)
        });
        if let Ok(v) = value {
            return FieldValue::Value(v);
        }
    }
    FieldValue::Raw(r.rest().to_vec())
}

fn read_value(r: &mut Reader, ty: FieldType) -> Result<Value> {
    Ok(match ty {
        FieldType::Bool => Value::Bool(r.bool()?),
        FieldType::Int | FieldType::Enum => Value::Int(r.i32()?),
        FieldType::UInt => Value::UInt(r.u32()?),
        FieldType::Short => Value::Short(r.i16()?),
        FieldType::UShort => Value::UShort(r.u16()?),
        FieldType::Long => Value::Long(r.i64()?),
        FieldType::Float => Value::Float(r.f32()?),
        FieldType::Double => Value::Double(r.f64()?),
        FieldType::Char => Value::Char(r.char()?),
        FieldType::String => Value::String(r.string()?),
        FieldType::Vector2 => Value::Vector2(Pod::read(r)?),
        FieldType::Vector3 => Value::Vector3(Pod::read(r)?),
        FieldType::Vector4 => Value::Vector4(Pod::read(r)?),
        FieldType::Quaternion => Value::Quaternion(Pod::read(r)?),
        FieldType::Color => Value::Color(Pod::read(r)?),
        FieldType::Bounds => Value::Bounds {
            center: Pod::read(r)?,
            extents: Pod::read(r)?,
        },
        FieldType::Reference => Value::Reference(r.u32()?),
        FieldType::Array(element) => {
            let mark_pos = r.pos();
            let found = r.i32()?;
            if found != mark::ARRAY {
                let kind = ErrorKind::UnexpectedMark {
                    expected: &[mark::ARRAY],
                    found,
                };
                return Err(Error::new(mark_pos, kind));
            }
            let count = r.count()?;
            let mut items = Vec::with_capacity(count.min(r.remaining()));
            for _ in 0..count {
                items.push(read_value(r, *element)?);
            }
            Value::Array(items)
        }
    })
}

pub(crate) fn write_fields(w: &mut Writer, fields: &[Field], format: FieldFormat) {
    w.count(fields.len());
    for field in fields {
        w.string(&field.name);
        let value = |w: &mut Writer| match &field.value {
            FieldValue::Empty => w.empty_marker(),
            FieldValue::Value(v) => write_value(w, v),
            FieldValue::Raw(bytes) => w.bytes(bytes),
        };
        match format {
            FieldFormat::EndOffsets => w.end_offset(value),
            FieldFormat::NoOffsets => value(w),
        }
    }
}

pub(crate) fn write_value(w: &mut Writer, value: &Value) {
    match value {
        Value::Bool(v) => w.u8((*v).into()),
        Value::Int(v) => w.i32(*v),
        Value::UInt(v) => w.u32(*v),
        Value::Short(v) => w.i16(*v),
        Value::UShort(v) | Value::Char(v) => w.u16(*v),
        Value::Long(v) => w.i64(*v),
        Value::Float(v) => w.f32(*v),
        Value::Double(v) => w.f64(*v),
        Value::String(s) => w.string(s),
        Value::Vector2(v) => w.f32s(v),
        Value::Vector3(v) => w.f32s(v),
        Value::Vector4(v) | Value::Quaternion(v) | Value::Color(v) => w.f32s(v),
        Value::Bounds { center, extents } => {
            w.f32s(center);
            w.f32s(extents);
        }
        Value::Reference(v) => w.u32(*v),
        Value::Array(items) => {
            w.i32(mark::ARRAY);
            w.count(items.len());
            for item in items {
                write_value(w, item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_types_from_schema() {
        let id = ComponentId::CAR_SCREEN_TEXT_DECODE_TRIGGER;
        assert_eq!(field_type(id, "PerCharSpeed"), Some(FieldType::Float));
        assert_eq!(
            field_type(id, "AnnouncerPhrases"),
            Some(FieldType::Array(&FieldType::String))
        );
        assert_eq!(field_type(id, "NoSuchField"), None);
    }

    #[test]
    fn typed_values_round_trip_in_both_formats() {
        let id = ComponentId::CAR_SCREEN_TEXT_DECODE_TRIGGER;
        let fields = vec![
            Field {
                name: "PerCharSpeed".into(),
                value: FieldValue::Value(Value::Float(0.5)),
            },
            Field {
                name: "Text".into(),
                value: FieldValue::Empty,
            },
            Field {
                name: "AnnouncerPhrases".into(),
                value: FieldValue::Value(Value::Array(vec![Value::String("hi".into())])),
            },
        ];
        for format in [FieldFormat::NoOffsets, FieldFormat::EndOffsets] {
            let mut w = Writer::new();
            write_fields(&mut w, &fields, format);
            let buf = w.finish();
            let mut r = Reader::new(&buf);
            assert_eq!(read_fields(&mut r, id, format).unwrap(), fields);
            assert!(r.is_empty());
        }
    }

    #[test]
    fn unknown_field_without_offsets_fails() {
        let fields = vec![Field {
            name: "Nope".into(),
            value: FieldValue::Value(Value::Int(1)),
        }];
        let mut w = Writer::new();
        write_fields(&mut w, &fields, FieldFormat::NoOffsets);
        let buf = w.finish();
        let id = ComponentId::CAR_SCREEN_TEXT_DECODE_TRIGGER;
        let err = read_fields(&mut Reader::new(&buf), id, FieldFormat::NoOffsets).unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnknownField);
    }

    #[test]
    fn end_offsets_are_recomputed() {
        let fields = vec![
            Field {
                name: "A".into(),
                value: FieldValue::Value(Value::Int(7)),
            },
            Field {
                name: "B".into(),
                value: FieldValue::Empty,
            },
            Field {
                name: "C".into(),
                value: FieldValue::Value(Value::Array(vec![
                    Value::Reference(1),
                    Value::Reference(2),
                ])),
            },
        ];
        for prefix in [0, 5] {
            let mut w = Writer::new();
            w.bytes(&vec![0; prefix]);
            write_fields(&mut w, &fields, FieldFormat::EndOffsets);
            let buf = w.finish();
            let mut r = Reader::new(&buf);
            r.bytes(prefix).unwrap();
            let read = read_fields(&mut r, ComponentId::NONE, FieldFormat::EndOffsets).unwrap();
            // Without a schema entry the values aren't typed, but the structure is the same.
            assert_eq!(read.len(), 3);
            assert_eq!(read[1].value, FieldValue::Empty);
            let mut w2 = Writer::new();
            w2.bytes(&vec![0; prefix]);
            write_fields(&mut w2, &read, FieldFormat::EndOffsets);
            assert_eq!(w2.finish(), buf);
        }
    }
}
