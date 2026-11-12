// Compile external JSON fixtures/devtool inputs into genuine OCaml values.
// The live application consumes the native binding API directly.
import { readFileSync, writeFileSync, existsSync } from "node:fs";
const options = Object.fromEntries(
  Array.from({ length: Math.floor((process.argv.length - 2) / 2) }, (_, i) => [
    process.argv[2 + i * 2],
    process.argv[3 + i * 2],
  ]),
);
if (!options["--schema"] || !options["--out"]) throw new Error("Use --schema PATH --out PATH");
const typeModule = options["--module"] ?? "Domain";
const schema = JSON.parse(readFileSync(options["--schema"], "utf8"));
const snake = (s) =>
  s
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase();
const keywords = new Set(
  "from default effect mod land lor lxor lsl lsr asr and as assert begin class constraint do done downto else end exception external false for fun function functor if in include inherit initializer lazy let match method module mutable new nonrec object of open or private rec sig struct then to true try type val virtual when while with".split(
    " ",
  ),
);
const label = (s) => {
  s = snake(s);
  return keywords.has(s) ? s + "_" : s;
};
const quote = JSON.stringify;
const attr = (s, k) => s?.match(new RegExp(`(?:^|[ (,])${k}\\s*=\\s*"([^"]+)"`))?.[1];
function renamed(name, attrs, parent, variant = false) {
  const rename = attr(attrs, "rename");
  if (rename) return rename;
  const rule = attr(parent, "rename_all");
  if (rule === "lowercase") return name.toLowerCase();
  if (rule === "camelCase")
    return variant
      ? name[0].toLowerCase() + name.slice(1)
      : name.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
  return name;
}
const mapKeyName = (t) => {
  if (t.kind === "tuple") return `tuple_${t.args.map(mapKeyName).join("_")}`;
  if (t.kind === "array") return `array_${t.len}_${mapKeyName(t.inner)}`;
  return snake(t.name) + (t.args ?? []).map((arg) => `_${mapKeyName(arg)}`).join("");
};
const mapModule = (t) => {
  const name = mapKeyName(t);
  return name[0].toUpperCase() + name.slice(1) + "_map";
};
const mapKeys = new Map();
function ty(t, v, encode) {
  const call = (name, x) => `${encode ? "to" : "of"}_${name} (${x})`;
  if (t.kind === "array" || t.kind === "slice")
    return `${encode ? "put_array" : "get_array"} (fun v -> ${ty(t.inner, "v", encode)}) (${v})`;
  if (t.kind === "tuple")
    return t.args.length === 0
      ? encode
        ? "Js.Json.null"
        : "()"
      : `${encode ? "put_tuple" : "get_tuple"}_${t.args.length} ${t.args.map((a, i) => `(fun v -> ${ty(a, "v", encode)})`).join(" ")} (${v})`;
  if (t.kind !== "named") throw new Error(JSON.stringify(t));
  if (t.name === "String" || t.name === "str")
    return `${encode ? "put_string" : "get_string"} (${v})`;
  if (t.name === "bool") return `${encode ? "Js.Json.boolean" : "get_bool"} (${v})`;
  if (["i32", "isize"].includes(t.name))
    return encode
      ? `Js.Json.number (Int32.to_float (${v}))`
      : `Int32.of_float (${options["--strict"] ? "get_integer (-2147483648.) 2147483647." : "get_number"} (${v}))`;
  if (/^[ui](8|16)$/.test(t.name))
    return encode
      ? `Js.Json.number (float_of_int (${v}))`
      : `int_of_float (${options["--strict"] ? `get_integer ${t.name[0] === "i" ? -Math.pow(2, Number(t.name.slice(1)) - 1) : 0}. ${t.name[0] === "i" ? Math.pow(2, Number(t.name.slice(1)) - 1) - 1 : Math.pow(2, Number(t.name.slice(1))) - 1}.` : "get_number"} (${v}))`;
  if (/^u(32|size)$|^f(32|64)$/.test(t.name))
    return `${encode ? "Js.Json.number" : options["--strict"] && /^[u](32|size)$/.test(t.name) ? "get_integer 0. 4294967295." : "get_number"} (${v})`;
  if (t.name === "Option")
    return `${encode ? "put_option" : "get_option"} (fun v -> ${ty(t.args[0], "v", encode)}) (${v})`;
  if (t.name === "Vec")
    return `${encode ? "put_array" : "get_array"} (fun v -> ${ty(t.args[0], "v", encode)}) (${v})`;
  if (["Box", "Rc", "Arc"].includes(t.name)) return ty(t.args[0], v, encode);
  if (t.name === "HashMap" || t.name === "BTreeMap") {
    const codecs = `(fun v -> ${ty(t.args[0], "v", encode)}) (fun v -> ${ty(t.args[1], "v", encode)})`;
    if (t.map === "dict")
      return encode
        ? `put_map ${codecs} (Array.to_list (Js.Dict.entries (${v})))`
        : `(let table = dict_create Js.Nullable.null in List.iter (fun (key,value) -> Js.Dict.set table key value) (get_map ${codecs} (${v})); table)`;
    if (t.name === "HashMap") {
      const key = `hash_${mapKeyName(t.args[0])}`;
      mapKeys.set(key, "Hashtbl");
      return encode
        ? `put_map ${codecs} (List.of_seq (Hashtbl.to_seq (${v})))`
        : `Hashtbl.of_seq (List.to_seq (get_map ${codecs} (${v})))`;
    }
    const key = mapKeyName(t.args[0]);
    const module = mapModule(t.args[0]);
    mapKeys.set(key, module);
    return encode
      ? `put_map ${codecs} (${module}.bindings (${v}))`
      : `(List.fold_left (fun table (key,value) -> ${module}.add key value table) ${module}.empty (get_map ${codecs} (${v})))`;
  }
  return call(snake(t.name), v);
}
const enumModule = (e) =>
  typeModule + "." + snake(e.name)[0].toUpperCase() + snake(e.name).slice(1);
const nativeType = (e) => (e.kind === "enum" ? `${enumModule(e)}.t` : label(e.name));
const enumConstructor = (e, v) =>
  `${enumModule(e)}.${snake(v.name)[0].toUpperCase()}${snake(v.name).slice(1)}`;
// A selected dictionary alias carries JS strings/numbers through anonymous
// containers; named payloads keep their own declared representation.
function jsDictionaryType(t, v, encode) {
  if (t.kind === "named" && ["BTreeMap", "HashMap"].includes(t.name)) {
    if (t.args[0].name !== "String") throw new Error("Fixture JS dictionaries require string keys");
    return `${encode ? "put_js_dict" : "get_js_dict"} (fun v -> ${jsDictionaryType(t.args[1], "v", encode)}) (${v})`;
  }
  if (t.kind === "named" && t.name === "Vec")
    return `${encode ? "put_array" : "get_array"} (fun v -> ${jsDictionaryType(t.args[0], "v", encode)}) (${v})`;
  if (t.kind === "named" && ["String", "str"].includes(t.name))
    return `${encode ? "Js.Json.string" : "get_js_string"} (${v})`;
  if (t.kind === "named" && /^[ui](8|16|32)$/.test(t.name)) {
    const bits = Number(t.name.slice(1));
    const min = t.name[0] === "i" ? -(2 ** (bits - 1)) : 0;
    const max = t.name[0] === "i" ? 2 ** (bits - 1) - 1 : 2 ** bits - 1;
    return `${encode ? "Js.Json.number" : options["--strict"] ? `get_integer ${min}. ${max}.` : "get_number"} (${v})`;
  }
  return ty(t, v, encode);
}
let entries = Object.values(schema).filter(
  (e) => ["struct", "enum", "alias"].includes(e.kind) && !e.params.length,
);
if (options["--only"]) {
  const byName = new Map(entries.map((entry) => [entry.name, entry]));
  const needed = new Set();
  function include(name) {
    if (needed.has(name)) return;
    const entry = byName.get(name);
    if (!entry) throw new Error(`Unknown JSON codec root: ${name}`);
    needed.add(name);
    function visit(value) {
      if (!value || typeof value !== "object") return;
      if (value.kind === "named" && byName.has(value.name)) include(value.name);
      for (const child of Object.values(value)) visit(child);
    }
    visit(entry);
  }
  options["--only"].split(",").forEach(include);
  entries = entries.filter((entry) => needed.has(entry.name));
}
let output = `(* Generated JSON boundary codecs. No JS knowledge of OCaml representations. *)
[@@@warning "-30-39"]
open ${typeModule}
external has_property : Js.Json.t -> string -> bool = "hasOwn" [@@mel.scope "Object"]
external dict_create : unit Js.Nullable.t -> 'a Js.Dict.t = "create" [@@mel.scope "Object"]
external object_from_entries : (string * Js.Json.t) array -> Js.Json.t = "fromEntries" [@@mel.scope "Object"]
let get_string v = match Js.Json.decodeString v with Some x->Utf8.of_js x|None->invalid_arg "fixture string"
let put_string v = Js.Json.string (Utf8.to_js v)
let get_js_string v = match Js.Json.decodeString v with Some x->x|None->invalid_arg "fixture JS string"
let get_js_dict f v = match Js.Json.decodeObject v with Some values -> let result = dict_create Js.Nullable.null in Array.iter (fun (key,value) -> Js.Dict.set result key (f value)) (Js.Dict.entries values); result | None -> invalid_arg "fixture JS dictionary"
let put_js_dict f values = object_from_entries (Array.map (fun (key,value) -> (key,f value)) (Js.Dict.entries values))
let get_number v = match Js.Json.decodeNumber v with Some x->x|None->invalid_arg "fixture number"
let get_integer minimum maximum v = let n = get_number v in if floor n <> n || n < minimum || n > maximum then invalid_arg "Invalid historical integer" else n
let get_bool v = match Js.Json.decodeBoolean v with Some x->x|None->invalid_arg "fixture bool"
let list_to_array value = Array.of_list value
let list_of_array value = Array.to_list value
let sequence_to_array value = value
let sequence_of_array value = value
let get_array f v = match Js.Json.decodeArray v with Some a->Array.map f a|None->invalid_arg "fixture array"
let put_array f values = Js.Json.array (Array.map f values)
let get_list f v = match Js.Json.decodeArray v with Some a->Array.to_list (Array.map f a)|None->invalid_arg "fixture array"
let put_list f values = Js.Json.array (Array.of_list (List.map f values))
let get_option f v = if Js.Json.classify v=Js.Json.JSONNull then None else Some(f v)
let put_option f = function None->Js.Json.null|Some v->f v
let field v key = match Js.Json.decodeObject v with Some d->(match Js.Dict.get d key with Some value when not (Js.Types.test value Js.Types.Undefined) -> value | _ -> Js.Json.null)|None->invalid_arg "fixture object"
let has v key = has_property v key && (match Js.Json.decodeObject v with Some d -> (match Js.Dict.get d key with Some value -> not (Js.Types.test value Js.Types.Undefined) | None -> false) | None -> false)
let obj fields = object_from_entries (Array.of_list fields)
let get_map k f v = match Js.Json.decodeObject v with Some d->Array.to_list (Array.map(fun (key,v)->k(Js.Json.string key),f v)(Js.Dict.entries d))|None->invalid_arg "fixture map"
let put_map k f v = obj (List.map(fun (key,v)->Utf8.to_js(get_string(k key)),f v)v)
`;
for (let n = 2; n <= 4; n++) {
  const ns = Array.from({ length: n }, (_, i) => i);
  output += `let get_tuple_${n} ${ns.map((i) => "f" + i).join(" ")} v = let a=match Js.Json.decodeArray v with Some a when Array.length a=${n}->a|_->invalid_arg "fixture tuple" in (${ns.map((i) => `f${i} a.(${i})`).join(",")})\n`;
  output += `let put_tuple_${n} ${ns.map((i) => "f" + i).join(" ")} (${ns.map((i) => "v" + i).join(",")}) = Js.Json.array [|${ns.map((i) => `f${i} v${i}`).join(";")}|]\n`;
}
function zero(t) {
  if (["Box", "Rc", "Arc"].includes(t.name)) return zero(t.args[0]);
  if (t.kind === "array") return Array.from({ length: t.len }, () => zero(t.inner));
  if (t.kind === "tuple") return t.args.map(zero);
  if (t.name === "Option") return null;
  if (t.name === "Vec") return [];
  if (t.name === "HashMap" || t.name === "BTreeMap") return {};
  if (t.name === "String") return "";
  if (t.name === "bool") return false;
  if (!schema[t.name]) return 0;
  const e = schema[t.name];
  if (e.kind === "alias") return zero(e.ty);
  if (e.transparent) return zero(e.body.fields[0].ty);
  if (e.kind === "enum") return renamed(e.variants[0].name, e.variants[0].serde, e.serde, true);
  return Object.fromEntries(
    e.body.fields.map((f) => [renamed(f.name, f.serde, e.serde), zero(f.ty)]),
  );
}
function body(e, b, encode, v, parent = e.serde) {
  if (b.style === "unit") return encode ? "Js.Json.null" : "()";
  if (e.transparent) return ty(b.fields[0].ty, v, encode);
  if (b.style === "tuple") return ty({ kind: "tuple", args: b.fields.map((f) => f.ty) }, v, encode);
  const fields = b.fields.map((f) => {
    const name = renamed(f.name, f.serde, parent),
      access = `field ${v} ${quote(name)}`;
    if (encode) {
      if (/\bskip\b/.test(f.serde)) return "[]";
      const value = `${v}.${label(f.name)}`;
      if (f.serde.includes("flatten"))
        return `(Js.Dict.entries (match Js.Json.decodeObject (${ty(f.ty, value, true)}) with Some d->d|None->assert false)|>Array.to_list)`;
      const pair = `[${quote(name)},${ty(f.ty, value, true)}]`;
      if (f.serde.includes("Option::is_none")) return `(if ${value}=None then [] else ${pair})`;
      if (f.serde.includes("Vec::is_empty"))
        return `(if Array.length ${value}=0 then [] else ${pair})`;
      return pair;
    }
    let read = ty(f.ty, f.serde.includes("flatten") ? v : access, false);
    if (f.serde.includes("present_value"))
      return `${label(f.name)}=(if has ${v} ${quote(name)} then Some (${ty(f.ty.args[0], access, false)}) else None)`;
    if (f.serde.includes("default") || /\bskip\b/.test(f.serde)) {
      let def =
        f.ty.name === "String"
          ? '""'
          : f.ty.name === "bool"
            ? "false"
            : f.ty.name === "Vec"
              ? "[||]"
              : f.ty.name === "Option"
                ? "None"
                : f.serde.includes("one_attack")
                  ? "1"
                  : ty(f.ty, `Js.Json.parseExn ${quote(JSON.stringify(zero(f.ty)))}`, false);
      read = `(if Js.Json.classify (${access})=Js.Json.JSONNull then ${def} else ${read})`;
    }
    return `${label(f.name)}=${read}`;
  });
  return encode ? `obj (${fields.join(" @ ") || "[]"})` : `{${fields.join(";")}}`;
}
for (const encode of [false, true]) {
  let first = true;
  for (const e of entries) {
    output += `${first ? "let rec" : "and"} ${encode ? "to" : "of"}_${snake(e.name)} (v : ${encode ? nativeType(e) : "Js.Json.t"}) : ${encode ? "Js.Json.t" : nativeType(e)} =\n`;
    first = false;
    if (e.name === "WeaponSound") {
      output += encode
        ? "put_string (Enum.string_of_weapon_sound v)\n"
        : "Enum.weapon_sound_of_string (get_string v)\n";
      continue;
    }
    if (e.kind === "alias") {
      output +=
        (e.js_value && e.js_selector === "dict"
          ? jsDictionaryType(e.ty, "v", encode)
          : ty(e.ty, "v", encode)) + "\n";
      continue;
    }
    if (e.kind === "struct") {
      output += body(e, e.body, encode, "v") + "\n";
      continue;
    }
    const tag = attr(e.serde, "tag");
    if (encode) {
      output += "match v with\n";
      for (const variant of e.variants) {
        const constructor = enumConstructor(e, variant),
          name = renamed(variant.name, variant.serde, e.serde, true),
          b = variant.body;
        const fields = b.fields.map((f, i) => (b.style === "named" ? label(f.name) : "v" + i));
        const pattern =
          b.style === "unit"
            ? constructor
            : b.style === "named"
              ? `${constructor} {${fields.join(";")}}`
              : `${constructor} (${fields.join(",")})`;
        let value;
        if (b.style === "unit")
          value = e.serde.includes("untagged")
            ? encoded
            : tag
              ? `obj [${quote(tag)},Js.Json.string ${quote(name)}]`
              : `Js.Json.string ${quote(name)}`;
        else {
          const fake = { ...e, transparent: false };
          const shape = b.style === "named" ? `{${fields.join(";")}}` : `(${fields.join(",")})`;
          // Inline variant records cannot escape, so encode their named fields here.
          const encoded =
            b.style === "named"
              ? `obj (${b.fields
                  .map((f) => {
                    const value = label(f.name);
                    const pair = `[${quote(renamed(f.name, f.serde, ""))},${ty(f.ty, value, true)}]`;
                    if (/\bskip\b/.test(f.serde)) return "[]";
                    if (f.serde.includes("flatten"))
                      return `(Js.Dict.entries (match Js.Json.decodeObject (${ty(f.ty, value, true)}) with Some d->d|None->assert false)|>Array.to_list)`;
                    if (f.serde.includes("Option::is_none"))
                      return `(if ${value}=None then [] else ${pair})`;
                    return pair;
                  })
                  .join(" @ ")})`
              : b.fields.length === 1
                ? ty(b.fields[0].ty, fields[0], true)
                : body(fake, b, true, shape);
          value = e.serde.includes("untagged")
            ? encoded
            : tag
              ? `obj ((${quote(tag)},Js.Json.string ${quote(name)})::(Js.Dict.entries (match Js.Json.decodeObject (${encoded}) with Some d->d|None->assert false)|>Array.to_list))`
              : `obj [${quote(name)},${encoded}]`;
        }
        output += `| ${pattern} -> ${value}\n`;
      }
    } else if (e.serde.includes("untagged")) {
      output +=
        e.variants
          .map((variant, index) => {
            const constructor = enumConstructor(e, variant);
            const b = variant.body;
            const decode =
              b.style === "named"
                ? `${constructor} ${body({ ...e, transparent: false }, b, false, "v", "")}`
                : `${constructor} (${b.fields.map((f) => ty(f.ty, "v", false)).join(",")})`;
            return index < e.variants.length - 1 ? `try ${decode} with _ -> ` : decode;
          })
          .join("") + "\n";
    } else {
      output += `match get_string (${tag ? `field v ${quote(tag)}` : '(match Js.Json.decodeString v with Some _->v|None-> let d=match Js.Json.decodeObject v with Some d->d|None->invalid_arg "fixture variant" in Js.Json.string (Js.Dict.keys d).(0))'}) with\n`;
      for (const variant of e.variants) {
        const name = renamed(variant.name, variant.serde, e.serde, true),
          constructor = enumConstructor(e, variant),
          b = variant.body;
        output += `| ${quote(name)} -> `;
        if (b.style === "unit") output += constructor;
        else if (b.style === "named")
          output += `${constructor} ${body({ ...e, transparent: false }, b, false, tag ? "v" : `(field v ${quote(name)})`, "")}`;
        else
          output += `${constructor} (${b.fields.map((f) => ty(f.ty, `field v ${quote(name)}`, false)).join(",")})`;
        output += "\n";
      }
      output += '| _ -> invalid_arg "unknown fixture variant"\n';
    }
  }
}
output +=
  '\nlet is_ok = function Ok _ -> true | Error _ -> false\nlet result_error = function Error error -> error | Ok _ -> invalid_arg "fixture error"\nlet ok value = Ok value\nlet error value = Error value\nlet value = function Ok value -> value | Error _ -> invalid_arg \"fixture result\"\n';
for (const [key, module] of mapKeys)
  output +=
    module === "Hashtbl"
      ? `let bindings_${key}_map value = Array.of_seq (Hashtbl.to_seq value)\n`
      : `let bindings_${key}_map value = Array.of_list (${module}.bindings value)\n`;
if (options["--utf8"]) output = output.replaceAll("Utf8.", options["--utf8"] + ".");
const outputPath = options["--out"];
if (!existsSync(outputPath) || readFileSync(outputPath, "utf8") !== output)
  writeFileSync(outputPath, output);
