import { hex, cellKey } from "./hex-fixture.mjs";
// Historical fixture spelling stays JSON. The compiled OCaml helper constructs
// and deconstructs native values; tests never manufacture compiler variants.
import * as Json from "@frontend/Fixture_json";
import * as Utf8 from "@frontend/Utf8";
import schema from "@wasm/schema.json" with { type: "json" };
const snake = (s) =>
  s
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toLowerCase();
const keyword = new Set([
  "type",
  "from",
  "default",
  "done",
  "end",
  "method",
  "object",
  "class",
  "open",
]);
const label = (s) => (keyword.has(s) ? s + "_" : s);
const cache = new WeakMap();
export const listToArray = (value) => Json.list_to_array(value);
export const listFromArray = (value) => Json.list_of_array(value);
export const toArray = (value) => Json.sequence_to_array(value);
export const fromArray = (value) => Json.sequence_of_array(value);
export const mapBindings = (key, value, kind = "BTreeMap") =>
  Json[`bindings_${kind === "HashMap" ? "hash_" : ""}${snake(key)}_map`](value);
const rename = (f, e) =>
  f.serde.match(/(?:^|[ (,])rename\s*=\s*"([^"]+)"/)?.[1] ??
  (e.serde.includes('rename_all = "camelCase"')
    ? f.name.replace(/_([a-z])/g, (_, c) => c.toUpperCase())
    : f.name);
function remember(t, n, j, js = false) {
  if (!j || typeof j !== "object") return j;
  if (!n || typeof n !== "object") return j;
  if (["Box", "Rc", "Arc"].includes(t.name)) return remember(t.args[0], n, j);
  const entry = schema[t.name];
  if (entry?.kind === "alias") return remember(entry.ty, n, j, Boolean(entry.js_value));
  if (entry?.transparent) return remember(entry.body.fields[0].ty, n, j);
  const previous = cache.get(n);
  if (previous) return previous;
  cache.set(n, j);
  if (t.kind === "array" || t.name === "Vec")
    toArray(n).forEach((value, i) => (j[i] = remember(t.inner ?? t.args[0], value, j[i], js)));
  else if (t.name === "Option") return remember(t.args[0], n, j);
  else if (t.name === "HashMap" || t.name === "BTreeMap")
    for (const [k, v] of js || t.map === "dict"
      ? Object.entries(n)
      : mapBindings(t.args[0].name, n, t.name)) {
      const key =
        t.args[0].name === "String"
          ? js
            ? k
            : Utf8.to_js(k)
          : (Json["to_" + snake(t.args[0].name)]?.(k) ?? k);
      if (Object.hasOwn(j, key)) j[key] = remember(t.args[1], v, j[key], js);
    }
  else {
    const e = schema[t.name];
    if (e?.body?.style === "named")
      for (const f of e.body.fields) {
        const k = rename(f, e);
        if (k in j) j[k] = remember(f.ty, n[label(f.name)], j[k]);
      }
  }
  return j;
}
export function toJSON(name, value) {
  if (value === undefined) return undefined;
  return (
    cache.get(value) ?? remember({ kind: "named", name }, value, Json["to_" + snake(name)](value))
  );
}
export const fromJSON = (name, value) =>
  value === undefined ? undefined : Json["of_" + snake(name)](value);
export function command(value) {
  const v = structuredClone(value);
  if (v.tile !== undefined) v.tile = hex(v.tile);
  for (const key of ["unit", "target"])
    if (typeof v[key] === "string" && /^u\d+$/.test(v[key])) v[key] = Number(v[key].slice(1));
  if (v.units) v.units = v.units.map((id) => (typeof id === "string" ? Number(id.slice(1)) : id));
  return fromJSON("Command", v);
}
