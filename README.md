# distance-bytes

A Rust library for reading and writing the `.bytes` files of the game [Distance](https://store.steampowered.com/app/233610/Distance/). See [PLAN.md](PLAN.md) for the format notes and roadmap.

## Status

- Reads every binary `.bytes` file the game writes into a tree of levels, layers, game objects and components, and writes it back byte-identical. Covers level versions 0–4 and non-level files (custom objects, profiles, leaderboards, replays).
- Decodes `Transform` components (position, rotation, scale, children) and components in the game's generic field format (field names with typed values). The generic format stores absolute file offsets, which are recomputed on write, so objects can be added, removed or moved.
- Decodes every component with its own serializer into a typed struct (`distance_bytes::components`), in every version the game reads, along with the level settings of level versions 0 and 1. Bodies that don't decode are kept as raw bytes. In the corpus, these are only bodies the game itself doesn't read.
- Converts files to JSON and back losslessly (`distance_bytes::json`). Components are `{"type": "TeleporterEntrance", "version": 4, "guid": 12, "data": {...}}`, and values equal to the prefab's are left out. The module documentation describes the format.
- Generates a JSON Schema (draft 2020-12) of the JSON format, with the optional `schema` feature (`distance_bytes::json::schema`). By default the schema describes JSON for a new level, with components in the versions the game writes and only the fields those versions have, for checking JSON written by hand or by an LLM. The full schema describes everything `to_json` writes; every file in the corpus fits it.
- Builds files: `Level::new`, `GameObject::new("CubeGS").with_position(...).with(component)`, and components in the versions the game writes. `assign_guids` gives new objects and components GUIDs above the largest in the file, and `check` reports duplicate GUIDs and references to missing targets or to types the game can't use there (which make it fail to load the level). No file in the corpus has such problems.
- Handles objects nested any number of levels deep (one Workshop level nests them 469 deep) by growing the stack, on native targets.

## Layout

- `crates/distance-bytes`: the library.
- `crates/distance-bytes-cli`: the `dbytes` tool.
- `tools/schema-extract`: generates `schema/components.json`, the Rust `ComponentId` constants and the table of generic field types from the game's `Assembly-CSharp.dll`.
- `tools/BytesTrace`: a BepInEx plugin that runs the game as an HTTP server and reports every value the game's own deserializer reads, so our decoder can be checked against it.

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

## Tests

```bash
cargo test --workspace --all-features
```

`--all-features` includes the schema tests. `cargo test -- --ignored` also checks that every `f32` bit pattern survives JSON (about half a minute).

The corpus test round-trips every binary `.bytes` file, directly and through JSON, under the directories in `DISTANCE_BYTES_CORPUS` (separated like `PATH`, so `;` on Windows). With the `schema` feature it also validates the JSON against the full schema, which triples its time. Use absolute paths, since tests run in the crate directory:

```bash
DISTANCE_BYTES_CORPUS="C:/<repo>/game_assets_dump/Assets/Resources;C:/Users/<you>/Documents/My Games/Distance" cargo test --release --all-features --test corpus
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
- `/ping` (GET)

Each line of the response is a JSON object for one read, with the absolute offset `o`, length `l`, type `t`, field name `n` and value `v`. Floats are given as their bit patterns. Scope headers have `mark` and `len`. `skip` lines mean the game didn't read a scope exactly to its end and seeked to the end; reads past a scope's end that the game seeked back from are marked `overread`. The last line reports whether the game loaded the file successfully.
