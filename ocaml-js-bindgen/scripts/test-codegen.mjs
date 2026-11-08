import assert from "node:assert/strict";
import {
  readFileSync,
  existsSync,
  writeFileSync,
  mkdtempSync,
  rmSync,
  mkdirSync,
} from "node:fs";
import { resolve } from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
const root = resolve(import.meta.dirname, "..");
const execute = promisify(execFile);
async function compile(command, args, options) {
  try {
    return { status: 0, ...(await execute(command, args, options)) };
  } catch (error) {
    return {
      status: typeof error.code === "number" ? error.code : null,
      stdout: error.stdout ?? "",
      stderr: error.stderr ?? "",
      error: typeof error.code === "number" ? undefined : error,
    };
  }
}
const n = (name, args = []) => ({ kind: "named", name, args });
const st = (name, params, fields) => ({
  kind: "struct",
  name,
  params,
  body: { style: "named", fields: fields.map(([name, ty]) => ({ name, ty })) },
});
function leb(n) {
  const bytes = [];
  do {
    const byte = n & 127;
    n >>>= 7;
    bytes.push(byte | (n ? 128 : 0));
  } while (n);
  return bytes;
}
mkdirSync(resolve(root, "tmp"), { recursive: true });
const base = readFileSync(
  resolve(
    root,
    "demos/calculator/rust/target/wasm32-unknown-unknown/release/calculator.wasm",
  ),
);
const compiler = process.env.BSC ?? resolve(root, "node_modules/.bin/bsc");
const cases = [
  [
    "recursive direct alias",
    false,
    [{ kind: "alias", name: "AliasCycle", params: [], ty: n("AliasCycle") }],
  ],
  [
    "recursive indirect alias through Box",
    false,
    [
      {
        kind: "alias",
        name: "AliasCycleA",
        params: [],
        ty: n("Box", [n("AliasCycleB")]),
      },
      { kind: "alias", name: "AliasCycleB", params: [], ty: n("AliasCycleA") },
    ],
  ],
  [
    "recursive generic alias through nominal argument",
    false,
    [
      st("AliasCarrier", ["T"], [["value", n("T")]]),
      {
        kind: "alias",
        name: "AliasCycle",
        params: ["T"],
        ty: n("AliasCarrier", [n("AliasCycle", [n("T")])]),
      },
    ],
  ],
  [
    "alias recursion through nominal body",
    true,
    [
      { kind: "alias", name: "RecursiveAlias", params: [], ty: n("AliasNode") },
      st("AliasNode", [], [["children", n("Vec", [n("RecursiveAlias")])]]),
    ],
  ],
  [
    "former instance type name",
    true,
    [st("Instance", [], [["value", n("u32")]])],
  ],
  [
    "colliding key module names",
    true,
    [
      {
        ...st(
          "CollisionKey",
          ["T", "U"],
          [
            ["first", n("T")],
            ["second", n("U")],
          ],
        ),
        key_eq: true,
        key_ord: true,
      },
      {
        ...st("CollisionKeyU32", ["T"], [["value", n("T")]]),
        key_eq: true,
        key_ord: true,
      },
      st(
        "KeyMaps",
        [],
        [
          [
            "first",
            n("BTreeMap", [n("CollisionKey", [n("u32"), n("u64")]), n("u8")]),
          ],
          [
            "second",
            n("BTreeMap", [n("CollisionKeyU32", [n("u64")]), n("u8")]),
          ],
        ],
      ),
    ],
  ],
  ...["ExplicitKey", "ExplicitContractKey"].map((key) => [
    `explicit discriminant key ${key}`,
    false,
    [st("ExplicitMap", [], [["entries", n("BTreeMap", [n(key), n("u32")])]])],
  ]),
  [
    "empty enum map key",
    true,
    [
      {
        kind: "enum",
        name: "NeverKey",
        params: [],
        key_eq: true,
        key_ord: true,
        variants: [],
      },
      st(
        "NeverMap",
        [],
        [["entries", n("BTreeMap", [n("NeverKey"), n("u32")])]],
      ),
    ],
  ],
  [
    "generic map key",
    false,
    [
      st(
        "GenericMap",
        ["Key"],
        [["entries", n("BTreeMap", [n("Key"), n("String")])]],
      ),
    ],
  ],
  [
    "unverified map key",
    false,
    [
      st("UnverifiedKey", [], [["value", n("u32")]]),
      st(
        "UnverifiedMap",
        [],
        [["entries", n("HashMap", [n("UnverifiedKey"), n("u32")])]],
      ),
    ],
  ],
  [
    "float key",
    false,
    [st("FloatMap", [], [["entries", n("BTreeMap", [n("f64"), n("u32")])]])],
  ],
  [
    "map dictionary",
    true,
    [
      st(
        "Dictionary",
        [],
        [
          [
            "entries",
            {
              ...n("BTreeMap", [n("String"), n("Option", [n("u32")])]),
              map: "dict",
            },
          ],
        ],
      ),
    ],
  ],
  [
    "map dictionary nonstring",
    false,
    [
      st(
        "BadDictionary",
        [],
        [
          [
            "entries",
            { ...n("HashMap", [n("u32"), n("String")]), map: "dict" },
          ],
        ],
      ),
    ],
  ],
  ["uppercase field", false, [st("BadField", [], [["Value", n("u32")]])]],
  ["native type collision", false, [st("Float", [], [["value", n("u32")]])]],
  ["escaped keyword", true, [st("HasKeyword", [], [["or", n("u32")]])]],
  [
    "uppercase function",
    false,
    [{ kind: "function", name: "Compute", params: [], result: n("u32") }],
  ],
  [
    "enum names do not reserve conversion helpers",
    true,
    ["COfString", "StringOfC"].map((name) => ({
      kind: "enum",
      name,
      params: [],
      variants: [{ name: "Value", body: { style: "unit", fields: [] } }],
    })),
  ],
  [
    "qualified generic",
    true,
    [
      st("GlobalThing", [], [["x", n("u32")]]),
      st(
        "ExtraGeneric",
        ["GlobalThing"],
        [
          ["generic", n("GlobalThing")],
          ["concrete", { ...n("GlobalThing"), qualified: true }],
        ],
      ),
    ],
  ],
  [
    "polymorphic codecs",
    true,
    [
      st("ExtraWrapper", ["Item"], [["value", n("Item")]]),
      st(
        "Uses",
        [],
        [
          ["number", n("ExtraWrapper", [n("u8")])],
          ["string", n("ExtraWrapper", [n("String")])],
        ],
      ),
    ],
  ],
  [
    "empty enum",
    true,
    [{ kind: "enum", name: "Never", params: [], variants: [] }],
  ],
  [
    "empty tuple",
    true,
    [
      {
        kind: "struct",
        name: "ExtraEmpty",
        params: [],
        body: { style: "tuple", fields: [] },
      },
    ],
  ],
  [
    "enum helper names",
    true,
    [
      {
        kind: "enum",
        name: "W",
        params: [],
        variants: [{ name: "U8", body: { style: "unit", fields: [] } }],
      },
    ],
  ],
  [
    "opaque keyword",
    true,
    [
      { kind: "opaque", name: "Type", params: [] },
      {
        kind: "function",
        name: "qualified_bool",
        module: "Type",
        member: "make",
        params: [{ name: "value", ty: n("bool"), mode: "value" }],
        result: n("Type"),
      },
    ],
    true,
  ],
  [
    "builtin generic",
    false,
    [st("ExtraGeneric", ["String"], [["value", n("String")]])],
  ],
  [
    "generic collision",
    false,
    [
      st(
        "ExtraGeneric",
        ["Type", "Type_"],
        [
          ["a", n("Type")],
          ["b", n("Type_")],
        ],
      ),
    ],
  ],
  [
    "normalized name collision",
    false,
    [
      st("ExtraHTTPServer", [], [["x", n("u32")]]),
      st("ExtraHttpServer", [], [["x", n("u32")]]),
    ],
  ],
  ["missing ref type", false, [st("BadRef", [], [["value", { kind: "ref" }]])]],
  [
    "bad primitive arity",
    false,
    [st("BadArity", [], [["value", n("u8", [n("String")])]])],
  ],
  [
    "expanding generic recursion",
    false,
    [
      st(
        "Expander",
        ["T"],
        [["next", n("Box", [n("Expander", [n("Vec", [n("T")])])])]],
      ),
      {
        kind: "alias",
        name: "Expand",
        params: [],
        ty: n("Rc", [n("Expander", [n("u8")])]),
      },
    ],
  ],
  [
    "hidden Rc opaque",
    false,
    [
      { kind: "opaque", name: "Object", params: [] },
      st("Hidden", ["T"], [["value", n("T")]]),
      {
        kind: "alias",
        name: "Alias",
        params: [],
        ty: n("Rc", [n("Hidden", [n("Object")])]),
      },
    ],
  ],
];
const fn = (ty, mode = "value") => ({
  kind: "function",
  name: "unsupported_signature",
  params: [{ name: "value", ty, mode }],
  result: n("u32"),
  asynchronous: true,
});
const ref = (inner) => ({ kind: "ref", mutable: false, inner });
cases.push(
  ["move primitive", false, [fn(n("u32"), "move")]],
  ["borrowed callback payload", false, [fn(n("Callback", [ref(n("String"))]))]],
  [
    "tuple callback payload",
    false,
    [fn(n("Callback", [{ kind: "tuple", args: [n("u32")] }]))],
  ],
  [
    "opaque callback payload",
    false,
    [
      { kind: "opaque", name: "Object", params: [] },
      fn(n("Callback", [n("Object")])),
    ],
  ],
  [
    "generic callback payload",
    false,
    [
      st("Event", ["T"], [["value", n("T")]]),
      fn(n("Callback", [n("Event", [n("u32")])])),
    ],
  ],
  [
    "task type collision",
    false,
    [st("OcamljsTask", [], [["value", n("u32")]])],
  ],
);

const api = JSON.parse(
  readFileSync(
    resolve(root, "demos/calculator/generated/melange/schema.json"),
    "utf8",
  ),
);
cases.push(
  ...[
    [
      "opaque T preserves qualified map types",
      true,
      [
        { kind: "opaque", name: "T", params: [] },
        { ...api.qualified_bool, module: "T", member: "make", result: n("T") },
      ],
    ],
    [
      "resource and map module name collision",
      true,
      [
        { kind: "opaque", name: "String_map", params: [] },
        {
          ...api.qualified_bool,
          module: "String_map",
          member: "make",
          result: n("String_map"),
        },
      ],
    ],
    [
      "opaque without local creation",
      true,
      [{ kind: "opaque", name: "ForeignToken", params: [] }],
    ],
    [
      "opaque producer keeps its name",
      true,
      [{ ...api.counter, member: "parse" }],
    ],
    [
      "constructor metadata rejected",
      false,
      [{ ...api.Counter, constructor: "current" }],
    ],
    ["unassociated opaque input", false, [{ ...api.current, module: "" }]],
    [
      "unassociated nested opaque result",
      false,
      [{ ...api.many_counters, module: "" }],
    ],
    [
      "duplicate module operation",
      false,
      [{ ...api.current, member: "create" }],
    ],
    [
      "reserved disposal operation",
      false,
      [{ ...api.current, member: "dispose" }],
    ],
    ["unknown module", false, [{ ...api.current, module: "Missing" }]],
  ].map(([name, valid, entries]) => [name, valid, entries, true]),
);
const namespace = (name, ty) => ({
  kind: "module",
  name: `OcamljsModule__${name}`,
  member: name,
  rust_path: name.toLowerCase(),
  ...(ty ? { ty: n(ty) } : {}),
});
cases.push(
  ["ordinary empty module", true, [namespace("Utilities")]],
  [
    "ordinary type-only module",
    true,
    [
      namespace("Labels2"),
      {
        ...st("Labels2__Label", ["T"], [["value", n("T")]]),
        module: "Labels2",
        member: "Label",
      },
    ],
  ],
  [
    "ordinary optional t without constructor",
    true,
    [
      namespace("Coordinates", "Coordinates__Point"),
      {
        ...st("Coordinates__Point", [], [["x", n("f64")]]),
        module: "Coordinates",
        member: "Point",
      },
    ],
  ],
  [
    "ordinary enum t without constructor",
    true,
    [
      namespace("Directions", "Directions__Axis"),
      {
        kind: "enum",
        name: "Directions__Axis",
        module: "Directions",
        member: "Axis",
        variants: [{ name: "X", body: { style: "unit", fields: [] } }],
      },
    ],
  ],
  [
    "ordinary recursive types",
    true,
    [
      namespace("Trees"),
      {
        ...st(
          "Trees__Branch",
          [],
          [["leaf", n("Option", [n("Box", [n("Trees__Leaf")])])]],
        ),
        module: "Trees",
        member: "Branch",
      },
      {
        ...st(
          "Trees__Leaf",
          [],
          [["branch", n("Option", [n("Box", [n("Trees__Branch")])])]],
        ),
        module: "Trees",
        member: "Leaf",
      },
    ],
  ],
  [
    "ordinary modules reference each other",
    true,
    [
      namespace("Left"),
      namespace("Right"),
      {
        ...st(
          "Left__Node",
          [],
          [["right", n("Option", [n("Box", [n("Right__Node")])])]],
        ),
        module: "Left",
        member: "Node",
      },
      {
        ...st(
          "Right__Node",
          [],
          [["left", n("Option", [n("Box", [n("Left__Node")])])]],
        ),
        module: "Right",
        member: "Node",
      },
    ],
  ],
  [
    "ordinary type referenced by root record",
    true,
    [
      namespace("Coordinates"),
      {
        ...st("Coordinates__Point", [], [["x", n("f64")]]),
        module: "Coordinates",
        member: "Point",
      },
      st("Position", [], [["point", n("Coordinates__Point")]]),
    ],
  ],
  [
    "ordinary selected type missing",
    false,
    [namespace("Coordinates", "Missing")],
  ],
  [
    "ordinary selected type outside namespace",
    false,
    [
      namespace("Coordinates", "Unrelated"),
      st("Unrelated", [], [["x", n("f64")]]),
    ],
  ],
  ["ordinary reserved module name", false, [namespace("Wasm")]],
  ["ordinary lowercase module name", false, [namespace("utility")]],
  [
    "ordinary unregistered namespace",
    false,
    [
      {
        ...st("Coordinates__Point", [], [["x", n("f64")]]),
        module: "Coordinates",
        member: "Point",
      },
    ],
  ],
  [
    "ordinary duplicate local type name",
    false,
    [
      namespace("Coordinates", "Coordinates__Point"),
      {
        ...st("Coordinates__Point", [], [["x", n("f64")]]),
        module: "Coordinates",
        member: "Point",
      },
      {
        ...st("Coordinates__T", [], [["x", n("f64")]]),
        module: "Coordinates",
        member: "T",
      },
    ],
  ],
);
cases.push(
  [
    "ordinary generic t",
    true,
    [
      namespace("Labels3", "Labels3__Label"),
      {
        ...st("Labels3__Label", ["T"], [["value", n("T")]]),
        module: "Labels3",
        member: "Label",
      },
    ],
  ],
  [
    "ordinary alias t",
    true,
    [
      namespace("Numbers", "Numbers__Number"),
      {
        kind: "alias",
        name: "Numbers__Number",
        module: "Numbers",
        member: "Number",
        ty: n("f64"),
      },
    ],
  ],
  [
    "ordinary local built-in collision",
    false,
    [
      namespace("Coordinates2"),
      {
        ...st("Coordinates2__Float", [], [["x", n("f64")]]),
        module: "Coordinates2",
        member: "Float",
      },
    ],
  ],
  [
    "ordinary function-only namespace",
    true,
    [namespace("Functions2"), { ...api.add, module: "Functions2" }],
    true,
  ],
  ["ordinary resource module collision", false, [namespace("Counter")], true],
  [
    "ordinary explicit conversion name",
    true,
    [{ ...api.qualified_bool, module: "Direction", member: "string_of_t" }],
    true,
  ],
);
cases.push([
  "ordinary field labels survive resource T",
  true,
  [
    { kind: "opaque", name: "T", params: [] },
    { ...api.qualified_bool, module: "T", member: "make", result: n("T") },
    namespace("Fields"),
    {
      ...st("Fields__Fields", [], [["t", n("u8")]]),
      module: "Fields",
      member: "Fields",
    },
  ],
  true,
]);
for (const [operation, params] of [
  ["getter", []],
  ["setter", []],
  ["call", []],
  ["indexing_getter", []],
  ["method", []],
]) {
  cases.push([
    `invalid host ${operation} receiver`,
    false,
    [
      {
        kind: "import",
        name: "__ocamljs_host_bad",
        js_name: "bad",
        operation,
        params,
        result: n("u32"),
      },
    ],
  ]);
}
cases.push([
  "nested native promises flatten",
  false,
  [
    {
      kind: "alias",
      name: "NestedPromise",
      ty: n("Promise", [n("Promise", [n("HostError")])]),
      params: [],
    },
  ],
]);
cases.push([
  "nested native promise import",
  false,
  [
    {
      kind: "import",
      name: "__ocamljs_host_nested",
      js_name: "nested",
      operation: "function",
      params: [],
      result: n("Promise", [n("Promise", [n("HostError")])]),
    },
  ],
]);
function withoutSchema(bytes) {
  const sections = [bytes.subarray(0, 8)];
  let at = 8;
  const unsigned = () => {
    let n = 0,
      shift = 0,
      b;
    do {
      b = bytes[at++];
      n |= (b & 127) << shift;
      shift += 7;
    } while (b & 128);
    return n >>> 0;
  };
  while (at < bytes.length) {
    const start = at,
      id = bytes[at++],
      length = unsigned(),
      end = at + length;
    let name;
    if (id === 0) {
      const size = unsigned();
      name = bytes.subarray(at, at + size).toString();
    }
    if (name !== "ocamljs.schema") sections.push(bytes.subarray(start, end));
    at = end;
  }
  return Buffer.concat(sections);
}

const docText =
  'Documentation keeps Counter.t and Geometry.point literally, "quotes", (* comment markers *), \\ paths, λ🦀 and \u0000 bytes.';
cases.push([
  "documentation text cannot alter declarations",
  true,
  [
    { ...api.Counter, docs: docText },
    { ...api.current, docs: docText },
    {
      ...api.Calculation,
      docs: docText,
      body: {
        ...api.Calculation.body,
        fields: api.Calculation.body.fields.map((f) => ({
          ...f,
          docs: docText,
        })),
      },
    },
    {
      ...api.CalculationError,
      docs: docText,
      variants: api.CalculationError.variants.map((v) => ({
        ...v,
        docs: docText,
      })),
    },
  ],
  true,
]);

const variant = (name, fields = []) => ({
  name,
  body: {
    style: fields.length ? "tuple" : "unit",
    fields: fields.map((ty, index) => ({ name: String(index), ty })),
  },
});
cases.push(
  [
    "enum namespace client",
    true,
    [
      {
        kind: "enum",
        name: "EnumContract",
        params: [],
        variants: [
          variant("Empty"),
          {
            name: "Payload",
            body: { style: "named", fields: [{ name: "value", ty: n("f64") }] },
          },
        ],
      },
    ],
  ],
  [
    "enum record recursion",
    true,
    [
      {
        kind: "enum",
        name: "Rose",
        params: ["T"],
        variants: [
          variant("Empty"),
          variant("Branch", [n("Box", [n("RoseBranch", [n("T")])])]),
        ],
      },
      st(
        "RoseBranch",
        ["T"],
        [
          ["value", n("T")],
          ["children", n("Vec", [n("Rose", [n("T")])])],
        ],
      ),
    ],
  ],
  [
    "enum constructors do not shadow builtins",
    true,
    [
      {
        kind: "enum",
        name: "FirstChoice",
        params: ["T"],
        variants: [
          variant("None"),
          variant("Some", [
            n("T"),
            n("Box", [n("Option", [n("OtherChoice", [n("T")])])]),
          ]),
        ],
      },
      {
        kind: "enum",
        name: "OtherChoice",
        params: ["T"],
        variants: [
          variant("None"),
          variant("Some", [n("Vec", [n("FirstChoice", [n("T")])])]),
        ],
      },
      {
        // Compiler-only fixture: reuse an existing framed export to reach
        // both codecs; this altered schema is never executed against Rust.
        ...api.generic_roundtrip,
        params: [
          { name: "value", ty: n("FirstChoice", [n("u32")]) },
          api.generic_roundtrip.params[1],
        ],
        result: n("FirstChoice", [n("u32")]),
      },
    ],
    true,
  ],
  [
    "enum namespace collision",
    false,
    [
      namespace("Enum_conflict"),
      { kind: "enum", name: "EnumConflict", variants: [variant("Empty")] },
    ],
  ],
);

async function checkCase([name, valid, edits, replace]) {
  const entries = replace
    ? Object.values({
        ...api,
        ...Object.fromEntries(edits.map((e) => [e.name, e])),
      })
    : edits;
  const dir = mkdtempSync(resolve(root, "tmp/codegen-"));
  try {
    const payload = Buffer.concat([
      Buffer.from([14]),
      Buffer.from("ocamljs.schema"),
      Buffer.from(entries.map((e) => JSON.stringify(e)).join("\n") + "\n"),
    ]);
    const input = resolve(dir, "input.wasm");
    writeFileSync(
      input,
      Buffer.concat([
        replace ? withoutSchema(base) : base,
        Buffer.from([0]),
        Buffer.from(leb(payload.length)),
        payload,
      ]),
    );
    const generated = await compile(
      resolve(root, "target/debug/ocamljs_pack"),
      [
        "generate",
        "--wasm",
        input,
        "--out-dir",
        dir,
        "--backend",
        "rescript",
        "--rescript-compiler",
        compiler,
      ],
      { cwd: root, encoding: "utf8", timeout: 5000 },
    );
    assert.equal(
      generated.status,
      valid ? 0 : 1,
      `${name}: ${generated.error ?? generated.stderr}`,
    );
    const diagnostics = {
      "recursive direct alias": /recursive type alias/,
      "recursive indirect alias through Box": /recursive type alias/,
      "recursive generic alias through nominal argument":
        /recursive type alias/,
      "move primitive": /consume requires a direct by-value opaque/,
      "borrowed callback payload": /Callback payload must be a concrete/,
      "tuple callback payload": /Callback payload must be a concrete/,
      "opaque callback payload": /Callback requires one owned structural/,
      "generic callback payload": /Callback payload must be a concrete/,
      "task type collision": /collides with an ABI built-in/,
    };
    if (diagnostics[name])
      assert.match(generated.stderr, diagnostics[name], name);
    if (name === "qualified generic") {
      const pack = resolve(root, "target/debug/ocamljs_pack");
      const args = [
        "generate",
        "--wasm",
        input,
        "--out-dir",
        dir,
        "--backend",
        "rescript",
        "--rescript-compiler",
        compiler,
      ];
      assert(!existsSync(resolve(dir, "bindings.worker.mjs")));
      assert(
        !readFileSync(resolve(dir, "Bindings.mli"), "utf8").includes(
          "module Remote",
        ),
      );
      const unsupported = await compile(pack, [...args, "--worker"], {
        cwd: root,
        encoding: "utf8",
        timeout: 5000,
      });
      assert.equal(unsupported.status, 1);
      assert.match(unsupported.stderr, /unknown option --worker/);
      const runtime = readFileSync(
        resolve(dir, "bindings.runtime.mjs"),
        "utf8",
      );
      assert(
        !/serveWorker|call_remote_|worker_load|remoteHandles/.test(runtime),
      );
    }
    if (name === "opaque without local creation") {
      const signature = readFileSync(resolve(dir, "Bindings.mli"), "utf8");
      const module = signature
        .split("module ForeignToken : sig")[1]
        ?.split("end")[0];
      assert.match(module, /type t[\s\S]*val dispose : t -> unit/);
      assert(!/val create/.test(module));
    }
    if (name === "opaque producer keeps its name") {
      const signature = readFileSync(resolve(dir, "Bindings.mli"), "utf8");
      const module = signature.split("Counter : sig")[1]?.split("end")[0];
      assert.match(module, /val parse : float -> t/);
      assert(!/val create/.test(module));
    }
    if (name === "documentation text cannot alter declarations") {
      const signature = readFileSync(resolve(dir, "Bindings.mli"), "utf8");
      assert(
        signature.includes(
          '[@@ocaml.doc "Documentation keeps Counter.t and Geometry.point literally',
        ),
      );
      assert(
        signature.includes(
          '[@ocaml.doc "Documentation keeps Counter.t and Geometry.point literally',
        ),
      );
      assert(signature.includes("\\000 bytes."));
      assert(signature.includes("Rust [Err] raises [Divide_error]"));
      assert(
        signature.includes(
          "Rust [Ok] and [Err] map to native OCaml [Ok] and [Error].",
        ),
      );
      assert(!signature.includes('ocaml.doc "\\n\\n'));
      assert(!signature.includes("[__ocamljs_self]"));
      assert(signature.includes("Runs Rust Drop and invalidates this value."));
    }
    if (valid) {
      const compiled = await compile(
        compiler,
        [
          "-I",
          resolve(root, "node_modules/rescript/lib/ocaml"),
          "-I",
          dir,
          "Bindings.res",
        ],
        { cwd: dir, encoding: "utf8", timeout: 5000 },
      );
      assert.equal(
        compiled.status,
        0,
        `${name}: ${compiled.error ?? compiled.stderr}`,
      );
      if (name === "enum namespace client") {
        const signature = readFileSync(resolve(dir, "Bindings.mli"), "utf8");
        assert(signature.includes("module Enum_contract : sig"));
        assert(!/^type enum_contract\b/m.test(signature));
        writeFileSync(
          resolve(dir, "Client.ml"),
          `
let construct () =
  let module Error = Bindings.Enum_contract in
  let value : Error.t = Error.Payload { value = 3. } in
  match value with
  | Error.Empty -> None
  | Error.Payload { value } -> Some value
`,
        );
        const formatted = await compile(
          compiler,
          ["-format", resolve(dir, "Client.ml")],
          { cwd: dir, encoding: "utf8" },
        );
        assert.equal(formatted.status, 0, formatted.stderr);
        writeFileSync(resolve(dir, "Client.res"), formatted.stdout);
        const client = await compile(
          compiler,
          [
            "-I",
            resolve(root, "node_modules/rescript/lib/ocaml"),
            "-I",
            dir,
            "Client.res",
          ],
          { cwd: dir, encoding: "utf8" },
        );
        assert.equal(client.status, 0, client.stderr);
      }
    }
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}
// Cases use separate temporary directories. Bound compiler concurrency so
// npm's portable compiler wrapper does not serialize its startup overhead.
// Two workers leave CPU headroom for each packager's compiler child and keep
// the existing per-command deadlines meaningful on shared build workers.
let nextCase = 0;
await Promise.all(
  Array.from({ length: 2 }, async () => {
    while (nextCase < cases.length) await checkCase(cases[nextCase++]);
  }),
);
console.log(
  `${cases.length} schema and compiler cases plus worker API removal regression passed`,
);
