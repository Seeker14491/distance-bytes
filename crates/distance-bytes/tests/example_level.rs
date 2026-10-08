//! The example level, `examples/sprint-level.json`, which the game loads: it builds into a level
//! without problems in its references, leaves the game nothing to skip, and with the `schema`
//! feature fits the authoring schema.

use std::path::Path;

use distance_bytes::{BytesFile, PrefabDefaults, json};

fn read(path: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap()
}

#[test]
fn example_level_builds() {
    let mut file: BytesFile = json::from_slice(&read("examples/sprint-level.json")).unwrap();
    assert!(matches!(file, BytesFile::Level(_)));
    assert!(file.assign_guids() > 0);
    assert_eq!(file.check(), []);
    let data = file.write();
    let mut back = BytesFile::read(&data).unwrap();
    assert_eq!(back.write(), data);
    let defaults = PrefabDefaults::read(&read("data/prefab-defaults.bytes")).unwrap();
    assert_eq!(back.resolve(&defaults), []);
}

#[cfg(feature = "schema")]
#[test]
fn example_level_fits_the_authoring_schema() {
    let validator =
        jsonschema::validator_for(&json::schema(&json::SchemaOptions::default())).unwrap();
    let value: serde_json::Value =
        serde_json::from_slice(&read("examples/sprint-level.json")).unwrap();
    let errors: Vec<_> = validator
        .iter_errors(&value)
        .map(|e| format!("{} (schema {})", e.instance_path(), e.schema_path()))
        .collect();
    assert_eq!(errors, Vec::<String>::new());
}
