# Direct Rust async: cat photos

A page calls one Rust `async fn`, awaits its native result, and displays a photo.
There is no worker, RPC, progress registration or task-completion envelope.

```rust
#[ocamljs_bindgen]
pub async fn fetch_cat(api_url: &str) -> Result<Cat, CatError> {
    let body = fetch::get_text_limited(api_url, 64 * 1024)
        .await
        .map_err(|error| match error {
            fetch::FetchError::Network => CatError::Network,
            fetch::FetchError::TooLarge => CatError::TooLarge,
        })?;
    parse_cat(&body)
}
```

Fetching, bounded collection and JSON parsing run in Rust. Small DOM primitives display the native OCaml record.

js_of_ocaml:

```ocaml
module B = Bindings
module P = Js_of_ocaml.Promise
let ( let* ) pending fn = P.bind fn pending

let load api_url =
  let* result = B.fetch_cat api_url in
  match result with
  | Ok cat -> show_cat cat; P.resolve ()
  | Error error -> show_error error; P.resolve ()
```

Melange uses `(cat, cat_error) result Js.Promise.t` with `Js.Promise.then_`.
ReScript uses its native `async`/`await` syntax:

```rescript
module B = Bindings
let load = async apiUrl => {
  switch await B.fetch_cat(apiUrl) {
  | Ok(cat) => showCat(cat)
  | Error(error) => showError(error)
  }
}
```

See the three frontend files for complete UI code. All use `B` as the alias for
`Bindings`. Rust errors resolve to native `Error`; `fetch_cat_throw` separately
exercises an explicit typed-exception export.

From the project root, run `make prepare`, then serve `tests/` over HTTP
and open `cat-pics/dist/index.html`. `make test` executes native async composition
and typed errors with all three compilers. `make test-browser` also verifies a
real cross-origin HTTP stream: Rust is Pending before input arrives, the page
continues processing turns, no worker can be constructed, and completion displays
an image. It covers malformed/oversized bodies and HTTP failure, with zero final
Rust tasks/resources/fetches.

The default URL is [TheCatAPI image search](https://docs.thecatapi.com/reference).
The example sends no API key and accepts another HTTPS cat JSON endpoint in the
input. Cloud egress blocks `api.thecatapi.com` here (HTTP403), so the recorded
browser pass uses a controlled local CORS server and a synthetic image fixture;
it is not a live-provider availability claim.

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

`tests/lwt/` separately validates `--async-runtime lwt`, including the existing
cancellation contracts where applicable. Its Lwt dependency is for that profile;
the default frontend and generated native-Promise API do not link Lwt.
