# distance-bytes

A Rust library for reading and writing the `.bytes` files of the game [Distance](https://store.steampowered.com/app/233610/Distance/). See [PLAN.md](PLAN.md) for the format notes and roadmap.

## Status

- Reads every binary `.bytes` file the game writes into a tree of levels, layers, game objects and components, and writes it back byte-identical. Covers level versions 0–4 and non-level files (custom objects, profiles, leaderboards, replays).
- Decodes `Transform` components (position, rotation, scale, children) and components in the game's generic field format (field names with typed values). The generic format stores absolute file offsets, which are recomputed on write, so objects can be added, removed or moved.
- Decodes every component with its own serializer into a typed struct (`distance_bytes::components`), in every version the game reads, along with the level settings of level versions 0 and 1. Bodies that don't decode are kept as raw bytes. In the corpus, these are only bodies the game itself doesn't read.
- Converts files to JSON and back losslessly (`distance_bytes::json`). Components are `{"type": "TeleporterEntrance", "version": 4, "guid": 12, "data": {...}}`, and values equal to the prefab's are left out. Integers that a JavaScript number can't hold exactly, beyond ±(2⁵³ − 1), such as a level's save time, are strings. The module documentation describes the format.
- Generates a JSON Schema (draft 2020-12) of the JSON format, with the optional `schema` feature (`distance_bytes::json::schema`). By default the schema describes JSON for a new level, with components in the versions the game writes and only the fields those versions have, for checking JSON written by hand or by an LLM. The full schema describes everything `to_json` writes; every file in the corpus fits it.
- Builds files: `Level::new`, `GameObject::new("CubeGS").with_position(...).with(component)`, and components in the versions the game writes. `assign_guids` gives new objects and components GUIDs above the largest in the file, and `check` reports duplicate GUIDs and references to missing targets or to types the game can't use there (which make it fail to load the level). No file in the corpus has such problems.
- Fills in the values a file leaves to the prefabs of its objects, as the game reads them (`resolve`), and leaves out those that are the same again (`minimize`), with `PrefabDefaults`. `resolve` also reports what the game skips or merges: objects without a prefab, children and components the prefab lacks, components the prefab has only as a subclass, repeats, and materials a renderer lacks. The defaults are in `data/prefab-defaults.bytes`: every prefab of the game's level editor (1,524) with every value, as the game creates them, dumped from the game with the BytesTrace plugin.
- Levels written by hand as JSON load in the game. The example, `examples/sprint-level.json`, is a short Sprint track: the game reads every value as written, and what it has after loading matches `resolve`, apart from values it sets from others as it reads them (described on those fields).
- Handles objects nested any number of levels deep (one Workshop level nests them 469 deep): natively by growing the stack, and when reading JSON by reading the children of deep objects separately. On wasm, where the stack can't grow, everything else handles about 1,100 levels in a browser.
- Decodes the recorded runs of replays (`distance_bytes::replay`): the car's position, velocity, rotation, angular velocity and the player's inputs, every 3 frames, as the game decompresses them to play a ghost. The states are the game's to the bit, checked against the game for the corpus's 80 replays and for them converted to the older formats.
- Runs in browsers and Node as a wasm package (`crates/distance-bytes-wasm`), which reads and writes files and their JSON, checks, resolves and minimizes them, and gives the JSON Schema. Every file in the corpus round-trips through it, directly and through JSON.

## Layout

- `crates/distance-bytes`: the library.
- `crates/distance-bytes-cli`: the `dbytes` tool.
- `crates/distance-bytes-wasm`: the JavaScript bindings, for browsers and Node.
- `tools/schema-extract`: generates `schema/components.json`, the Rust `ComponentId` constants and the table of generic field types from the game's `Assembly-CSharp.dll`.
- `tools/BytesTrace`: a BepInEx plugin that runs the game as an HTTP server and reports every value the game's own deserializer reads, so our decoder can be checked against it. It also dumps the prefab defaults.
- `data/prefab-defaults.bytes`: the prefab defaults, the default for the `dbytes` commands that use them.
- `examples/sprint-level.json`: a level written by hand as JSON.

Game files (`game/`, `game_assets_dump/`) are not part of the repository.

## `dbytes`

```bash
cargo run --release -p distance-bytes-cli -- <command>
```

- `dump <file>`: print the object tree.
- `to-json <file> [out] [--compact]`: convert a file to JSON.
- `from-json <json> <out> [--assign-guids]`: convert JSON back to a `.bytes` file, warning about problems with GUIDs and references. With `--assign-guids`, objects and components whose GUID is 0 or missing get new ones.
- `check <paths>...`: report problems with GUIDs and references in `.bytes` files under the paths and in JSON files given by name.
- `roundtrip <paths>... [--json]`: read and re-write every binary `.bytes` file under the paths and check that the output is identical. With `--json`, each file goes through JSON first.
- `schema [out] [--full] [--components A,B] [--compact]`: write the JSON Schema of the JSON format. `--components` limits it to those component types (and `Transform` and `LevelSettings`), which keeps it small enough for an LLM prompt (13 KB compact for one type); `--full` describes all JSON `to-json` writes.
- `stats <paths>...`: statistics about component types and how many bytes are still undecoded.
- `trace <file>`: every value our decoder reads, as JSON lines.
- `relocate <in> <out> [--pad N]`: re-write a level with N characters added to its name, which moves everything after it. Use it with `trace-check` to test that offsets are recomputed.
- `trace-check <paths>...`: compare our decoder against the game, using the BytesTrace server. It reports mismatches, failed game loads and the share of the game-read bytes that we decode.
- `coverage <paths>... [--per N]`: for every decoded (component, version) pair, the N smallest files that contain it. Gives a small set of files for `trace-check`.
- `synth --out <dir> <paths>...`: for every decoded component version that no file under the paths has, write a copy of a file with that component, replaced by a sample of that version with every value set. Use it with `trace-check` to test those versions.
- `resolve <in> <out> [--defaults <file>]`: fill in the values a binary or JSON file leaves to the prefabs, and print what the game skips. Writes JSON if `out` ends in `.json`.
- `minimize <in> <out> [--defaults <file>]`: leave out the values that are the same as the prefabs'.
- `resolve-check <paths>...`: check that resolving each binary file under the paths gives the same after minimizing it, count what the game skips, and total the sizes.
- `prefabs [<paths>...] [--out <file>]`: dump the prefab defaults from the game, using the BytesTrace server, along with the prefabs that levels under the paths use.
- `resolve-game-check <paths>... [--max N]`: load each level under the paths in the game, using the BytesTrace server, and compare what the game has afterwards with what resolving the level gives.
- `replay <file> [out]`: the car's states of a replay, as JSON lines, with the inputs unpacked.
- `replay-check <paths>... [--older] [--max N]`: compare the states of the replays under the paths with those the game decodes, using the BytesTrace server. With `--older`, also check each replay converted to versions 5 and 4.

## Writing a level as JSON

A level lists its objects by prefab name, with only the values that differ from the prefab's; `examples/sprint-level.json` is one. `dbytes schema` describes the JSON, and `--components` cuts it to the types a level uses. To build the level and play it, put the file in `Documents/My Games/Distance/Levels/MyLevels`:

```bash
dbytes from-json level.json "My Level.bytes" --assign-guids
```

- Give GUIDs only to what something refers to, such as the `TrackLink`s of connected track pieces. `--assign-guids` gives the rest theirs.
- Track pieces connect through the `TrackLink`s at their ends, each with `link_ref` set to the other's GUID. A spline road's ends are its children `BezierSplineTrackEmpty1` and `BezierSplineTrackEmpty2`, and a start or end zone's is its child `SnapPoint`. In the level editor, one link of each connected pair owns the node between them (`manipulator_node_should_be_created`).
- A `GoldenSimples` body that sets values needs a `preset` of 0, or the prefab's preset replaces them.
- Sizes: the simple shapes (`CubeGS`, `CylinderGS` and so on) are about 64 units across at scale 1, so a 4-unit post has a scale of about 0.06. A spline road at scale 16 with its ends 8 units apart is 128 units long, and the checkpoint's trigger over it is 70 units wide.
- A `WorldText` reads correctly from its -z side, so a sign facing a car that drives along +z needs no rotation.

## JavaScript

The wasm package needs the `wasm32-unknown-unknown` target and the `wasm-bindgen` command at the version of the `wasm-bindgen` crate (0.2.129):

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129
```

Build it into `crates/distance-bytes-wasm/pkg`:

```bash
cargo build -p distance-bytes-wasm --target wasm32-unknown-unknown --profile wasm
wasm-bindgen --target web --out-dir crates/distance-bytes-wasm/pkg target/wasm32-unknown-unknown/wasm/distance_bytes_wasm.wasm
```

`pkg` then holds an ES module, `distance_bytes_wasm.js`, with its TypeScript types, and the wasm (2.3 MB, 0.6 MB gzipped). The `wasm` profile makes it less than half the size of a release build, for about a fifth more time.

```js
import init, { BytesFile, PrefabDefaults, schema } from './pkg/distance_bytes_wasm.js';

await init(); // fetches distance_bytes_wasm_bg.wasm; in Node, initSync({ module: bytes })
using file = BytesFile.read(new Uint8Array(await (await fetch('level.bytes')).arrayBuffer()));
const level = JSON.parse(file.toJson());
```

- `BytesFile.read(bytes)` and `BytesFile.fromJson(text)` read a file; `write()` and `toJson(pretty)` write it. `kind` is `"level"` or `"game_object"`, and `name` the level's or object's name.
- `assignGuids()`, `maxGuid()` and `check()` work as in Rust, and `resolve(defaults)` and `minimize(defaults)` take a `PrefabDefaults.read(bytes)` of `data/prefab-defaults.bytes`, which also gives `prefabNames()` and each `prefab(name)` as a file. `check()` and `resolve()` return `{message, guid}` objects.
- `replayStates()` decodes a replay's car states, or gives `undefined` for other files. Its `frames`, `positions`, `velocities`, `rotations` (quaternions as x, y, z, w), `angularVelocities` and `directives` are typed arrays with the values of every state in turn, and `inputs(i)` unpacks the player's inputs at a state.
- `schema(full, components)` is the JSON Schema as text, as `dbytes schema` writes it.
- Errors are thrown as `Error`s with the library's message, for JSON with the path, line and column.
- Files live in the module's memory until `free()` is called on them or they're declared with `using`. The garbage collector frees them too, but too late when reading many files.
- `JSON.stringify` writes `-0` as `0`, and most files have negative zeros, so a file whose JSON is changed as parsed objects comes back with zeros there, which the game doesn't tell apart. Everything else survives `JSON.parse` and `JSON.stringify`.
- Objects can nest about 1,100 levels deep in a browser's stack (reading JSON has no limit). Running out of stack leaves the module unusable.

## Tests

```bash
cargo test --workspace --all-features
```

`--all-features` includes the schema tests. `cargo test -- --ignored` also checks that every `f32` bit pattern survives JSON (about half a minute). The example level test checks that `examples/sprint-level.json` builds without problems in its references, leaves the game nothing to skip, and fits the authoring schema.

The corpus test round-trips every binary `.bytes` file, directly and through JSON, checks that minimizing it against `data/prefab-defaults.bytes` keeps what resolving it gives, and decodes the states of replays, under the directories in `DISTANCE_BYTES_CORPUS` (separated like `PATH`, so `;` on Windows). With the `schema` feature it also validates the JSON against the full schema, which triples its time. Use absolute paths, since tests run in the crate directory:

```bash
DISTANCE_BYTES_CORPUS="C:/<repo>/game_assets_dump/Assets/Resources;C:/Users/<you>/Documents/My Games/Distance" cargo test --release --all-features --test corpus
```

`crates/distance-bytes-wasm/tests/corpus.mjs` tests the built `pkg` with Node: the example level builds, and with `DISTANCE_BYTES_CORPUS` set (separated by `;`), every binary file round-trips, directly and through JSON, its JSON has no integers that JavaScript numbers can't hold, and replays decode.

```bash
node crates/distance-bytes-wasm/tests/corpus.mjs
```

## Regenerating the schema

```bash
dotnet run --project tools/schema-extract -- game/Distance_Data/Managed/Assembly-CSharp.dll .
```

## The trace server

1. Install BepInEx 5 into a copy of the game. Don't use a copy that has BytesServer installed: its level-settings cleanup interferes with this plugin.
2. Build the plugin and copy it to the game's `BepInEx/plugins` folder. The build references `game/Distance_Data/Managed/Assembly-CSharp.dll` by default; pass `-p:DistanceManaged=<path>` to use another copy.

   ```bash
   dotnet build -c Release tools/BytesTrace
   ```
3. Start the game headless:

   ```bash
   Distance.exe -batchmode -nographics
   ```

It listens on `localhost:18501` (set `BYTES_TRACE_PORT` to change it). Endpoints (POST the file as the body):

- `/level-bytes-trace`
- `/gameobject-bytes-trace`
- `/level-bytes-full`
- `/prefab-names` (with an empty body)
- `/prefab-defaults`
- `/replay-states`
- `/ping` (GET)

For the trace endpoints, each line of the response is a JSON object for one read, with the absolute offset `o`, length `l`, type `t`, field name `n` and value `v`. Floats are given as their bit patterns. Scope headers have `mark` and `len`. `skip` lines mean the game didn't read a scope exactly to its end and seeked to the end; reads past a scope's end that the game seeked back from are marked `overread`. The last line reports whether the game loaded the file successfully.

`/level-bytes-full` loads a level and returns it as the game writes it, with every component and value rather than only those that differ from the prefabs (status 422 if the game can't load it). `/prefab-names` lists the level editor's prefabs, and `/prefab-defaults` takes prefab names, one per line, and returns a JSON line per prefab with the `.bytes` of an object created from it, in full, and one per component type the game adds to objects. `/replay-states` loads a replay as the game does for a ghost and returns its states, decompressed, as version 4 replays keep them (status 422 if the game can't load it).
