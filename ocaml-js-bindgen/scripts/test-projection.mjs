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
const scratch = mkdtempSync(resolve(root, "tmp/projection-tests-"));
try {
  for (const backend of ["melange", "rescript", "js_of_ocaml"]) {
    const folder = resolve(scratch, backend);
    run("mkdir", ["-p", folder], root);
    const input = resolve(folder, "input.wasm");
    writeFileSync(input, base);
    const generate = (extra = [], wasm = input, out = folder) => [
      "generate",
      "--wasm",
      wasm,
      "--backend",
      backend,
      "--out-dir",
      out,
      "--name",
      "projection_test",
      ...extra,
    ];
    if (backend === "melange") {
      const nativeFolder = resolve(folder, "native-only");
      run(
        resolve(root, "target/debug/ocamljs_pack"),
        generate(
          [],
          (() => {
            const nativeInput = resolve(folder, "native.wasm");
            writeFileSync(
              nativeInput,
              withSchemaEntries(base, (entries) => {
                const strip = (value) => {
                  if (value && typeof value === "object") {
                    delete value.js_value;
                    delete value.js_value_rec;
                    for (const child of Object.values(value)) strip(child);
                  }
                };
                strip(entries);
              }),
            );
            return nativeInput;
          })(),
          nativeFolder,
        ),
        root,
      );
      run(
        process.execPath,
        ["--check", resolve(nativeFolder, "bindings.runtime.mjs")],
        root,
      );
    }
    run(resolve(root, "target/debug/ocamljs_pack"), generate(), root);
    const publicInterface = readFileSync(
      resolve(folder, "Bindings.mli"),
      "utf8",
    );
    assert(!publicInterface.includes("module Js_projection"));
    assert(
      !/^val projection_owned/m.test(publicInterface),
      "scoped exports must stay inside their module",
    );
    const counterModule = publicInterface.match(
      /(?:module(?: rec)?|and) Counter : sig\n([\s\S]*?)\nend/,
    );
    assert(counterModule, "missing Counter module signature");
    const counterInterface = counterModule[1];
    assert.match(
      counterInterface,
      /val sample : t -> bool -> \(projection_sample, Calculation_error\.t\) result/,
    );
    assert.match(
      counterInterface,
      /val checked : t -> bool -> projection_sample/,
    );
    assert.match(
      counterInterface,
      /exception Checked_error of Calculation_error\.t/,
    );
    writeFileSync(resolve(folder, "package.json"), '{"type":"commonjs"}');
    const jsoo = backend === "js_of_ocaml";
    const propertyName = (raw) => {
      const invalid = !/^[a-z]/.test(raw),
        keyword = ["type", "done"].includes(raw);
      if (jsoo && raw.includes("_")) return `${invalid ? "_" : ""}${raw}_`;
      if (!jsoo && (raw.includes("__") || raw.startsWith("_")))
        return `${raw}__`;
      return invalid || keyword ? `_${raw}` : raw;
    };
    const prop = (value, name) =>
      `((${value})${jsoo ? "##." : "##"}${propertyName(name)})`;
    const field = (name) => prop("retained", name);
    const bigint = (value) =>
      jsoo
        ? `text((${value})##toString)`
        : `${backend === "melange" ? "Js.Bigint" : "Js.BigInt"}.toString (${value})`;
    const dictionaries = jsoo
      ? `
  let words=${field("words")} in
  assert(${prop("Js.object_keys words", "length")}=6);
  let values : Js.js_string Js.t Js.js_array Js.t = Js.Unsafe.get words "__proto__" in
  assert(text(get values 0)="value:__proto__");
  let count : Js.number_t = Js.Unsafe.get ${field("counts")} "constructor" in assert(number count=9.);
`
      : `
  let words=${field("words")} in
  assert(Array.length(Js.Dict.keys words)=6);
  (match Js.Dict.get words "__proto__" with Some values -> assert(text(get values 0)="value:__proto__") | None -> assert false);
  (match Js.Dict.get ${field("counts")} "constructor" with Some count -> assert(number count=9.) | None -> assert false);
`;

    const consumer = `module B=Bindings
${jsoo ? "open Js_of_ocaml" : ""}
let retained = match B.projection_value false with Ok value -> value | Error _ -> assert false
let text = ${jsoo ? "Js.to_string" : "fun value -> value"}
let number = ${jsoo ? "Js.float_of_number" : "fun value -> value"}
let boolean = ${jsoo ? "Js.to_bool" : "fun value -> value"}
let lambda = ${jsoo ? '"λ"' : `${backend === "melange" ? "Js.String" : "Js.String2"}.fromCharCode 0x3bb`}
let crab = ${jsoo ? '"🦀"' : `${backend === "melange" ? "Js.String" : "Js.String2"}.fromCodePoint 0x1f980`}
let get = ${jsoo ? "fun values i -> Js.Optdef.get (Js.array_get values i) (fun () -> assert false)" : "Array.get"}
let some = ${jsoo ? "fun value -> Js.Optdef.get value (fun () -> assert false)" : "Js.Undefined.getExn"}
let check_retained () =
  let flags=${field("flags")} in
  assert(${jsoo ? prop("flags", "length") : "Array.length flags"}=2);
  assert(not(boolean(get flags 0)) && boolean(get flags 1));
  let numbers=${field("numbers")} in
  assert(1. /. number(get numbers 0)=neg_infinity && number(get numbers 1)=1.25 && number(get numbers 2)=infinity);
  let nan=number(get numbers 3) in assert(nan <> nan);
  assert(${bigint(field("signed"))}="-9223372036854775808" && ${bigint(field("unsigned"))}="18446744073709551615");
${dictionaries}
  let pair=${field("pair")} in assert(boolean(${prop("pair", "0")}) && text(${prop("pair", "1")})=("tuple" ^ String.make 1 (Char.chr 0)));
  let options=${field("options")} in
  let empty=get options 0 and nested=get options 1 and present=get options 2 in
  assert(number(${prop("empty", "tag")})=0.);
  assert(number(${prop("nested", "tag")})=1. && number(${prop("some (" + prop("nested", "value") + ")", "tag")})=0.);
  assert(text(some (${prop("some (" + prop("present", "value") + ")", "value")}))=lambda);
  let cases=${field("cases")} in
  let flag=some (${prop("get cases 1", "flag")}) in assert(boolean(${prop("flag", "0")}));
  let named=some (${prop("get cases 2", "named")}) in assert(text(${prop("named", "text")})=("payload " ^ crab));
  assert(text(${field("wrapped")})=("wrapped " ^ lambda ^ String.make 1 (Char.chr 0)));
  assert(text(${field("type")})="keyword" && text(${field("trailing_")})="trailing" && text(${field("under_score")})="underscore" && text(${field("double__under")})="double");
  let nested=${field("nested")} in assert(number(${prop("nested", "tag")})=0. && text(some (${prop("nested", "ok")}))="nested");
  let proto=some (${prop("get cases 3", "__proto__")}) in assert(boolean(${prop("proto", "value")}));
  let tuple=B.projection_tuple () in assert(boolean(${prop("tuple", "0")}) && text(${prop("tuple", "1")})=("tuple root " ^ lambda));
  let recursive=B.projection_recursive () in
  let child=get (${prop("recursive", "children")}) 0 in assert(text(${prop("child", "label")})="child");
  assert(${field("key")}=B.Key.Equals);
  let mixed=B.projection_mixed () in
  let rows=${prop("mixed", "rows")} in let native=get rows 0 in assert(native.B.flag && native.B.text="native λ");
  let mixed=B.projection_mixed_echo mixed in assert(not((${prop("mixed", "item")}).B.flag));
  let envelope=B.projection_envelope_echo (B.projection_envelope ()) in
  let native=get envelope.B.rows 0 in assert(native.B.flag);
  let alias=B.projection_js_result false in let nested=B.projection_js_result_envelope false in
  assert(number(${prop("alias", "tag")})=0. && number(${prop("nested.B.value", "tag")})=0.);
  let alias=B.projection_js_result true in
  (match some (${prop("alias", "error")}) with B.Calculation_error.Invalid_operation _ -> () | _ -> assert false);
  let echoed=B.projection_roundtrip retained in assert(text(${prop("echoed", "wrapped")})=text(${field("wrapped")}));
  let sum=ref 0. in
  for i=0 to 1 do sum := !sum +. number(${jsoo ? "get numbers i" : "numbers.(i)"}) done;
  assert(!sum=1.25)
let validate_recursive value = ignore(B.projection_recursive_echo value)
let load_failure () = ignore(B.projection_value false)
let () =
  check_retained ();
  (match B.projection_flat_error true with Error error -> assert(error.B.code=13.) | Ok _ -> assert false);
  ignore(B.projection_flat_error false);
  (match B.projection_sample false with Ok value -> assert(text(${prop("value", "wrapped")})=text(${field("wrapped")})) | Error _ -> assert false);
  (match B.projection_value true with Error(B.Calculation_error.Invalid_operation {operation}) -> assert(String.length operation>5) | _ -> assert false);
  (try ignore(B.projection_throw true); assert false with B.Projection_throw_error(B.Calculation_error.Invalid_operation _) -> ());
  ignore(B.projection_throw false);
  let owner=B.Counter.create 23. in
  (match B.Counter.sample owner false with Ok value -> assert(text(${prop("value", "wrapped")})=text(${field("wrapped")})) | Error _ -> assert false);
  (try ignore(B.Counter.checked owner true); assert false with B.Counter.Checked_error(B.Calculation_error.Invalid_operation _) -> ());
  ignore(B.Counter.checked owner false);
  B.Counter.dispose owner
`;
    const rescriptConsumer = `module B = Bindings

let retained = switch B.projection_value(false) {
| Ok(value) => value
| Error(_) => assert(false)
}
let text = value => value
let number = value => value
let boolean = value => value
let lambda = Js.String2.fromCharCode(0x3bb)
let crab = Js.String2.fromCodePoint(0x1f980)
let get = Array.get
let some = Js.Undefined.getExn
let check_retained = () => {
  let flags = retained["flags"]
  assert(Array.length(flags) == 2)
  assert(!boolean(get(flags, 0)) && boolean(get(flags, 1)))
  let numbers = retained["numbers"]
  assert(1. /. number(get(numbers, 0)) == neg_infinity &&
    (number(get(numbers, 1)) == 1.25 &&
    number(get(numbers, 2)) == infinity))
  let nan = number(get(numbers, 3))
  assert(nan != nan)
  assert(Js.BigInt.toString(retained["signed"]) == "-9223372036854775808" &&
    Js.BigInt.toString(retained["unsigned"]) == "18446744073709551615")

  let words = retained["words"]
  assert(Array.length(Js.Dict.keys(words)) == 6)
  switch Js.Dict.get(words, "__proto__") {
  | Some(values) => assert(text(get(values, 0)) == "value:__proto__")
  | None => assert(false)
  }
  switch Js.Dict.get(retained["counts"], "constructor") {
  | Some(count) => assert(number(count) == 9.)
  | None => assert(false)
  }

  let pair = retained["pair"]
  assert(boolean(pair["0"]) && text(pair["1"]) == "tuple" ++ String.make(1, Char.chr(0)))
  let options = retained["options"]
  let empty = get(options, 0) and nested = get(options, 1) and present = get(options, 2)
  assert(number(empty["tag"]) == 0.)
  assert(number(nested["tag"]) == 1. && number(some(nested["value"])["tag"]) == 0.)
  assert(text(some(some(present["value"])["value"])) == lambda)
  let cases = retained["cases"]
  let flag = some(get(cases, 1)["flag"])
  assert(boolean(flag["0"]))
  let named = some(get(cases, 2)["named"])
  assert(text(named["text"]) == "payload " ++ crab)
  assert(text(retained["wrapped"]) == "wrapped " ++ (lambda ++ String.make(1, Char.chr(0))))
  assert(text(retained["type"]) == "keyword" &&
    (text(retained["trailing_"]) == "trailing" &&
    (text(retained["under_score"]) == "underscore" &&
      text(retained["double__under"]) == "double")))
  let nested = retained["nested"]
  assert(number(nested["tag"]) == 0. && text(some(nested["ok"])) == "nested")
  let proto = some(get(cases, 3)["__proto__"])
  assert(boolean(proto["value"]))
  let tuple = B.projection_tuple()
  assert(boolean(tuple["0"]) && text(tuple["1"]) == "tuple root " ++ lambda)
  let recursive = B.projection_recursive()
  let child = get(recursive["children"], 0)
  assert(text(child["label"]) == "child")
  assert(retained["key"] == B.Key.Equals)
  let mixed = B.projection_mixed()
  let native = mixed["rows"][0]
  assert(native.B.flag && native.B.text == "native λ")
  let mixed = B.projection_mixed_echo(mixed)
  assert(!mixed["item"].B.flag)
  let envelope = B.projection_envelope_echo(B.projection_envelope())
  assert(envelope.B.rows[0].B.flag)
  let alias = B.projection_js_result(false)
  let nested = B.projection_js_result_envelope(false)
  assert(alias["tag"] == 0. && nested.B.value["tag"] == 0.)
  switch Js.Undefined.getExn(B.projection_js_result(true)["error"]) {
  | B.Calculation_error.Invalid_operation(_) => ()
  | _ => assert(false)
  }
  let echoed = B.projection_roundtrip(retained)
  assert(echoed["wrapped"] == retained["wrapped"])
  let sum = ref(0.)
  for i in 0 to 1 {
    sum := sum.contents +. number(numbers[i])
  }
  assert(sum.contents == 1.25)
}
let validate_recursive = value => ignore(B.projection_recursive_echo(value))
let load_failure = () => ignore(B.projection_value(false))
let () = {
  check_retained()
  switch B.projection_flat_error(true) {
  | Error(error) => assert(error.B.code == 13.)
  | Ok(_) => assert(false)
  }
  ignore(B.projection_flat_error(false))
  switch B.projection_sample(false) {
  | Ok(value) => assert(value["wrapped"] == retained["wrapped"])
  | Error(_) => assert(false)
  }
  switch B.projection_value(true) {
  | Error(B.Calculation_error.Invalid_operation({operation})) =>
    assert(String.length(operation) > 5)
  | _ => assert(false)
  }
  try {
    ignore(B.projection_throw(true))
    assert(false)
  } catch {
  | B.Projection_throw_error(B.Calculation_error.Invalid_operation(_)) => ()
  }
  ignore(B.projection_throw(false))
  let owner = B.Counter.create(23.)
  switch B.Counter.sample(owner, false) {
  | Ok(value) => assert(text(value["wrapped"]) == text(retained["wrapped"]))
  | Error(_) => assert(false)
  }
  try {
    ignore(B.Counter.checked(owner, true))
    assert(false)
  } catch {
  | B.Counter.Checked_error(B.Calculation_error.Invalid_operation(_)) => ()
  }
  ignore(B.Counter.checked(owner, false))
  B.Counter.dispose(owner)
}
`;
    const selectedMl = `
let selected = B.selector_sample ()
let echo_selected value = B.selector_echo value
let selected_async () = B.selector_async ()
let selected_number value = B.selector_alias value
let selected_number_async value = B.selector_alias_async value
let selected_scalar_async () = B.selector_scalar_async ()
let selected_text_result_async fail = B.selected_text_result_async fail
let selected_unit_text_result_async () = B.selected_unit_text_result_async ()
let check_selected () =
  let ok=B.selected_text_result false in
  assert(number(${prop("ok", "tag")})=0. && text(some (${prop("ok", "ok")}))=("selected " ^ lambda));
  let error=B.selected_text_result true in
  assert(number(${prop("error", "tag")})=1. && ${bigint("some (" + prop("error", "error") + ")")}="18446744073709551615");
  assert(text(B.selector_text_echo (${prop("selected", "string")}))=text(${prop("selected", "string")}));
  assert(B.selector_text_borrowed (${prop("selected", "string")})=10.);
  let noted=ref 0. in
  assert(text(B.selector_text_notify (${prop("selected", "string")}) (fun n -> noted:=n.B.value))=text(${prop("selected", "string")}));
  assert(!noted=10.);
  let native_string="λ🦀" ^ String.make 1 (Char.chr 0) in
  assert(B.native_alias_echo native_string=native_string);
  assert(B.native_alias_borrowed native_string=7.);
  assert(B.selector_slice_sum (${prop("selected", "floats")})=infinity);
  assert(number(B.selector_number (Int64.of_string "9007199254740991"))=9007199254740991.);
  assert(${bigint("B.selector_bigint (-32768)")}="-32768");
  assert(B.selector_native_list [-32768;0;32767]=[-32768;0;32767]);
  let item : B.projection_native_item = {flag=true;text="native λ"} in
  let generic : B.projection_native_item B.selector_generic = ${jsoo ? "object%js val values = Js.array [|item|] end" : "[%mel.obj {values=[|item|]}]"} in
  let echoed=B.selector_generic generic in
  assert((get (${prop("echoed", "values")}) 0).B.text=item.B.text);
  let flat : B.selector_flat = ${jsoo ? "object%js val x=Js.number_of_float 1.25 val y=Js.number_of_float (-0.) end" : "[%mel.obj {x=1.25;y=(-0.)}]"} in
  let echoed=B.selector_flat flat in
  assert(number(${prop("echoed", "x")})=1.25 && 1. /. number(${prop("echoed", "y")})=neg_infinity);
  let mixed : B.selector_mixed_flat = {x=${jsoo ? "Js.number_of_float (-2147483648.)" : "(-2147483648.)"};y=4294967295.} in
  let echoed=B.selector_mixed_flat mixed in
  assert(number echoed.B.x=(-2147483648.) && echoed.B.y=4294967295.)
let () = check_selected ()
`;
    const selectedRes = `
let selected = B.selector_sample()
let echo_selected = value => B.selector_echo(value)
let selected_async = () => B.selector_async()
let selected_number = value => B.selector_alias(value)
let selected_number_async = value => B.selector_alias_async(value)
let selected_scalar_async = () => B.selector_scalar_async()
let selected_text_result_async = fail => B.selected_text_result_async(fail)
let selected_unit_text_result_async = () => B.selected_unit_text_result_async()
let check_selected = () => {
  let ok=B.selected_text_result(false)
  assert(ok["tag"] == 0. && Js.Undefined.getExn(ok["ok"]) == "selected λ")
  let error=B.selected_text_result(true)
  assert(error["tag"] == 1. && Js.Undefined.getExn(error["error"]) == 18446744073709551615n)

  assert(B.selector_text_echo(selected["string"]) == selected["string"])
  assert(B.selector_text_borrowed(selected["string"]) == 10.)
  let noted=ref(0.)
  assert(B.selector_text_notify(selected["string"], n => {noted.contents=n.value}) == selected["string"])
  assert(noted.contents == 10.)
  let native_string="λ🦀" ++ String.make(1, Char.chr(0))
  assert(B.native_alias_echo(native_string) == native_string)
  assert(B.native_alias_borrowed(native_string) == 7.)
  assert(B.selector_slice_sum(selected["floats"]) == infinity)
  assert(B.selector_number(Int64.of_string("9007199254740991")) == 9007199254740991.)
  assert(Js.BigInt.toString(B.selector_bigint(-32768)) == "-32768")
  assert(B.selector_native_list(list{-32768,0,32767}) == list{-32768,0,32767})
  let item: B.projection_native_item = {flag:true,text:"native λ"}
  let generic: B.selector_generic<B.projection_native_item> = {"values":[item]}
  assert(B.selector_generic(generic)["values"][0].B.text == item.B.text)
  let flat: B.selector_flat = {"x":1.25,"y":-0.}
  let echoed=B.selector_flat(flat)
  assert(echoed["x"] == 1.25 && 1. /. echoed["y"] == neg_infinity)
  let mixed: B.selector_mixed_flat = {x:-2147483648.,y:4294967295.}
  let echoed=B.selector_mixed_flat(mixed)
  assert(echoed.B.x == -2147483648. && echoed.B.y == 4294967295.)
}
let ()=check_selected()
`;
    writeFileSync(
      resolve(folder, "Test.ml"),
      consumer +
        selectedMl +
        (backend === "js_of_ocaml"
          ? '\nlet ()=Js_of_ocaml.Js.export "selected" selected; Js_of_ocaml.Js.export "echo_selected" (Js_of_ocaml.Js.wrap_callback echo_selected); Js_of_ocaml.Js.export "selected_number" (Js_of_ocaml.Js.wrap_callback selected_number); Js_of_ocaml.Js.export "selected_async" (Js_of_ocaml.Js.wrap_callback (fun () -> Js_of_ocaml.Promise.to_any (selected_async ()))); Js_of_ocaml.Js.export "selected_number_async" (Js_of_ocaml.Js.wrap_callback (fun value -> Js_of_ocaml.Promise.to_any (selected_number_async value))); Js_of_ocaml.Js.export "selected_unit_text_result_async" (Js_of_ocaml.Js.wrap_callback (fun () -> Js_of_ocaml.Promise.to_any (selected_unit_text_result_async ()))); Js_of_ocaml.Js.export "selected_text_result_async" (Js_of_ocaml.Js.wrap_callback (fun fail -> Js_of_ocaml.Promise.to_any (selected_text_result_async (Js_of_ocaml.Js.to_bool fail)))); Js_of_ocaml.Js.export "selected_scalar_async" (Js_of_ocaml.Js.wrap_callback (fun () -> Js_of_ocaml.Promise.to_any (selected_scalar_async ()))); Js_of_ocaml.Js.export "retained" retained; Js_of_ocaml.Js.export "check_retained" (Js_of_ocaml.Js.wrap_callback check_retained); Js_of_ocaml.Js.export "load_failure" (Js_of_ocaml.Js.wrap_callback load_failure); Js_of_ocaml.Js.export "validate_recursive" (Js_of_ocaml.Js.wrap_callback validate_recursive)\n'
          : ""),
    );
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
        rescriptConsumer + selectedRes,
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
        run(
          ocamlCompiler,
          [
            ...includes,
            "-ppx",
            join(target("js_of_ocaml-ppx"), "lib/js_of_ocaml-ppx/ppx.exe") +
              " --as-ppx",
            "-c",
            file,
          ],
          folder,
        );
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

    const implementation = readFileSync(resolve(folder, "Bindings.ml"), "utf8");
    if (!jsoo) {
      const compiled = readFileSync(resolve(folder, "Test.js"), "utf8");
      assert.match(compiled, /\.flags\b/);
      assert.match(compiled, /for\s*\(/);
      assert(!compiled.includes("Js_projection"));
    }

    const runner = `const assert=require('node:assert/strict');
(async()=>{
 const r=await import('./bindings.runtime.mjs');
 const i=await r.initialize(require('node:fs').readFileSync('./backend.wasm'));
 let armed=false,closed=0,lastReader;
 globalThis.Ocamljs_projection_test={...r.api,
   r_js_named_projection_sample:(reader,...native)=>{lastReader=reader;if(armed){r.api.r_js_string(reader);throw Error('projection failure');}return r.api.r_js_named_projection_sample(reader,...native);},
   close:reader=>{closed++;return r.api.close(reader);}};
 const t=require('./Test.js'),v=t.retained;
 const selected=t.selected;
 const liveBeforeStringCallback=i.e.__ocamljs_live(),stringCallbackError={stringCallback:true};
 assert.throws(()=>r.api.call_selector_text_notify(selected.string,reader=>{r.api.close(reader);throw stringCallbackError;}),error=>error===stringCallbackError);
 assert.equal(i.e.__ocamljs_live(),liveBeforeStringCallback,'callback failure leaked raw string result');

 assert.equal(selected.number,9007199254740991);assert.equal(selected.bigint,-32768n);
 assert.equal(selected.boolean,true);assert.equal(selected.string,'\\ufeffλ🦀\\0');
 assert.deepEqual(selected.array,[0,65535]);assert.deepEqual([...selected.bytes],[0,255]);
 assert(selected.bytes instanceof Uint8Array);assert(selected.floats instanceof Float64Array);assert(selected.wide instanceof BigUint64Array);
 assert(Object.is(selected.floats[0],-0));assert.equal(selected.floats[1],Infinity);assert.deepEqual([...selected.wide],[0n,18446744073709551615n]);
 assert.equal(Object.getPrototypeOf(selected.dict),null);assert.equal(selected.dict.__proto__,-1);
 assert(selected.map instanceof Map);assert.equal(selected.map.get(-2),'negative');
 assert.equal(selected.nullable,null);assert.equal(selected.optional,'present');assert.equal(selected.unit,undefined);
 const echoed=t.echo_selected(selected);assert.equal(echoed.nullable,null);assert.equal(echoed.unit,undefined);
 for(const [field,value] of [['number',9007199254740992],['number',1.5],['bigint',32768n],['bigint',1],['boolean',1],['string',1],['array',[65536]],['bytes',new Int8Array(2)],['floats',new Float64Array(1)],['nullable',undefined],['optional',null],['map',{}]]) {
   assert.throws(()=>t.echo_selected({...selected,[field]:value}),field);
 }
 assert.equal(t.selected_number(9007199254740991),9007199254740991);
 assert.throws(()=>t.selected_number(9007199254740992));
 const bytes=new Uint8Array([4,5,6]);assert.deepEqual([...t.echo_selected({...selected,bytes:bytes.subarray(1)}).bytes],[5,6]);
 const detached=new Uint8Array([1]);structuredClone(detached,{transfer:[detached.buffer]});assert.throws(()=>t.echo_selected({...selected,bytes:detached}));
 const later=await t.selected_async();assert.equal(later.number,selected.number);
 assert.equal(await t.selected_number_async(42),42);assert.equal(await t.selected_scalar_async(),42);
 const selectedOk=await t.selected_text_result_async(false);assert.equal(selectedOk.tag,0);assert.equal(selectedOk.ok,"selected λ");
 const selectedUnit=await t.selected_unit_text_result_async();assert.equal(selectedUnit.tag,1);assert.equal(selectedUnit.error,undefined);
 const selectedError=await t.selected_text_result_async(true);assert.equal(selectedError.tag,1);assert.equal(selectedError.error,18446744073709551615n);

 assert.equal(v.first,'\ufeffλ');assert.equal(v.second,'🙂\0');assert.equal(v.third,'\ue000🦀');
 assert.deepEqual(v.flags,[false,true]);assert.equal(v.signed,-9223372036854775808n);assert.equal(v.unsigned,18446744073709551615n);
 assert(Array.isArray(v.numbers)&&Array.isArray(v.cases)&&!Array.isArray(v.pair));
 assert.equal(Object.getPrototypeOf(v.words),null);assert.equal(Object.getPrototypeOf(v.counts),null);
 for(const key of ['','__proto__','constructor','λ','🙂','\ue000']){assert(Object.hasOwn(v.words,key));assert.equal(v.words[key][0],'value:'+key);}
 assert.equal(v.wrapped,'wrapped λ\0');assert.equal(Object.getPrototypeOf(v.wrapped_words),null);assert.equal(v.wrapped_words.__proto__,'wrapped:__proto__');assert.equal(v.wrapped_words['λ'],'wrapped:λ');
 assert.equal(v.options[0].tag,0);assert.equal(v.options[1].tag,1);assert.equal(v.options[1].value.tag,0);assert.equal(v.options[2].value.value,'λ');
 assert.equal(lastReader.closed,true);
 assert(Object.hasOwn(v.cases[3],'__proto__'));assert.equal(v.cases[3].__proto__.value,true);assert.equal(Object.getPrototypeOf(v.cases[3]),Object.prototype);
 const readKey=reader=>{r.api.enter(reader);try{return r.api.r_u32(reader);}finally{r.api.leave(reader);}};
 const native=s=>${backend === "rescript" ? "s" : "Buffer.from(s).toString('latin1')"};
 // Exercise one reader with interleaved native byte strings and raw slices.
 for(const rawFirst of [false,true]){
   const reader=r.api.call_projection_sample(false);assert.equal(r.api.r_u8(reader),0);
   assert.equal((rawFirst?r.api.r_js_string:r.api.r_string)(reader),rawFirst?'\ufeffλ':native('\ufeffλ'));
   assert.equal((rawFirst?r.api.r_string:r.api.r_js_string)(reader),rawFirst?native('🙂\0'):'🙂\0');
   assert.equal(r.api.r_js_string(reader),'\ue000🦀');r.api.close(reader);
 }
 // The DTO plus nested enum/transparent wrapper consume two depth levels;
 // aliases must delegate without charging another depth boundary.
 for(const seed of [126,127]){
   const reader=r.api.call_projection_sample(false);r.api.r_u8(reader);reader.depth=seed;
   try{if(seed===126){r.api.r_js_named_projection_sample(reader,readKey);r.api.finish(reader);}else assert.throws(()=>r.api.r_js_named_projection_sample(reader,readKey));}finally{r.api.close(reader);}
 }
 // Batched primitive reads must retain the same validation and range checks.
 {
   const reader=r.api.call_projection_sample(false);r.api.r_u8(reader);
   reader.view().setUint8(reader.ptr+reader.position+16,2);
   try{assert.throws(()=>r.api.r_js_named_projection_sample(reader,readKey),/invalid bool/);}finally{r.api.close(reader);}
 }
 {
   const reader=r.api.call_projection_sample(false);r.api.r_u8(reader);reader.length=reader.position+2;
   try{assert.throws(()=>r.api.r_js_named_projection_sample(reader,readKey),/truncated value/);}finally{r.api.close(reader);}
 }
 const cycle={label:'cycle',children:[]};cycle.children.push(cycle);
 assert.throws(()=>t.validate_recursive(cycle),/nesting limit/);
 t.validate_recursive({label:'valid',children:[]});
 const before=closed,live=i.e.__ocamljs_live();armed=true;
 try{assert.throws(()=>t.load_failure());}finally{armed=false;}
 assert.equal(lastReader.closed,true);assert.equal(lastReader.poolText,null);assert.equal(lastReader.poolBytes,null);assert.equal(closed,before+1);assert.equal(i.e.__ocamljs_live(),live);
 i.e.memory.grow(1);t.check_retained();assert.equal(v.second,'🙂\0');
 assert.strictEqual(await r.initialize(require('node:fs').readFileSync('./backend.wasm')),i);
 console.log('${backend}: direct JavaScript projections and native boundary passed');
})().catch(error=>{console.error(error);process.exitCode=1});`;
    writeFileSync(resolve(folder, "run.cjs"), runner);
    process.stdout.write(
      run(process.execPath, ["run.cjs"], folder, {
        NODE_PATH: [runtime, resolve(root, "node_modules")].join(":"),
      }),
    );

    if (backend === "melange") {
      const n = (name, args = []) => ({ kind: "named", name, args });
      const rejects = (label, mutate, pattern) => {
        const file = resolve(folder, `rejected-${label}.wasm`);
        writeFileSync(file, withSchemaEntries(base, mutate));
        const p = spawnSync(
          resolve(root, "target/debug/ocamljs_pack"),
          generate([], file, resolve(folder, "rejected")),
          { cwd: root, encoding: "utf8" },
        );
        assert.equal(p.status, 1, label);
        assert.match(p.stderr, pattern, label);
      };
      const sample = (entries) =>
        entries.find((e) => e.name === "ProjectionSample");
      const selected = (entries) =>
        entries.find((e) => e.name === "projection_value");
      rejects(
        "opaque",
        (entries) => {
          entries.find((entry) => entry.name === "Counter").js_value = true;
        },
        /concrete structural declaration/,
      );
      rejects(
        "named-boundary",
        (entries) => {
          const field = sample(entries).body.fields[0];
          field.ty = n("CalculationError");
          field.js_value = true;
        },
        /annotate that type with js_value/,
      );
      rejects(
        "mutable",
        (entries) => {
          sample(entries).body.fields[0].ty = { ...n("String"), mutable: true };
        },
        /requires owned/,
      );
      rejects(
        "reference",
        (entries) => {
          sample(entries).body.fields[0].ty = { kind: "ref", inner: n("str") };
        },
        /requires owned|reference/,
      );
      rejects(
        "map-key",
        (entries) => {
          sample(entries).body.fields.find(
            (f) => f.name === "counts",
          ).js_selector = "dict";
          sample(entries).body.fields.find(
            (f) => f.name === "counts",
          ).js_value = true;
          sample(entries).body.fields.find(
            (f) => f.name === "counts",
          ).ty.args[0] = n("u32");
        },
        /requires String keys/,
      );
    }
  }
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
