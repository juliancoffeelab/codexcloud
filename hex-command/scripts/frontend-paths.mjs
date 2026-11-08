import { readdirSync, existsSync } from "node:fs";
import { resolve, relative } from "node:path";
import { fileURLToPath } from "node:url";
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
      ["@wasm", new URL("../rust/pkg", import.meta.url).href],
      ["@presentation", new URL("../src/presentation", import.meta.url).href],
      ["@assets/pixel-art", new URL("../src/presentation/pixel-art.mjs", import.meta.url).href],
      [
        "@migrations",
        new URL("../_build/default/frontend/output/packages/save-migrations/ocaml", import.meta.url)
          .href,
      ],
    ].map(([name, path]) => [name, path.startsWith("file:") ? fileURLToPath(path) : path]),
  ),
};
export { frontendAliases };
