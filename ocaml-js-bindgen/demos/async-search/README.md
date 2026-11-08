# Search in a worker

This application owns its worker and message protocol. The binding generator has
no worker mode or RPC API. Compiled `Worker` loads Wasm through the generated
`Bindings.Wasm.load` entry point, then creates its Rust resources in that realm.

The page sends application commands: build a URL, cancel, query, validate and
shutdown. The worker replies with progress, index-ready status and search text.
Application code uses `Channel.command` and `Channel.event` variants, with integer
progress counts and boolean validation results. Only Channel translates these to
wire strings and validates incoming messages; unknown kinds and invalid payloads
are ignored. Corpus, index, input and callback owners stay inside the worker.
`src/ocaml/Channel.ml` and `src/rescript/Channel.res` own this protocol and worker lifetime.

Indexing calls ordinary `B.Index.build_url`, which returns native promises by default and
accepts an OCaml progress function. The application exposes Rust `search_cancel`
and `cancel` operations. Each build gets a fresh one-shot token; cancelling wakes
the pending fetch future, returns `Error Index_error_cancelled`, drops the stream
and releases the corpus lease before completion. Reusing a started token returns
`Closed`. This is Rust application behavior, not a binding task API.

The previous index survives an unsuccessful replacement. Shutdown requests
application cancellation and waits for native completion before disposing the
cancellation token, index and corpus and closing the worker. Rust drops progress
callbacks automatically. Abrupt termination does not run Rust destructors.
Unexpected worker failures ask the user to reload.

Commands are gated until worker readiness. One build, one validation and one
query batch can be in flight; starting a build during validation is disabled.
Commands are limited to 32768 JavaScript characters, incoming text to 1 MiB, matching
the corpus cap. Progress emits at most eight messages and query returns at most 128
matches. These application bounds do not constitute a generic RPC credit system.

`Browser_checks.ml` runs direct local binding ownership checks inside the worker:
exclusive-borrow rejection, Unicode query and cleanup. It also creates a native
Blob, awaits `Blob.text()` in Rust through `Index.of_blob`, converts text directly
into bounded Rust UTF-8 storage, builds an index and queries Unicode data. The
corpus's exclusive borrow and the OCaml progress callback remain alive across the
browser await; disposal is rejected while pending. This path is exercised by both
the browser worker and Node's compiled worker test, without adding an RPC layer.
The browser suite exercises cancellation during a real held HTTP response,
restart, query, large Unicode results and shutdown. The collision-game demo also
runs its compiled callback/reentry checks inside an ordinary browser worker.

From the project root, run `make prepare`, serve `tests/` over HTTP and
open `async-search/dist/melange/index.html`. Run `make test` and `make test-browser` for
the prepared validation suites.

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
applicable. The worker lifetime-validation command uses these cases directly.

The js_of_ocaml adapter uses the compiler’s shipped `Dom`/`Dom_html` APIs and
official `js_of_ocaml-ppx` for property access. Browser APIs missing from those
bindings use local FFI declarations. OCaml/browser string conversion calls the
generated runtime; demos contain no custom ASCII checks or text codecs.

`tests/lwt/` separately validates `--async-runtime lwt`, including the existing
cancellation contracts where applicable. Its Lwt dependency is for that profile;
the default frontend and generated native-Promise API do not link Lwt.
