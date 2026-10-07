# distance-bytes

A Rust library for reading and writing the `.bytes` files of the game [Distance](https://store.steampowered.com/app/233610/Distance/). See [PLAN.md](PLAN.md) for the format notes and roadmap.

## Status

- Reads every binary `.bytes` file the game writes into a tree of levels, layers, game objects and components, and writes it back byte-identical. Covers level versions 0–4 and non-level files (custom objects, profiles, leaderboards, replays).
- Decodes `Transform` components (position, rotation, scale, children). Other component bodies are kept as raw bytes.

## Layout

- `crates/distance-bytes`: the library.
- `crates/distance-bytes-cli`: the `dbytes` tool.
- `tools/schema-extract`: generates `schema/components.json` and the Rust `ComponentId` constants from the game's `Assembly-CSharp.dll`.
- `tools/BytesTrace`: a BepInEx plugin that runs the game as an HTTP server and reports every value the game's own deserializer reads, so our decoder can be checked against it.

Game files (`game/`, `game_assets_dump/`) are not part of the repository.

## `dbytes`

```bash
cargo run --release -p distance-bytes-cli -- <command>
```

- `dump <file>`: print the object tree.
- `roundtrip <paths>...`: read and re-write every binary `.bytes` file under the paths and check that the output is identical.
- `stats <paths>...`: statistics about component types and how many bytes are still undecoded.
- `trace <file>`: every value our decoder reads, as JSON lines.
- `trace-check <paths>...`: compare our decoder against the game, using the BytesTrace server. It reports mismatches and the share of the game-read bytes that we decode.

## Tests

```bash
cargo test
```

The corpus test round-trips every binary `.bytes` file under the directories in `DISTANCE_BYTES_CORPUS` (separated like `PATH`, so `;` on Windows):

```bash
DISTANCE_BYTES_CORPUS="game_assets_dump/Assets/Resources;C:/Users/<you>/Documents/My Games/Distance" cargo test --test corpus
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
