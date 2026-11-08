// Input strings prepared before Benchmark.js invokes the measured functions.
const savePayload = (count, unicode = false) =>
  JSON.stringify({
    seed: 1234,
    state: {
      round: 12,
      units: Array.from({ length: count }, (_, id) => ({
        id: `unit_${id}`,
        tile: `tile_${id % 1024}`,
        faction: "enemy",
        hp: 100,
        notes: unicode ? "弓兵 café 🦀" : "unit_archer",
      })),
    },
  });
export const stringPayloads = {
  empty: "",
  ascii4k: "The quick brown fox jumps over the lazy dog. ".repeat(96),
  id16: "unit_archer_0123",
  ascii64: "a".repeat(64),
  latin8: "café",
  lateLatin1k: "a".repeat(1024) + "é",
  latinThenWide: "café déjà vu. ".repeat(64) + "λ🦀",
  latin1k: "café déjà vu. ".repeat(64),
  cjk1k: "攻撃力と防御力。".repeat(48),
  emoji1k: "🦀🚀🎮🙂".repeat(64),
  early1k: "λ🦀" + "a".repeat(1024),
  late1k: "a".repeat(1024) + "λ🦀",
  json16k: JSON.stringify({
    version: 1,
    units: Array.from({ length: 128 }, (_, id) => ({
      id,
      name: "弓兵",
      hp: 100,
      x: id % 32,
      y: 3,
      notes: "café 🦀",
    })),
  }),
  bomNul: "\ufeffkey\0value\n",
  saveAscii: savePayload(512),
  saveUnicode: savePayload(512, true),
  saveLarge: savePayload(4096),
};
export const stringCases = [];
const add = (payload, operation) =>
  stringCases.push({
    name: `strings/${payload}/${operation}`,
    kind: "string-control",
    payload,
    operation,
  });
for (const payload of ["id16", "latin1k", "json16k"])
  for (const operation of [
    "alias-in",
    "native-literal-in",
    "native-borrowed-in",
  ])
    add(payload, operation);
for (const payload of [
  "empty",
  "cjk1k",
  "emoji1k",
  "early1k",
  "late1k",
  "bomNul",
])
  add(payload, "alias-in");
for (const payload of ["latin1k", "json16k"])
  for (const operation of ["out", "host", "echo"]) add(payload, operation);
for (const payload of ["id16", "latin1k"]) add(payload, "mixed-in");

for (const operation of ["alias-in", "out", "host", "echo"])
  add("ascii4k", operation);

for (const operation of ["out", "host", "echo"]) add("id16", operation);

for (const payload of ["ascii64", "latin8", "lateLatin1k", "latinThenWide"])
  for (const operation of ["alias-in", "out", "host", "echo"])
    add(payload, operation);
for (const payload of ["early1k", "late1k", "bomNul"])
  for (const operation of ["out", "host", "echo"]) add(payload, operation);

// These paths deliberately retain Result/record framing. Hex's save boundary
// uses Result<String, EngineError>, unlike the root String fast-path controls.
for (const payload of [
  "id16",
  "latin1k",
  "json16k",
  "saveAscii",
  "saveUnicode",
  "saveLarge",
])
  for (const operation of ["result-out", "result-host"])
    add(payload, operation);
add("id16", "result-error");
for (const payload of [
  "id16",
  "latin1k",
  "json16k",
  "saveAscii",
  "saveUnicode",
  "saveLarge",
]) {
  add(payload, "throws-out");
  add(payload, "throws-native-out");
}
add("id16", "throws-error");
for (const payload of ["id16", "saveAscii", "saveUnicode"])
  for (const operation of [
    "envelope-in",
    "envelope-out",
    "envelope-host",
    "envelope-echo",
  ])
    add(payload, operation);
for (const payload of ["saveAscii", "saveUnicode", "saveLarge"])
  for (const operation of ["alias-in", "out", "host", "echo"])
    add(payload, operation);
for (const labels of [128, 1024])
  for (const labelUnicode of [false, true])
    stringCases.push({
      name: `strings/labels${labels}-${labelUnicode ? "unicode" : "ascii"}/envelope-out`,
      kind: "string-control",
      payload: "id16",
      operation: "envelope-out",
      labels,
      labelUnicode,
    });
