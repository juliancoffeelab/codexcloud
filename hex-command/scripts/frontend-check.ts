/** One compiler program checks the selected frontend contracts and declarations. */
import ts from "typescript";
import { readFileSync } from "node:fs";
import { generateBindings } from "./melange-bindings";

const groups = ["wasm", "ui", "types"] as const;
type Group = (typeof groups)[number];
const requested = process.argv.slice(2);
const selected = new Set<Group>((requested.length ? requested : groups) as Group[]);
const diagnosticHost: ts.FormatDiagnosticsHost = {
  getCanonicalFileName: (path) => path,
  getCurrentDirectory: () => process.cwd(),
  getNewLine: () => ts.sys.newLine,
};
function report(diagnostics: readonly ts.Diagnostic[]): void {
  if (diagnostics.length)
    throw new Error(ts.formatDiagnosticsWithColorAndContext(diagnostics, diagnosticHost));
}

try {
  if (requested.some((name) => !groups.includes(name as Group)))
    throw new Error(`Choose frontend checks from: ${groups.join(", ")}`);
  const configPath = selected.has("types") ? "tsconfig.tests.json" : "tsconfig.json";
  const config = ts.readConfigFile(configPath, ts.sys.readFile);
  if (config.error) report([config.error]);
  const parsed = ts.parseJsonConfigFileContent(config.config, ts.sys, process.cwd());
  report(parsed.errors);
  const files = new Set(selected.has("types") ? parsed.fileNames : []);
  const boundary = "scripts/frontend-boundary.ts";
  if (selected.has("ui")) files.add(boundary);
  if (selected.has("wasm")) files.add("rust/pkg/hex_command.d.ts");
  const options: ts.CompilerOptions = { ...parsed.options, noEmit: true };
  if (selected.has("wasm")) {
    // Preserve strict checking of generated and authored declarations without
    // repeating checks of TypeScript's own standard library declarations.
    options.skipLibCheck = false;
    options.skipDefaultLibCheck = true;
  }
  const program = ts.createProgram([...files], options);
  report(ts.getPreEmitDiagnostics(program));
  if (selected.has("ui")) {
    const { implementation, intf } = generateBindings(program, boundary);
    for (const [ext, expected] of [
      ["ml", implementation],
      ["mli", intf],
    ]) {
      let actual: string | undefined;
      try {
        actual = readFileSync(`frontend/Domain.${ext}`, "utf8");
      } catch {
        // A missing generated file is stale too; report the regeneration command.
      }
      if (actual !== expected)
        throw new Error("Melange bindings are stale. Run npm run bindings:frontend.");
    }
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exitCode = 1;
}
