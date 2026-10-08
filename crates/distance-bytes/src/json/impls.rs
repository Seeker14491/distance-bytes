//! Serde implementations that don't follow from the types' structure.

use std::fmt;

use serde::de::{
    self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Unexpected,
};
use serde::de::{Error as _, Visitor};
use serde::ser::{Error as _, SerializeMap, SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use super::adapter::{parse_f32, parse_f64};
use super::hex::{Hex, HexVisitor};
use crate::component_id::ComponentId;
use crate::components::ComponentData;
use crate::fields::{Field, FieldType, FieldValue, Value, field_type};
use crate::io::mark;
use crate::legacy_settings::LegacyLevelSettings;
use crate::string::DString;
use crate::tree::{
    Component, ComponentBody, ComponentKind, GameObject, LayerFlags, LegacySettingsBody,
    LevelSettingsData,
};
use crate::visit::BodyEnd;

// --- Strings and IDs ---

impl Serialize for DString {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.as_str() {
            Some(text) => s.serialize_str(text),
            None => {
                let mut m = s.serialize_map(Some(1))?;
                m.serialize_entry("utf16le", &Hex(&self.to_utf16le_bytes()))?;
                m.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for DString {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(DStringVisitor)
    }
}

struct DStringVisitor;

impl<'de> Visitor<'de> for DStringVisitor {
    type Value = DString;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(r#"a string or {"utf16le": hex}"#)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<DString, E> {
        Ok(v.into())
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<DString, E> {
        Ok(v.into())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<DString, A::Error> {
        if map.next_key::<String>()?.as_deref() != Some("utf16le") {
            return Err(A::Error::invalid_type(Unexpected::Map, &self));
        }
        let bytes = map.next_value_seed(HexSeed)?;
        end_map(&mut map, &self)?;
        Ok(DString::from_utf16le_bytes(&bytes))
    }
}

/// Fails if the map has entries left.
fn end_map<'de, A: MapAccess<'de>>(map: &mut A, exp: &dyn de::Expected) -> Result<(), A::Error> {
    match map.next_key::<IgnoredAny>()? {
        Some(_) => Err(A::Error::invalid_type(Unexpected::Map, exp)),
        None => Ok(()),
    }
}

struct HexSeed;

impl<'de> DeserializeSeed<'de> for HexSeed {
    type Value = Vec<u8>;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Vec<u8>, D::Error> {
        d.deserialize_str(HexVisitor)
    }
}

/// An `Option<Vec<u8>>` field written as hex.
fn hex_option<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<u8>>, D::Error> {
    HexSeed.deserialize(d).map(Some)
}

impl Serialize for ComponentId {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.name() {
            Some(name) => s.serialize_str(name),
            None => s.serialize_i32(self.0),
        }
    }
}

impl<'de> Deserialize<'de> for ComponentId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(ComponentIdVisitor)
    }
}

struct ComponentIdVisitor;

impl Visitor<'_> for ComponentIdVisitor {
    type Value = ComponentId;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a component type name or number")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<ComponentId, E> {
        ComponentId::from_name(v)
            .ok_or_else(|| E::custom(format_args!("unknown component type {v:?}")))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<ComponentId, E> {
        i32::try_from(v)
            .map(ComponentId)
            .map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<ComponentId, E> {
        i32::try_from(v)
            .map(ComponentId)
            .map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &self))
    }
}

// --- Level settings and layers ---

impl Serialize for LevelSettingsData {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        const NAME: &str = "LevelSettingsData";
        match self {
            LevelSettingsData::GameObject(obj) => {
                s.serialize_newtype_variant(NAME, 0, "game_object", obj)
            }
            LevelSettingsData::Legacy { version, body } => {
                let legacy = match body {
                    LegacySettingsBody::Data { data, end } => LegacyRef {
                        version: *version,
                        data: Some(data),
                        end: Some(end).filter(|e| **e != BodyEnd::Complete),
                        raw: None,
                    },
                    LegacySettingsBody::Raw(bytes) => LegacyRef {
                        version: *version,
                        data: None,
                        end: None,
                        raw: Some(Hex(bytes)),
                    },
                };
                s.serialize_newtype_variant(NAME, 1, "legacy", &legacy)
            }
        }
    }
}

#[derive(Serialize)]
struct LegacyRef<'a> {
    version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<&'a LegacyLevelSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<&'a BodyEnd>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw: Option<Hex<'a>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum LevelSettingsRepr {
    GameObject(GameObject),
    Legacy(LegacyRepr),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyRepr {
    version: i32,
    data: Option<Box<LegacyLevelSettings>>,
    end: Option<BodyEnd>,
    #[serde(default, deserialize_with = "hex_option")]
    raw: Option<Vec<u8>>,
}

impl<'de> Deserialize<'de> for LevelSettingsData {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let legacy = match LevelSettingsRepr::deserialize(d)? {
            LevelSettingsRepr::GameObject(obj) => return Ok(LevelSettingsData::GameObject(obj)),
            LevelSettingsRepr::Legacy(legacy) => legacy,
        };
        let body = match (legacy.data, legacy.raw, legacy.end) {
            (Some(data), None, end) => LegacySettingsBody::Data {
                data,
                end: end.unwrap_or_default(),
            },
            (None, Some(raw), None) => LegacySettingsBody::Raw(raw),
            (None, Some(_), Some(_)) => {
                return Err(D::Error::custom(r#""end" goes only with "data""#));
            }
            _ => {
                return Err(D::Error::custom(
                    r#"legacy settings need one of "data" and "raw""#,
                ));
            }
        };
        Ok(LevelSettingsData::Legacy {
            version: legacy.version,
            body,
        })
    }
}

impl LayerFlags {
    pub(crate) fn is_absent(&self) -> bool {
        *self == LayerFlags::Absent
    }

    pub(crate) fn absent() -> Self {
        LayerFlags::Absent
    }
}

/// Flags are written with their layer version; version 1, which the game writes, is left out.
impl Serialize for LayerFlags {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        const NAME: &str = "LayerFlags";
        match *self {
            LayerFlags::Absent => s.serialize_none(),
            LayerFlags::V0 {
                unfrozen,
                active,
                visible,
            } => {
                let mut st = s.serialize_struct(NAME, 4)?;
                st.serialize_field("version", &0)?;
                st.serialize_field("unfrozen", &unfrozen)?;
                st.serialize_field("active", &active)?;
                st.serialize_field("visible", &visible)?;
                st.end()
            }
            LayerFlags::V1 {
                version,
                active,
                frozen,
                visible,
                is_background,
            } => {
                let mut st = s.serialize_struct(NAME, 5)?;
                if version != 1 {
                    st.serialize_field("version", &version)?;
                }
                st.serialize_field("active", &active)?;
                st.serialize_field("frozen", &frozen)?;
                st.serialize_field("visible", &visible)?;
                st.serialize_field("is_background", &is_background)?;
                st.end()
            }
            LayerFlags::None { version } => {
                let mut st = s.serialize_struct(NAME, 1)?;
                st.serialize_field("version", &version)?;
                st.end()
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerFlagsRepr {
    version: Option<i32>,
    unfrozen: Option<bool>,
    active: Option<bool>,
    frozen: Option<bool>,
    visible: Option<bool>,
    is_background: Option<bool>,
}

/// Missing flags of a version 1 layer are the game's defaults for a new layer.
impl<'de> Deserialize<'de> for LayerFlags {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let Some(r) = Option::<LayerFlagsRepr>::deserialize(d)? else {
            return Ok(LayerFlags::Absent);
        };
        let LayerFlags::V1 {
            active: default_active,
            frozen: default_frozen,
            visible: default_visible,
            is_background: default_background,
            ..
        } = LayerFlags::DEFAULT
        else {
            unreachable!()
        };
        let version = r.version.unwrap_or(1);
        let unexpected = |names: &[(&str, bool)]| match names.iter().find(|(_, set)| *set) {
            Some((name, _)) => Err(D::Error::custom(format_args!(
                "layer version {version} has no flag {name:?}"
            ))),
            None => Ok(()),
        };
        Ok(match version {
            0 => {
                unexpected(&[
                    ("frozen", r.frozen.is_some()),
                    ("is_background", r.is_background.is_some()),
                ])?;
                let required = |v: Option<bool>, name| {
                    v.ok_or_else(|| D::Error::custom(format_args!("missing flag {name:?}")))
                };
                LayerFlags::V0 {
                    unfrozen: required(r.unfrozen, "unfrozen")?,
                    active: required(r.active, "active")?,
                    visible: required(r.visible, "visible")?,
                }
            }
            1.. => {
                unexpected(&[("unfrozen", r.unfrozen.is_some())])?;
                LayerFlags::V1 {
                    version,
                    active: r.active.unwrap_or(default_active),
                    frozen: r.frozen.unwrap_or(default_frozen),
                    visible: r.visible.unwrap_or(default_visible),
                    is_background: r.is_background.unwrap_or(default_background),
                }
            }
            _ => {
                unexpected(&[
                    ("unfrozen", r.unfrozen.is_some()),
                    ("active", r.active.is_some()),
                    ("frozen", r.frozen.is_some()),
                    ("visible", r.visible.is_some()),
                    ("is_background", r.is_background.is_some()),
                ])?;
                LayerFlags::None { version }
            }
        })
    }
}

// --- Components ---

fn mark_name(mark: i32) -> Option<&'static str> {
    match mark {
        mark::SERIALIZABLE => Some("built_in"),
        mark::SERIAL_COMPONENT => Some("serial"),
        mark::SERIALIZABLE_ALT => Some("added"),
        _ => None,
    }
}

struct Mark(i32);

impl Serialize for Mark {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match mark_name(self.0) {
            Some(name) => s.serialize_str(name),
            None => s.serialize_i32(self.0),
        }
    }
}

impl<'de> Deserialize<'de> for Mark {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let name = String::deserialize(d)?;
        [
            mark::SERIALIZABLE,
            mark::SERIAL_COMPONENT,
            mark::SERIALIZABLE_ALT,
        ]
        .into_iter()
        .find(|&m| mark_name(m) == Some(&name))
        .map(Mark)
        .ok_or_else(|| D::Error::unknown_variant(&name, &["built_in", "serial", "added"]))
    }
}

struct DataRef<'a>(&'a ComponentData);

impl Serialize for DataRef<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize_body(s)
    }
}

impl Serialize for Component {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        let id = match &self.kind {
            ComponentKind::Known { mark, id, version } => {
                m.serialize_entry("type", id)?;
                m.serialize_entry("version", version)?;
                if *mark != id.written_mark() {
                    m.serialize_entry("mark", &Mark(*mark))?;
                }
                Some(*id)
            }
            ComponentKind::Unknown { name } => {
                m.serialize_entry("unknown_type", name)?;
                None
            }
        };
        m.serialize_entry("guid", &self.guid)?;
        match &self.body {
            ComponentBody::Empty => {}
            ComponentBody::Transform(t) => {
                if id != Some(ComponentId::TRANSFORM) {
                    return Err(S::Error::custom(
                        "a Transform body on another component type",
                    ));
                }
                m.serialize_entry("data", t)?;
            }
            ComponentBody::Fields(fields) => m.serialize_entry("fields", &FieldsRef(fields))?,
            ComponentBody::Data { data, end } => {
                if id != Some(data.id()) {
                    return Err(S::Error::custom(format_args!(
                        "a {} body on another component type",
                        data.id()
                    )));
                }
                m.serialize_entry("data", &DataRef(data))?;
                if *end != BodyEnd::Complete {
                    m.serialize_entry("end", end)?;
                }
            }
            ComponentBody::Raw(bytes) => m.serialize_entry("raw", &Hex(bytes))?,
        }
        m.end()
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum ComponentKey {
    Type,
    UnknownType,
    Version,
    Mark,
    Guid,
    Data,
    Fields,
    Raw,
    End,
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_map(ComponentVisitor)
    }
}

struct ComponentVisitor;

/// Sets a value that may appear only once.
fn set_once<T, E: de::Error>(slot: &mut Option<T>, value: T, name: &'static str) -> Result<(), E> {
    if slot.is_some() {
        return Err(E::duplicate_field(name));
    }
    *slot = Some(value);
    Ok(())
}

impl<'de> Visitor<'de> for ComponentVisitor {
    type Value = Component;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a component")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Component, A::Error> {
        let mut id = None;
        let mut unknown_type = None;
        let mut version = None;
        let mut mark = None;
        let mut guid = None;
        let mut body = None;
        let mut end = None;
        while let Some(key) = map.next_key()? {
            match key {
                ComponentKey::Type => set_once(&mut id, map.next_value()?, "type")?,
                ComponentKey::UnknownType => {
                    set_once(&mut unknown_type, map.next_value()?, "unknown_type")?
                }
                ComponentKey::Version => set_once(&mut version, map.next_value()?, "version")?,
                ComponentKey::Mark => set_once(&mut mark, map.next_value::<Mark>()?.0, "mark")?,
                ComponentKey::Guid => set_once(&mut guid, map.next_value()?, "guid")?,
                ComponentKey::Data | ComponentKey::Fields => {
                    let is_data = matches!(key, ComponentKey::Data);
                    let name = if is_data { "data" } else { "fields" };
                    let Some(id) = id else {
                        return Err(A::Error::custom(format_args!(
                            "\"type\" must come before {name:?}"
                        )));
                    };
                    if body.is_some() {
                        return Err(A::Error::custom(
                            r#"only one of "data", "fields" and "raw" is allowed"#,
                        ));
                    }
                    body = Some(if !is_data {
                        ComponentBody::Fields(map.next_value_seed(FieldsSeed(id))?)
                    } else if id == ComponentId::TRANSFORM {
                        ComponentBody::Transform(map.next_value()?)
                    } else {
                        ComponentBody::Data {
                            data: map.next_value_seed(DataSeed(id))?,
                            end: BodyEnd::Complete,
                        }
                    });
                }
                ComponentKey::Raw => {
                    if body.is_some() {
                        return Err(A::Error::custom(
                            r#"only one of "data", "fields" and "raw" is allowed"#,
                        ));
                    }
                    body = Some(ComponentBody::Raw(map.next_value_seed(HexSeed)?));
                }
                ComponentKey::End => set_once(&mut end, map.next_value()?, "end")?,
            }
        }
        let mut body = body.unwrap_or(ComponentBody::Empty);
        if let Some(e) = end {
            match &mut body {
                ComponentBody::Data { end, .. } => *end = e,
                _ => {
                    return Err(A::Error::custom(
                        r#""end" goes only with a typed "data" body"#,
                    ));
                }
            }
        }
        let kind = match (id, unknown_type) {
            (Some(id), None) => {
                let version = match (version, &body) {
                    (Some(v), _) => v,
                    (None, ComponentBody::Data { data, .. }) => data.latest_version(),
                    (None, body) => default_version(id, body).map_err(A::Error::custom)?,
                };
                ComponentKind::Known {
                    mark: mark.unwrap_or(id.written_mark()),
                    id,
                    version,
                }
            }
            (None, Some(name)) => {
                if version.is_some() || mark.is_some() {
                    return Err(A::Error::custom(
                        "a component of unknown type has no version or mark",
                    ));
                }
                if !matches!(body, ComponentBody::Empty | ComponentBody::Raw(_)) {
                    return Err(A::Error::custom(
                        "a component of unknown type has only a raw body",
                    ));
                }
                ComponentKind::Unknown { name }
            }
            (None, None) => return Err(A::Error::missing_field("type")),
            (Some(_), Some(_)) => {
                return Err(A::Error::custom(
                    r#"only one of "type" and "unknown_type" is allowed"#,
                ));
            }
        };
        Ok(Component {
            kind,
            guid: guid.unwrap_or(0),
            body,
        })
    }
}

/// The version of a component of type `id` whose body isn't typed data.
fn default_version(id: ComponentId, body: &ComponentBody) -> Result<i32, String> {
    let Some(version) = id.written_version() else {
        return Err(format!(
            "{id} has no version the game writes; give a \"version\""
        ));
    };
    // The game would read a generic-format body as typed data at this version.
    if matches!(body, ComponentBody::Fields(_)) && ComponentData::decodes(id, version) {
        return Err(format!(
            "{id} is written as typed \"data\" at version {version}; give the \"version\" of \
             these \"fields\""
        ));
    }
    Ok(version)
}

struct DataSeed(ComponentId);

impl<'de> DeserializeSeed<'de> for DataSeed {
    type Value = ComponentData;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<ComponentData, D::Error> {
        ComponentData::deserialize_body(self.0, d)
    }
}

// --- Generic fields ---

/// Fields as an object of their names, or as a list of `[name, value]` pairs when names repeat
/// or aren't valid UTF-16.
struct FieldsRef<'a>(&'a [Field]);

impl Serialize for FieldsRef<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut names: Vec<_> = self.0.iter().filter_map(|f| f.name.as_str()).collect();
        names.sort_unstable();
        let as_object = names.len() == self.0.len() && names.windows(2).all(|w| w[0] != w[1]);
        if as_object {
            let mut m = s.serialize_map(Some(self.0.len()))?;
            for f in self.0 {
                m.serialize_entry(&f.name, &f.value)?;
            }
            m.end()
        } else {
            s.collect_seq(self.0.iter().map(|f| (&f.name, &f.value)))
        }
    }
}

struct FieldsSeed(ComponentId);

impl<'de> DeserializeSeed<'de> for FieldsSeed {
    type Value = Vec<Field>;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Vec<Field>, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for FieldsSeed {
    type Value = Vec<Field>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an object of fields or a list of [name, value] pairs")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Vec<Field>, A::Error> {
        let mut fields = Vec::new();
        while let Some(name) = map.next_key::<DString>()? {
            let ty = name.as_str().and_then(|n| field_type(self.0, n));
            let value = map.next_value_seed(FieldValueSeed(ty))?;
            fields.push(Field { name, value });
        }
        Ok(fields)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<Field>, A::Error> {
        let mut fields = Vec::new();
        while let Some(field) = seq.next_element_seed(FieldPairSeed(self.0))? {
            fields.push(field);
        }
        Ok(fields)
    }
}

struct FieldPairSeed(ComponentId);

impl<'de> DeserializeSeed<'de> for FieldPairSeed {
    type Value = Field;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Field, D::Error> {
        d.deserialize_tuple(2, self)
    }
}

impl<'de> Visitor<'de> for FieldPairSeed {
    type Value = Field;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a [name, value] pair")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Field, A::Error> {
        let name: DString = seq
            .next_element()?
            .ok_or_else(|| A::Error::invalid_length(0, &self))?;
        let ty = name.as_str().and_then(|n| field_type(self.0, n));
        let value = seq
            .next_element_seed(FieldValueSeed(ty))?
            .ok_or_else(|| A::Error::invalid_length(1, &self))?;
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(A::Error::invalid_length(3, &self));
        }
        Ok(Field { name, value })
    }
}

impl Serialize for FieldValue {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            FieldValue::Empty => s.serialize_unit(),
            FieldValue::Value(v) => v.serialize(s),
            FieldValue::Raw(bytes) => {
                let mut m = s.serialize_map(Some(1))?;
                m.serialize_entry("raw", &Hex(bytes))?;
                m.end()
            }
        }
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Bool(v) => s.serialize_bool(*v),
            Value::Int(v) => s.serialize_i32(*v),
            Value::UInt(v) | Value::Reference(v) => s.serialize_u32(*v),
            Value::Short(v) => s.serialize_i16(*v),
            Value::UShort(v) | Value::Char(v) => s.serialize_u16(*v),
            Value::Long(v) => s.serialize_i64(*v),
            Value::Float(v) => s.serialize_f32(*v),
            Value::Double(v) => s.serialize_f64(*v),
            Value::String(v) => v.serialize(s),
            Value::Vector2(v) => v.serialize(s),
            Value::Vector3(v) => v.serialize(s),
            Value::Vector4(v) | Value::Quaternion(v) | Value::Color(v) => v.serialize(s),
            Value::Bounds { center, extents } => {
                let mut st = s.serialize_struct("Bounds", 2)?;
                st.serialize_field("center", center)?;
                st.serialize_field("extents", extents)?;
                st.end()
            }
            Value::Array(items) => items.serialize(s),
        }
    }
}

/// The JSON expected for a field type, for error messages.
fn describe(ty: FieldType) -> &'static str {
    match ty {
        FieldType::Bool => "a boolean",
        FieldType::Int | FieldType::Enum => "a 32-bit integer",
        FieldType::UInt => "an unsigned 32-bit integer",
        FieldType::Short => "a 16-bit integer",
        FieldType::UShort => "an unsigned 16-bit integer",
        FieldType::Long => "a 64-bit integer",
        FieldType::Float | FieldType::Double => "a number",
        FieldType::Char => "a UTF-16 code unit",
        FieldType::String => "a string",
        FieldType::Vector2 => "an array of 2 numbers",
        FieldType::Vector3 => "an array of 3 numbers",
        FieldType::Vector4 | FieldType::Quaternion | FieldType::Color => "an array of 4 numbers",
        FieldType::Bounds => r#"{"center": [x, y, z], "extents": [x, y, z]}"#,
        FieldType::Reference => "an object ID",
        FieldType::Array(_) => "an array",
    }
}

/// A field's value, typed by the field's type if known.
struct FieldValueSeed(Option<FieldType>);

impl<'de> DeserializeSeed<'de> for FieldValueSeed {
    type Value = FieldValue;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<FieldValue, D::Error> {
        d.deserialize_any(self)
    }
}

macro_rules! typed_visit {
    ($($method:ident($ty:ty);)*) => {
        $(fn $method<E: de::Error>(self, v: $ty) -> Result<FieldValue, E> {
            match self.0 {
                Some(ty) => ValueSeed(ty).$method(v).map(FieldValue::Value),
                None => Err(E::invalid_type(Unexpected::Other("a value"), &self)),
            }
        })*
    };
}

impl<'de> Visitor<'de> for FieldValueSeed {
    type Value = FieldValue;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(ty) => write!(f, r#"{}, null or {{"raw": hex}}"#, describe(ty)),
            None => f.write_str(r#"null or {"raw": hex}, since the field's type isn't known"#),
        }
    }

    fn visit_unit<E: de::Error>(self) -> Result<FieldValue, E> {
        Ok(FieldValue::Empty)
    }

    fn visit_none<E: de::Error>(self) -> Result<FieldValue, E> {
        Ok(FieldValue::Empty)
    }

    typed_visit! {
        visit_bool(bool);
        visit_i64(i64);
        visit_u64(u64);
        visit_f64(f64);
        visit_str(&str);
    }

    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<FieldValue, A::Error> {
        match self.0 {
            Some(ty) => ValueSeed(ty).visit_seq(seq).map(FieldValue::Value),
            None => Err(A::Error::invalid_type(Unexpected::Seq, &self)),
        }
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<FieldValue, A::Error> {
        let Some(key) = map.next_key::<String>()? else {
            return Err(A::Error::invalid_type(Unexpected::Map, &self));
        };
        if key == "raw" {
            let bytes = map.next_value_seed(HexSeed)?;
            end_map(&mut map, &self)?;
            return Ok(FieldValue::Raw(bytes));
        }
        match self.0 {
            Some(ty) => ValueSeed(ty)
                .visit_map_from(key, map)
                .map(FieldValue::Value),
            None => Err(A::Error::invalid_type(Unexpected::Map, &self)),
        }
    }
}

/// A value of the given type.
#[derive(Clone, Copy)]
struct ValueSeed(FieldType);

impl<'de> DeserializeSeed<'de> for ValueSeed {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        d.deserialize_any(self)
    }
}

impl ValueSeed {
    fn int<E: de::Error>(self, n: i128, unexp: Unexpected) -> Result<Value, E> {
        let out_of_range = || E::invalid_value(unexp, &self);
        Ok(match self.0 {
            FieldType::Int | FieldType::Enum => {
                Value::Int(n.try_into().map_err(|_| out_of_range())?)
            }
            FieldType::UInt => Value::UInt(n.try_into().map_err(|_| out_of_range())?),
            FieldType::Reference => Value::Reference(n.try_into().map_err(|_| out_of_range())?),
            FieldType::Short => Value::Short(n.try_into().map_err(|_| out_of_range())?),
            FieldType::UShort => Value::UShort(n.try_into().map_err(|_| out_of_range())?),
            FieldType::Char => Value::Char(n.try_into().map_err(|_| out_of_range())?),
            FieldType::Long => Value::Long(n.try_into().map_err(|_| out_of_range())?),
            FieldType::Float => Value::Float(n as f32),
            FieldType::Double => Value::Double(n as f64),
            _ => return Err(E::invalid_type(unexp, &self)),
        })
    }

    /// A map value whose first key has been read.
    fn visit_map_from<'de, A: MapAccess<'de>>(
        self,
        first_key: String,
        mut map: A,
    ) -> Result<Value, A::Error> {
        match self.0 {
            FieldType::String if first_key == "utf16le" => {
                let bytes = map.next_value_seed(HexSeed)?;
                end_map(&mut map, &self)?;
                Ok(Value::String(DString::from_utf16le_bytes(&bytes)))
            }
            FieldType::Bounds => {
                let (mut center, mut extents) = (None, None);
                let mut key = Some(first_key);
                while let Some(k) = key {
                    match k.as_str() {
                        "center" => set_once(&mut center, map.next_value()?, "center")?,
                        "extents" => set_once(&mut extents, map.next_value()?, "extents")?,
                        other => {
                            return Err(A::Error::unknown_field(other, &["center", "extents"]));
                        }
                    }
                    key = map.next_key()?;
                }
                Ok(Value::Bounds {
                    center: center.ok_or_else(|| A::Error::missing_field("center"))?,
                    extents: extents.ok_or_else(|| A::Error::missing_field("extents"))?,
                })
            }
            _ => Err(A::Error::invalid_type(Unexpected::Map, &self)),
        }
    }
}

/// Reads exactly `N` floats.
fn floats<'de, const N: usize, A: SeqAccess<'de>>(
    mut seq: A,
    exp: &dyn de::Expected,
) -> Result<[f32; N], A::Error> {
    let mut out = [0.0; N];
    for (i, v) in out.iter_mut().enumerate() {
        *v = seq
            .next_element()?
            .ok_or_else(|| A::Error::invalid_length(i, exp))?;
    }
    if seq.next_element::<IgnoredAny>()?.is_some() {
        return Err(A::Error::invalid_length(N + 1, exp));
    }
    Ok(out)
}

impl<'de> Visitor<'de> for ValueSeed {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(describe(self.0))
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        match self.0 {
            FieldType::Bool => Ok(Value::Bool(v)),
            _ => Err(E::invalid_type(Unexpected::Bool(v), &self)),
        }
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        self.int(v.into(), Unexpected::Signed(v))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        self.int(v.into(), Unexpected::Unsigned(v))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        match self.0 {
            FieldType::Float => Ok(Value::Float(v as f32)),
            FieldType::Double => Ok(Value::Double(v)),
            _ => Err(E::invalid_type(Unexpected::Float(v), &self)),
        }
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        let float = match self.0 {
            FieldType::String => return Ok(Value::String(v.into())),
            FieldType::Float => parse_f32(v).map(Value::Float),
            FieldType::Double => parse_f64(v).map(Value::Double),
            _ => return Err(E::invalid_type(Unexpected::Str(v), &self)),
        };
        float.ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        Ok(match self.0 {
            FieldType::Vector2 => Value::Vector2(floats(seq, &self)?),
            FieldType::Vector3 => Value::Vector3(floats(seq, &self)?),
            FieldType::Vector4 => Value::Vector4(floats(seq, &self)?),
            FieldType::Quaternion => Value::Quaternion(floats(seq, &self)?),
            FieldType::Color => Value::Color(floats(seq, &self)?),
            FieldType::Array(element) => {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element_seed(ValueSeed(*element))? {
                    items.push(item);
                }
                Value::Array(items)
            }
            _ => return Err(A::Error::invalid_type(Unexpected::Seq, &self)),
        })
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        match map.next_key::<String>()? {
            Some(key) => self.visit_map_from(key, map),
            None => Err(A::Error::invalid_type(Unexpected::Map, &self)),
        }
    }
}
