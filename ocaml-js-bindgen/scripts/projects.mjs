import { spawnSync } from "node:child_process";
import {
  closeSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  openSync,
  readdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { relative, resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const projects = readdirSync(resolve(root, "demos"), { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();
const sites = projects.filter((name) => name !== "reference-conformance");
const esbuild = resolve(root, "node_modules/.bin/esbuild");

function run(command, args, cwd = root, stdout = "inherit") {
  console.log(`[${relative(root, cwd) || "."}] ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, {
    cwd,
    stdio: ["inherit", stdout, "inherit"],
  });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(
      `${command} failed: ${result.signal ?? `exit ${result.status}`}`,
    );
}

function copyFiles(from, to, files) {
  for (const file of files)
    copyFileSync(resolve(from, file), resolve(to, file));
}

function matrix() {
  for (const name of projects)
    run(
      "ocamljs_pack",
      [
        "build",
        "--backend",
        "all",
        "--out-dir",
        "../generated",
        ...([
          "calculator",
          "todo",
          "image-editor",
          "cat-pics",
          "document-explorer",
        ].includes(name)
          ? ["--features", "conformance,ocamljs_bindgen/resource-diagnostics"]
          : ["--features", "ocamljs_bindgen/resource-diagnostics"]),
      ],
      resolve(root, "demos", name, "rust"),
    );

  for (const name of [
    "async-search",
    "reference-conformance",
    "document-explorer",
    "cat-pics",
  ])
    run(
      "ocamljs_pack",
      [
        "generate",
        "--wasm",
        "generated/js_of_ocaml/backend.wasm",
        "--schema",
        "generated/js_of_ocaml/schema.json",
        "--backend",
        "js_of_ocaml",
        "--async-runtime",
        "lwt",
        "--name",
        readFileSync(
          resolve(
            root,
            "demos",
            name,
            "generated/js_of_ocaml/bindings.runtime.mjs",
          ),
          "utf8",
        )
          .match(/globalThis\.(Ocamljs_\w+) = api/)[1]
          .slice("Ocamljs_".length),
        "--out-dir",
        "generated/lwt",
      ],
      resolve(root, "demos", name),
    );

  const conformance = resolve(root, "demos/reference-conformance");
  for (const backend of ["melange", "js_of_ocaml", "rescript"])
    copyFiles(
      resolve(conformance, "tests"),
      resolve(conformance, "generated", backend),
      ["third-party.mjs"],
    );
  copyFiles(
    resolve(conformance, "tests"),
    resolve(conformance, "generated/lwt"),
    ["third-party.mjs"],
  );

  for (const name of projects)
    run(
      "dune",
      ["build", "@validation", ...(sites.includes(name) ? ["@frontend"] : [])],
      resolve(root, "demos", name),
    );
  for (const name of projects)
    run("node_modules/.bin/rescript", ["build"], resolve(root, "demos", name));

  for (const name of sites) {
    const project = resolve(root, "demos", name);
    for (const backend of ["melange", "js_of_ocaml", "rescript"]) {
      const dist = resolve(project, "dist", backend);
      rmSync(dist, { recursive: true, force: true });
      mkdirSync(dist, { recursive: true });
      copyFiles(resolve(project, "web"), dist, ["index.html", "style.css"]);
      copyFiles(resolve(project, "generated", backend), dist, ["backend.wasm"]);
      const runtime = resolve(
        project,
        "generated",
        backend,
        "bindings.runtime.mjs",
      );
      const entry =
        backend === "js_of_ocaml"
          ? resolve(project, "_build/default/adapters/js_of_ocaml/Main.bc.js")
          : backend === "rescript"
            ? resolve(project, "lib/es6/src/rescript/Main.js")
            : resolve(project, "_build/default/output/src/ocaml/Main.js");
      const entries = [`frontend=${entry}`, `bindings.runtime=${runtime}`];
      if (existsSync(resolve(project, "src/ocaml/Worker.ml"))) {
        entries.push(
          `worker=${
            backend === "js_of_ocaml"
              ? resolve(
                  project,
                  "_build/default/adapters/js_of_ocaml/Worker.bc.js",
                )
              : backend === "rescript"
                ? resolve(project, "lib/es6/src/rescript/Worker.js")
                : resolve(project, "_build/default/output/src/ocaml/Worker.js")
          }`,
        );
      }
      run(esbuild, [
        ...entries,
        "--bundle",
        "--splitting",
        "--format=esm",
        "--minify",
        "--external:node:*",
        `--inject:${runtime}`,
        "--out-extension:.js=.mjs",
        `--outdir=${dist}`,
      ]);
      if (existsSync(resolve(project, "web/corpus.ndjson")))
        copyFiles(resolve(project, "web"), dist, ["corpus.ndjson"]);
    }
    copyFileSync(
      resolve(project, "tests/backends.html"),
      resolve(project, "dist/index.html"),
    );
  }
}

try {
  switch (process.argv[2]) {
    case "demos":
      for (const name of sites)
        run("make", ["build"], resolve(root, "demos", name));
      break;
    case "matrix":
      matrix();
      break;
    case "reference-fixtures":
      for (const feature of [
        "primitive-only",
        "builtin-only",
        "error-only",
        "text-only",
        "pixels-only",
      ])
        run(
          "ocamljs_pack",
          [
            "build",
            "--features",
            feature,
            "--out-dir",
            `../generated/${feature}`,
            "--name",
            feature.replaceAll("-", "_"),
          ],
          resolve(root, "demos/reference-conformance/rust"),
        );
      break;
    case "deps":
      run("npm", ["ci"]);
      run("make", ["deps"], resolve(root, "examples/rust-to-ocaml"));
      for (const name of projects)
        run("make", ["deps"], resolve(root, "demos", name));
      break;
    case "native-tests":
      mkdirSync(resolve(root, "tmp"), { recursive: true });
      for (const name of projects) {
        const stdout = openSync(
          resolve(root, "tmp", `${name}-tests.jsonl`),
          "w",
        );
        try {
          run(
            "cargo",
            ["test", "--locked", "--no-run", "--message-format=json"],
            resolve(root, "demos", name, "rust"),
            stdout,
          );
        } finally {
          closeSync(stdout);
        }
      }
      break;
    case "clean":
      for (const name of projects)
        run("make", ["clean"], resolve(root, "demos", name));
      break;
    default:
      throw new Error(
        "Usage: node scripts/projects.mjs demos|matrix|reference-fixtures|deps|native-tests|clean",
      );
  }
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
