// Make a source distribution so consumers remain independent monorepo projects.
import {
  mkdirSync,
  readFileSync,
  writeFileSync,
  rmSync,
  readdirSync,
  existsSync,
} from "node:fs";
import { resolve } from "node:path";
const root = resolve(import.meta.dirname, ".."),
  dest = resolve(process.argv[2] ?? "tmp/vendor");
function sync(source, destination) {
  mkdirSync(destination, { recursive: true });
  const entries = readdirSync(source, { withFileTypes: true });
  const names = new Set(entries.map((entry) => entry.name));
  for (const entry of readdirSync(destination))
    if (!names.has(entry))
      rmSync(resolve(destination, entry), { recursive: true, force: true });
  for (const entry of entries) {
    const from = resolve(source, entry.name),
      to = resolve(destination, entry.name);
    if (entry.isDirectory()) sync(from, to);
    else {
      const content = readFileSync(from);
      if (!existsSync(to) || !readFileSync(to).equals(content))
        writeFileSync(to, content);
    }
  }
}
mkdirSync(dest, { recursive: true });
sync(resolve(root, "crates"), resolve(dest, "crates"));
const manifest = readFileSync(resolve(root, "Cargo.toml"), "utf8");
const output = resolve(dest, "Cargo.toml");
if (!existsSync(output) || readFileSync(output, "utf8") !== manifest)
  writeFileSync(output, manifest);
