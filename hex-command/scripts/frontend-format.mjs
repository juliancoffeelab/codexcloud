import { readdirSync, readFileSync, writeFileSync } from "node:fs";
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
let failed = false;
for (const path of [
  ...sources("frontend"),
  ...readdirSync("scripts").map((file) => `scripts/${file}`),
  ...readdirSync("packages/save-migrations/ocaml").map(
    (file) => `packages/save-migrations/ocaml/${file}`,
  ),
  ...readdirSync("tests/ocaml").map((file) => `tests/ocaml/${file}`),
]
  .filter(
    (file2) => /\.(re|rei|ml|mli)$/.test(file2) && !/(?:^|\/)(Domain|Fixture_json)\./.test(file2),
  )
  .sort()) {
  const file = path;
  const syntax = file.endsWith(".ml") || file.endsWith(".mli") ? "ml" : "re";
  const result = spawnSync(
    syntax === "ml" ? ocamlformat : refmt,
    syntax === "ml"
      ? [path]
      : [
          "--parse",
          syntax,
          "--print",
          syntax,
          "--print-width",
          "100",
          "--interface",
          String(file.endsWith("i")),
          path,
        ],
    {
      encoding: "utf8",
    },
  );
  if (result.status !== 0 || result.error) {
    failed = true;
    console.error(result.stderr || result.error?.message);
    continue;
  }
  const current = readFileSync(path, "utf8");
  const formatted = result.stdout.endsWith("\n") ? result.stdout : result.stdout + "\n";
  if (formatted !== current) {
    if (write) writeFileSync(path, formatted);
    else {
      failed = true;
      console.error(`Unformatted: ${path}`);
    }
  }
}
process.exit(failed ? 1 : 0);
