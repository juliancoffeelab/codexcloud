// Optional benchmark build; no changes to the SDK runtime or the upstream checkout.
import assert from "node:assert/strict";
import {
  readFileSync,
  writeFileSync,
  readdirSync,
  mkdirSync,
  copyFileSync,
} from "node:fs";
import { resolve, join } from "node:path";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { build } from "esbuild";
const root = resolve(import.meta.dirname, "../.."),
  args = process.argv.slice(2),
  get = (k, f) => {
    let i = args.indexOf(k);
    return i < 0 ? f : args[i + 1];
  };
const paranoid =
  args.includes("--paranoid") || process.env.OCAMLJS_PARANOID === "1";
const upstream = resolve(get("--upstream", "")),
  out = resolve(get("--out", join(root, "tmp/glue-benchmark"))),
  sdk = resolve(get("--sdk", root)),
  legacy = args.includes("--legacy"),
  crate = join(sdk, "benchmarks/glue/rust");
const git = (...a) =>
  execFileSync("git", ["-C", upstream, ...a], { encoding: "utf8" }).trim();
assert.equal(
  git("rev-parse", "HEAD"),
  "78289b2745c4dfbff5555c14f539ed6284a47802",
);
assert.equal(git("status", "--porcelain"), "");
const wire = join(upstream, "rust/src/wasm/wire.rs");
const run = (cmd, a, cwd = root, extra = {}) =>
  execFileSync(cmd, a, {
    cwd,
    stdio: "inherit",
    env: { ...process.env, ...extra },
  });
mkdirSync(out, { recursive: true });
const target = resolve(get("--target-dir", join(out, "target"))),
  env = {
    CARGO_TARGET_DIR: target,
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? "4",
    GLUE_BENCH_UPSTREAM_WIRE: wire,
    OCAMLJS_PARANOID: "0",
  };
const pkg = join(root, "demos/calculator/_build/_private/default/.pkg"),
  melange = get(
    "--melange",
    (() => {
      let p = readdirSync(pkg).find((p) => p.startsWith("melange.7.0.1-55-"));
      assert(p, "Prepare calculator dependencies first");
      return join(pkg, p, "target");
    })(),
  );
const wasmBindgen = get(
    "--wasm-bindgen",
    process.env.WASM_BINDGEN ?? "wasm-bindgen",
  ),
  pack = get("--pack", join(sdk, "target/debug/ocamljs_pack"));
const runtime = join(
  root,
  "demos/calculator/_build/default/output/node_modules",
);
const metadata = {
  paranoid,
  debugWasm: args.includes("--debug-wasm"),
  directNumeric: !legacy,
  stringControls: true,
  recordHostProtocol: "convert-native-carrier-only",
  stringLifecycleControls:
    args.includes("--lifecycle-probes") &&
    readFileSync(join(crate, "src/lib.rs"), "utf8").includes(
      "pub fn text_result_notify",
    ),
  pooledLabelControls: readFileSync(join(crate, "src/lib.rs"), "utf8").includes(
    "pub fn string_envelope_store_new",
  ),
  projectStringControls: readFileSync(
    join(crate, "src/lib.rs"),
    "utf8",
  ).includes("pub fn text_result_out"),
  throwsStringControls: readFileSync(
    join(crate, "src/lib.rs"),
    "utf8",
  ).includes("pub fn text_throws_out"),
  nativeThrowsStringControls: readFileSync(
    join(crate, "src/lib.rs"),
    "utf8",
  ).includes("pub fn text_throws_native_out"),
  sdkCommit: execFileSync("git", ["-C", sdk, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim(),
  node: process.version,
  v8: process.versions.v8,
  rust: execFileSync("rustc", ["--version"], { encoding: "utf8" }).trim(),
  wasmBindgen: execFileSync(wasmBindgen, ["--version"], {
    encoding: "utf8",
  }).trim(),
  upstream: git("rev-parse", "HEAD"),
  wireSha256: createHash("sha256").update(readFileSync(wire)).digest("hex"),
  variants: {},
};
for (const variant of [
  "native",
  "js",
  ...(readFileSync(join(crate, "Cargo.toml"), "utf8").includes("js-record =")
    ? ["js-record"]
    : []),
  "baseline",
]) {
  const folder = join(out, variant);
  mkdirSync(folder, { recursive: true });
  run(
    "cargo",
    [
      "build",
      "--locked",
      "--release",
      "--target",
      "wasm32-unknown-unknown",
      "--manifest-path",
      join(
        variant === "baseline" ? root : sdk,
        "benchmarks/glue/rust/Cargo.toml",
      ),
      "--features",
      (variant === "native"
        ? "ours"
        : variant === "js"
          ? "js"
          : variant === "js-record"
            ? "js-record"
            : "baseline") +
        (variant !== "baseline" && !legacy ? ",direct-numeric" : "") +
        (variant !== "baseline" && args.includes("--lifecycle-probes")
          ? ",lifecycle-probes"
          : "") +
        (paranoid && variant !== "baseline" ? ",ocamljs_bindgen/paranoid" : ""),
    ],
    root,
    env,
  );
  const wasm = join(
    target,
    "wasm32-unknown-unknown/release/glue_benchmark.wasm",
  );
  let entry, wasmFile;
  if (variant === "baseline") {
    run(wasmBindgen, [
      wasm,
      "--target",
      "web",
      "--out-dir",
      folder,
      "--out-name",
      "reference",
    ]);
    entry = `import initialize,* as Raw from ${JSON.stringify(join(folder, "reference.js"))};export const B={...Raw,text_result_unwrap:store=>Raw.text_result_out(store,7,0),text_result_error_code:store=>{try{Raw.text_result_out(store,0,0);throw Error("expected result error");}catch(error){if(typeof error!=="number")throw error;return error;}},text_throws_error_code:store=>{try{Raw.text_throws_out(store,0,0);throw Error("expected throws error");}catch(error){if(typeof error!=="number")throw error;return error;}}};export async function load(bytes){await initialize({module_or_path:bytes});}export const host=x=>x;export const nativeString=x=>x;`;
    wasmFile = join(folder, "reference_bg.wasm");
  } else {
    run(pack, [
      "generate",
      ...(paranoid ? ["--paranoid"] : []),
      ...(args.includes("--debug-wasm") ? ["--dev"] : []),
      "--wasm",
      wasm,
      "--backend",
      "melange",
      "--name",
      "glue_benchmark",
      "--out-dir",
      folder,
    ]);
    writeFileSync(
      join(folder, "Consumer.ml"),
      metadata.projectStringControls
        ? `include Bindings
let text_result_unwrap store = match Store.text_result_out store 7. 0. with
  | Ok text -> text | Error _ -> failwith "unexpected result error"
let text_result_error_code store = match Store.text_result_out store 0. 0. with
  | Ok _ -> failwith "expected result error" | Error code -> int_of_float code
${metadata.throwsStringControls ? 'let text_throws_error_code store = try ignore (Store.text_throws_out store 0. 0.); failwith "expected throws error" with Store.Text_throws_out_error code -> int_of_float code' : ""}
${metadata.stringLifecycleControls ? 'let text_encoded_unwrap store = match Store.text_encoded_result_out store with Ok text -> text | Error _ -> failwith "unexpected encoded error"' : ""}
`
        : "include Bindings\n",
    );
    for (const file of ["Bindings.mli", "Bindings.ml", "Consumer.ml"])
      run(
        join(melange, "bin/melc"),
        [
          "-ppx",
          join(melange, "bin/melppx"),
          "-I",
          ".",
          "-I",
          join(melange, "lib/melange/melange"),
          "-I",
          join(melange, "lib/melange/js/melange"),
          "--mel-package-output",
          "esm:.",
          "-o",
          file.replace(/\.mli?$/, file.endsWith(".mli") ? ".cmi" : ".cmj"),
          file,
        ],
        folder,
      );
    entry = `import * as Generated from ${JSON.stringify(join(folder, "Consumer.js"))};import {initialize,api} from ${JSON.stringify(join(folder, "bindings.runtime.mjs"))};export const B={...Generated,...Generated.Store,...Generated.Counter};export const disposeStore=Generated.Store.dispose;export const disposeCounter=Generated.Counter.dispose;export async function load(bytes,options){return await initialize(bytes,options);}export const host=${variant === "native" ? "api.host_string" : "x=>x"};export const nativeHost=api.host_string;export const nativeString=api.native_string;`;
    wasmFile = join(folder, "backend.wasm");
  }
  const result = await build({
    stdin: { contents: entry, resolveDir: root },
    outfile: join(folder, "adapter.mjs"),
    bundle: true,
    format: "esm",
    platform: "node",
    target: "es2022",
    metafile: true,
    nodePaths: [runtime],
  });
  const hash = (p) =>
    createHash("sha256").update(readFileSync(p)).digest("hex");
  metadata.variants[variant] = {
    wasm: wasmFile,
    wasmBytes: readFileSync(wasmFile).length,
    wasmSha256: hash(wasmFile),
    adapter: join(folder, "adapter.mjs"),
    adapterSha256: hash(join(folder, "adapter.mjs")),
  };
  writeFileSync(
    join(folder, "metafile.json"),
    JSON.stringify(result.metafile, null, 2),
  );
}
metadata.fixtureSha256 = createHash("sha256")
  .update(readFileSync(join(crate, "src/lib.rs")))
  .update(readFileSync(join(crate, "Cargo.lock")))
  .digest("hex");
writeFileSync(
  join(out, "build.json"),
  JSON.stringify(metadata, null, 2) + "\n",
);
console.log("Built benchmark artifacts:", out);
