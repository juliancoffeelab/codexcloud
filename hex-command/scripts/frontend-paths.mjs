import { readdirSync, existsSync } from "node:fs";
import { resolve, relative } from "node:path";
import { fileURLToPath } from "node:url";
export const wasmMode = process.env.HEX_COMMAND_PROFILE ?? "dev";
if (!["dev", "release"].includes(wasmMode)) throw new Error("Choose dev or release WASM");
const projectRoot = fileURLToPath(new URL("../", import.meta.url));
function sourceModules(root) {
  const modules = new Map();
  const walk = (directory) => {
    if (!existsSync(directory)) return;
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = resolve(directory, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (/\.(ml|re)$/.test(entry.name))
        modules.set(
          entry.name.replace(/\.(ml|re)$/, ""),
          relative(resolve(root, "frontend"), path).replace(/\.(ml|re)$/, ".js"),
        );
    }
  };
  walk(resolve(root, "frontend"));
  modules.set("Domain", "engine/Domain.js");
  modules.set("Domain_resource_walk", "engine/Domain_resource_walk.js");
  modules.set("Fixture_json", "Fixture_json.js");
  return modules;
}
export function frontendModulePath(name, root = projectRoot) {
  const path = sourceModules(root).get(name);
  if (!path) throw new Error(`Unknown frontend module: ${name}`);
  return resolve(root, "_build/default/frontend/output/frontend", path);
}
export function frontendModuleAliases(root = projectRoot) {
  return Object.fromEntries(
    [...sourceModules(root)].flatMap(([name, path]) => {
      const target = resolve(root, "_build/default/frontend/output/frontend", path);
      return [
        [`@frontend/${name}.js`, target],
        [`@frontend/${name}`, target],
      ];
    }),
  );
}
const frontendAliases = {
  ...frontendModuleAliases(),
  ...Object.fromEntries(
    [
      // ReasonReact supplies typed JSX/events; Preact supplies the runtime contract.
      ["react/jsx-runtime", import.meta.resolve("preact/compat/jsx-runtime")],
      ["react/jsx-dev-runtime", import.meta.resolve("preact/compat/jsx-dev-runtime")],
      ["react", import.meta.resolve("preact/compat")],
      [
        "@frontend/Save_legacy",
        new URL(
          "../_build/default/frontend/output/packages/save-migrations/ocaml/Save_legacy.js",
          import.meta.url,
        ).href,
      ],
      ["@frontend", new URL("../_build/default/frontend/output/frontend", import.meta.url).href],
      [
        "melange",
        new URL("../_build/default/frontend/output/node_modules/melange", import.meta.url).href,
      ],
      ["@wasm", new URL(`../_build/default/wasm/${wasmMode}`, import.meta.url).href],
      ["@presentation", new URL("../src/presentation", import.meta.url).href],
      [
        "@assets/pixel-art",
        new URL("../_build/default/artwork/pixel-art.mjs", import.meta.url).href,
      ],
      [
        "@migrations",
        new URL("../_build/default/frontend/output/packages/save-migrations/ocaml", import.meta.url)
          .href,
      ],
    ].map(([name, path]) => [name, path.startsWith("file:") ? fileURLToPath(path) : path]),
  ),
};
export { frontendAliases };

export function wasmPath(name, root = projectRoot) {
  return resolve(root, "_build/default/wasm", wasmMode, name);
}
export function assertBuildProfile(expected = wasmMode) {
  for (const name of ["backend.wasm", "bindings.runtime.mjs", "schema.json"])
    if (!existsSync(resolve(projectRoot, "_build/default/wasm", expected, name)))
      throw new Error(
        `Missing ${expected} WASM package; run node scripts/toolchain/with-toolchain.mjs bash scripts/build-wasm.sh ${expected}, then dune build --root=. @melange @artwork`,
      );
}
