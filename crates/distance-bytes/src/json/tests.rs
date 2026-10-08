use super::*;
use crate::generated::component_ids::NAMES;
use crate::io::mark;
use crate::{
    BodyEnd, BytesFile, Component, ComponentBody, ComponentData, ComponentId, ComponentKind,
    DString, Field, FieldValue, GameObject, Layer, LayerFlags, LegacyLevelSettings,
    LegacySettingsBody, Level, LevelSettingsData, Transform, Value,
};

fn object(name: &str, guid: u32, components: Vec<Component>) -> GameObject {
    GameObject {
        name: name.into(),
        prefab: DString::default(),
        guid,
        components,
    }
}

fn component(id: ComponentId, version: i32, guid: u32, body: ComponentBody) -> Component {
    Component {
        kind: ComponentKind::Known {
            mark: id.written_mark(),
            id,
            version,
        },
        guid,
        body,
    }
}

fn level(objects: Vec<GameObject>) -> BytesFile {
    BytesFile::Level(Level {
        name: "Test".into(),
        version: 4,
        settings: LevelSettingsData::GameObject(object("LevelSettings", 1, vec![])),
        layers: vec![Layer {
            name: "Default".into(),
            flags: LayerFlags::DEFAULT,
            objects,
        }],
    })
}

/// Converts to pretty and compact JSON and back, checking that the binary output is unchanged
/// and that the JSON fits the full schema. Returns the pretty JSON.
fn round_trip(file: &BytesFile) -> String {
    let text = to_string_pretty(file).unwrap();
    let back: BytesFile = from_str(&text).unwrap_or_else(|e| panic!("{e}\n{text}"));
    assert_eq!(back.write(), file.write(), "{text}");
    let compact: BytesFile = from_slice(&to_vec(file).unwrap()).unwrap();
    assert_eq!(compact.write(), file.write());
    assert_fits_full_schema(file);
    text
}

fn assert_fits_full_schema(file: &BytesFile) {
    #[cfg(feature = "schema")]
    schema_tests::assert_fits(schema_tests::full(), file);
    #[cfg(not(feature = "schema"))]
    let _ = file;
}

fn read_error(text: &str) -> String {
    from_str::<BytesFile>(text).unwrap_err().to_string()
}

/// Every decoded version of every component, with every value set, converts to JSON and back.
#[test]
fn samples_round_trip() {
    let mut components = Vec::new();
    for &(id, _) in NAMES {
        let id = ComponentId(id);
        for version in 0..=64 {
            if let Some(data) = ComponentData::sample(id, version) {
                let body = ComponentBody::Data {
                    data,
                    end: BodyEnd::Complete,
                };
                components.push(component(id, version, 2, body));
            }
        }
    }
    assert!(components.len() > 300);
    let file = level(vec![object("Samples", 1, components)]);
    let back: BytesFile = from_str(&to_string(&file).unwrap()).unwrap();
    assert_eq!(back, file);
    assert_fits_full_schema(&file);

    for version in 0..=8 {
        let file = BytesFile::Level(Level {
            name: "Old".into(),
            version: 1,
            settings: LevelSettingsData::Legacy {
                version,
                body: LegacySettingsBody::Data {
                    data: Box::new(LegacyLevelSettings::sample(version)),
                    end: BodyEnd::Complete,
                },
            },
            layers: vec![],
        });
        let back: BytesFile = from_str(&to_string(&file).unwrap()).unwrap();
        assert_eq!(back, file, "v{version}");
        assert_fits_full_schema(&file);
    }
}

#[test]
fn non_finite_floats() {
    let transform = Transform {
        position: Some([f32::NAN, f32::INFINITY, f32::NEG_INFINITY]),
        rotation: Some([f32::from_bits(0xffc0_0001), -0.0, 0.0, 1.0]),
        ..Transform::default()
    };
    let fields = vec![
        Field {
            name: "EmitFrequency".into(),
            value: FieldValue::Value(Value::Float(f32::NAN)),
        },
        Field {
            name: "LightScale".into(),
            value: FieldValue::Value(Value::Float(f32::NEG_INFINITY)),
        },
    ];
    let emit = ComponentId::from_name("ParticleEmitLogic").unwrap();
    let file = level(vec![object(
        "Thing",
        2,
        vec![
            component(
                ComponentId::TRANSFORM,
                0,
                3,
                ComponentBody::Transform(transform),
            ),
            component(emit, 0, 4, ComponentBody::Fields(fields)),
        ],
    )]);
    let text = round_trip(&file);
    for expected in [
        r#""position": ["NaN", "Infinity", "-Infinity"]"#,
        r#""rotation": ["NaN:0xffc00001", -0.0, 0.0, 1.0]"#,
        r#""EmitFrequency": "NaN""#,
        r#""LightScale": "-Infinity""#,
    ] {
        assert!(text.contains(expected), "{expected} not in {text}");
    }
}

#[test]
fn f32_that_needs_more_digits() {
    // The shortest decimal of this f32 is read back as the next f32 up when parsed as an f64
    // first.
    let v = f32::from_bits(363742205);
    let text = to_string(&v).unwrap();
    assert_ne!(text, "7.038531e-26");
    assert_eq!(from_str::<f32>(&text).unwrap().to_bits(), v.to_bits());
}

/// serde_json reads an `f32` by parsing an `f64` and rounding it, which for a few values rounds
/// differently than parsing the `f32` directly. Checks every bit pattern.
#[test]
#[ignore = "takes about half a minute on 20 threads"]
fn every_f32_round_trips() {
    let threads = std::thread::available_parallelism().map_or(8, |n| n.get()) as u64;
    let chunk = (1u64 << 32).div_ceil(threads);
    std::thread::scope(|s| {
        for t in 0..threads {
            s.spawn(move || {
                let end = ((t + 1) * chunk).min(1 << 32);
                for bits in t * chunk..end {
                    let v = f32::from_bits(bits as u32);
                    let text = to_string(&v).unwrap();
                    let back: f32 = from_str(&text).unwrap();
                    assert_eq!(back.to_bits(), v.to_bits(), "{text}");
                }
            });
        }
    });
}

#[test]
fn invalid_utf16_strings_and_field_names() {
    let bad = DString::from_utf16le_bytes(&[0x00, 0xd8, 0x41]);
    let fields = vec![
        Field {
            name: bad.clone(),
            value: FieldValue::Raw(vec![1, 2]),
        },
        Field {
            name: "PulsingShaderProperty".into(),
            value: FieldValue::Value(Value::String(bad.clone())),
        },
    ];
    let pulse = ComponentId::from_name("PulseMaterial").unwrap();
    let file = level(vec![object(
        "Thing",
        2,
        vec![
            component(pulse, 0, 3, ComponentBody::Fields(fields)),
            Component {
                kind: ComponentKind::Unknown { name: bad.clone() },
                guid: 4,
                body: ComponentBody::Raw(vec![0xab]),
            },
        ],
    )]);
    round_trip(&file);
    let text = to_string(&file).unwrap();
    // A field name that isn't valid UTF-16 makes the fields a list of pairs.
    for expected in [
        r#"[{"utf16le":"00d841"},{"raw":"0102"}]"#,
        r#"["PulsingShaderProperty",{"utf16le":"00d841"}]"#,
        r#""unknown_type":{"utf16le":"00d841"}"#,
    ] {
        assert!(text.contains(expected), "{expected} not in {text}");
    }
}

#[test]
fn repeated_field_names_are_kept() {
    let field = |v| Field {
        name: "FlickerModNumber".into(),
        value: FieldValue::Value(Value::Int(v)),
    };
    let flicker = ComponentId::from_name("LightFlickerLogic").unwrap();
    let body = ComponentBody::Fields(vec![field(1), field(2)]);
    let file = level(vec![object(
        "Thing",
        2,
        vec![component(flicker, 0, 3, body)],
    )]);
    let text = round_trip(&file);
    assert!(text.contains(r#"["FlickerModNumber", 1]"#), "{text}");
}

#[test]
fn unusual_marks_and_body_ends() {
    let group = |guid, end| {
        let body = ComponentBody::Data {
            data: ComponentData::sample(ComponentId::GROUP, 1).unwrap(),
            end,
        };
        component(ComponentId::GROUP, 1, guid, body)
    };
    let mut trailing = group(3, BodyEnd::Trailing(vec![9, 9]));
    if let ComponentKind::Known { mark, .. } = &mut trailing.kind {
        *mark = mark::SERIALIZABLE_ALT;
    }
    let truncated = group(4, BodyEnd::Truncated(1));
    let file = level(vec![object("Thing", 2, vec![trailing, truncated])]);
    round_trip(&file);
    let text = to_string(&file).unwrap();
    for expected in [
        r#""mark":"added""#,
        r#""end":{"trailing":"0909"}"#,
        r#""end":{"truncated":1}"#,
    ] {
        assert!(text.contains(expected), "{expected} not in {text}");
    }
}

#[test]
fn raw_bodies_and_legacy_settings() {
    let file = BytesFile::Level(Level {
        name: "Old".into(),
        version: 1,
        settings: LevelSettingsData::Legacy {
            version: 3,
            body: LegacySettingsBody::Raw(vec![1, 2, 3]),
        },
        layers: vec![Layer {
            name: "Default".into(),
            flags: LayerFlags::Absent,
            objects: vec![object(
                "Thing",
                2,
                vec![
                    component(ComponentId::GROUP, 0, 3, ComponentBody::Empty),
                    component(ComponentId::GROUP, 0, 4, ComponentBody::Raw(vec![0xff])),
                ],
            )],
        }],
    });
    let text = round_trip(&file);
    assert!(text.contains(r#""legacy": {"#), "{text}");
    assert!(text.contains(r#""raw": "010203""#), "{text}");
    assert!(text.contains(r#""raw": "ff""#), "{text}");
    assert!(!text.contains("flags"), "{text}");
}

#[test]
fn written_json_is_typed_by_the_field_table() {
    let text = r#"{"level": {"name": "L", "settings": {"game_object": {"name": "LevelSettings"}},
        "layers": [{"name": "Default", "objects": [{"name": "Thing", "components": [
            {"type": "CarScreenTextDecodeTrigger", "version": 0, "fields": {
                "PerCharSpeed": 1,
                "AnnouncerPhrases": ["hi"],
                "NoSuchField": {"raw": "00"},
                "Text": null
            }}
        ]}]}]}}"#;
    let BytesFile::Level(level) = from_str(text).unwrap() else {
        panic!("not a level");
    };
    assert_eq!(level.version, Level::LATEST_VERSION);
    assert_eq!(level.layers[0].flags, LayerFlags::Absent);
    let c = &level.layers[0].objects[0].components[0];
    assert_eq!(c.guid, 0);
    let ComponentBody::Fields(fields) = &c.body else {
        panic!("not fields");
    };
    let values: Vec<_> = fields.iter().map(|f| f.value.clone()).collect();
    assert_eq!(
        values,
        [
            FieldValue::Value(Value::Float(1.0)),
            FieldValue::Value(Value::Array(vec![Value::String("hi".into())])),
            FieldValue::Raw(vec![0]),
            FieldValue::Empty,
        ]
    );
}

#[test]
fn missing_versions_are_the_written_ones() {
    let text = r#"{"game_object": {"name": "Thing", "components": [
        {"type": "Transform", "data": {"position": [1, 2, 3]}},
        {"type": "TeleporterEntrance", "data": {"teleports_to": 5}},
        {"type": "TeleporterExit"},
        {"type": "LightFlickerLogic", "fields": {"FlickerModNumber": 2}}
    ]}}"#;
    let BytesFile::GameObject(obj) = from_str(text).unwrap() else {
        panic!("not a game object");
    };
    let versions: Vec<_> = obj
        .components
        .iter()
        .map(|c| match c.kind {
            ComponentKind::Known { mark, id, version } => {
                assert_eq!(mark, id.written_mark(), "{id}");
                version
            }
            ComponentKind::Unknown { .. } => panic!("unknown component"),
        })
        .collect();
    assert_eq!(versions, [0, 4, 1, 0]);

    // Generic fields of a type that is written as typed data need their version.
    let err = read_error(
        r#"{"game_object": {"name": "T", "components": [
            {"type": "TeleporterEntrance", "fields": {}}]}}"#,
    );
    assert!(err.contains(r#"give the "version""#), "{err}");
}

#[test]
fn errors_say_what_and_where() {
    let cases = [
        (
            r#"{"game_object": {"name": "T", "components": [{"data": {}, "type": "Group"}]}}"#,
            r#""type" must come before "data""#,
        ),
        (
            r#"{"game_object": {"name": "T", "components": [{"type": "Group", "colour": 1}]}}"#,
            "unknown field `colour`",
        ),
        (
            r#"{"game_object": {"name": "T", "components": [
                {"type": "Group", "data": {"links": ["a"]}}]}}"#,
            "game_object.components[0].data.links[0]",
        ),
        (
            r#"{"game_object": {"name": "T", "components": [
                {"type": "Group", "data": {"linkz": []}}]}}"#,
            "unknown field `linkz`",
        ),
        (
            r#"{"game_object": {"name": "T", "components": [
                {"type": "LightFlickerLogic", "version": 0, "fields": {"OnTime": true}}]}}"#,
            "at game_object.components[0].fields.OnTime: invalid type: boolean `true`, expected a \
             number",
        ),
        (
            r#"{"game_object": {"name": "T", "components": [{"type": "Nope"}]}}"#,
            r#"unknown component type "Nope""#,
        ),
        (
            r#"{"game_object": {"name": "T", "components": [
                {"type": "LightFlickerLogic", "version": 0, "data": {}}]}}"#,
            "LightFlickerLogic has no typed body",
        ),
        (
            r#"{"game_object": {"name": "T", "components": [
                {"type": "Group", "raw": "00", "data": {}}]}}"#,
            r#"only one of "data", "fields" and "raw""#,
        ),
        (
            r#"{"game_object": {"name": "T", "components": [
                {"type": "Group", "raw": "00", "end": "complete"}]}}"#,
            r#""end" goes only with a typed "data" body"#,
        ),
    ];
    for (text, expected) in cases {
        let err = read_error(text);
        assert!(err.contains(expected), "{err:?} lacks {expected:?}");
    }
}

#[test]
fn layer_flags() {
    let cases = [
        (
            LayerFlags::DEFAULT,
            r#"{"active":false,"frozen":false,"visible":true,"is_background":false}"#,
        ),
        (
            LayerFlags::V1 {
                version: 2,
                active: true,
                frozen: true,
                visible: false,
                is_background: true,
            },
            r#"{"version":2,"active":true,"frozen":true,"visible":false,"is_background":true}"#,
        ),
        (
            LayerFlags::V0 {
                unfrozen: true,
                active: false,
                visible: true,
            },
            r#"{"version":0,"unfrozen":true,"active":false,"visible":true}"#,
        ),
        (LayerFlags::None { version: -1 }, r#"{"version":-1}"#),
    ];
    for (flags, json) in cases {
        assert_eq!(to_string(&flags).unwrap(), json);
        assert_eq!(from_str::<LayerFlags>(json).unwrap(), flags);
    }
    assert_eq!(from_str::<LayerFlags>("{}").unwrap(), LayerFlags::DEFAULT);
    assert_eq!(from_str::<LayerFlags>("null").unwrap(), LayerFlags::Absent);
    assert!(from_str::<LayerFlags>(r#"{"version":0,"active":true}"#).is_err());
    assert!(from_str::<LayerFlags>(r#"{"unfrozen":true}"#).is_err());
}

/// Objects nested far deeper than a small stack holds without growing it, through every recursive
/// operation.
#[test]
fn deep_nesting() {
    let mut obj = object("Leaf", 0, vec![]);
    for i in 0..20_000 {
        let transform = Transform {
            children: vec![obj],
            ..Transform::default()
        };
        let body = ComponentBody::Transform(transform);
        let transform = component(ComponentId::TRANSFORM, 0, 0, body);
        obj = object("Group", i + 1, vec![transform]);
    }
    let file = BytesFile::GameObject(obj);
    let same = std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(move || {
            let bytes = file.write();
            let read = BytesFile::read(&bytes).unwrap();
            // Pretty JSON would be indented by up to a megabyte per line.
            let back: BytesFile = from_str(&to_string(&read).unwrap()).unwrap();
            let copy = back.clone();
            assert!(format!("{copy:?}").len() > 20_000);
            back.write() == bytes && copy == read && copy == file
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(same);
}

#[cfg(feature = "schema")]
mod schema_tests {
    use std::sync::LazyLock;

    use jsonschema::Validator;

    use super::*;
    use crate::generated::classes::CLASSES;

    fn validator(options: &SchemaOptions) -> Validator {
        jsonschema::validator_for(&schema(options)).unwrap()
    }

    pub(super) fn full() -> &'static Validator {
        static FULL: LazyLock<Validator> = LazyLock::new(|| {
            validator(&SchemaOptions {
                full: true,
                components: None,
            })
        });
        &FULL
    }

    fn authoring() -> &'static Validator {
        static AUTHORING: LazyLock<Validator> =
            LazyLock::new(|| validator(&SchemaOptions::default()));
        &AUTHORING
    }

    /// Where the JSON doesn't fit the schema.
    fn errors(validator: &Validator, text: &str) -> Vec<String> {
        let value: serde_json::Value = serde_json::from_str(text).unwrap();
        validator
            .iter_errors(&value)
            .map(|e| format!("at {} (schema {})", e.instance_path(), e.schema_path()))
            .collect()
    }

    pub(super) fn assert_fits(validator: &Validator, file: &BytesFile) {
        let text = to_string(file).unwrap();
        let errors = errors(validator, &text);
        assert!(errors.is_empty(), "{errors:?}\n{text}");
    }

    /// A level whose only object has the given components.
    fn level_with(components: &str) -> String {
        r#"{"level": {"name": "L", "settings": {"game_object": {"name": "LevelSettings"}},
            "layers": [{"name": "Default", "objects": [{"name": "Thing", "components": [
                COMPONENTS
            ]}]}]}}"#
            .replace("COMPONENTS", components)
    }

    /// Every component in the version the game writes, as the library writes it, fits the
    /// authoring schema.
    #[test]
    fn authoring_schema_fits_written_versions() {
        let mut components = Vec::new();
        for &(id, version, _) in CLASSES {
            let id = ComponentId(id);
            if let Some(data) = ComponentData::sample(id, version) {
                let body = ComponentBody::Data {
                    data,
                    end: BodyEnd::Complete,
                };
                components.push(component(id, version, 2, body));
            }
        }
        assert!(components.len() > 90);
        assert_fits(authoring(), &level(vec![object("Samples", 1, components)]));
    }

    /// A new level written by hand fits the authoring schema, also when it has only the
    /// components used, and the schemas' promise that the reader takes it holds.
    #[test]
    fn authoring_schema_fits_a_new_level() {
        let text = r#"{"level": {"name": "L", "settings": {"game_object": {
            "name": "LevelSettings", "guid": 1, "components": [
                {"type": "Transform", "guid": 2},
                {"type": "LevelSettings", "guid": 3, "data": {"level_name": "L"}}
            ]}},
            "layers": [{"name": "Default", "objects": [
                {"name": "Teleporter", "guid": 4, "components": [
                    {"type": "Transform", "guid": 5, "data": {"position": [1, 2, 3], "children": [
                        {"name": "Light", "guid": 6, "components": [
                            {"type": "LightFlickerLogic", "guid": 7,
                                "fields": {"FlickerModNumber": 2, "OnTime": 0.5}}
                        ]}
                    ]}},
                    {"type": "TeleporterEntrance", "version": 4, "guid": 8,
                        "data": {"teleports_to": 9}},
                    {"type": "TeleporterExit", "guid": 9, "data": {"link_id": 9}}
                ]}
            ]}]}}"#;
        // `Transform` and `LevelSettings` come with any components.
        let names = ["TeleporterEntrance", "TeleporterExit", "LightFlickerLogic"];
        let ids = names.map(|n| ComponentId::from_name(n).unwrap()).to_vec();
        let some = validator(&SchemaOptions {
            full: false,
            components: Some(ids[..2].to_vec()),
        });
        let only_used = validator(&SchemaOptions {
            full: false,
            components: Some(ids),
        });
        assert_eq!(errors(authoring(), text), Vec::<String>::new());
        assert_eq!(errors(&only_used, text), Vec::<String>::new());
        assert_eq!(errors(full(), text), Vec::<String>::new());
        assert!(!errors(&some, text).is_empty());
        from_str::<BytesFile>(text).unwrap();
    }

    /// The authoring schema takes only components as the game writes them now, the full schema
    /// takes what the reader takes.
    #[test]
    fn schemas_agree_with_the_reader() {
        let components = [
            // Fits the authoring schema.
            (
                r#"{"type": "TeleporterEntrance", "data": {"teleports_to": 1}}"#,
                true,
            ),
            (
                r#"{"type": "LightFlickerLogic", "fields": {"OnTime": 1}}"#,
                true,
            ),
            (r#"{"type": "TeleporterExit"}"#, true),
            // Fits only the full schema: other versions, values and bodies.
            (r#"{"type": "TeleporterEntrance", "version": 3}"#, false),
            (
                r#"{"type": "MusicTrigger", "data": {"music_cue_id": 1}}"#,
                false,
            ),
            (
                r#"{"type": "LightFlickerLogic", "fields": {"OnTime": null}}"#,
                false,
            ),
            (
                r#"{"type": "LightFlickerLogic", "fields": {"Nope": {"raw": "00"}}}"#,
                false,
            ),
            (
                r#"{"type": "LightFlickerLogic", "fields": {"OnTime": "NaN"}}"#,
                false,
            ),
            (
                r#"{"type": "LightFlickerLogic", "fields": [["OnTime", 1]]}"#,
                false,
            ),
            (
                r#"{"type": "GlobalFogLogic", "version": 0, "fields": {"X": null}}"#,
                false,
            ),
            (r#"{"type": "Group", "mark": "added", "raw": "00"}"#, false),
            (r#"{"unknown_type": "Gone", "raw": "00"}"#, false),
            (r#"{"type": 5000, "version": 0}"#, false),
            (
                r#"{"type": "TeleporterEntrance", "version": 0, "fields": {}}"#,
                false,
            ),
            (
                r#"{"type": "Transform", "data": {"scale": ["Infinity", 1, 1]}}"#,
                false,
            ),
        ];
        for (component, authored) in components {
            let text = level_with(component);
            assert_eq!(
                errors(authoring(), &text).is_empty(),
                authored,
                "{component}"
            );
            assert_agree(&text, true);
        }

        // The reader doesn't take these either.
        let bad = [
            r#"{"type": "TeleporterEntrance", "data": {"teleports_to": 1, "linkz": 2}}"#,
            r#"{"type": "TeleporterEntrance", "data": {"teleports_to": 1.5}}"#,
            r#"{"type": "TeleporterEntrance", "data": {"teleports_to": 3000000000}}"#,
            r#"{"type": "LightFlickerLogic", "fields": {"FlickerModNumber": 3000000000}}"#,
            r#"{"type": "LightFlickerLogic", "fields": {"OnTime": "fast"}}"#,
            r#"{"type": "LightFlickerLogic", "fields": [["OnTime", "fast"]]}"#,
            r#"{"type": "LightFlickerLogic", "fields": [["Nope", 1]]}"#,
            r#"{"type": "Nope"}"#,
            r#"{"type": "Group", "colour": 1}"#,
            // Versions the reader can't tell.
            r#"{"type": 5000}"#,
            r#"{"type": "TeleporterEntrance", "fields": {}}"#,
            // Bodies.
            r#"{"type": "Group", "raw": "00", "data": {}}"#,
            r#"{"type": "Group", "raw": "00", "end": {"truncated": 1}}"#,
            r#"{"unknown_type": "Gone", "version": 1}"#,
        ];
        for component in bad {
            let text = level_with(component);
            assert!(!errors(authoring(), &text).is_empty(), "{component}");
            assert_agree(&text, false);
        }

        let object = r#"{"game_object": {"name": "Thing"}}"#;
        assert!(!errors(authoring(), object).is_empty());
        assert_agree(object, true);

        let flags = [
            (r#"{}"#, true),
            (r#"{"version": 2, "frozen": true}"#, true),
            (
                r#"{"version": 0, "unfrozen": true, "active": false, "visible": true}"#,
                true,
            ),
            (r#"{"version": -1}"#, true),
            (r#"{"version": 0, "active": true}"#, false),
            (r#"{"unfrozen": true}"#, false),
            (r#"{"version": -1, "active": true}"#, false),
        ];
        for (flags, ok) in flags {
            let text = r#"{"level": {"name": "L",
                "settings": {"game_object": {"name": "LevelSettings"}},
                "layers": [{"name": "Default", "flags": FLAGS}]}}"#;
            assert_agree(&text.replace("FLAGS", flags), ok);
        }

        let legacy = [
            (r#"{"version": 3, "raw": "00"}"#, true),
            (
                r#"{"version": 3, "data": {}, "end": {"truncated": 1}}"#,
                true,
            ),
            (
                r#"{"version": 3, "raw": "00", "end": {"truncated": 1}}"#,
                false,
            ),
            (r#"{"version": 3, "raw": "00", "data": {}}"#, false),
            (r#"{"version": 3}"#, false),
        ];
        for (legacy, ok) in legacy {
            let text = r#"{"level": {"name": "L", "version": 1,
                "settings": {"legacy": LEGACY}}}"#;
            assert_agree(&text.replace("LEGACY", legacy), ok);
        }
    }

    /// Checks that the full schema and the reader both take the JSON, or both don't.
    fn assert_agree(text: &str, takes: bool) {
        let errors = errors(full(), text);
        assert_eq!(errors.is_empty(), takes, "{errors:?} {text}");
        let read = from_str::<BytesFile>(text);
        assert_eq!(read.is_ok(), takes, "{:?} {text}", read.err());
    }
}
