//! A JSON Schema of the JSON format, for validating JSON and for LLM structured output.

use std::borrow::Cow;

use schemars::generate::SchemaSettings;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde_json::{Map, Value, json};

use super::hex::Hex;
use crate::component_id::ComponentId;
use crate::components::ComponentData;
use crate::fields::{FieldInfo, FieldType, fields_of};
use crate::generated::classes::CLASSES;
use crate::generated::component_ids::NAMES;
use crate::legacy_settings::LegacyLevelSettings;
use crate::string::DString;
use crate::tree::Level;
use crate::visit::BodyEnd;

/// What a schema describes.
#[derive(Debug, Clone, Default)]
pub struct SchemaOptions {
    /// Describe all JSON that [`to_string`](super::to_string) writes, for any file: non-level
    /// files, every component version, undecoded bodies, components of unknown types, legacy
    /// level settings, invalid UTF-16 and non-finite floats.
    ///
    /// Otherwise the schema describes JSON for a new level: components in the version the game
    /// writes, with only the fields that version has.
    pub full: bool,
    /// Only these component types, and `Transform` and `LevelSettings`, which every level has.
    /// All types if `None`.
    pub components: Option<Vec<ComponentId>>,
}

/// The JSON Schema (draft 2020-12) of a file in JSON.
///
/// The reader accepts any JSON the schema allows. It also accepts some JSON the schema doesn't
/// allow: without [`SchemaOptions::full`], that includes the JSON of most existing files.
pub fn schema(options: &SchemaOptions) -> Value {
    let mut b = Builder {
        generator: SchemaSettings::draft2020_12().into_generator(),
        full: options.full,
        defs: Map::new(),
        filters: Vec::new(),
    };
    let ids = match &options.components {
        Some(ids) => {
            let mut ids = ids.clone();
            ids.extend([ComponentId::TRANSFORM, ComponentId::LEVEL_SETTINGS]);
            ids.sort();
            ids.dedup();
            ids
        }
        None if options.full => NAMES.iter().map(|&(id, _)| ComponentId(id)).collect(),
        None => CLASSES
            .iter()
            .map(|&(id, _, _)| ComponentId(id))
            .filter(|id| id.0 > 0)
            .collect(),
    };
    let root = b.root(&ids);
    b.finish(root)
}

struct Builder {
    generator: SchemaGenerator,
    full: bool,
    /// The hand-written definitions.
    defs: Map<String, Value>,
    /// Typed bodies whose definitions should keep only these properties.
    filters: Vec<(String, Vec<String>)>,
}

fn reference(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}

fn array_of(items: Value) -> Value {
    json!({ "type": "array", "items": items })
}

/// An object with the given properties and no others.
fn object(description: &str, properties: Value, required: &[&str]) -> Value {
    let mut o = json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false,
    });
    if !description.is_empty() {
        o["description"] = description.into();
    }
    if !required.is_empty() {
        o["required"] = json!(required);
    }
    o
}

fn with_description(mut schema: Value, description: &str) -> Value {
    schema["description"] = description.into();
    schema
}

const GUID_DESCRIPTION: &str = "The ID of the object or component, unique among the objects and \
    components of the file. Components refer to objects and components by it. Left out (or 0), \
    it is assigned when the level is built, so give it only to what something refers to.";

impl Builder {
    fn sub<T: JsonSchema>(&mut self) -> Value {
        self.generator.subschema_for::<T>().to_value()
    }

    fn guid(&mut self) -> Value {
        if !self.defs.contains_key("Guid") {
            let guid = with_description(self.sub::<u32>(), GUID_DESCRIPTION);
            self.defs.insert("Guid".into(), guid);
        }
        reference("Guid")
    }

    fn raw_value(&mut self) -> Value {
        if !self.defs.contains_key("RawValue") {
            let raw = object(
                "A value that doesn't decode as its field's type, as its bytes.",
                json!({ "raw": self.sub::<Hex>() }),
                &["raw"],
            );
            self.defs.insert("RawValue".into(), raw);
        }
        reference("RawValue")
    }

    fn root(&mut self, ids: &[ComponentId]) -> Value {
        self.level();
        self.game_object(ids);
        let level = object("", json!({ "level": reference("Level") }), &["level"]);
        if self.full {
            let object = object(
                "A non-level file: a single object.",
                json!({ "game_object": reference("GameObject") }),
                &["game_object"],
            );
            json!({ "anyOf": [level, object] })
        } else {
            level
        }
    }

    fn level(&mut self) {
        let version = if self.full {
            with_description(
                self.sub::<i32>(),
                "The level file version, 0 to 4. The game writes version 4.",
            )
        } else {
            json!({
                "const": Level::LATEST_VERSION,
                "description": "The level file version, the one the game writes if left out.",
            })
        };
        let level = object(
            "A level.",
            json!({
                "name": with_description(reference("DString"), "The level's name."),
                "version": version,
                "settings": reference("LevelSettingsData"),
                "layers": array_of(reference("Layer")),
            }),
            &["name", "settings"],
        );
        self.defs.insert("Level".into(), level);

        let game_object = object(
            "",
            json!({
                "game_object": with_description(
                    reference("GameObject"),
                    "An object named `LevelSettings` with a `Transform` and a `LevelSettings` \
                     component.",
                ),
            }),
            &["game_object"],
        );
        let settings = if self.full {
            let mut legacy = object(
                "The settings of level versions 0 and 1, which have a format of their own.",
                json!({
                    "version": self.sub::<i32>(),
                    "data": self.sub::<LegacyLevelSettings>(),
                    "end": self.sub::<BodyEnd>(),
                    "raw": self.sub::<Hex>(),
                }),
                &["version"],
            );
            legacy["anyOf"] = json!([
                { "required": ["data"], "properties": { "raw": false } },
                { "required": ["raw"], "properties": { "data": false, "end": false } },
            ]);
            let legacy = object("", json!({ "legacy": legacy }), &["legacy"]);
            json!({ "anyOf": [game_object, legacy] })
        } else {
            game_object
        };
        self.defs.insert("LevelSettingsData".into(), settings);

        let layer = object(
            "A layer of the level editor.",
            json!({
                "name": reference("DString"),
                "flags": reference("LayerFlags"),
                "objects": array_of(reference("GameObject")),
            }),
            &["name"],
        );
        self.defs.insert("Layer".into(), layer);

        let flag = |description: &str| json!({ "type": "boolean", "description": description });
        let active = flag("Whether the layer is the active one in the editor. Default false.");
        let visible = flag("Default true.");
        let mut flags = json!({
            "active": active.clone(),
            "frozen": flag("Whether the layer's objects can't be selected. Default false."),
            "visible": visible.clone(),
            "is_background": flag("Whether the layer is a background layer. Default false."),
        });
        let description = "The layer's flags. Levels before version 3 have none.";
        let flags = if self.full {
            flags["version"] = json!({
                "type": "integer",
                "minimum": 1,
                "description": "The layer version, 1 (the game's, if left out) or more.",
            });
            let v0 = object(
                "Layer version 0, whose flags are all given.",
                json!({
                    "version": { "const": 0 },
                    "unfrozen": flag("In place of `frozen`."),
                    "active": active,
                    "visible": visible,
                }),
                &["version", "unfrozen", "active", "visible"],
            );
            let none = object(
                "A negative layer version, which has no flags.",
                json!({ "version": { "type": "integer", "maximum": -1 } }),
                &["version"],
            );
            json!({
                "description": description,
                "anyOf": [object("", flags, &[]), v0, none],
            })
        } else {
            object(description, flags, &[])
        };
        self.defs.insert("LayerFlags".into(), flags);
    }

    fn game_object(&mut self, ids: &[ComponentId]) {
        let mut properties = json!({
            "name": with_description(
                reference("DString"),
                "The name of the object's prefab, such as `CubeGS`.",
            ),
            "guid": self.guid(),
            "components": array_of(reference("Component")),
        });
        if self.full {
            properties["prefab"] = with_description(
                reference("DString"),
                "Always empty in files the game writes.",
            );
        }
        let game_object = object(
            "An object created from a prefab, with the components that differ from the prefab's.",
            properties,
            &["name"],
        );
        self.defs.insert("GameObject".into(), game_object);

        let vector =
            |b: &mut Self, description| with_description(b.sub::<Option<[f32; 3]>>(), description);
        let rotation = with_description(
            self.sub::<Option<[f32; 4]>>(),
            "The local rotation, a quaternion `[x, y, z, w]`.",
        );
        let transform = object(
            "The body of a `Transform`. Values left out are the prefab's.",
            json!({
                "position": vector(self, "The local position `[x, y, z]`."),
                "rotation": rotation,
                "scale": vector(self, "The local scale `[x, y, z]`."),
                "children": with_description(
                    array_of(reference("GameObject")),
                    "The object's children, positioned relative to it.",
                ),
            }),
            &[],
        );
        self.defs.insert("Transform".into(), transform);

        let mut variants: Vec<Value> = ids.iter().filter_map(|&id| self.component(id)).collect();
        if self.full {
            variants.push(object(
                "A component of a type the game no longer has, of which only the name is stored.",
                json!({
                    "unknown_type": reference("DString"),
                    "guid": self.guid(),
                    "raw": self.sub::<Hex>(),
                }),
                &["unknown_type"],
            ));
            let ty = with_description(self.sub::<i32>(), "A component ID the game doesn't name.");
            let fields = self.fields(&[]);
            variants.push(self.component_variant(ty, None, None, Some(fields)));
        }
        let component = json!({
            "description": "A component. `type` is its type, and `version` its version, which \
                is the version the game writes if left out. Its body is either `data`, a typed \
                body, or `fields`, the game's generic format; `type` must come before them. \
                Without a body, the component is the same as the prefab's.",
            "anyOf": variants,
        });
        self.defs.insert("Component".into(), component);
    }

    /// The variant of `Component` for a component type, if the schema has one.
    fn component(&mut self, id: ComponentId) -> Option<Value> {
        let name = id.name()?;
        let written = id.written_version();
        if !self.full && written.is_none() {
            return None;
        }
        let ty = json!({ "const": name });
        let data = if id == ComponentId::TRANSFORM {
            Some(reference("Transform"))
        } else if self.full {
            ComponentData::data_schema(id, &mut self.generator).map(Schema::to_value)
        } else {
            let written = written?;
            ComponentData::sample(id, written).map(|sample| {
                let schema = ComponentData::data_schema(id, &mut self.generator).unwrap();
                let schema = schema.to_value();
                if let Some(name) = schema["$ref"].as_str().and_then(|r| r.rsplit('/').next()) {
                    self.filters
                        .push((name.to_owned(), used_properties(&sample)));
                }
                schema
            })
        };
        // Old files have some types in the generic format that the game no longer has fields
        // for. Their values are raw.
        let fields = match fields_of(id) {
            Some(fields) if self.full || data.is_none() => Some(self.fields(fields)),
            None if self.full => Some(self.fields(&[])),
            _ => None,
        };
        Some(self.component_variant(ty, Some(id), data, fields))
    }

    /// A variant of `Component`. `id` is `None` for types the game doesn't name.
    fn component_variant(
        &mut self,
        ty: Value,
        id: Option<ComponentId>,
        data: Option<Value>,
        fields: Option<Value>,
    ) -> Value {
        let written = id.and_then(ComponentId::written_version);
        // Where the reader can't tell the version.
        let version_needed = match (id, written) {
            (_, None) if data.is_some() => Some(json!([
                { "required": ["version"] },
                { "required": ["data"] },
            ])),
            (_, None) => Some(json!([{ "required": ["version"] }])),
            (Some(id), Some(written))
                if fields.is_some() && ComponentData::decodes(id, written) =>
            {
                Some(json!([
                    { "required": ["version"] },
                    { "not": { "required": ["fields"] } },
                ]))
            }
            _ => None,
        };
        let mut properties = json!({ "type": ty });
        properties["version"] = if self.full {
            self.sub::<i32>()
        } else {
            json!({ "const": written.unwrap_or(0) })
        };
        if self.full {
            properties["mark"] = json!({
                "enum": ["built_in", "serial", "added"],
                "description": "The scope mark, when it isn't the one the game writes for the type.",
            });
        }
        properties["guid"] = self.guid();
        if let Some(data) = data {
            properties["data"] = data;
        }
        if let Some(fields) = fields {
            properties["fields"] = fields;
        }
        if !self.full {
            return object("", properties, &["type"]);
        }
        properties["raw"] = self.sub::<Hex>();
        properties["end"] = self.sub::<BodyEnd>();
        let mut variant = object("", properties, &["type"]);
        // At most one body, and `end` only with `data`.
        variant["not"] = json!({
            "anyOf": [
                { "required": ["data", "fields"] },
                { "required": ["data", "raw"] },
                { "required": ["fields", "raw"] },
            ],
        });
        variant["dependentRequired"] = json!({ "end": ["data"] });
        if let Some(version_needed) = version_needed {
            variant["anyOf"] = version_needed;
        }
        variant
    }

    /// The body of a component in the generic format.
    fn fields(&mut self, fields: &[FieldInfo]) -> Value {
        let mut properties = Map::new();
        let pair = |name: Value, value: Value| {
            json!({
                "type": "array",
                "prefixItems": [name, value],
                "minItems": 2,
                "maxItems": 2,
            })
        };
        let mut pair_variants = Vec::new();
        for &(name, cs_type, ty) in fields {
            let Some(ty) = ty else {
                continue; // The game can't read these.
            };
            let mut value = self.field_value(ty);
            if self.full {
                value = json!({ "anyOf": [value, { "type": "null" }, self.raw_value()] });
                pair_variants.push(pair(json!({ "const": name }), value.clone()));
            }
            let description = match ty {
                FieldType::Reference => {
                    format!("The `guid` of a `{cs_type}`, or 0 for none.")
                }
                FieldType::Enum => format!("A `{cs_type}` value."),
                _ => format!("`{cs_type}`."),
            };
            value["description"] = description.into();
            properties.insert(name.into(), value);
        }
        let description = "The component's fields by name. Fields left out (or null) are the \
            prefab's.";
        if !self.full {
            return object(description, properties.into(), &[]);
        }
        let untyped = json!({ "anyOf": [{ "type": "null" }, self.raw_value()] });
        let names: Vec<_> = properties.keys().cloned().collect();
        let other_name = if names.is_empty() {
            reference("DString")
        } else {
            json!({ "allOf": [reference("DString"), { "not": { "enum": names } }] })
        };
        pair_variants.push(pair(other_name, untyped.clone()));
        let as_object = json!({
            "type": "object",
            "description": description,
            "properties": properties,
            "additionalProperties": untyped,
        });
        let pairs = array_of(json!({ "anyOf": pair_variants }));
        let pairs = with_description(
            pairs,
            "`[name, value]` pairs, used when names repeat or aren't valid UTF-16.",
        );
        json!({ "anyOf": [as_object, pairs] })
    }

    fn field_value(&mut self, ty: FieldType) -> Value {
        match ty {
            FieldType::Bool => self.sub::<bool>(),
            FieldType::Int | FieldType::Enum => self.sub::<i32>(),
            FieldType::UInt | FieldType::Reference => self.sub::<u32>(),
            FieldType::Short => self.sub::<i16>(),
            FieldType::UShort | FieldType::Char => self.sub::<u16>(),
            FieldType::Long => self.sub::<i64>(),
            FieldType::Float => self.sub::<f32>(),
            FieldType::Double => self.sub::<f64>(),
            FieldType::String => reference("DString"),
            FieldType::Vector2 => self.sub::<[f32; 2]>(),
            FieldType::Vector3 => self.sub::<[f32; 3]>(),
            FieldType::Vector4 => self.sub::<[f32; 4]>(),
            FieldType::Quaternion => with_description(self.sub::<[f32; 4]>(), "`[x, y, z, w]`"),
            FieldType::Color => with_description(self.sub::<[f32; 4]>(), "`[r, g, b, a]`"),
            FieldType::Bounds => {
                let vector = self.sub::<[f32; 3]>();
                object(
                    "",
                    json!({ "center": vector, "extents": vector }),
                    &["center", "extents"],
                )
            }
            FieldType::Array(element) => array_of(self.field_value(*element)),
        }
    }

    fn finish(mut self, root: Value) -> Value {
        let mut defs = self.generator.take_definitions(true);
        for (name, keep) in &self.filters {
            if let Some(Value::Object(properties)) =
                defs.get_mut(name).and_then(|def| def.get_mut("properties"))
            {
                properties.retain(|k, _| keep.contains(k));
            }
        }
        defs.append(&mut self.defs);
        let mut schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Distance .bytes file",
            "description": "A .bytes file of the game Distance as JSON, as written and read by \
                the distance-bytes library.",
        });
        let Value::Object(root) = root else {
            unreachable!()
        };
        schema.as_object_mut().unwrap().extend(root);
        if self.full {
            defs.insert(
                "DString".into(),
                json!({
                    "anyOf": [
                        { "type": "string" },
                        object(
                            "A string that isn't valid UTF-16, as its bytes.",
                            json!({ "utf16le": reference("Hex") }),
                            &["utf16le"],
                        ),
                    ],
                }),
            );
            let mut defs = Value::Object(defs);
            allow_non_finite(&mut defs);
            schema["$defs"] = defs;
        } else {
            defs.remove("DString");
            let mut defs = Value::Object(defs);
            inline_strings(&mut defs);
            schema["$defs"] = defs;
            inline_strings(&mut schema);
        }
        bound_integers(&mut schema);
        unwrap_descriptions(&mut schema);
        schema
    }
}

/// The properties of a typed body that are written for a sample of it, which sets every value
/// its version has. Values left unset are left out.
fn used_properties(sample: &ComponentData) -> Vec<String> {
    let value = sample
        .serialize_body(serde_json::value::Serializer)
        .unwrap();
    let Value::Object(map) = value else {
        return Vec::new();
    };
    map.into_iter().map(|(k, _)| k).collect()
}

/// Gives integers the range of their format, which validators don't check by default.
fn bound_integers(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let range: Option<(Value, Value)> = match map.get("format").and_then(Value::as_str) {
                Some("int8") => Some((i8::MIN.into(), i8::MAX.into())),
                Some("uint8") => Some((u8::MIN.into(), u8::MAX.into())),
                Some("int16") => Some((i16::MIN.into(), i16::MAX.into())),
                Some("uint16") => Some((u16::MIN.into(), u16::MAX.into())),
                Some("int32") => Some((i32::MIN.into(), i32::MAX.into())),
                Some("uint32") => Some((u32::MIN.into(), u32::MAX.into())),
                Some("int64") => Some((i64::MIN.into(), i64::MAX.into())),
                Some("uint64") => Some((u64::MIN.into(), u64::MAX.into())),
                _ => None,
            };
            if let Some((min, max)) = range {
                map.entry("minimum").or_insert(min);
                map.entry("maximum").or_insert(max);
            }
            // Beyond 2^53 - 1, 64-bit integers are written as strings (see `adapter`).
            if matches!(
                map.get("format").and_then(Value::as_str),
                Some("int64" | "uint64")
            ) {
                match map.get_mut("type") {
                    Some(Value::Array(types)) => types.push("string".into()),
                    Some(ty) => *ty = Value::Array(vec![ty.take(), "string".into()]),
                    None => {}
                }
                let sign = if map["format"] == "int64" { "-?" } else { "" };
                map.insert("pattern".into(), format!("^{sign}[0-9]+$").into());
            }
            map.values_mut().for_each(bound_integers);
        }
        Value::Array(items) => items.iter_mut().for_each(bound_integers),
        _ => {}
    }
}

/// Joins the lines of each paragraph of the descriptions, which come from wrapped doc comments.
fn unwrap_descriptions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(description)) = map.get_mut("description") {
                let paragraphs: Vec<_> = description
                    .split("\n\n")
                    .map(|p| p.replace('\n', " "))
                    .collect();
                *description = paragraphs.join("\n\n");
            }
            map.values_mut().for_each(unwrap_descriptions);
        }
        Value::Array(items) => items.iter_mut().for_each(unwrap_descriptions),
        _ => {}
    }
}

/// Replaces references to `DString` with plain strings.
fn inline_strings(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.get("$ref").and_then(Value::as_str) == Some("#/$defs/DString") {
                map.remove("$ref");
                map.insert("type".into(), "string".into());
            }
            map.values_mut().for_each(inline_strings);
        }
        Value::Array(items) => items.iter_mut().for_each(inline_strings),
        _ => {}
    }
}

/// Lets every number be one of the strings written for non-finite floats.
fn allow_non_finite(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.values_mut().for_each(allow_non_finite);
            let is_number = match map.get("type") {
                Some(Value::String(t)) => t == "number",
                Some(Value::Array(types)) => types.iter().any(|t| t == "number"),
                _ => false,
            };
            if is_number {
                let number = std::mem::take(map);
                map.insert(
                    "anyOf".into(),
                    json!([
                        number,
                        {
                            "type": "string",
                            "pattern": "^(NaN(:0x[0-9a-f]+)?|-?Infinity)$",
                        },
                    ]),
                );
            }
        }
        Value::Array(items) => items.iter_mut().for_each(allow_non_finite),
        _ => {}
    }
}

impl JsonSchema for DString {
    fn schema_name() -> Cow<'static, str> {
        "DString".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        schemars::json_schema!({ "type": "string" })
    }
}
