# Rust → OCaml: complete generated example

This combines the three examples into one Rust library: an ordinary
`Geometry.point` record, opaque `Token.t` objects produced by `Tokens.parse`,
and `fetch_cat : string -> (cat, cat_error) result Js_of_ocaml.Promise.t`.

- [Rust source](rust/src/lib.rs)
- [Complete generated Bindings.ml](Bindings.ml)
- [Complete generated public interface](Bindings.mli)
- [Complete generated JavaScript runtime](bindings.runtime.js)
- [Compiler-emitted schema](schema.json)
- [OCaml application](Frontend.ml)

The generated files are checked in for inspection. They are copied directly from
`ocamljs_pack`, without hand edits or formatting. The packager emits
`bindings.runtime.mjs`; `make` copies those exact bytes to the requested
`bindings.runtime.js` filename. It remains an ES module.

From the `ocaml-js-bindgen` directory, with the toolchains described in its README:

```sh
make install
cd examples/rust-to-ocaml
make deps
make build
make serve
```

Open http://localhost:8000. The page runs OCaml compiled by js_of_ocaml, backed by
Rust WASM. Its compiled OCaml entry point initializes WASM once before starting the UI. The token action uses `Fun.protect` for disposal. Cat fetching
uses ordinary browser fetch and native js_of_ocaml promises; the URL field defaults to TheCatAPI.

Cargo runs inside `rust/`; Dune builds in this directory using the local
`dune-project`, `dune-workspace` and dependency lock. The installed
`ocamljs_pack` command is shared with the other projects. Browser-check
dependencies are declared in this example's own npm manifest and lock.

The build generates the full package under `generated/`, copies the readable
outputs here, and builds the browser site in `dist/` (ignored build products).
The OCaml entry point calls `B.Wasm.load`; esbuild links the generated runtime
and emits shared ESM modules. The example requires no handwritten JavaScript bootstrap.
The generation command in its Makefile is:

```sh
cd rust
ocamljs_pack build --backend js_of_ocaml --name example --out-dir ../generated
```

Run the focused browser check after building:

```sh
make test
```

It checks native records, token success/error and the full `u32` range, explicit
owner disposal, genuinely pending asynchronous fetch, success and typed errors,
and zero live ABI entries after cleanup. Browser tests use a controlled local
HTTP response and an intercepted image; they do not require provider access.
