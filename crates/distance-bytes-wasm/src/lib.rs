//! JavaScript bindings of [`distance_bytes`] for browsers and Node, built with `wasm-bindgen`.
//!
//! Files go in and out as binary `.bytes` (`Uint8Array`) or as the library's JSON text (see
//! [`distance_bytes::json`]), which `JSON.parse` reads exactly: integers that a JavaScript
//! number can't hold are strings.
//!
//! Files live in the module's memory until `free()` is called on them (or they're declared
//! with `using`). The garbage collector frees them too, but it doesn't see that memory, so it
//! may not run soon enough when reading many files.
//!
//! The stack can't grow on wasm, and a browser's holds about 1 MB, so objects can nest about
//! 1,100 levels deep (the deepest known level has 470). Beyond that, everything but reading
//! JSON runs out of stack, which, like a panic, leaves the module unusable.

use distance_bytes::{Problem, Unresolved, json};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const NOTE: &str = r#"
/** Something about an object or component of a file, with its GUID. */
export interface Note {
    message: string;
    guid: number;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(message: &str);
}

/// Logs panics to the console. A panic aborts, and the module can't be used after one.
#[wasm_bindgen(start)]
fn start() {
    std::panic::set_hook(Box::new(|info| error(&info.to_string())));
}

#[derive(Serialize)]
struct Note {
    message: String,
    guid: u32,
}

/// `Note` objects for JavaScript.
fn notes<T: ToString>(items: &[T], guid: impl Fn(&T) -> u32) -> JsValue {
    let notes: Vec<_> = items
        .iter()
        .map(|item| Note {
            message: item.to_string(),
            guid: guid(item),
        })
        .collect();
    serde_wasm_bindgen::to_value(&notes).expect("strings and numbers convert")
}

/// A `.bytes` file: a level, or a single game object (replays, leaderboards, profiles, custom
/// objects and other user data).
#[wasm_bindgen]
pub struct BytesFile(distance_bytes::BytesFile);

#[wasm_bindgen]
impl BytesFile {
    /// Reads a binary `.bytes` file.
    pub fn read(data: &[u8]) -> Result<BytesFile, JsError> {
        Ok(BytesFile(distance_bytes::BytesFile::read(data)?))
    }

    /// Reads a file from JSON text, as `toJson` writes it or as written by hand.
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(text: &str) -> Result<BytesFile, JsError> {
        Ok(BytesFile(json::from_str(text)?))
    }

    /// The binary `.bytes` file. A file read and written unchanged is byte-identical.
    pub fn write(&self) -> Vec<u8> {
        self.0.write()
    }

    /// The file as JSON text, indented if `pretty`.
    ///
    /// `JSON.stringify` writes `-0` as `0`, so JSON changed as parsed objects comes back with
    /// zeros for negative zeros (which most files have, and which the game doesn't tell apart
    /// from zeros), and the file isn't byte-identical. Every other value survives `JSON.parse`
    /// and `JSON.stringify`.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self, pretty: Option<bool>) -> Result<String, JsError> {
        Ok(if pretty.unwrap_or(false) {
            json::to_string_pretty(&self.0)?
        } else {
            json::to_string(&self.0)?
        })
    }

    /// Whether the file is a level or a single game object, as the key of its JSON.
    #[wasm_bindgen(getter, unchecked_return_type = r#""level" | "game_object""#)]
    pub fn kind(&self) -> String {
        match self.0 {
            distance_bytes::BytesFile::Level(_) => "level",
            distance_bytes::BytesFile::GameObject(_) => "game_object",
        }
        .into()
    }

    /// The level's name, or the object's.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        match &self.0 {
            distance_bytes::BytesFile::Level(level) => level.name.to_string(),
            distance_bytes::BytesFile::GameObject(obj) => obj.name.to_string(),
        }
    }

    /// Gives every object and component whose GUID is 0 a new one, counting up from the
    /// largest in the file, and returns how many it gave.
    #[wasm_bindgen(js_name = assignGuids)]
    pub fn assign_guids(&mut self) -> usize {
        self.0.assign_guids()
    }

    /// The largest GUID of an object or component, or 0 if there are none.
    #[wasm_bindgen(js_name = maxGuid)]
    pub fn max_guid(&self) -> u32 {
        self.0.max_guid()
    }

    /// Problems with the GUIDs and references of the file, which make the game lose links
    /// between objects or fail to load the level. A problem's GUID is the repeated one, or the
    /// component with the reference.
    #[wasm_bindgen(unchecked_return_type = "Note[]")]
    pub fn check(&self) -> JsValue {
        notes(&self.0.check(), Problem::guid)
    }

    /// Fills in every value that the file leaves to the prefabs of its objects, so that it has
    /// the values the game reads it with, and returns what the game skips or merges, which is
    /// left as it is. See `Level::resolve` in the Rust library for what isn't applied.
    #[wasm_bindgen(unchecked_return_type = "Note[]")]
    pub fn resolve(&mut self, defaults: &PrefabDefaults) -> JsValue {
        notes(&self.0.resolve(&defaults.0), Unresolved::guid)
    }

    /// Leaves out every value that is the same as what the game reads in its place, the
    /// reverse of `resolve`.
    pub fn minimize(&mut self, defaults: &PrefabDefaults) {
        self.0.minimize(&defaults.0);
    }

    /// A copy of the file.
    #[wasm_bindgen(js_name = clone)]
    pub fn duplicate(&self) -> BytesFile {
        BytesFile(self.0.clone())
    }
}

/// The prefabs of the game, with every value, as `data/prefab-defaults.bytes` has them.
#[wasm_bindgen]
pub struct PrefabDefaults(distance_bytes::PrefabDefaults);

#[wasm_bindgen]
impl PrefabDefaults {
    /// Reads prefab defaults, such as `data/prefab-defaults.bytes`.
    pub fn read(data: &[u8]) -> Result<PrefabDefaults, JsError> {
        Ok(PrefabDefaults(distance_bytes::PrefabDefaults::read(data)?))
    }

    /// The names of the prefabs.
    #[wasm_bindgen(js_name = prefabNames)]
    pub fn prefab_names(&self) -> Vec<String> {
        self.0
            .prefabs()
            .map(|prefab| prefab.object.name.to_string())
            .collect()
    }

    /// The object the game creates from the prefab of this name, as a file of one object.
    pub fn prefab(&self, name: &str) -> Option<BytesFile> {
        let prefab = self.0.get(name)?;
        Some(BytesFile(distance_bytes::BytesFile::GameObject(
            prefab.object.clone(),
        )))
    }
}

/// The JSON Schema (draft 2020-12) of the JSON format, as JSON text.
///
/// It describes a new level, with components in the versions the game writes, or, if `full`,
/// everything that `toJson` writes. `components` limits it to those types (and `Transform` and
/// `LevelSettings`, which every level has).
#[cfg(feature = "schema")]
#[wasm_bindgen]
pub fn schema(full: Option<bool>, components: Option<Vec<String>>) -> Result<String, JsError> {
    let components = components
        .map(|names| {
            names
                .iter()
                .map(|name| {
                    distance_bytes::ComponentId::from_name(name)
                        .ok_or_else(|| JsError::new(&format!("unknown component type {name:?}")))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let options = json::SchemaOptions {
        full: full.unwrap_or(false),
        components,
    };
    Ok(serde_json::to_string(&json::schema(&options))?)
}
