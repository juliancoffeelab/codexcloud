import { withSchemaEntries } from "./wasm-schema-fixture.mjs";
import assert from "node:assert/strict";
import {
  readFileSync,
  writeFileSync,
  mkdtempSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { resolve, join } from "node:path";
import { spawnSync } from "node:child_process";
const root = resolve(import.meta.dirname, ".."),
  pkg = resolve(root, "demos/calculator/_build/_private/default/.pkg");
const target = (name) => {
  const locked = readdirSync(resolve(root, "demos/calculator/dune.lock")).find(
    (file) => file.startsWith(`${name}.`) && file.endsWith(".pkg"),
  );
  assert(locked, `missing locked package ${name}`);
  const prefix = locked.slice(0, -4) + "-";
  const directory = readdirSync(pkg).find((file) => file.startsWith(prefix));
  assert(directory, `missing prepared package ${locked}`);
  return resolve(pkg, directory, "target");
};
function run(cmd, args, cwd, env = {}) {
  const p = spawnSync(cmd, args, {
    cwd,
    env: { ...process.env, ...env },
    encoding: "utf8",
    timeout: 60000,
  });
  assert.equal(p.status, 0, `${cmd}: ${p.error ?? p.stderr}`);
  return p.stdout;
}
const base = readFileSync(
  resolve(
    root,
    "demos/calculator/rust/target/wasm32-unknown-unknown/release/calculator.wasm",
  ),
);
const melange = target("melange"),
  runtime = resolve(
    root,
    "demos/calculator/_build/default/output/node_modules",
  );
const compiler = process.env.BSC ?? resolve(root, "node_modules/.bin/bsc"),
  reslib = resolve(root, "node_modules/rescript/lib/ocaml");
const project = resolve(root, "demos/calculator");
const ocamlCompiler = run(
  "dune",
  ["exec", "--", "which", "ocamlc.opt"],
  project,
).trim();
const ocamlLibrary = run(
  "dune",
  ["exec", "--", "ocamlc.opt", "-where"],
  project,
).trim();
const scratch = mkdtempSync(resolve(root, "tmp/collection-tests-"));
try {
  for (const policy of ["array", "dict"])
    for (const backend of ["melange", "rescript", "js_of_ocaml"]) {
      const folder = resolve(scratch, policy + "-" + backend);
      run("mkdir", ["-p", folder], root);
      const input = resolve(folder, "input.wasm");
      writeFileSync(
        input,
        withSchemaEntries(base, (entries) => {
          if (policy.startsWith("dict")) {
            for (const entry of entries)
              if (entry.name === "Collections")
                for (const field of entry.body.fields)
                  if (field.ty.args?.[0]?.name === "String")
                    field.ty.map = "dict";
          }
          if (policy === "array") {
            // Compile-only types force a recursive reader/list-reader SCC,
            // nested direct-return/fixed-array routing and a generic fallback.
            // Reuse an unused real buffer-returning export; its fixture
            // result is compile-only and never called against the real Wasm.
            const named = (name, args = []) => ({ kind: "named", name, args });
            // Adjacent fixed-width fields take span_read, including the
            // transparent BatchId boundary inside an alias instantiation.
            const batch = entries.find((entry) => entry.name === "BatchRecord");
            for (const field of batch.body.fields)
              if (field.name === "first" || field.name === "id")
                field.ty = named("DepthAlias0", [field.ty]);
            entries.push(
              {
                kind: "struct",
                name: "SequenceProbeNode",
                params: [],
                body: {
                  style: "named",
                  fields: [
                    {
                      name: "children",
                      ty: named("Vec", [named("SequenceProbeNode")]),
                    },
                  ],
                },
              },
              {
                kind: "struct",
                name: "SequenceProbeGeneric",
                params: ["T"],
                body: {
                  style: "named",
                  fields: [{ name: "values", ty: named("Vec", [named("T")]) }],
                },
              },
            );
            const probe = entries.find(
              (entry) => entry.kind === "function" && entry.name === "unicode",
            );
            assert(probe, "missing existing Unicode fixture export");
            assert.equal(probe.params.length, 0);
            assert.equal(probe.result.name, "String");
            // String and structural tuple results both use () -> output-buffer-id.
            probe.result = {
              kind: "tuple",
              args: [
                named("Vec", [named("Vec", [named("SequenceProbeNode")])]),
                { kind: "array", inner: named("String"), len: 2 },
                named("SequenceProbeGeneric", [named("SequenceProbeNode")]),
              ],
            };
          }
        }),
      );
      const args = [
        "generate",
        "--wasm",
        input,
        "--backend",
        backend,
        "--out-dir",
        folder,
        "--name",
        "collection_test",
      ];
      if (policy.startsWith("dict") && backend === "js_of_ocaml") {
        const p = spawnSync(resolve(root, "target/debug/ocamljs_pack"), args, {
          cwd: root,
          encoding: "utf8",
        });
        assert.equal(p.status, 1);
        assert.match(p.stderr, /dict.*requires.*Melange/);
        continue;
      }
      run(resolve(root, "target/debug/ocamljs_pack"), args, root);
      const body = policy === "array"
        ? `assert(B.String_map.find "__proto__" data.words=[|1.|]); assert(Hashtbl.find data.counts "constructor"=12.); assert(B.String_map.find "none" data.optional=None); assert(B.fixed_numbers ()=[|3.;2.;1.|]); assert(List.map fst(B.Vec_u32_map.bindings data.vectors)=[[|0.;2.|];[|1.|];[|1.;0.|]]); let owners=B.Counter.many_counters 20000. in Array.iteri (fun index owner->assert(B.Counter.current owner=float_of_int index);B.Counter.dispose owner) owners; ignore(B.collections data)`
        : `assert(Js.Dict.get data.words "__proto__"=Some [|1.|]); assert(Js.Dict.get data.counts "constructor"=Some 12.); assert(Js.Dict.get data.optional "none"=Some None); ignore(B.collections data)`;
      const numericLists =
        policy === "array"
          ? `
let () =
  let sample = B.float_samples false in
  let values = sample.B.values in
  assert(Array.length values=8 && Array.get values 0=1.25 && Array.get values 1=(-3.5) && Array.get values 7=2.75);
  assert(1. /. Array.get values 2 = neg_infinity && 1. /. Array.get values 3 = infinity);
  let unordered=Array.get values 4 in assert(unordered <> unordered);
  assert(Array.get values 5=infinity && Array.get values 6=neg_infinity);
  let ordinals=sample.B.ordinals in
  assert(Array.length ordinals=20000 && Array.get ordinals 0=0. && Array.get ordinals 19999=19999.);
  let empty=B.float_samples true in
  assert(empty.B.values=[||] && empty.B.ordinals=[||])
`
          : "";
      const depthProbes =
        policy === "array"
          ? `
let load_depth () =
  let rec count (node : B.depth_probe_node) =
    1 + (if Array.length node.B.children = 0 then 0 else count node.B.children.(0)) in
  assert(count (B.depth_probe 128.) = 128)
let load_batch () =
  let batch = B.Counter.batched_output () in
  assert(batch.B.first = "${backend === "rescript" ? "\ufeffλ" : "\\239\\187\\191λ"}" && batch.B.id = 17. && batch.B.flag);
  B.Counter.dispose batch.B.owner
`
          : "";
      writeFileSync(
        resolve(folder, "Test.ml"),
        `module B=Bindings\nlet data=B.sample_collections ()\nlet ()=${body}; assert(B.bytes_sum (Bytes.of_string "abc")=294.)\nlet load_owners ()=B.MapOwned.map_owners ()\nlet drops ()=B.MapOwned.map_drop_count ()\n${policy === "array" ? "let load_sequence ()=Array.iter B.Counter.dispose (B.Counter.many_counters 8.)\n" : ""}${numericLists}${depthProbes}`,
      );
      if (backend === "js_of_ocaml")
        writeFileSync(
          resolve(folder, "Test.ml"),
          readFileSync(resolve(folder, "Test.ml"), "utf8") +
            '\nlet ()=Js_of_ocaml.Js.export "load_owners" (Js_of_ocaml.Js.wrap_callback load_owners); Js_of_ocaml.Js.export "drops" (Js_of_ocaml.Js.wrap_callback drops)\n' +
            (policy === "array"
              ? 'let ()=Js_of_ocaml.Js.export "load_sequence" (Js_of_ocaml.Js.wrap_callback load_sequence)\n'
              : "") +
            (depthProbes
              ? 'let ()=Js_of_ocaml.Js.export "load_depth" (Js_of_ocaml.Js.wrap_callback load_depth); Js_of_ocaml.Js.export "load_batch" (Js_of_ocaml.Js.wrap_callback load_batch)\n'
              : ""),
        );
      writeFileSync(resolve(folder, "package.json"), '{"type":"commonjs"}');
      if (backend === "melange") {
        const includes = [
          "-I",
          ".",
          "-I",
          join(melange, "lib/melange/melange"),
          "-I",
          join(melange, "lib/melange/js/melange"),
        ];
        for (const file of ["Bindings.mli", "Bindings.ml", "Test.ml"])
          run(
            join(melange, "bin/melc"),
            [
              "-ppx",
              join(melange, "bin/melppx"),
              ...includes,
              "--mel-package-output",
              "commonjs:.",
              "-o",
              file.replace(/\.mli?$/, file.endsWith(".mli") ? ".cmi" : ".cmj"),
              file,
            ],
            folder,
          );
      } else if (backend === "rescript") {
        writeFileSync(
          resolve(folder, "Test.res"),
          run(compiler, ["-format", "Test.ml"], folder),
        );
        for (const file of ["Bindings.resi", "Bindings.res", "Test.res"]) {
          const text = run(compiler, ["-I", reslib, "-I", ".", file], folder);
          if (file.endsWith(".res"))
            writeFileSync(resolve(folder, file.replace(".res", ".js")), text);
        }
      } else {
        const js = join(target("js_of_ocaml"), "lib/js_of_ocaml"),
          jc = target("js_of_ocaml-compiler");
        const includes = ["-I", js];
        for (const file of ["Bindings.mli", "Bindings.ml", "Test.ml"])
          run(ocamlCompiler, [...includes, "-c", file], folder);
        run(
          ocamlCompiler,
          [
            "-no-check-prims",
            "-noautolink",
            ...includes,
            join(jc, "lib/js_of_ocaml-compiler/runtime/jsoo_runtime.cma"),
            join(js, "js_of_ocaml.cma"),
            "Bindings.cmo",
            "Test.cmo",
            "-o",
            "Test.bc",
          ],
          folder,
        );
        const find = join(folder, "findlib.conf");
        writeFileSync(find, "");
        run(join(jc, "bin/js_of_ocaml"), ["Test.bc", "-o", "Test.js"], folder, {
          OCAMLFIND_CONF: find,
          OCAMLLIB: ocamlLibrary,
          OCAMLPATH: join(jc, "lib"),
        });
      }
      writeFileSync(
        resolve(folder, "run.cjs"),
        `const assert=require('node:assert/strict'); (async()=>{const r=await import('./bindings.runtime.mjs');const i=await r.initialize(require('node:fs').readFileSync('./backend.wasm'));let armed=false,sequenceClaims=0,failSequence=false,depthSeed=0,maxDepth=0,enterCount=0;const seeded=new WeakSet();globalThis.Ocamljs_collection_test={...r.api,enter:reader=>{enterCount++;if(!seeded.has(reader)){seeded.add(reader);reader.depth=depthSeed;}r.api.enter(reader);maxDepth=Math.max(maxDepth,reader.depth);},r_handle_counter:reader=>{if(failSequence && ++sequenceClaims===4)throw Error("sequence decode fixture failure");return r.api.r_handle_counter(reader);},r_string:reader=>{const value=r.api.r_string(reader);return armed?'duplicate':value;}};const test=require('./Test.js');${policy.startsWith("dict") ? `assert.equal(Object.getPrototypeOf(test.data.words),null);assert.equal(Object.getPrototypeOf(test.data.counts),null);assert(Object.hasOwn(test.data.optional,'none'));` : ""}const before=test.drops();armed=true;try{assert.throws(()=>test.load_owners());}finally{armed=false;}assert.equal(test.drops(),before+2);${policy === "array" ? 'const live=i.e.__ocamljs_live();failSequence=true;try{assert.throws(()=>test.load_sequence());assert.equal(sequenceClaims,4);}finally{failSequence=false;}assert.equal(i.e.__ocamljs_live(),live,"Partial array decoding must release claimed and unclaimed owners");' : ""}${depthProbes ? `maxDepth=0;test.load_depth();assert.equal(maxDepth,128,"Aliases must delegate without charging reader depth");depthSeed=1;try{assert.throws(()=>test.load_depth());}finally{depthSeed=0;}depthSeed=126;maxDepth=0;enterCount=0;try{test.load_batch();assert.equal(enterCount,4,"BatchRecord, BatchId, Key and Counter each retain one depth boundary");assert.equal(maxDepth,128,"Span aliases must retain the transparent struct boundary");}finally{depthSeed=0;}depthSeed=127;try{assert.throws(()=>test.load_batch());}finally{depthSeed=0;}test.load_batch();` : ""}console.log('${policy}/${backend}: native collections, boundaries and cleanup passed');})().catch(e=>{console.error(e);process.exitCode=1});`,
      );
      process.stdout.write(
        run(process.execPath, ["run.cjs"], folder, {
          NODE_PATH: [runtime, resolve(root, "node_modules")].join(":"),
        }),
      );
    }
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
