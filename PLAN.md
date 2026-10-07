# distance-bytes — plan

A Rust library to read and write Distance's `.bytes` files, aimed at a browser level viewer (wasm) and AI-generated levels.

## Format summary (confirmed from the decompiled `Assembly-CSharp.dll`)

The game uses one visitor walk (`IVisitor`) for both `BinarySerializer` and `XmlSerializer`. Binary drops the field names and XML keeps them.

- **Primitives**: .NET `BinaryWriter`, little-endian. Strings are a 7-bit varint *byte* length followed by UTF-16LE (they may contain unpaired surrogates). Bool is 1 byte. `DateTime` is an i64 (`ToBinary`).
- **Scopes**: an `i32` marker plus an `i64` length, backpatched after the contents are written.

  | Marker | Meaning |
  |---|---|
  | `99999999` | Level: name, layer count, level version, then LevelSettings |
  | `77777777` | Layer: name, object count, layer version, flags |
  | `66666666` | GameObject: name, empty string, ID, component count |
  | `55555555` | Transform children: count |
  | `33333333` | Built-in component (Unity wrapper): type, version, ID |
  | `22222222` | `SerialComponent` (game logic): type, version, ID |
  | `23232323` | Unknown component: name, ID |
  | `44444444` | Spline track transform |

  Arrays and lists are `11111111` + count, and dictionaries are `12121212` + count. Neither is a length scope.
- **`EMPTY` = `0x7FFFFFFD`**: "same as the prefab".
- **Component payload styles**:
  1. **Hand-written `Visit(visitor, prefab, version)`** (126 classes): positional, with `switch (version)` branches.
  2. **Reflection (`VisitSerialComponentData`)**: field count, then for each field a name (`Trim('_')` of the first-letter-uppercased C# field name), an i64 **absolute end offset**, and the value or `EMPTY`. Level version 0 uses the older layout with no end offsets (`VisitSerialComponentDataV0`).
- **Absolute offsets**: the game seeks to them and drops any value whose length doesn't match. The writer must recompute them, and raw blobs that contain reflection components can't be moved.
- **How defaults are omitted**:
  1. A whole component body is empty when `IsDifferent(prefab)` is false.
  2. A reflection field is written as `EMPTY` when it equals the prefab's value.
  3. Transform position, rotation and scale are each `EMPTY` when equal to the prefab's.

  Which prefab is used: a root object is matched by name; a child by the prefab's child at the same index (when child counts match), otherwise by name; a component the prefab lacks against the shared added-component prefab.

## Corpus (not committed; tests read paths from env vars and skip if absent)

- `game_assets_dump/Assets/Resources/{levels,levelbackups,...}`: 779 levels (v3: 110, v4: 669) and 34 non-level object files.
- `C:\Users\seekr\Documents\My Games\Distance\Levels\WorkshopLevels`: 1,391 levels (v0: 2, v1: 1, v2: 15, v3: 581, v4: 792). The 18 files at v0–v2 are the only samples of the old code paths.
- distanceutils `tests/in` fixtures (MIT): old component versions.
- About 129 component types and about 240 (type, version) pairs appear in real files.

## Decisions

- **Read every version the game can read**: level versions 0–4 and every component version branch.
- **No upgrade logic**: write back in the version that was read; new content always uses the latest versions.
- **JSON, not XML, is the interchange format.** XML only arrives later, for the user-data files that use it (Settings, ControlSchemes, LevelPlaylists, Profiles), and only for the components those files use.
- **Prefab defaults come from a game-side dump**, not from the dumped Unity `.prefab` YAML.

## Architecture

Workspace crates:
- `distance-bytes`: core, I/O-free (`&[u8]` → model → `Vec<u8>`), wasm-friendly, never panics on input.
- `distance-bytes-cli`: `stats`, `dump`, `trace`, `roundtrip`, `to-json`/`from-json`.
- `tools/schema-extract` (C#): reads `Assembly-CSharp.dll` metadata and emits the `ComponentID` enum and reflection field-type tables as JSON. Generated Rust is checked in.

Layers:
1. **Lossless tree**: Level → Layer → GameObject → Component → children. A payload is `Decoded(T)` or `Opaque(bytes)`. Reflection components are always parsed generically as `(name, Empty | Raw)` so their offsets can be recomputed. Lengths are backpatched the way the game does it.
2. **Visitor-based components**: a `Visitor` trait mirrors `IVisitor` and **takes field names**. One `visit(&mut self, v, version)` per component, translated from the decompiled C#, drives the reader, the writer, the trace emitter and later the XML reader/writer. Each struct holds the superset of fields across versions plus its stored version. No `binrw`/`deku`.
3. **Semantic API**: builders, ID allocation and reference handling, serde JSON (with `schemars` JSON Schema for LLM structured output and TS types for the web), and `resolve(&PrefabDefaults)` producing filled-in values. The writer never needs the defaults; an optional `minimize` step re-diffs against them.

## Prefab defaults

BytesServer endpoint: go through the level prefabs and the added-component prefab, and call `SaveGameObjectToBytes<BinarySerializer>(prefab, null)`. With `null`, the game writes every component and field in full. A Harmony patch also stops children from being matched against prefabs looked up by name. The output is in the same `.bytes` format the library already parses. Generate it once and ship it as a separate data file loaded at runtime, not built into the crate.

## Verification

1. All corpus files round-trip byte-identical. Track coverage: the percentage of component bytes decoded vs. opaque.
2. **BytesServer `/level-bytes-trace`**: a Harmony postfix on `BinaryDeserializer.ReadSet(string name, ref T)` and the scope reads emits `(offset, name, type, value)` as JSONL. The library emits the same, and a diff shows the first divergent field.
3. Existing endpoints as a semantic check:
   - Edited, generated and relocated files must load through `/level-bytes-to-xml` and match.
   - Insert an object at the start of a level and confirm everything else loads unchanged.
4. A BytesServer "load the level, then save every object with `prefab = null`" endpoint to check `resolve()`.
5. proptest for typed write→read; cargo-fuzz on the reader.

## Roadmap

| Phase | Deliverable | Done when |
|---|---|---|
| 0 | `schema-extract` (ComponentID, field tables); trace endpoint; `cli stats` | Stats reproduce the corpus scan |
| 1 | Primitives, scope tree, backpatching writer | Whole corpus round-trips byte-identical |
| 2 | Level/Layer/GameObject/Transform/children; generic reflection components (V0 and V1) | Relocation test passes in the game |
| 3 | Port hand-written components by frequency, all version branches | 100% coverage; traces match |
| 4 | Builder API, IDs/references, JSON + JSON Schema, prefab-defaults dump + `resolve` | A level generated from JSON loads in the game |
| 5 | Other binary files (LocalLeaderboards, CustomObjects, replays, LevelInfos); wasm package | — |
| later | XML visitor for user-data files (Settings, ControlSchemes, LevelPlaylists, Profiles) | — |

## Progress

- **Phase 0 (done)**:
  - `tools/schema-extract` generates `schema/components.json` (182 component classes, 218 IDs) and `ComponentId` constants.
  - `tools/BytesTrace` traces the game's reads per field; it ignores reads from other deserializers created during loading.
  - `dbytes stats` reproduces the corpus scan.
- **Phase 1 (done)**:
  - Every binary file in both collections (2,362 files) round-trips byte-identical.
  - All 5.7M Transforms decode. No raw component body contains a game object mark, so children only appear under Transform.
  - `trace-check` shows no mismatches against the game on the oldest levels (v0–v2); our structural decode covers roughly 45–50% of the bytes the game reads.
- **Next**: Phase 2 (generic reflection components with relocatable end offsets). After that, port MeshRenderer v1/v2: it is about 870 MB of the 1.19 GB of raw component bytes.

## Tooling notes

- Decompile with `ilspycmd` 9.1.0.7988 (11.x fails to install on the .NET 9 SDK). Don't commit the decompiled source.
- `game_assets_dump/Assets/Scripts` are stubs with fields only, no method bodies.
- Python is not installed. Throwaway analysis scripts are in Rust.
