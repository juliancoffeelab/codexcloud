import { createHash, randomUUID } from "node:crypto";
import {
  readdirSync,
  readFileSync,
  writeFileSync,
  existsSync,
  mkdirSync,
  renameSync,
  rmSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
const write = process.argv.includes("--write");
const tools = spawnSync("dune", ["exec", "--", "which", "refmt", "ocamlformat"], {
  encoding: "utf8",
});
if (tools.status !== 0 || tools.error) throw tools.error ?? new Error(tools.stderr);
const [refmt, ocamlformat] = tools.stdout.trim().split("\n");
const sources = (directory) =>
  readdirSync(directory, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? sources(`${directory}/${entry.name}`) : [`${directory}/${entry.name}`],
  );
const files = [
  ...sources("frontend"),
  ...readdirSync("scripts").map((file) => `scripts/${file}`),
  ...readdirSync("packages/save-migrations/ocaml").map(
    (file) => `packages/save-migrations/ocaml/${file}`,
  ),
  ...readdirSync("tests/ocaml").map((file) => `tests/ocaml/${file}`),
]
  .filter(
    (file) =>
      /\.(re|rei|ml|mli)$/.test(file) &&
      !/(?:^|\/)(Domain|Fixture_json)\./.test(file) &&
      // Generated visitors compile as emitted; explicit formatting still includes them.
      (write || !/(?:^|\/)Domain_resource_walk\.ml$/.test(file)),
  )
  .sort();
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
// Include all potential ancestor configs, even absent ones: adding a config must invalidate hits.
const configPaths = new Set();
const projectBoundaries = new Set();
for (const file of files) {
  let directory = dirname(resolve(file));
  while (true) {
    for (const name of [
      ".ocamlformat",
      ".ocamlformat-ignore",
      ".ocamlformat-enable",
      ".ocp-indent",
    ])
      configPaths.add(join(directory, name));
    for (const name of [".git", ".hg", "dune-project"])
      projectBoundaries.add(join(directory, name));
    if (dirname(directory) === directory) break;
    directory = dirname(directory);
  }
}
const globalConfig = process.env.XDG_CONFIG_HOME || join(process.env.HOME ?? "", ".config");
for (const name of [".ocamlformat", "ocamlformat"]) configPaths.add(join(globalConfig, name));
function contextKey() {
  const hash = createHash("sha256").update(readFileSync(import.meta.filename));
  for (const path of [refmt, ocamlformat, ...[...configPaths].sort()]) {
    hash.update(path).update("\0");
    if (existsSync(path)) hash.update("present\0").update(readFileSync(path));
    else hash.update("absent\0");
    hash.update("\0");
  }
  for (const path of [...projectBoundaries].sort())
    hash.update(path).update(String(existsSync(path)));
  hash.update(
    JSON.stringify(
      Object.entries(process.env)
        .filter(([name]) => /^(OCAMLFORMAT|REFMT_)/.test(name))
        .sort(),
    ),
  );
  return hash.digest("hex");
}
const key = contextKey();
const cachePath = "tmp/frontend-format.json";
let previous;
try {
  previous = JSON.parse(readFileSync(cachePath, "utf8"));
} catch {
  /* Cache is optional. */
}
const cached = previous?.key === key ? (previous.files ?? {}) : {};
const next = {};
const pending = [];
for (const file of files) {
  const hash = digest(readFileSync(file));
  if (cached[file] === hash) next[file] = hash;
  else pending.push({ file, hash });
}
let failed = false;
function run(command, args) {
  const result = spawnSync(command, args, { encoding: "utf8" });
  if (result.status !== 0 || result.error) {
    failed = true;
    console.error(result.stderr || result.error?.message || result.stdout);
    return false;
  }
  return result;
}
const native = pending.filter(({ file }) => /\.mli?$/.test(file));
if (
  native.length &&
  run(ocamlformat, [write ? "--inplace" : "--check", ...native.map(({ file }) => file)]) &&
  !write
)
  for (const { file, hash } of native) if (digest(readFileSync(file)) === hash) next[file] = hash;
// refmt has no check-only batch mode; successful unchanged files are cached individually.
for (const { file, hash } of pending.filter(({ file }) => /\.rei?$/.test(file))) {
  const result = run(refmt, [
    "--parse",
    "re",
    "--print",
    "re",
    "--print-width",
    "100",
    "--interface",
    String(file.endsWith("i")),
    file,
  ]);
  if (!result) continue;
  const formatted = result.stdout.endsWith("\n") ? result.stdout : result.stdout + "\n";
  const current = readFileSync(file, "utf8");
  if (formatted !== current) {
    if (write) {
      writeFileSync(file, formatted);
      next[file] = digest(formatted);
    } else {
      failed = true;
      console.error(`Unformatted: ${file}`);
    }
  } else if (digest(current) === hash) next[file] = hash;
}
// Don't persist results if a config/tool changed while formatting was in flight.
if (contextKey() === key) {
  const temporary = `${cachePath}.${randomUUID()}.tmp`;
  try {
    mkdirSync(dirname(cachePath), { recursive: true });
    writeFileSync(temporary, JSON.stringify({ key, files: next }) + "\n", { flag: "wx" });
    renameSync(temporary, cachePath);
  } catch (error) {
    console.warn(`Formatting cache unavailable: ${error.message}`);
  } finally {
    try {
      rmSync(temporary, { force: true });
    } catch {
      /* Cache cleanup is best effort. */
    }
  }
}
process.exit(failed ? 1 : 0);
