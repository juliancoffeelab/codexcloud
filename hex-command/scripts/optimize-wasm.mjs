import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
const path = process.argv[2];
if (!path || process.argv.length !== 3) throw new Error("Usage: optimize-wasm.mjs module.wasm");
let source = readFileSync(path);
if (!WebAssembly.validate(source)) throw new Error(`Invalid WASM module: ${path}`);
function unsigned(bytes, offset) {
  let value = 0;
  for (let shift = 0; shift < 35; shift += 7) {
    const byte = bytes[offset++];
    if (byte === void 0) throw new Error("Truncated WASM integer");
    value += (byte & 127) * 2 ** shift;
    if (byte < 128) return [value, offset];
  }
  throw new Error("Invalid WASM integer");
}
function encoded(value) {
  const bytes = [];
  do {
    const byte = value % 128;
    value = Math.floor(value / 128);
    bytes.push(byte | (value ? 128 : 0));
  } while (value);
  return Buffer.from(bytes);
}
function string(bytes) {
  return Buffer.concat([encoded(bytes.length), bytes]);
}
function section(kind, bytes) {
  return Buffer.concat([Buffer.from([kind]), encoded(bytes.length), bytes]);
}
const optimizer = createRequire(import.meta.url).resolve("binaryen/bin/wasm-opt");
function transform(bytes, passes) {
  const directory = mkdtempSync(join(tmpdir(), "hex-wasm-"));
  try {
    const input = join(directory, "input.wasm");
    const output = join(directory, "output.wasm");
    writeFileSync(input, bytes);
    execFileSync(
      process.execPath,
      [
        optimizer,
        input,
        "-o",
        output,
        "--enable-bulk-memory",
        "--enable-nontrapping-float-to-int",
        ...passes,
      ],
      { stdio: "inherit" },
    );
    return readFileSync(output);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
source = transform(source, ["--duplicate-function-elimination", "--optimize-instructions", "-g"]);
const sections = [source.subarray(0, 8)];
const names = [];
const imports = WebAssembly.Module.imports(new WebAssembly.Module(source)).filter(
  (entry) => entry.kind === "function",
).length;
for (let index = 0; index < imports; index++) {
  names.push(encoded(index), string(Buffer.from(` ${index}`)));
}
for (let offset = 8; offset < source.length;) {
  const start = offset;
  const kind = source[offset++];
  const [length, payload] = unsigned(source, offset);
  offset = payload + length;
  if (offset > source.length) throw new Error("Truncated WASM section");
  if (kind === 0) {
    const [length2, nameStart] = unsigned(source, payload);
    const name = source.toString("utf8", nameStart, nameStart + length2);
    if (name === "name" || name.startsWith(".debug_")) continue;
  }
  if (kind === 10) {
    const [count, first] = unsigned(source, payload);
    let cursor = first;
    for (let index = 0; index < count; index++) {
      const [length2, body] = unsigned(source, cursor);
      cursor = body + length2;
      if (cursor > offset) throw new Error("Truncated WASM function");
      const key = source.subarray(body, Math.min(cursor, body + 128)).toString("hex");
      names.push(encoded(imports + index), string(Buffer.from(`${key} ${imports + index}`)));
    }
    if (cursor !== offset) throw new Error("Invalid WASM code section");
  }
  sections.push(source.subarray(start, offset));
}
if (names.length === 0) throw new Error("WASM code section missing");
names.unshift(encoded(names.length / 2));
const functionNames = section(1, Buffer.concat(names));
sections.push(section(0, Buffer.concat([string(Buffer.from("name")), functionNames])));
const optimized = transform(Buffer.concat(sections), ["--reorder-functions-by-name"]);
if (!WebAssembly.validate(optimized)) throw new Error("Invalid optimized WASM module");
writeFileSync(path, optimized);
console.log(`WASM release layout: ${optimized.length} bytes`);
