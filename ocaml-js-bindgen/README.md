# OCamlJS bindings for Rust/WASM

First-principles bindings for js_of_ocaml, Melange, and ReScript. The Rust and
JavaScript implementations contain no code or runtime dependency from
wasm-bindgen, tsify, or wasm-pack.

A [small runnable example](examples/rust-to-ocaml/README.md) includes the full
generated `Bindings.ml` and JavaScript runtime alongside its Rust and OCaml source.

The workspace provides:

| Crate                   | Responsibility                                                                                  |
| ----------------------- | ----------------------------------------------------------------------------------------------- |
| `ocamljs_bindgen`       | ABI runtime, checked opaque ownership and borrowing; reexports the export attribute             |
| `ocamljs_bindgen_macro` | Rust wrappers, codecs and embedded schema metadata                                              |
| `ocamljsify`            | Data attribute; optional `generator` feature supplies host-side validation and OCaml generation |
| `ocamljs_pack`          | Cargo/WASM build, schema extraction, validation and packaging CLI                               |

## Use

Declare a Rust `cdylib` and depend on `ocamljs_bindgen` and `ocamljsify`.
The examples use local paths; a source distribution can be made with
`node scripts/vendor.mjs /path/to/consumer/ocamljs`.

```rust
use ocamljs_bindgen::ocamljs_bindgen;

#[ocamljsify::ocamljsify]
pub enum ChangeError { EmptyTitle }

#[ocamljs_bindgen(opaque)]
pub struct Todo { titles: Vec<String> }

#[ocamljs_bindgen]
impl Todo {
    pub fn create() -> Self { Self { titles: Vec::new() } }

    pub fn add(&mut self, title: String) -> Result<(), ChangeError> {
        if title.is_empty() { return Err(ChangeError::EmptyTitle); }
        self.titles.push(title);
        Ok(())
    }
}
```

```sh
rustup target add wasm32-unknown-unknown
make install
cd demos/todo/rust
ocamljs_pack build --backend melange --out-dir ../pkg
```

Use `--backend js_of_ocaml`, `rescript`, or `all` for the other compilers.
ReScript output uses the installed compiler's syntax printer. The packager finds
`node_modules/.bin/bsc` in the current project or its ancestors, so commands run
inside `rust/` use that project's compiler too. Supply `BSC` or
`--rescript-compiler /path/to/bsc` to select another installation.
`generate --wasm input.wasm ...` packages an already-built Rust module.
To regenerate bindings from a release package, also pass `--schema schema.json`;
the packager validates the sidecar against the module's ABI stamp and exports.
`--dev` selects Cargo's debug profile and retains embedded schema, names,
producers and debug sections. Release packages remove that metadata, retain the
full schema in `schema.json`, and check a small ABI stamp when loading Wasm.
Loading needs neither browser cryptography nor schema JSON parsing. Builds use
release and `--locked` by default.

Output includes `Bindings.ml`, `Bindings.mli`, `bindings.runtime.mjs`,
`backend.wasm` and `schema.json`. ReScript packaging additionally writes
`Bindings.res` and `Bindings.resi`. Compile the binding with the selected OCaml
compiler and initialize Wasm from the application entry point. For js_of_ocaml:

```ocaml
module B = Bindings

let main () =
  let todo = B.Todo.create () in
  Fun.protect ~finally:(fun () -> B.Todo.dispose todo)
    (fun () ->
      match B.Todo.add todo "Ship the OCaml frontend" with
      | Ok () -> ()
      | Error B.Change_error_empty_title -> failwith "Enter a title")

let () = ignore (Js_of_ocaml.Promise.map main (B.Wasm.load "./backend.wasm"))
```

The demo Makefiles link the generated runtime with the compiled application using
esbuild injection and shared ESM chunks. HTML loads the resulting `frontend.mjs`;
application projects need no handwritten JavaScript loader.

Initialize each generated module once. Concurrent or repeated initialization
shares its existing load. Ordinary calls and type-scoped disposal use that module
directly, without an instance argument. For initialization from OCaml,
`Wasm.load url` returns `unit Js_of_ocaml.Promise.t` for js_of_ocaml and `unit Js.Promise.t` for
Melange and ReScript.
The executable examples show complete Dune and ReScript compiler setup.

## Modules

An annotated inline Rust module exports its annotated public types and functions
as an ordinary OCaml module. It does not need a `t` or a constructor:

```rust
#[ocamljs_bindgen::ocamljs_bindgen]
pub mod geometry {
    #[ocamljsify::ocamljsify]
    pub struct Point { pub x: f64, pub y: f64 }

    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn distance(a: Point, b: Point) -> f64 {
        (a.x - b.x).hypot(a.y - b.y)
    }
}
```

```ocaml
module G = Bindings.Geometry

let a : G.point = { x = 0.; y = 0. }
let b : G.point = { x = 3.; y = 4. }
let distance = G.distance a b
```

`Geometry.point` is a normal record defined in the public module signature, with
accessible fields. Variants support ordinary pattern matching; aliases, generic
data, type-only modules, function-only modules and empty modules also work.
Native type names are scoped: different modules may both define `Point`.
Cross-module signatures use paths such as `Geometry.point`. Annotated exports
must be public; unannotated helpers stay private to Rust.
Use explicit Rust paths such as `geometry::Point` in cross-module export
signatures so the schema preserves the type's namespace.

Use `#[ocamljs_bindgen::ocamljs_bindgen(t = Point)]` on the Rust module to name
that type `Geometry.t` instead. This works with ordinary records, variants and
aliases and does not require a constructor. The old `main` option is rejected.
No constructor or disposal operation is inferred for native data.

### Resource modules

An opaque struct generates a module containing abstract `t` and `dispose`, which
invokes normal Rust field destruction and any `Drop` implementation. It needs no
creation function. An annotated inherent `impl` exports its public methods and
associated functions under that module. A consuming `self` method transfers
ownership automatically. Private methods remain Rust implementation.

An inline Rust module can instead declare `opaque_t = Universe`. This both names
the public type `t` and marks the selected concrete struct opaque; no separate
attribute is needed on that struct. The module is named after the Rust module
(`mod universe` becomes `Universe`). Other annotated types and functions are
also exported inside it; see the [Game of Life source](demos/game-of-life/rust/src/lib.rs).

```rust
#[ocamljs_bindgen::ocamljs_bindgen(opaque_t = Point)]
pub mod point {
    pub struct Point { pub(crate) x: f64, pub(crate) y: f64 }

    #[ocamljs_bindgen::ocamljs_bindgen]
    pub fn distance(a: &Point, b: &Point) -> f64 {
        (a.x - b.x).hypot(a.y - b.y)
    }
}
```

Another exported module may produce `Point.t`, directly or inside `Result`,
`Option` or a collection. Ownership and Rust drop glue follow the concrete type,
independently of which function produced it. Functions retain their declared
names, with normal keyword escaping; `name = "create"` is an ordinary explicit
rename when desired. There is no constructor designation or automatic renaming.
The obsolete `constructor` option and schema metadata are rejected.
[Todo](demos/todo/rust/src/lib.rs) demonstrates the struct/impl form.
Existing free Rust functions can explicitly select `module = Type`; `name = "operation"`
selects a member name. This is useful when binding an existing Rust module without
rewriting its native callers, as in Hex's factory. Unassociated free exports
that accept or return opaque values, including nested owners, are rejected.

Ordinary opaque types appear only as `Type.t`. Mixed native data containing an
opaque value can require an abstract carrier alias shared with that module; this
preserves type equality without wrappers or copies. Cross-type method signatures
use `Other.t`; mutually referring signatures use recursive groups while their
implementations remain direct aliases, without recursive-module trampolines.
Copied values use ordinary by-value parameters; borrowed/consuming receiver
methods currently target opaque types. Opaque types must be concrete.
Opaque Rust names and ungrouped schema type names must remain unique.
Mixed root data and standard map-key signatures can require shared carrier
aliases to preserve type equality; self-contained native modules keep their
carriers private. Module grouping adds no runtime conversion or bridge crossing.

Disposal is explicit, because OCaml scope exit does not deterministically destroy
Rust owners. Use `Fun.protect` when cleanup must follow a scope; GC finalizers are
a fallback. There is no public `Owned` support module.

## Native values and ownership

Records, ordinary variants, generic types, tuples, arrays, `option` and `result`
are constructed by generated OCaml code. `Vec<T>` and `[T; N]` become OCaml
arrays; fixed arrays retain their length contract. `LinkedList<T>` becomes an
OCaml list. Returned native collections retain ordinary OCaml mutability. Rust `Result<T,E>` becomes OCaml
`(t,e) result`. `#[ocamljs_bindgen(throws)]` explicitly opts a function into a
generated typed exception carrying its error value. `serde` naming attributes
can supply enum string helpers; the transport does not use JSON or JavaScript
objects to imitate OCaml variants.

Rust `HashMap<K,V>` becomes a detached, mutable `('k, 'v) Hashtbl.t`.
`BTreeMap<K,V>` becomes a persistent `'v Key_map.t`; generated modules such as
`String_map` and `U64_map` expose the ordinary `Map.S` interface. Maps are decoded
directly into these collections. They do not retain a second pair-array cache.
Use `Hashtbl.replace` to update unique bindings. `Hashtbl.add` can introduce
duplicates, which violate the Rust map contract. Keys must be unique, and ordered
map entries must follow Rust key order; paranoid codecs check these conditions.

Primitive key comparators preserve Rust ordering, including unsigned `u64` bit
patterns and Unicode string order on ReScript. Structured keys need verified
structural equality/order: place `#[ocamljsify::ocamljsify]` **before** the standard
`#[derive(PartialEq, Eq, PartialOrd, Ord, ...)]` so the macro can read that evidence.
When a derive has already expanded, `#[ocamljsify::ocamljsify(key = "structural")]`
is an explicit contract that Rust key equality/order follows the described fields
and variant order. It can be combined with `transparent`. Do not use this contract
for custom equality or ordering with different semantics. Unsupported abstract
generic, opaque, floating-point, recursive, and map-valued keys receive diagnostics.
Keys containing mutable values must remain unchanged while stored in a collection.
Explicit enum discriminants are not supported as ordered map keys.

For an explicitly JavaScript-oriented string map, described containers and aliases
can select `#[ocamljsify::ocamljsify(map = "dict")]`. It projects syntactically
nested `String`-keyed maps to native `'v Js.Dict.t` on Melange/ReScript. Decoding
creates a null-prototype object and uses own-property checks, preserving keys such
as `__proto__` and `constructor`, including bindings whose native value is `None`.
Encoding enumerates own keys; ordered Rust maps sort keys by Rust string order
before serialization. JavaScript dictionary enumeration is **not** Rust map order,
particularly for integer-looking keys. Dictionaries are mutable; other native collections retain their default
representations. Native `map = "dict"` is rejected for js_of_ocaml; use
`js_value("dict")` for a JavaScript dictionary on that backend.

### JavaScript representations

Use `js_value` locally on a type declaration or field. A named type has one
representation everywhere it occurs; generation does not create a second
`*_js` type. A selected aggregate uses real JavaScript objects, arrays, strings,
booleans, numbers, BigInts and string dictionaries for its anonymous builtin
structure. Traversal stops at each named declaration, whose own policy applies.

```rust
#[ocamljsify::ocamljsify]
pub struct Row { pub label: String }

#[ocamljsify::ocamljsify(js_value)]
pub struct View { pub rows: Vec<Row>, pub values: Vec<f64> }

#[ocamljsify::ocamljsify]
pub struct Envelope {
    #[ocamljsify(js_value)]
    pub rows: Vec<Row>,
}

#[ocamljs_bindgen]
pub fn view() -> View {
    View { rows: vec![], values: vec![1.0, 2.0] }
}
```

`View` is a JavaScript object. Its arrays contain `Row` values in their declared
native representation. `Envelope` remains a native record with a JavaScript
array field containing the same native `Row` type. Mark `Row` explicitly when it
should instead be a JavaScript object everywhere.

Melange consumers use ordinary properties and arrays:

```ocaml
let total () =
  let view = Bindings.view () in
  let values = view##values in
  let sum = ref 0. in
  for i = 0 to Array.length values - 1 do
    sum := !sum +. values.(i)
  done;
  !sum
```

A function marker selects its success value's anonymous builtin structure. When
it directly names `T`, `T` must already declare the JavaScript representation.
Existing function names, module ownership and the loaded Wasm instance remain
in use. A syntactic outer `Result` keeps its native transport wrapper; named
aliases follow their own canonical representation, including aliases to
`Result`. `throws` requires the supported direct `Result` signature.
`js_value_rec` is an explicit escape hatch that selects reachable named types
globally; ordinary usage should keep selections local and visible as above.

ReScript receives native structural object types with their actual quoted
JavaScript property names. js_of_ocaml uses `Js.t`, `Js.js_array Js.t`,
`Js.js_string Js.t`, `Js.number_t` and `bool Js.t`, with its standard interop
operations. Melange/ReScript dictionaries use `Js.Dict.t`; js_of_ocaml uses
null-prototype `Js.t` dictionaries. All returned strings detach from Wasm memory;
retaining a substring may retain its owned decoded pool.

Tuples use numeric-key objects. Nested JavaScript options use `{tag:0}` or
`{tag:1,value:...}`; results use `{tag:0,ok:...}` or `{tag:1,error:...}`. Payload
enums expose a numeric tag and an optional typed variant property. Missing
payloads use the compiler's standard undefined type. Native named children cross
the boundary through typed compiler callbacks, including during input encoding.

#### Representation selectors

`js_value` selects JavaScript representation on declarations, fields and exports.
An explicit selector requests one compatible representation:

```rust
#[ocamljsify::ocamljsify(js_value)]
pub struct Samples { pub values: Vec<f64> }

#[ocamljsify::ocamljsify]
pub struct Counters {
    #[ocamljsify(js_value("number"))]
    pub count: u64,
}
```

| Selector      | JavaScript representation            | Compatible values                                                                   |
| ------------- | ------------------------------------ | ----------------------------------------------------------------------------------- |
| `number`      | Number                               | Numeric types; integers must be exact and within the JS safe integer range.         |
| `bigint`      | BigInt                               | Integer types within their Rust range.                                              |
| `boolean`     | Boolean                              | Rust booleans.                                                                      |
| `string`      | Unicode string                       | Rust strings.                                                                       |
| `array`       | Ordinary JS array                    | Vectors, linked lists and fixed arrays.                                             |
| `object`      | Object with typed properties         | Structural aggregates.                                                              |
| `dict`        | Null-prototype string-key dictionary | String-key maps, preserving own keys such as `__proto__`.                           |
| `map`         | JS `Map`                             | Maps with supported key types.                                                      |
| `typed_array` | Element-matched typed array          | Numeric sequences, such as `Vec<u8>` → `Uint8Array` or `Vec<f64>` → `Float64Array`. |
| `nullable`    | `null` or the present payload        | Options, explicitly collapsing absence layers.                                      |
| `optional`    | `undefined` or the present payload   | Options, explicitly collapsing absence layers.                                      |

Plain `js_value` chooses Number for numeric types through 32 bits and floats,
BigInt for 64-bit integers, Boolean for booleans, Unicode strings for strings,
ordinary arrays for vectors/linked lists/fixed arrays, and typed objects for
aggregates. String-key maps use dictionaries; other supported maps use JS `Map`.
Tuples, options, results and payload enums use the shapes described above.
Incompatible selectors are rejected during generation. Value ranges and shapes
remain caller obligations in trusted builds; [paranoid builds](#trusted-and-paranoid-builds)
check them. For example, a `u64` selected as Number must fit within `2^53 - 1`.

Named declarations have one representation everywhere. Local selection stops at
named children and uses each child's declared policy. Generic declarations,
selected fields, aliases, scalar/flat calls and async returns use that same
policy. Host references and mutable references keep their separate interop and
ownership contracts; structural `js_value` does not replace those contracts.

In Melange, native arrays already use JavaScript arrays, so selecting the
container can leave it unchanged while changing element conversion. Native
js_of_ocaml arrays and `Js.js_array Js.t` are distinct. Consumers use their
compiler's ordinary object, array, dictionary and typed-array APIs.

Default options preserve every absence layer. Explicit `nullable` maps both
`None` and `Some None` to null; null decodes to outer Rust `None`, while a present
payload reconstructs the nested `Some` layers. `optional` uses undefined in the
same way, including collapsing `Some ()` when unit is represented by undefined.
Generated types describe this intentional flattening.

Typed-array outputs own their copied storage independently of Wasm memory.
Selecting a typed array does not introduce a borrowed Wasm-memory view.

Private codecs are generated only in directions reachable from exported functions
and callbacks, with recursive groups limited to actual dependency cycles. All
intentionally described public types and their native collection modules remain
available even when no codec is needed.

Application JSON decoding, equality and enum spellings live in frontend modules;
the SDK generates only bindings and necessary transport support.

By default `i32` becomes `int32`, small integers become `int`, wasm32 unsigned
integers become lossless `float`, and 64-bit integers become `int64`. Unsigned
64-bit values retain all bits using the signed `int64` bit pattern. The current
portable 64-bit bridge converts decimal strings and BigInt; it is more expensive
than the 32-bit scalar path. Use an explicit `js_value` selector to request a
JavaScript numeric representation; native mappings stay fixed.

Opaque fields remain private in Rust. References to opaque objects borrow the
same Rust allocation with checked shared/exclusive access. Dispose owners
explicitly; the API does not depend on garbage collection to run destructors.
Returned records may contain owned opaque handles, which also need disposal.
Use `#[ocamljs_bindgen(consume(image))]` on a direct opaque argument to transfer
ownership after all argument checks succeed. Every alias becomes invalid, even
when the Rust function returns an error. Async exports retain shared/exclusive
object guards and borrowed input storage across suspension. Explicit owned view
tokens can retain parent storage without exporting raw Rust references.

Raw returned references, arbitrary exported lifetimes/generics, mutable strings
or slices, native WASM threads and shared memories remain unsupported.

## Async

Ordinary Rust `async fn` exports return native asynchronous values directly:

| Backend     | Generated return type for `Result<Cat, CatError>` |
| ----------- | ------------------------------------------------- |
| js_of_ocaml | `(cat, cat_error) result Js_of_ocaml.Promise.t`   |
| Melange     | `(cat, cat_error) result Js.Promise.t`            |
| ReScript    | `promise<result<cat, cat_error>>`                 |

Rust errors remain native `Error` values. `throws` explicitly opts into a typed
exception that rejects the Promise (or fails the explicitly selected Lwt computation). Argument validation and start
failures also fail/reject the returned native async value. There is no completion
envelope or worker involved in an ordinary call.

```ocaml
module B = Bindings
module P = Js_of_ocaml.Promise
let ( let* ) pending fn = P.bind fn pending

let load_cat url =
  let* result = B.fetch_cat url in
  match result with
  | Ok cat -> show_cat cat; P.resolve ()
  | Error error -> show_error error; P.resolve ()
```

[The cat-photo demo](demos/cat-pics/README.md) has complete js_of_ocaml Promise, Melange Promise,
and native ReScript `async`/`await` frontends. It fetches and parses JSON in Rust,
then displays a native OCaml record in the page.

The js_of_ocaml backend defaults to the shipped native `Js_of_ocaml.Promise`
API (js_of_ocaml 6.5.0 in these projects). It does not require Lwt. Native
promises do not cancel work when dropped; use the crate's exported Rust
cancellation operations. Returned opaque owners still need explicit disposal.

Select an Lwt-compatible generated API explicitly:

```sh
ocamljs_pack build --backend js_of_ocaml --async-runtime lwt
```

This changes `Wasm.load`, exported async functions and host Promise projections
to `Lwt.t`; link `lwt` when compiling that package. Cancelling a generated Rust
future's Lwt computation drops that future and releases its leases. Cancelling
an adapter for an existing host Promise does not cancel its source. Cancellation
does not roll back Rust mutations. The default is `--async-runtime promise`.
For `--backend all`, this option applies only to the js_of_ocaml output; selecting
Lwt for Melange or ReScript alone is rejected.

The public support module remains `Wasm`. Task IDs, host roots and completion
delivery stay inside generated glue. Both modes retain native OCaml records,
variants and Result values, with their respective native failure channels.

## Workers and callbacks

Bindings have the same API on the page and inside a worker. Initialize the WASM
instance in the worker, then call ordinary generated functions there. The binding
tools do not create workers or generate RPC, proxy objects or worker entry files.

The search demo keeps every Rust object in its worker. Its separate application
channel sends commands such as build URL, cancel and query, and receives progress,
search text and status messages. The scripted-game worker runs compiled OCaml
callback checks locally. See [the search demo](demos/async-search/README.md).
Graceful application shutdown cancels work and disposes owners before closing;
`Worker.terminate()` does not promise Rust destructors.

A Rust `Callback<Event>` parameter accepts an ordinary OCaml function
`event -> unit`. Generated glue decodes each owned event snapshot before invoking
it. A borrowed Wasm `externref` slot lives through the call; Rust roots the
function when it stores a callback or captures one in a future. Rust drop,
handler replacement or application `clear_handler` releases that retention.
There is no explicit callback factory or generic remove operation.

A handler exception is reported after Rust releases guards. Same-instance
synchronous reentry during a call/poll/destructor is rejected; queue subsequent
commands. Callbacks run in the realm owning WASM; cross-realm progress is
application messaging. A closure capturing its own Rust owner can form a rooted
cycle: explicitly clear its handler or dispose its owner to break that cycle;
finalizers cannot collect it while the Wasm reference table roots the closure.

Melange strings contain native UTF-8 bytes, while DOM/JSON strings use JavaScript
UTF-16. Convert at those I/O boundaries; the demo DOM adapter and Hex's independent
`Utf8` module show this. Generated codecs handle the compiler representation;
application logic receives ordinary OCaml values.

## Performance and boundary guarantees

Scalar and opaque-reference calls use direct numeric WASM exports. Opaque
handles are pointers to individually owned typed allocations. Ordinary typed
access dereferences the pointer and checks a borrow counter; there is no global
directory, generation/type lookup, page store, downcast or reference count.
The allocation carries a destructor for heterogeneous packet rollback/finalization.
Async and consuming anchors hold borrows that prevent disposal until release.
Only paranoid builds check a live-pointer/type registry. Native Rust tests use a
separate u32-to-64-bit-pointer bridge; it is absent from trusted wasm32.

Synchronous root opaque returns create no serialized output packet, Reader or
claim transaction. Nested owners and async results retain transactional buffered
publication, including rollback. JS owners store their pointer directly,
without a WeakMap or freezing. Disposing/moving clears the pointer so aliases
cannot reuse it. Finalizers hold only the numeric pointer and defer destruction
while an async borrow is outstanding. Promise subscriptions and fetch requests
have explicit cancellation identities to reject late events after memory reuse.
Raw Rust pointer entry points are unsafe and require the generated ownership
contract. Initialization rejects older Wasm/glue pairs.

The generator specializes each exported signature at generation time. Nongeneric records
with one to eight direct scalar fields and tuples with up to eight scalar leaves
lower directly to Wasm arguments, preserving their native OCaml API. Synchronous
numeric record results use a 64-byte internal return area: the stub reads all
fields into host values before releasing its call guard. No view escapes, and
later calls cannot overwrite earlier OCaml results. Async results and callbacks
retain owning output buffers. Larger, nested and generic records use the bounded
codec. Numeric vectors have bulk operations. Consecutive fixed-width record and
tuple fields use one span (checked in paranoid builds) and typed DataView getters, followed by native
ML construction, without an intermediate field array or OCaml object-layout
assumptions. Spans stop before enums, variable-width fields, opaque claims and
host operations. Enums reuse their existing decoder; transparent scalar wrappers
retain their depth checks in paranoid builds.

ABI7 numeric vector inputs write directly into aligned final Rust allocations,
then transfer ownership at export entry. Numeric slices use separate borrowed
storage guards, including across async suspension. Generated copies specialize by
numeric type and representation; native arrays fill directly, selected typed
arrays bulk-copy, and single-element/empty inputs avoid unnecessary copy setup.
All numeric argument lengths/offsets are captured before allocations can grow
Wasm memory. Preparation failure rolls back earlier allocations. Heavy validation
remains paranoid-only; ownership and memory-growth handling remain in production.
Explicit `js_value("typed_array")` also works through numeric slice aliases.

ABI8 root string inputs, including registered owned and borrowed aliases, use
pointer, initialized UTF-8 length and allocation capacity. Selected JS strings
encode directly into Rust-owned storage; native byte strings fill that storage
without a JS staging buffer. Rust adopts owned inputs without another copy;
borrowed inputs retain scoped storage, including across async suspension.
Synchronous root String results transfer their existing allocation and release
it after constructing the declared host representation. ABI12 also transfers
successful synchronous `Result<String, E>` payloads directly. Errors, explicit
`Encoded<Result<...>>`, other composite and async string results retain the owning
buffered codec. Native strings retain their compiler byte-string semantics; use
explicit JS string selection for host-facing
Unicode. Regenerate Rust, bindings and runtime together after this ABI change.

Composite values use a bounded little-endian binary codec and allocate decoded
snapshots. Large hot data should stay opaque, exposing small queries or batched
operations instead of repeated snapshots.

ABI4 buffered outputs own separate structural-body and UTF-8-pool segments. String
lengths remain in the body. Plain-data outputs read the live segments during a
non-reentrant decode region. Outputs that acquire host references/owners retain
an owned pool snapshot across those effects. Paranoid builds additionally
validate UTF-8 and each string boundary. Native-byte and JS-string codecs
preserve their declared representations and BOMs; returned strings own their
contents or retain a JS-owned decoded pool,
and publication moves both segments without assembling another packet. Inputs
and standalone `Encoder::new()` retain their inline format. Regenerate Wasm,
bindings and runtime together; exported body bytes alone are no longer an inline
packet.

Generated stubs contain no runtime signature interpreter. Native root strings
encode directly into Wasm input memory; host Unicode root strings use TextEncoder.encodeInto
in Rust-owned storage, while browser fetch streams transport raw bytes;
Rust performs incremental UTF-8 decoding, including split code points. JS retains
host APIs, host representation conversions and buffer copies. Runtime emission
includes only required primitive codecs and the selected native Promise or opt-in Lwt adapter.
Counters are disabled unless JS initialization explicitly passes
`initialize(source, { diagnostics: true })`.

Input/output buffers and output-owned handles are released on partial decode
and expected error paths. Paranoid codecs enforce limits of 64 MiB, one million
collection items and 128 nesting levels, with malformed-value diagnostics.
Unsupported schemas, stale handles, conflicting borrows and ABI/schema mismatches
still fail explicitly in production. Unexpected WASM
traps poison the module; reload the page or restart its worker to recover. `Wasm.failure ()`
returns `None` while healthy and `Some reason` after a trap. This query does not
enter Wasm and preserves the first failure reason. Repeating initialization does
not replace a poisoned module or revive its owners.

### Trusted and paranoid builds

Generated production codecs default to trusting the matching typed value contract.
Shapes, numeric ranges, fixed lengths and native UTF-8 string preconditions are
caller obligations. They avoid repeated value checks, depth/budget bookkeeping
and validation of Rust-produced output. Public Rust `decode`/`decode_segments`
byte parsers remain checked. Generated raw Rust FFI entrypoints are unsafe;
ordinary typed Rust functions and generated OCaml APIs remain the user surface.

Use `ocamljs_pack build --paranoid` to compile checked Rust codecs and emit checked
bindings together. For separate generation, build the Rust dependency with its
`ocamljs_bindgen/paranoid` Cargo feature and pass `generate --paranoid`.
`OCAMLJS_PARANOID=1` is the equivalent build/generation environment setting, used
by the SDK invalid-input conformance suite. Mixing new fast/checked artifacts is
rejected at initialization. The flags are independent of `--dev`/release profile.

Runtime `initialize(source, { diagnostics: true })` enables statistics in either
build; it does not enable validation. No per-value runtime paranoid switch is
emitted. Ownership, memory-growth handling, cleanup and trap poisoning remain.
Typed arrays use owned bulk copies in both modes; checks do not force scalarized
transport. Re-run the optional matrix with `build.mjs --paranoid --out DIR`, and
`run.mjs --artifacts DIR`, to compare checked and trusted builds.

Run `npm run benchmark` after preparing glue artifacts for the focused
14-case native/JS/wasm-bindgen comparison (42 measurements). It uses a 1-second
sampling budget per measurement and targets about two minutes or less.
`--case`, `--variants` and `--time` select deeper investigations; `--full` runs
the full glue matrix. `npm run benchmark:demo` retains the raw WASM, runtime and
compiled OCaml scalar demo. These are boundary measurements, not whole-application
throughput guarantees. `npm run benchmark:async` measures
runtime/compiled Promises and indexing using Benchmark.js deferred cases.
Benchmark.js owns calibration and sampling in both commands. See the
[runner guide](docs/benchmarkjs.md) for commands and workload boundaries.

The optional [glue benchmark matrix](benchmarks/glue/README.md) compares native
and `js_value` bindings against wasm-bindgen and the pinned Hex response transport.
It covers calls, flat/nested data, arrays and strings in both directions.

## Validation and real sites

Install Node, Rust's WASM target and Dune 3.24.2. Each project's `dune.lock` pins OCaml 5.5.2,
Melange 7 and js_of_ocaml 6; npm pins ReScript 11.1.4.

```sh
make install                 # Install ocamljs_pack into Cargo's binary directory
make deps                    # Install local npm dependencies for all projects
make prepare                 # Cold builds and all 27 browser sites
make test                    # Rust + all three compilers; shared advisory budget
make test-browser            # Optional Chromium interactions
```

Each of the ten projects under `demos/` has its own Cargo workspace and lock,
Dune workspace and lock, npm manifest and lock, browser sources and Makefile.
They build independently. The SDK workspace does not contain their Rust crates
or compile their OCaml sources. The SDK Makefile exposes the commands;
`scripts/projects.mjs` handles project loops and backend packaging. Each demo's
Makefile contains its own direct build commands.

Build a project from its directory:

```sh
cd demos/calculator
make -C ../.. install        # Install ocamljs_pack once
make deps                    # Install this project's npm dependencies
make wasm                    # Compile the Rust library to WebAssembly
make bindings                # Generate OCaml bindings and the JavaScript bridge
make frontend                # Compile the OCaml application to JavaScript
make bundle                  # Bundle that JavaScript for the browser
make assets                  # Assemble HTML, CSS and Wasm beside compiled modules
make build                   # Run the complete chain above
make serve                   # Build, then serve at http://localhost:8000
make clean
```

Each step builds its prerequisites. Rust commands run inside `rust/`, so Cargo
finds its own manifest and lock. Dune runs from each project's directory and
uses its local workspace, project configuration and dependency lock.

`make deps` installs only local npm dependencies; it does not install tools,
build sites or run tests. Demo and example Makefiles have no `install` target.
`make install` installs `ocamljs_pack` on Cargo's normal executable path.
Root and standalone builds invoke that same command; there is no parent override
to a debug executable. Test preparation reinstalls the current tool sources
before compiling the matrix.

`cargo build` compiles the Rust library.
`ocamljs_pack generate` generates bindings from that project's Wasm binary.
The calculator bundles `_build/default/output/src/ocaml/Main.js` inside its own directory.
`ESBUILD` holds the path to the project's local esbuild executable.
Cargo, Dune and `ocamljs_pack` are invoked directly from `PATH`.
Normal builds exclude `Test.ml`; the Dune `validation` alias compiles checks only
when explicitly requested. The SDK-level `make prepare` compiles the full
matrix and test executables; `make test` runs them. `make demos` builds only the
browser apps; the conformance project has its own checks build.

Each demo keeps application sources in `src/ocaml/` and `src/rescript/`, with
`Main.ml` and `Main.res` as the entry points. ReScript consumes checked-in `.res`
files exclusively. Melange and js_of_ocaml share OCaml application sources;
compiler-specific FFI and Promise observers live under `adapters/`. Tests and
fixtures are in `tests/`, and browser assets are in `web/`. The reference-conformance
project has checks rather than an application. Dune files and `rescript.json`
are checked in; the build performs no frontend syntax conversion.

The SDK Makefile coordinates the three-backend validation matrix:

```sh
make calculator              # Only the calculator's normal frontend
make demos                   # All normal demo frontends
make matrix                  # All three compilers and browser sites
make prepare                 # Matrix + native tests + freshness stamp
make test                    # Bounded compiler/runtime checks
make test-browser            # Optional Chromium interactions
```

Bindings/Wasm go into each demo's `generated/<backend>`, sites into
`dist/<backend>`, and ReScript compiler output into `lib/`. Dune owns its normal
`_build/` output. Preparation includes the checked-in compiler configuration and
backend source files in its input hash, and excludes these build outputs.
The generated example has its own Cargo and Dune workspaces too. Hex and the
example invoke the same installed `ocamljs_pack` command.
`npm run prepare:all` remains an alias for `make prepare`. Node handles aggregate
project operations, backend packaging, browser tests and the bounded test runner.
The preparation helper only hashes inputs and records freshness.

`npm test` rebuilds missing or stale preparation with `make prepare`, then runs
the checks. One shared 20-second advisory clock includes preparation and all
check subprocesses; exceeding it warns and continues. Individual compiled-client
results retain their five-second timeout.

`demos/hello-world`, `demos/calculator`, `demos/todo`, `demos/game-of-life`, `demos/image-editor`,
`demos/document-explorer`, `demos/async-search`, `demos/scripted-game` and
`demos/cat-pics` each
contain a Rust backend and an OCaml frontend. Serve each `dist` directory over HTTP to try its
three compiler variants. Browser validation uses Playwright with an existing
Chromium (`CHROMIUM=/path/to/chromium`); it does not install a browser.

`tests/hex-command` drives the larger consumer via an explicit source path.
Hex Command temporarily uses Cargo paths into this sibling SDK workspace, so
both projects exercise the same crate sources during development. Its frontend
is OCaml/ReasonReact, with native Rust rules, versioned JSON persistence and no
TypeScript application. Its build and validation instructions live in that
consumer's development documentation.

## Native host references

`#[ocamljs_bindgen(host)] pub struct Canvas;` declares an abstract host type.
Typed `extern "C"` import blocks describe JS functions, methods, properties and
ES-module classes. The packager emits real `externref` import/export signatures
and a Wasm reference table; it does not serialize host objects or maintain a JS
object-ID registry. Imported exceptions become `Result<T, HostError>`.

Rust `host::Promise<T>` implements `Future<Output = Result<T, HostError>>`.
OCaml sees native `Js.Promise.t` or `Js_of_ocaml.Promise.t` by default; js_of_ocaml can explicitly generate `Lwt.t`; no new task API.
See [reference conformance](demos/reference-conformance/README.md) for ownership
rules and supported limits. Real browser workflows live in
[image editor](demos/image-editor/README.md),
[document explorer](demos/document-explorer/README.md), and
[search in a worker](demos/async-search/README.md).

Demo browser applications contain no handwritten JavaScript. DOM access, input
handling, worker protocols and startup are OCaml/ReScript code using direct
browser FFI. Each entry point initializes Wasm through the generated `Wasm.load`.
esbuild injects the generated runtime and emits shared ESM chunks, preserving one
runtime instance per browser realm and a single module shared by application and
diagnostic imports. Browser source assets remain HTML, CSS and static data.

### Public binding surface: hard rule

`Bindings` may expose only bindings corresponding to explicit Rust declarations.
`Bindings.Wasm` is reserved for glue actually necessary for the binding runtime;
it is not a place to hide unrelated generated conveniences. Application JSON/save
decoding, typed structural equality, and enum/string conversion belong in the
frontend unless exposed as explicit Rust functions. Remove SDK JSON/equality
selection options and automatic enum conversion generation as consumers migrate.
Audit generated collection modules separately as representation dependencies of
Rust map types; do not assume that every convenience is justified by the schema.

The SDK does not generate application JSON decoders, equality functions or
enum/string conversions. Necessary map type support is audited separately.

### Implementation status

The representation rewrite remains unfinished; supported combinations and
limitations are described above and exercised by the compiler matrix.
