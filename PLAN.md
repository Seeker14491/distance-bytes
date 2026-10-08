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
  | `32323232` | Read like `33333333`; not written by the current game, but present in some Workshop levels |
  | `23232323` | Unknown component: name, ID |
  | `44444444` | Spline track transform |

  Arrays and lists are `11111111` + count, and dictionaries are `12121212` + count. Neither is a length scope.
- **`EMPTY` = `0x7FFFFFFD`**: "same as the prefab".
- **Component payload styles**:
  1. **Hand-written `Visit(visitor, prefab, version)`** (126 classes): positional, with `switch (version)` branches.
  2. **Reflection (`VisitSerialComponentData`)**: field count, then for each field a name (`Trim('_')` of the first-letter-uppercased C# field name), an i64 **absolute end offset**, and the value or `EMPTY`. Level version 0 uses the older layout with no end offsets (`VisitSerialComponentDataV0`).
- **Reflection values** follow `BinaryDeserializer`'s reader:
  - Vector2/3/4, Quaternion, Color and Bounds are floats, and strings are strings, read with `Visit` (which checks for `EMPTY` before each float).
  - Primitives are read as their type, and enums as `i32`.
  - Any other type is read as a `u32` object reference.
  - Arrays are `11111111` + count + elements. Elements that aren't primitives, strings or arrays are read as `u32` references.

  The serializer writes **nothing** for types it can't handle, which includes all object references. The reader then reads past the field's end and seeks back, so those values are lost. An array of references is written as only the mark and count.
- **Absolute offsets**: the game seeks to them and drops any value whose length doesn't match. The writer must recompute them. Moving a raw blob that contains reflection components makes the game misread the level (shown with `dbytes relocate`: two official levels failed to load after 6 bytes were inserted before them).
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
- `tools/schema-extract` (C#): reads `Assembly-CSharp.dll` metadata and emits the `ComponentID` enum and reflection field-type tables, as JSON and as generated Rust (checked in).

Layers:
1. **Lossless tree**: Level → Layer → GameObject → Component → children. A payload is decoded or kept as raw bytes. Reflection components are detected by structure and parsed as `(name, Empty | Value | Raw)`, so their offsets can be recomputed. Values are typed from a generated table of field types; a value that doesn't parse as its type (or whose field the current game doesn't have) stays raw. Lengths are backpatched the way the game does it.
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
- **Phase 2 (done)**:
  - Reflection bodies are detected by structure: at least one field, and every end offset exact. 42,790 bodies in the corpus decode this way, all at component version 0, and none of those components fall back to raw.
  - Field values are typed from `generated/fields.rs`: 87,969 typed values, with 1,313 left raw. Every raw value is one the game discards: fields the current game no longer has (e.g. `MusicTrigger.MusicCueID`, `KillGridBox.Event`, the removed `LaserLogic`), zero-byte object references, and reference arrays without elements.
  - The writer recomputes end offsets. Levels with 6 bytes inserted before their reflection components now load in the game with 0 trace mismatches; before, two of them failed to load.
  - Level version 0 (no end offsets) is decoded by field type. The corpus has no v0 level with reflection components, so this is covered by unit tests only.
  - String lengths are now only accepted in their shortest 7-bit encoding, so typed values always write back identically.
- **Phase 3 (done)**:
  - Every component with a hand-written `Visit` (96 types, 314 component versions) has a typed struct, translated from the decompiled C#. Most were drafted by a script and then checked by hand; the rest were written by hand. This includes the user-data components (replays, local leaderboards, level infos, profile progress and stats), so most of phase 5's binary files are covered.
  - The `Visitor` handles values, arrays, dictionaries, byte arrays, references and counted lists of records. It also handles bodies that end early or carry extra bytes the way the game does (`BodyEnd`).
  - The settings of level versions 0 and 1 (`LevelSettings.OldVisit`) are decoded as `LegacyLevelSettings`.
  - Corpus: 10.7M bodies decoded, and the 15,002 bodies still raw (926 KB) are ones the game doesn't read:
    - components the current game has no class for (CubeMapRenderer, GenerateCreditsNames, GlobalFogLogic, WorkshopPublishedFileInfos);
    - BezierSplineTrack v2;
    - one MeshFilter wrapper;
    - one empty-field SetAbilitiesTrigger v0;
    - six 5-byte GoldenSimples v0 bodies, which the game reads past.
  - `trace-check` on the 2 smallest files for every decoded (component, version) pair (258 files) gives 0 mismatches and 0 load failures. All but a few bytes the game reads are decoded; the exception is the 8-byte `SetActiveOnMIDIEvent.GameObjs` generic values, an array count with no elements, which the game reads past and discards.
  - Versions that no corpus file has (114 component versions, 7 legacy settings versions) are checked with `dbytes synth`, which writes a sample with every value set into a copy of a corpus file. All have 0 mismatches, and the game reads every value. Eight fail to load after the read because of the sample data: replays upgrade a garbage event buffer, and a TrackSegment v0 links to an object that isn't a TrackLink.
  - A unit test round-trips that sample for every decoded version.
- **Phase 4 (in progress)**:
  - **4a. Lossless JSON.** serde on the whole model, `dbytes to-json` / `from-json`, and a corpus test that binary → JSON → binary is byte-identical. The JSON shape:
    - A component is `{"type": "TeleporterEntrance", "version": 4, "guid": 12, "data": {...}}`. `data` holds a typed body (a `Transform` holds its `children` there), `fields` a generic-format body keyed by the game's field names and typed from the field table, and `raw` undecoded bytes as hex. A component with none of these has an empty body. `mark` appears only when it isn't the game's mark for the type, and `end` only when a body is truncated or has trailing bytes.
    - Struct fields use the Rust names. `null` or a missing key means the empty marker ("same as the prefab"), so `None` values are left out. Fields that aren't optional (lists, references, nested groups and Light's `has_spot_angle`) are left out when empty, 0 or false.
    - Non-finite floats are written as `"NaN"`, `"Infinity"` or `"-Infinity"` (with the bits when the NaN isn't the canonical one), since JSON has no such numbers. The corpus has one NaN.
    - Strings that aren't valid UTF-16 are `{"utf16le": "<hex>"}`. The corpus has none.
    - A generated table of the game's classes gives each component ID's written version and mark.
    - Objects nest up to 469 deep in one Workshop level, which overflows a 1 MB stack even in the binary reader. Recursion grows the stack with `stacker` on native targets.
  - **4a (done).** All 2,362 corpus files round-trip byte-identical through JSON (in the corpus test and `dbytes roundtrip --json`), at about 2.5× the time of the binary round trip. Findings:
    - serde_json reads an `f32` by parsing an `f64` and rounding it, which for some values gives the next `f32` (e.g. `7.038531e-26`). Those are written with more digits; an ignored test checks all 2³² bit patterns.
    - The derived `Clone` overflowed a 1 MB stack on the 469-deep level, and drop on about 2,000 levels. `GameObject` implements `Clone`, `PartialEq`, `Debug` and `Drop` by hand to grow the stack, so its fields can't be moved out by destructuring.
    - The pretty output keeps arrays of plain values on one line. Deep nesting makes it large (the 469-deep level is 71 MB pretty, 3 MB binary); `--compact` avoids the indentation.
  - **4b. JSON Schema** via `schemars`, with per-type schemas for typed bodies and for generic-format fields. `dbytes schema`.
  - **4b (done).** Optional `schema` feature; `json::schema(&SchemaOptions)` returns draft 2020-12, with `anyOf` rather than `oneOf` for LLM structured output.
    - The authoring schema (the default) describes a new level: each type in the version the game writes (`const`), typed bodies cut to the properties that version has (those a sample of it sets), and plain strings. It is 177 KB compact; `--components` cuts it to the types a level uses.
    - The full schema describes everything `to_json` writes: every version, `raw`/`end`/`mark`, unknown and unnamed types, `[name, value]` field pairs, `{"utf16le"}` strings and non-finite floats. It also states the reader's rules: when `version` is required, at most one body, `end` only with `data`, layer flags by layer version, and integer ranges (validators don't check `format`). It is 829 KB compact.
    - All 2,362 corpus files fit the full schema (in the corpus test with the feature). Unit tests check that every component's sample at its written version fits the authoring schema, and that the full schema and the reader accept and reject the same JSON in about 40 cases.
    - Findings: some old files have types in the generic format that the current game has no fields for (GlobalFogLogic v0), so in the full schema every type may have `fields`. Light's sample now has a spot angle (the filling visitor reports an unbounded body), which the authoring schema had dropped.
  - **4c. Builder API.** Create levels, layers and objects by prefab name; allocate GUIDs (one pool per file for objects and components, unique in every corpus level); check references; new components use the latest versions. Optional `version` and `guid` in authored JSON.
  - **4c (done).** `Level::new`, `Layer::new`, `GameObject::new(prefab)` with `with`, `with_position`/`rotation`/`scale`, `with_child` and `transform_mut`; `Component::new(data)` (or `From` any body type), `empty`, `fields` and `transform`, in the versions the game writes. `assign_guids`, `max_guid` and `check` on `Level` and `BytesFile`; `Component::references` and `references_mut`. CLI: `from-json --assign-guids`, `check`.
    - The game puts objects and components in one reference map by GUID (0 is never added), and the last one with a GUID wins. A missing target logs a warning and leaves the reference empty. A target of the wrong type fails the cast in the reference's callback, and the level fails to load.
    - The game's `ReferenceMap` hands out the lowest free ID, and files aren't numbered 1..n in file order. New GUIDs here count up from the largest. Six corpus levels have objects or components with GUID 0, so the JSON reader keeps 0 and assigning is a separate step.
    - References are the `VisitReference` values of typed bodies (Group and TrackSegment `Links`, TrackLink `SegRef`/`LinkRef`/`TrackNodeRef`, TrackAttachment `Attached To`), found with a visitor so versions are respected, and `Reference` values in the generic format. Targets are checked against the field's class and its subclasses (`TrackSegment` includes `SplineSegment`).
    - `check` finds no problems in any of the 2,362 corpus files (about 1.4 million references), in 2 s.
    - `Level::new` follows the level the game's editor makes: a `LevelSettings` object with the name and otherwise the prefab's settings, and a `Default` layer. On load the game fills in missing modes with its defaults, with Sprint on.
  - **4d. Prefab defaults.** A `tools/BytesTrace` endpoint that saves every level-editor prefab and the added-component prefab with `prefab = null`; `PrefabDefaults`, `resolve()` and `minimize()`. The dump isn't committed until the licensing question is settled.
  - **4e. Acceptance.** A level written as JSON loads in the game through `/level-bytes-trace`.

## Tooling notes

- Decompile with `ilspycmd` 9.1.0.7988 (11.x fails to install on the .NET 9 SDK). Don't commit the decompiled source.
- `game_assets_dump/Assets/Scripts` are stubs with fields only, no method bodies.
- Python is not installed. Throwaway analysis scripts are in Rust or Node.
- A running `trace-check` keeps `target/release/dbytes.exe` open, so builds fail with "Access denied" until it ends. Run long checks from a copy of the binary.
