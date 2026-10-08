//! A `Serializer` and `Deserializer` wrapper for what serde_json can't do on its own:
//!
//! - JSON has no NaN or infinities (serde_json writes them as `null`), so non-finite floats are
//!   written as strings, and read back from them where a float is expected.
//! - serde_json reads an `f32` by parsing an `f64` and rounding it, so the few `f32`s for which
//!   that differs from parsing the `f32` directly are written with more digits.
//! - Objects can nest hundreds of levels deep, so the stack grows as needed at every nested
//!   value.

use std::fmt;

use serde::de::{self, DeserializeSeed, Deserializer, Visitor};
use serde::ser::{self, Serialize, Serializer};

use crate::stack::grow;

/// The string for a non-finite `f32`: `NaN` for the canonical quiet NaN, otherwise the NaN with
/// its bits, like `NaN:0xffc00000`.
pub(crate) fn f32_string(v: f32) -> String {
    match v {
        f32::INFINITY => "Infinity".into(),
        f32::NEG_INFINITY => "-Infinity".into(),
        _ if v.to_bits() == f32::NAN.to_bits() => "NaN".into(),
        _ => format!("NaN:{:#010x}", v.to_bits()),
    }
}

pub(crate) fn f64_string(v: f64) -> String {
    match v {
        f64::INFINITY => "Infinity".into(),
        f64::NEG_INFINITY => "-Infinity".into(),
        _ if v.to_bits() == f64::NAN.to_bits() => "NaN".into(),
        _ => format!("NaN:{:#018x}", v.to_bits()),
    }
}

pub(crate) fn parse_f32(s: &str) -> Option<f32> {
    match s {
        "Infinity" => Some(f32::INFINITY),
        "-Infinity" => Some(f32::NEG_INFINITY),
        "NaN" => Some(f32::NAN),
        _ => {
            let bits = u32::from_str_radix(s.strip_prefix("NaN:0x")?, 16).ok()?;
            Some(f32::from_bits(bits)).filter(|v| v.is_nan())
        }
    }
}

pub(crate) fn parse_f64(s: &str) -> Option<f64> {
    match s {
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        "NaN" => Some(f64::NAN),
        _ => {
            let bits = u64::from_str_radix(s.strip_prefix("NaN:0x")?, 16).ok()?;
            Some(f64::from_bits(bits)).filter(|v| v.is_nan())
        }
    }
}

/// The `f64` to write in place of a finite `f32` whose shortest decimal would be read back
/// as another `f32` (rounding first to `f64` and then to `f32` can round across the midpoint
/// between two `f32`s), if any. It is the closest `f64` to the shortest decimal that is read
/// back correctly.
fn f32_with_more_digits(v: f32) -> Option<f64> {
    let reads_back = |x: f64| (x as f32).to_bits() == v.to_bits();
    let mut buf = ryu::Buffer::new();
    if buf.format_finite(v).parse().is_ok_and(reads_back) {
        return None;
    }
    // 17 digits identify an `f64`, so the last candidate is `v` exactly.
    (1..=17)
        .filter_map(|digits| format!("{v:.*e}", digits - 1).parse().ok())
        .find(|&x| reads_back(x))
}

// --- Serializer ---

pub(crate) struct Ser<S>(pub S);

/// A value serialized through [`Ser`].
struct Wrap<'a, T: ?Sized>(&'a T);

impl<T: Serialize + ?Sized> Serialize for Wrap<'_, T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        grow(|| self.0.serialize(Ser(s)))
    }
}

macro_rules! forward_ser {
    ($($method:ident($($arg:ident: $ty:ty),*);)*) => {
        $(fn $method(self, $($arg: $ty),*) -> Result<S::Ok, S::Error> {
            self.0.$method($($arg),*)
        })*
    };
}

impl<S: Serializer> Serializer for Ser<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = Ser<S::SerializeSeq>;
    type SerializeTuple = Ser<S::SerializeTuple>;
    type SerializeTupleStruct = Ser<S::SerializeTupleStruct>;
    type SerializeTupleVariant = Ser<S::SerializeTupleVariant>;
    type SerializeMap = Ser<S::SerializeMap>;
    type SerializeStruct = Ser<S::SerializeStruct>;
    type SerializeStructVariant = Ser<S::SerializeStructVariant>;

    forward_ser! {
        serialize_bool(v: bool);
        serialize_i8(v: i8);
        serialize_i16(v: i16);
        serialize_i32(v: i32);
        serialize_i64(v: i64);
        serialize_i128(v: i128);
        serialize_u8(v: u8);
        serialize_u16(v: u16);
        serialize_u32(v: u32);
        serialize_u64(v: u64);
        serialize_u128(v: u128);
        serialize_char(v: char);
        serialize_str(v: &str);
        serialize_bytes(v: &[u8]);
        serialize_none();
        serialize_unit();
        serialize_unit_struct(name: &'static str);
        serialize_unit_variant(name: &'static str, index: u32, variant: &'static str);
    }

    fn serialize_f32(self, v: f32) -> Result<S::Ok, S::Error> {
        if !v.is_finite() {
            self.0.serialize_str(&f32_string(v))
        } else if let Some(x) = f32_with_more_digits(v) {
            self.0.serialize_f64(x)
        } else {
            self.0.serialize_f32(v)
        }
    }

    fn serialize_f64(self, v: f64) -> Result<S::Ok, S::Error> {
        if v.is_finite() {
            self.0.serialize_f64(v)
        } else {
            self.0.serialize_str(&f64_string(v))
        }
    }

    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<S::Ok, S::Error> {
        self.0.serialize_some(&Wrap(v))
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        self.0.serialize_newtype_struct(name, &Wrap(v))
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        self.0
            .serialize_newtype_variant(name, index, variant, &Wrap(v))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, S::Error> {
        self.0.serialize_seq(len).map(Ser)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, S::Error> {
        self.0.serialize_tuple(len).map(Ser)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, S::Error> {
        self.0.serialize_tuple_struct(name, len).map(Ser)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, S::Error> {
        self.0
            .serialize_tuple_variant(name, index, variant, len)
            .map(Ser)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, S::Error> {
        self.0.serialize_map(len).map(Ser)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, S::Error> {
        self.0.serialize_struct(name, len).map(Ser)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, S::Error> {
        self.0
            .serialize_struct_variant(name, index, variant, len)
            .map(Ser)
    }

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

macro_rules! wrap_compound {
    ($($trait:ident { $method:ident($($name:ident: $ty:ty),*) })*) => {
        $(impl<S: ser::$trait> ser::$trait for Ser<S> {
            type Ok = S::Ok;
            type Error = S::Error;

            fn $method<T: Serialize + ?Sized>(
                &mut self,
                $($name: $ty,)*
                v: &T,
            ) -> Result<(), S::Error> {
                self.0.$method($($name,)* &Wrap(v))
            }

            fn end(self) -> Result<S::Ok, S::Error> {
                self.0.end()
            }
        })*
    };
}

wrap_compound! {
    SerializeSeq { serialize_element() }
    SerializeTuple { serialize_element() }
    SerializeTupleStruct { serialize_field() }
    SerializeTupleVariant { serialize_field() }
    SerializeStructVariant { serialize_field(key: &'static str) }
}

impl<S: ser::SerializeStruct> ser::SerializeStruct for Ser<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), S::Error> {
        self.0.serialize_field(key, &Wrap(v))
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), S::Error> {
        self.0.skip_field(key)
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::SerializeMap> ser::SerializeMap for Ser<S> {
    type Ok = S::Ok;
    type Error = S::Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<(), S::Error> {
        self.0.serialize_key(&Wrap(k))
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), S::Error> {
        self.0.serialize_value(&Wrap(v))
    }

    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

// --- Deserializer ---

pub(crate) struct De<D>(pub D);

/// A visitor whose nested values are deserialized through [`De`].
struct WrapVisitor<V>(V);

/// A seed whose value is deserialized through [`De`].
struct WrapSeed<T>(T);

impl<'de, T: DeserializeSeed<'de>> DeserializeSeed<'de> for WrapSeed<T> {
    type Value = T::Value;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<T::Value, D::Error> {
        grow(|| self.0.deserialize(De(d)))
    }
}

macro_rules! forward_de {
    ($($method:ident($($arg:ident: $ty:ty),*);)*) => {
        $(fn $method<V: Visitor<'de>>(self, $($arg: $ty,)* visitor: V) -> Result<V::Value, D::Error> {
            self.0.$method($($arg,)* WrapVisitor(visitor))
        })*
    };
}

impl<'de, D: Deserializer<'de>> Deserializer<'de> for De<D> {
    type Error = D::Error;

    forward_de! {
        deserialize_any();
        deserialize_bool();
        deserialize_i8();
        deserialize_i16();
        deserialize_i32();
        deserialize_i64();
        deserialize_i128();
        deserialize_u8();
        deserialize_u16();
        deserialize_u32();
        deserialize_u64();
        deserialize_u128();
        deserialize_char();
        deserialize_str();
        deserialize_string();
        deserialize_bytes();
        deserialize_byte_buf();
        deserialize_option();
        deserialize_unit();
        deserialize_unit_struct(name: &'static str);
        deserialize_newtype_struct(name: &'static str);
        deserialize_seq();
        deserialize_tuple(len: usize);
        deserialize_tuple_struct(name: &'static str, len: usize);
        deserialize_map();
        deserialize_struct(name: &'static str, fields: &'static [&'static str]);
        deserialize_enum(name: &'static str, variants: &'static [&'static str]);
        deserialize_identifier();
        deserialize_ignored_any();
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_any(FloatVisitor {
            inner: visitor,
            f64: false,
        })
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, D::Error> {
        self.0.deserialize_any(FloatVisitor {
            inner: visitor,
            f64: true,
        })
    }

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

/// Reads a float from a number or from one of the strings written for non-finite values.
struct FloatVisitor<V> {
    inner: V,
    f64: bool,
}

macro_rules! forward_visit {
    ($($method:ident($ty:ty);)*) => {
        $(fn $method<E: de::Error>(self, v: $ty) -> Result<V::Value, E> {
            self.inner.$method(v)
        })*
    };
}

impl<'de, V: Visitor<'de>> Visitor<'de> for FloatVisitor<V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.expecting(f)
    }

    forward_visit! {
        visit_bool(bool);
        visit_i64(i64);
        visit_u64(u64);
        visit_f64(f64);
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<V::Value, E> {
        if self.f64 {
            if let Some(x) = parse_f64(v) {
                return self.inner.visit_f64(x);
            }
        } else if let Some(x) = parse_f32(v) {
            return self.inner.visit_f32(x);
        }
        self.inner.visit_str(v)
    }

    fn visit_unit<E: de::Error>(self) -> Result<V::Value, E> {
        self.inner.visit_unit()
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<V::Value, A::Error> {
        self.inner.visit_seq(WrapAccess(seq))
    }

    fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<V::Value, A::Error> {
        self.inner.visit_map(WrapAccess(map))
    }
}

impl<'de, V: Visitor<'de>> Visitor<'de> for WrapVisitor<V> {
    type Value = V::Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.expecting(f)
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<V::Value, E> {
        self.0.visit_bool(v)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<V::Value, E> {
        self.0.visit_i64(v)
    }

    fn visit_i128<E: de::Error>(self, v: i128) -> Result<V::Value, E> {
        self.0.visit_i128(v)
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<V::Value, E> {
        self.0.visit_u64(v)
    }

    fn visit_u128<E: de::Error>(self, v: u128) -> Result<V::Value, E> {
        self.0.visit_u128(v)
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<V::Value, E> {
        self.0.visit_f64(v)
    }

    fn visit_char<E: de::Error>(self, v: char) -> Result<V::Value, E> {
        self.0.visit_char(v)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<V::Value, E> {
        self.0.visit_str(v)
    }

    fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<V::Value, E> {
        self.0.visit_borrowed_str(v)
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<V::Value, E> {
        self.0.visit_string(v)
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<V::Value, E> {
        self.0.visit_bytes(v)
    }

    fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<V::Value, E> {
        self.0.visit_borrowed_bytes(v)
    }

    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<V::Value, E> {
        self.0.visit_byte_buf(v)
    }

    fn visit_none<E: de::Error>(self) -> Result<V::Value, E> {
        self.0.visit_none()
    }

    fn visit_some<D2: Deserializer<'de>>(self, d: D2) -> Result<V::Value, D2::Error> {
        self.0.visit_some(De(d))
    }

    fn visit_unit<E: de::Error>(self) -> Result<V::Value, E> {
        self.0.visit_unit()
    }

    fn visit_newtype_struct<D2: Deserializer<'de>>(self, d: D2) -> Result<V::Value, D2::Error> {
        self.0.visit_newtype_struct(De(d))
    }

    fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<V::Value, A::Error> {
        self.0.visit_seq(WrapAccess(seq))
    }

    fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<V::Value, A::Error> {
        self.0.visit_map(WrapAccess(map))
    }

    fn visit_enum<A: de::EnumAccess<'de>>(self, data: A) -> Result<V::Value, A::Error> {
        self.0.visit_enum(WrapAccess(data))
    }
}

/// A `SeqAccess`, `MapAccess`, `EnumAccess` or `VariantAccess` whose values are deserialized
/// through [`De`].
struct WrapAccess<A>(A);

impl<'de, A: de::SeqAccess<'de>> de::SeqAccess<'de> for WrapAccess<A> {
    type Error = A::Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, A::Error> {
        self.0.next_element_seed(WrapSeed(seed))
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

impl<'de, A: de::MapAccess<'de>> de::MapAccess<'de> for WrapAccess<A> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, A::Error> {
        self.0.next_key_seed(WrapSeed(seed))
    }

    fn next_value_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<T::Value, A::Error> {
        self.0.next_value_seed(WrapSeed(seed))
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

impl<'de, A: de::EnumAccess<'de>> de::EnumAccess<'de> for WrapAccess<A> {
    type Error = A::Error;
    type Variant = WrapAccess<A::Variant>;

    fn variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<(T::Value, Self::Variant), A::Error> {
        let (value, variant) = self.0.variant_seed(WrapSeed(seed))?;
        Ok((value, WrapAccess(variant)))
    }
}

impl<'de, A: de::VariantAccess<'de>> de::VariantAccess<'de> for WrapAccess<A> {
    type Error = A::Error;

    fn unit_variant(self) -> Result<(), A::Error> {
        self.0.unit_variant()
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, A::Error> {
        self.0.newtype_variant_seed(WrapSeed(seed))
    }

    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, A::Error> {
        self.0.tuple_variant(len, WrapVisitor(visitor))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, A::Error> {
        self.0.struct_variant(fields, WrapVisitor(visitor))
    }
}
