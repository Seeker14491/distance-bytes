// Tests the browser build in `pkg` (`wasm-bindgen --target web`) with Node: the example level
// builds, and every binary .bytes file under the directories in DISTANCE_BYTES_CORPUS (separated
// by `;`) reads and writes back byte-identical, also through JSON, as in the Rust corpus test.
//
//   node crates/distance-bytes-wasm/tests/corpus.mjs

import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { BytesFile, PrefabDefaults, initSync, schema } from '../pkg/distance_bytes_wasm.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '../../..');
initSync({ module: readFileSync(new URL('../pkg/distance_bytes_wasm_bg.wasm', import.meta.url)) });
const defaults = PrefabDefaults.read(readFileSync(join(root, 'data/prefab-defaults.bytes')));

// The example level, as in the Rust test.
{
  const level = BytesFile.fromJson(readFileSync(join(root, 'examples/sprint-level.json'), 'utf8'));
  assert.equal(level.kind, 'level');
  assert.ok(level.assignGuids() > 0);
  assert.deepEqual(level.check(), []);
  const bytes = level.write();
  assert.deepEqual(BytesFile.read(bytes).write(), bytes);
  assert.deepEqual(level.clone().resolve(defaults), []);
  assert.ok(defaults.prefabNames().includes('CubeGS'));
  assert.equal(defaults.prefab('CubeGS').kind, 'game_object');
  assert.equal(defaults.prefab('NoSuchPrefab'), undefined);
  assert.ok(JSON.parse(schema()).$defs);
  assert.ok(schema(true).length > schema(false, ['Light']).length);
  assert.throws(() => schema(false, ['NoSuchType']), /unknown component type/);
  assert.throws(() => BytesFile.fromJson('{"level": 1}'), /line 1/);
  assert.throws(() => BytesFile.read(new Uint8Array([1, 2, 3])), /offset/);
  console.log('example level: ok');
}

function* files(dir) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) yield* files(p);
    else if (p.endsWith('.bytes')) yield p;
  }
}

/**
 * Integers in JSON text that a JavaScript number doesn't hold exactly. (A reviver passed to
 * `JSON.parse` would see them, but it recurses and runs out of stack on the deepest levels.)
 */
function unsafeIntegers(text) {
  let n = 0;
  for (const [token] of text.matchAll(/"(?:[^"\\]|\\.)*"|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/g)) {
    if (/^-?\d+$/.test(token) && !Number.isSafeInteger(Number(token))) n++;
  }
  return n;
}

const dirs = (process.env.DISTANCE_BYTES_CORPUS ?? '').split(';').filter(Boolean);
let count = 0, bytesIn = 0, failures = 0, notBinary = 0, largest = { size: 0 };
const start = performance.now();
corpus: for (const dir of dirs) {
  for (const path of files(dir)) {
    const data = readFileSync(path);
    const mark = data.length >= 4 ? data.readInt32LE(0) : 0;
    if (mark !== 99999999 && mark !== 66666666) {
      notBinary++;
      continue;
    }
    try {
      // Objects are freed when garbage collected, which doesn't happen soon enough here.
      using file = BytesFile.read(data);
      assert.ok(Buffer.from(file.write()).equals(data), 'binary round trip');
      const text = file.toJson();
      using back = BytesFile.fromJson(text);
      assert.ok(Buffer.from(back.write()).equals(data), 'JSON round trip');
      assert.equal(unsafeIntegers(text), 0, 'unsafe integers');
      if (data.length > largest.size) largest = { size: data.length, path };
    } catch (e) {
      failures++;
      console.log(`${path}: ${e.message ?? e}`);
      // After a panic, a trap or running out of stack, the module's state is lost.
      if (e instanceof WebAssembly.RuntimeError || e instanceof RangeError) break corpus;
    }
    count++;
    bytesIn += data.length;
  }
}
const seconds = (performance.now() - start) / 1000;
if (dirs.length) {
  console.log(
    `${count} files (${(bytesIn / 1e6).toFixed(0)} MB) in ${seconds.toFixed(0)} s, ${failures} failures, ${notBinary} not binary`,
  );
  console.log(`largest: ${largest.path} (${(largest.size / 1e6).toFixed(1)} MB)`);
} else {
  console.log('DISTANCE_BYTES_CORPUS not set: corpus skipped');
}
process.exitCode = failures ? 1 : 0;
