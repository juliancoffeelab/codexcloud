# Native host references

This independent project checks native host-reference contracts. It has no website. Real Canvas/ImageBitmap work
lives in [image-editor](../image-editor/README.md), file parsing in
[document-explorer](../document-explorer/README.md), and native Blob awaiting in
[async-search](../async-search/README.md). Scripted-game exercises real OCaml
collision callbacks. This project keeps deliberately artificial identity,
ES-module class, thrown-value, unsigned-range, buffer and promise cases.
All three compilers consume the same Rust declarations.

```rust
use ocamljs_bindgen::{ocamljs_bindgen, HostError, host::Promise};

#[ocamljs_bindgen(host)]
pub struct Blob;
#[ocamljs_bindgen(host)]
pub struct Text;

#[ocamljs_bindgen]
unsafe extern "C" {
    #[ocamljs_bindgen(method, js_name = "text")]
    fn text(blob: &Blob) -> Result<Promise<Text>, HostError>;
}

#[ocamljs_bindgen]
pub async fn read(blob: &Blob) -> Result<Text, HostError> {
    text(blob)?.await
}
```

The generated interface is ordinary OCaml:

```ocaml
module Blob : sig type t end
module Text : sig type t end
module HostError : sig type t end
val read : Blob.t -> (Text.t, HostError.t) result Js.Promise.t
```

js_of_ocaml uses `Js_of_ocaml.Promise.t` by default. Select
`--async-runtime lwt` to generate an Lwt-compatible API instead. For example:

```ocaml
module B = Bindings
module P = Js_of_ocaml.Promise

let show blob =
  P.map (function
    | Ok text -> display_native_text text
    | Error error -> display_host_error error)
    (B.read blob)
```

`Text.t` deliberately represents a native JavaScript string. Native host references preserve that string without an OCaml encoding round trip. Ordinary Rust `String`
exports elsewhere still become native OCaml strings with their compiler's proper
encoding. A typed host declaration asserts the JS API's type contract; it does
not perform reflection or validate a value's class at runtime.

## Lifetime and boundary costs

- Public Wasm imports and exports carry actual `externref` values. Host object
  identity survives, including backing buffers and imported class instances.
- Synchronous borrowed arguments occupy reusable Wasm table slots, cleared after
  the call. Retaining a host value in Rust creates an owning table root. Rust
  `Clone` retains, `Drop` releases. Async borrowed parameters retain an anchor
  until completion/cancellation. No JS object-ID map or proxy object is involved.
- OCaml host `Module.t` values are owned by the host GC. They have no generated
  `dispose`. A Rust-owned `Holder.t` still uses the checked generational arena and
  has `Holder.dispose`; disposing it drops its retained host reference.
- Structural return packets and callback records can contain host values. Their
  roots remain owned by the packet until decoding/cleanup finishes; partially
  decoded packets release unclaimed values.
- Rust awaits an existing promise through one native settlement subscription and
  a generation-tagged Rust waiter. Cancellation releases roots and the waiter;
  late settlement cannot revive it. Native JS promises themselves are not
  cancellable. Cancelling a Rust/Lwt operation does not cancel its input promise.
- Typed promise inputs/import results get an immediate shared rejection observer.
  This preserves identity and the original rejection, including thrown primitives,
  when Rust awaits later or drops/cancels before polling. It allocates one native
  observer promise per acquisition. Awaiting adds its own settlement subscription.
  js_of_ocaml additionally needs a native Promise adapter. These are real costs.
- A rejected host promise becomes Rust `Err(HostError)` when awaited in Rust.
  Default js_of_ocaml promises preserve the original reason in `Promise.error`.
  Explicit Lwt generation exposes `Promise_rejected of HostError.t`. Both profiles
  preserve rejection identity when that promise is passed back to Rust.
- Compiler promise observers are weakly associated with real promise objects.
  They preserve thenable host payloads inside generated Rust-await glue without
  native re-assimilation. Foreign JS code reading the underlying promise retains
  js_of_ocaml's documented carrier semantics. Await in Rust and pass the host
  object directly when that foreign API does not understand compiler carriers.
- Reference passing does not copy buffer contents. Arbitrary JS buffers are still
  separate from Wasm linear memory: Rust numeric processing of their bytes needs
  an explicit copy or a view into Wasm memory. `host::Uint8ClampedArray::copy_from` and `to_vec` provide bounded bulk
  copies. External arrays copy once; an input backed by this Wasm memory first
  takes a protective snapshot before Rust allocation can grow/detach the memory.
- `host::JsString::to_rust_string(max_bytes)` uses `TextEncoder.encodeInto` directly
  into Rust memory. It bounds the allocation, preserves Unicode/BOM, and follows
  JS replacement semantics for lone surrogates. There is no intermediate JS byte
  array. The Rust destination capacity can exceed the written length (up to three
  bytes per UTF-16 code unit, capped by the requested limit and SDK limit).

Import blocks support function, method, getter, setter, indexing_getter, call,
constructor, `js_namespace`, and static `js_module` declarations. Every native
import returns `Result<T, HostError>` so JS exceptions cannot unwind through Rust.
Native unsigned integers are normalized separately from the OCaml Int64 transport.

Reference Types support here means `externref` interop and preservation of the
module's existing `funcref` tables. It does not expose typed Wasm function
references or the Wasm GC type system as OCaml APIs.

Nested host **output** values work. Host inputs must be direct parameters;
arbitrary nested host-input serialization is rejected. `Promise<T>` currently
requires a concrete host payload. Nested promises, including aliases, are rejected
because foreign JS promises assimilate them. Native OCaml Promise nesting is supported by
the compiler, but is not a Rust host::Promise ABI payload. Await
and decode into ordinary Rust data explicitly instead. Same-instance synchronous
reentry from host calls/callbacks is rejected; queue the next call normally.

## Run and verify

From the SDK root:

```sh
make prepare
make test
make test-browser
```

There is no frontend or worker in this project. The imported `third-party.mjs`
is copied beside generated test runtimes; a consuming project supplies its own
declared module dependency.

The default suite checks native identity (including primitive thrown values),
retained Rust ownership, partial packet cleanup, full unsigned ranges, buffer
identity/mutation, native pending/rejected promises, delayed awaiting, immediate
cancellation and late settlement. It also builds primitive-only imports,
HostError-only signatures, built-in Promise aliases, text-only and pixel-only
exports without declaring host structs. Compiled tests exercise ordinary native Promise consumption. Reference tests cover
byte offsets, detached buffers, Wasm memory growth during copying, bounded Unicode
conversion, and cleanup. Chromium exercises actual browser APIs in the other demos.

## Setup

This directory is an independent Rust/OCaml project. Its Cargo workspace,
Dune workspace, npm dependencies and build outputs are local to this directory.
The Rust dependencies point at the SDK crates in this checkout for development.

Install Rust with the `wasm32-unknown-unknown` target, Node.js and Dune 3.24.2.
Dune uses this project's checked-in dependency lock.

```sh
make -C ../.. install        # Install ocamljs_pack once
make deps                    # npm ci for this project
make build
```

`make build` writes Rust artifacts to `rust/target/`, OCaml artifacts to
`_build/`, bindings to `generated/`, and site assets to `dist/melange/` where applicable.
It does not build other projects or run tests.

## Build commands

Run in this directory, with the Rust/OCaml toolchains and this project's npm dependencies installed:

```sh
make wasm                    # Compile the Rust library to WebAssembly
make bindings                # Generate OCaml bindings and the JavaScript bridge
make checks                  # Compile conformance checks without running them
make build                   # Same as make checks
make clean
```

This checks-only project keeps OCaml checks in `tests/ocaml/`, native ReScript
checks in `tests/rescript/`, js_of_ocaml overrides in `tests/js_of_ocaml/`,
compiler configuration in `adapters/js_of_ocaml/`, and Rust in `rust/src/`.
The imported JavaScript fixture is `tests/third-party.mjs`.
Compiler configuration is checked in next to its sources, with `rescript.json`
at the project root. Builds perform no frontend syntax conversion.
Run `make matrix` at the ocaml-js-bindgen root to build all three compiler variants,
or `make prepare` and `make test` for bounded validation. This project has no site or serve target.

`tests/lwt/` separately validates `--async-runtime lwt`, including the existing
cancellation contracts where applicable. Its Lwt dependency is for that profile;
the default frontend and generated native-Promise API do not link Lwt.
