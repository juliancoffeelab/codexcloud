# todo

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
make wasm                    # Rust library to WebAssembly
make bindings                # OCaml bindings and JavaScript bridge
make frontend                # OCaml to JavaScript
make bundle                  # Browser JavaScript bundle
make assets                  # HTML, CSS, Wasm and runtime in dist/melange
make build                   # Complete chain; no test compilation/execution
make serve                   # Build and serve at http://localhost:8000
make clean
```

Each step builds its prerequisites. `tests/ocaml/Test.ml` belongs to explicit validation,
not the normal frontend build. Rust commands run inside `rust/`; `ocamljs_pack`
is the installed binding generator.

Application source lives in `src/ocaml/` and `src/rescript/`; entry points are
`Main.ml` and `Main.res`. ReScript consumers use checked-in `.res` source exclusively.
The OCaml application is shared by Melange and js_of_ocaml; their FFI and
native Promise adapters live in `adapters/melange/` and `adapters/js_of_ocaml/`.
ReScript browser FFI declarations live in `adapters/rescript/`.
Tests and fixtures live in `tests/`, web assets in `web/`, and Rust in `rust/src/`.
Compiler configuration is checked in next to its sources, with `rescript.json`
at the project root. Builds perform no frontend syntax conversion.
Run `make matrix` at the ocaml-js-bindgen root to build all three compiler variants,
or `make prepare` and `make test` for bounded validation. Matrix sites are in
`dist/<backend>`; `make serve PORT=8001` selects another port for the normal site.

Browser behavior is authored in OCaml/ReScript, with direct typed browser FFI in
`adapters/*/Browser`. `Ui` owns DOM composition and input handling. Entry points call
`Bindings.Wasm.load` before constructing Rust resources. esbuild links the generated
runtime as a shared module; `frontend.mjs`, worker entry points and runtime chunks
are compiler/bundler output. `web/` contains only HTML, CSS and static data.
No handwritten JavaScript bootstrap, DOM proxy or worker channel is required.

Rust conformance cases live in `tests/rust/`, alongside native unit tests where
applicable. The compiler matrix enables the `conformance` feature; normal builds omit it.

The js_of_ocaml adapter uses the compiler’s shipped `Dom`/`Dom_html` APIs and
official `js_of_ocaml-ppx` for property access. Browser APIs missing from those
bindings use local FFI declarations. OCaml/browser string conversion calls the
generated runtime; demos contain no custom ASCII checks or text codecs.
