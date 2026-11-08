export const cases = [];
for (const kind of [
  "noop",
  "add",
  "eight",
  "counter-get",
  "counter-add",
  "counter-lifecycle",
])
  cases.push({ name: kind, kind });
for (const kind of [
  "numeric-slice",
  "numeric-pair",
  "numeric-mixed",
  "numeric-eight",
  "numeric-heterogeneous",
])
  for (const n of [0, 1, 16, 1024, 65536])
    cases.push({ name: `${kind}/${n}`, kind, n });
for (const type of ["numbers", "bytes", "flats"])
  for (const n of type === "flats" ? [1, 64, 4096] : [16, 1024, 65536])
    for (const direction of type === "bytes"
      ? ["out", "in"]
      : ["out", "in", "echo"])
      cases.push({
        name: `${type}/${n}/${direction}`,
        kind: `${type}-${direction}`,
        n,
      });
for (const [label, depth, width] of [
  ["small", 1, 2],
  ["wide", 2, 32],
  ["deep", 32, 1],
])
  for (const direction of ["out", "in", "echo"])
    cases.push({
      name: `tree/${label}/${direction}`,
      kind: `tree-${direction}`,
      depth,
      width,
    });
for (const unicode of [false, true])
  for (const units of [16, 16384, 262144])
    for (const direction of ["out", "host", "in", "echo"])
      cases.push({
        name: `text/${unicode ? "unicode" : "ascii"}/${units}/${direction}`,
        kind: `text-${direction}`,
        unicode,
        units,
        utf8Bytes: units * (unicode ? 6 : 1),
        utf16Units: units * (unicode ? 3 : 1),
      });
for (const direction of ["out", "in", "echo"])
  cases.push({
    name: "flat-single/" + direction,
    kind: "flat-" + direction,
    n: 1,
  });
