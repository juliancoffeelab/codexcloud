/** Remove diagnostic names from production WASM without changing executable sections. */
import { readFileSync, writeFileSync } from "node:fs";

const path = process.argv[2];
if (!path || process.argv.length !== 3) throw new Error("Usage: strip-wasm.ts module.wasm");
const source = readFileSync(path);
if (!WebAssembly.validate(source)) throw new Error(`Invalid WASM module: ${path}`);

function unsigned(offset: number): [value: number, next: number] {
  let value = 0;
  for (let index = 0; index < 5; index++) {
    const byte = source[offset++];
    if (byte === undefined) throw new Error("Truncated WASM section length");
    value += (byte & 127) * 2 ** (index * 7);
    if (byte < 128) return [value, offset];
  }
  throw new Error("Invalid WASM section length");
}

const sections = [source.subarray(0, 8)];
for (let offset = 8; offset < source.length;) {
  const start = offset;
  const kind = source[offset++]!;
  const [length, payload] = unsigned(offset);
  offset = payload + length;
  if (offset > source.length) throw new Error("Truncated WASM section");
  if (kind === 0) {
    const [nameLength, nameStart] = unsigned(payload);
    const nameEnd = nameStart + nameLength;
    if (nameEnd > offset) throw new Error("Truncated WASM custom-section name");
    const name = source.toString("utf8", nameStart, nameEnd);
    if (name === "name" || name.startsWith(".debug_")) continue;
  }
  sections.push(source.subarray(start, offset));
}
const stripped = Buffer.concat(sections);
if (!WebAssembly.validate(stripped)) throw new Error("Invalid stripped WASM module");
if (stripped.length !== source.length) writeFileSync(path, stripped);
console.log(`WASM diagnostic metadata: removed ${source.length - stripped.length} bytes`);
