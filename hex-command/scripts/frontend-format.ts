/** Pinned formatters preserve documentation in each compiler syntax. */
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { devNull } from "node:os";
const write = process.argv.includes("--write");
// Resolve the pinned package tools once; each file does not need another Dune scan.
const tools = spawnSync("dune", ["exec", "--", "which", "refmt", "ocp-indent"], {
  encoding: "utf8",
});
if (tools.status !== 0 || tools.error) throw tools.error ?? new Error(tools.stderr);
const [refmt, indent] = tools.stdout.trim().split("\n");
let failed = false;
for (const file of readdirSync("frontend")
  .filter((file) => /\.(re|rei|ml|mli)$/.test(file) && !/^Domain\./.test(file))
  .sort()) {
  const path = `frontend/${file}`;
  const syntax = file.endsWith(".ml") || file.endsWith(".mli") ? "ml" : "re";
  const result = spawnSync(
    syntax === "ml" ? indent! : refmt!,
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
      // No formatter plugins are used; avoid ocamlfind's build-path config in
      // relocated Dune packages. The formatter itself is statically linked.
      env: syntax === "ml" ? { ...process.env, OCAMLFIND_CONF: devNull } : process.env,
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
