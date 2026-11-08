// Alter compiler-emitted descriptions for generator rejection fixtures only.
function leb(n) {
  const out = [];
  do {
    let b = n & 127;
    n >>>= 7;
    out.push(b | (n ? 128 : 0));
  } while (n);
  return Buffer.from(out);
}
export function withSchemaEntries(bytes, mutate) {
  let at = 8;
  const sections = [];
  const read = () => {
    let n = 0,
      shift = 0,
      b;
    do {
      b = bytes[at++];
      n |= (b & 127) << shift;
      shift += 7;
    } while (b & 128);
    return n;
  };
  while (at < bytes.length) {
    const start = at,
      id = bytes[at++],
      length = read(),
      payload = at;
    let name = "";
    if (id === 0) {
      const n = read();
      name = bytes.subarray(at, at + n).toString();
    }
    at = payload + length;
    if (name !== "ocamljs.schema") sections.push(bytes.subarray(start, at));
  }
  const module = new WebAssembly.Module(bytes);
  const entries = WebAssembly.Module.customSections(
    module,
    "ocamljs.schema",
  ).flatMap((b) =>
    Buffer.from(b)
      .toString()
      .trim()
      .split("\n")
      .filter(Boolean)
      .map(JSON.parse),
  );
  mutate(entries);
  const payload = Buffer.concat([
    leb(14),
    Buffer.from("ocamljs.schema"),
    Buffer.from(entries.map(JSON.stringify).join("\n") + "\n"),
  ]);
  return Buffer.concat([
    bytes.subarray(0, 8),
    ...sections,
    Buffer.from([0]),
    leb(payload.length),
    payload,
  ]);
}
