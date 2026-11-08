import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdirSync, copyFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { createHash } from "node:crypto";
const root = resolve(import.meta.dirname, "../..");
const fixture = resolve(
  process.env.ASYNC_FIXTURE ?? join(import.meta.dirname, "rust"),
);
const out = resolve(process.argv[2]);
const pack = resolve(
  process.argv[3] ?? join(root, "target/debug/ocamljs_pack"),
);
const target = join(root, "tmp/glue-benchmark/target");
mkdirSync(out, { recursive: true });
const generatorSha256 = createHash("sha256")
  .update(readFileSync(pack))
  .digest("hex");
const invocationSource = execFileSync("git", ["rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();
let metadata;
try {
  metadata = JSON.parse(readFileSync(join(out, "build.json"), "utf8"));
} catch {
  metadata = {
    source: execFileSync("git", ["rev-parse", "HEAD"], {
      encoding: "utf8",
    }).trim(),
    variants: {},
  };
}
for (const [kind, features] of [
  ["sync", []],
  ["host", ["host"]],
  ["plain", ["async"]],
  ["borrowed", ["async", "opaque"]],
  ["owned", ["async", "owned-result"]],
  ["completion", ["completion"]],
  ["callbacks", ["callbacks"]],
]) {
  if (process.argv[4] && !process.argv[4].split(",").includes(kind)) continue;
  const input = join(out, kind + ".input.wasm");
  if (!process.argv.includes("--generate-only")) {
    execFileSync(
      "cargo",
      [
        "build",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "--manifest-path",
        join(fixture, "Cargo.toml"),
        ...(features.length ? ["--features", features.join(",")] : []),
      ],
      {
        stdio: "inherit",
        env: {
          ...process.env,
          CARGO_TARGET_DIR: target,
          CARGO_BUILD_JOBS: "2",
        },
      },
    );
    copyFileSync(
      join(target, "wasm32-unknown-unknown/release/async_mode_benchmark.wasm"),
      input,
    );
  }
  for (const [backend, runtime] of [
    ["melange", "promise"],
    ["rescript", "promise"],
    ["js_of_ocaml", "promise"],
    ["js_of_ocaml", "lwt"],
  ]) {
    const name = kind + "-" + backend + "-" + runtime,
      folder = join(out, name);
    execFileSync(
      pack,
      [
        "generate",
        "--wasm",
        input,
        "--backend",
        backend,
        "--async-runtime",
        runtime,
        "--name",
        "async_modes",
        "--out-dir",
        folder,
      ],
      { stdio: "inherit" },
    );
    metadata.variants[name] = {
      kind,
      backend,
      runtime,
      invocationSource,
      generatorSha256,
      wasmSha256: createHash("sha256")
        .update(readFileSync(join(folder, "backend.wasm")))
        .digest("hex"),
    };
  }
}
writeFileSync(
  join(out, "build.json"),
  JSON.stringify(metadata, null, 2) + "\n",
);
