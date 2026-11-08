// Compile native collection clients against the actual prepared interfaces.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { toolchainEnvironment } from "./toolchain/toolchain-env.mjs";

const root = resolve(import.meta.dirname, "..");
const env = toolchainEnvironment();
const located = spawnSync("dune", ["exec", "--", "which", "melc"], {
  cwd: root,
  env,
  encoding: "utf8",
});
if (located.error || located.status !== 0)
  throw located.error ?? new Error(located.stderr || "Cannot locate the prepared Melange compiler");
const compiler = located.stdout.trim();
const compilerRoot = resolve(dirname(compiler), "../lib/melange");
const includes = [
  join(compilerRoot, "melange"),
  join(compilerRoot, "js/melange"),
  join(compilerRoot, "dom/melange"),
  join(compilerRoot, "__private__/melange_mini_stdlib/melange/.public_cmi_melange"),
  ...Object.entries({
    platform: "hex_platform",
    engine: "hex_engine",
    game: "hex_game",
    audio: "hex_audio",
    persistence: "hex_persistence",
    phaser: "phaser_bindings",
  }).map(([folder, library]) => {
    const objects = join(root, "_build/default/frontend", folder, `.${library}.objs`);
    const publicInterface = join(objects, "public_cmi_melange");
    return existsSync(publicInterface) ? publicInterface : join(objects, "melange");
  }),
];
const out = mkdtempSync(join(root, "tmp/native-clients-"));
try {
  const compile = (name, source) => {
    const file = join(out, `${name}.ml`);
    writeFileSync(file, `open Hex_engine\nopen Hex_game\n${source}\n`);
    return spawnSync(
      compiler,
      [
        "--mel-stop-after-cmj",
        "-c",
        ...includes.flatMap((path) => ["-I", path]),
        "-o",
        join(out, `${name}.cmj`),
        file,
      ],
      { cwd: root, env, encoding: "utf8" },
    );
  };
  const valid = compile(
    "Valid",
    `
let count view = Array.length (Native.state view).units
let update_array view unit = Array.set (Native.state view).units 0 unit
let neighbors view = Array.map (fun (tile : Domain.tile) -> tile.id) (Native.neighbors view "0,0")
let extended view = Domain.String_map.add "new" 2. (Native.reachable view "u1")
let catalog_ids catalog = List.map (fun (unit : Domain.unit_rule) -> unit.id) (Catalog.units catalog)
let tile_ids (scene : Terrain_plan.t) = Array.map (fun (tile : Terrain_plan.tile) -> tile.id) scene.tiles
let update_scene (scene : Terrain_plan.t) tile = Array.set scene.tiles 0 tile
let objective_count view = List.length (Story.goal_statuses view)
`,
  );
  assert.equal(valid.status, 0, valid.stderr || valid.error?.message);
  for (const [name, source] of [
    ["Write_catalog", "let mutate catalog unit = Array.set (Catalog.units catalog) 0 unit"],
    ["Write_objectives", "let mutate view value = Array.set (Story.goal_statuses view) 0 value"],
    ["Write_dictionary", 'let mutate view = Js.Dict.set (Native.reachable view "u1") "0,0" 99.'],
    ["Write_hashtable", 'let mutate view = Hashtbl.replace (Native.reachable view "u1") "0,0" 99.'],
  ]) {
    const result = compile(name, source);
    assert(!result.error, `${name}: compiler must execute`);
    assert.notEqual(
      result.status,
      0,
      `${name}: published collection write unexpectedly type-checks`,
    );
    assert.match(
      result.stderr,
      /type|expected|not compatible/i,
      `${name}: must fail on its client type contract`,
    );
  }
  console.log(
    "PASS: native arrays accept reads/writes; list and map interfaces retain their OCaml types",
  );
} finally {
  rmSync(out, { recursive: true, force: true });
}
