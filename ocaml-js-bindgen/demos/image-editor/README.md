# Little Paint

A small, touch-friendly Paint app. Draw with a brush or eraser, fill enclosed
areas, drag out lines, rectangles and ellipses, choose a color and brush size,
and undo or redo complete pixel edits. The Hand tool (H) pans the view; use
two fingers to pinch-zoom and pan, the wheel or +/− controls to zoom, and Fit to recenter.
Pinching works in any tool and cancels the unfinished single-finger edit. The Select tool (V)
creates a rectangular selection: drag its contents to move them, drag a corner
to resize, or enter an exact width and height. Escape cancels an active edit
or deselects. Delete removes selected pixels; arrow keys nudge them (Shift: 10 px).
Panning, zooming, selecting and deselecting never enter image history or clear redo. Open a PNG from your device and save your
painting as a PNG. Files are processed locally; nothing is uploaded.

The default canvas is 768 × 512. PNG import accepts images up to 2048 × 2048
and files up to 8 MiB. Erasing paints white, like a traditional opaque Paint
canvas. Undo and redo share a 24 MiB / 40-step history budget; the oldest
states are discarded when that budget is reached. New and Open replace the
document; Clear can be undone. Reloading starts a new drawing, so save your work.

Keyboard shortcuts: B brush, E eraser, F fill, L line, R rectangle, O ellipse;
Ctrl/Cmd+Z undo, Ctrl/Cmd+Shift+Z or Ctrl/Cmd+Y redo, Ctrl/Cmd+S save.

## Implementation

Rust owns the pixels, stroke rasterization, scanline fill, PNG codecs and bounded
history behind an abstract `Paint.t`. Selection transforms sample the original
source pixels on every preview, reuse storage and commit one history step;
resizing uses nearest-neighbor sampling. Vacated pixels become white. A gesture takes one history snapshot.
Undo and redo exchange existing pixel buffers without copying them. PNG decoding
also reuses its decoded buffer when the input is already RGBA.
Brush segments retain coalesced pointer samples; shape previews keep the latest
position and rasterize at most once per animation frame. Rendering copies pixels
in bulk once per changed frame, never through OCaml records or per-pixel DOM.
Canvas dimensions are only reset when the document size changes.

OCaml owns tool selection, input, viewport and selection geometry, and document
lifetime. Selection borders and resize handles are DOM overlays and are not
included in PNG export. View transforms change CSS display size, not pixels. Pinch updates read both
fingers from one native touch event and apply the latest pair once per animation
frame, avoiding intermediate transforms from mismatched pointer samples.
The canvas position stays stable when selection settings open. Melange and js_of_ocaml share
`src/ocaml/App.ml`; ReScript uses `src/rescript/App.res`. `Ui` handles DOM events, paired touch sampling, pointer capture, frame scheduling,
shortcuts and downloads in OCaml/ReScript through direct typed browser FFI. Rust awaits
`File.arrayBuffer` for PNG import: Melange/ReScript use promises; js_of_ocaml
uses native promises. Late imports are discarded and disposed when the drawing changes.

The original Image/Texture APIs remain available for ownership and native
ImageBitmap validation. They are exercised in tests, rather than exposed as
buttons in the app.

## Setup and build

Install Rust with the `wasm32-unknown-unknown` target, Node.js and Dune 3.24.2.
This directory owns its Cargo, Dune and npm manifests and dependency locks.
Rust dependencies point at the SDK crates in this checkout for development.

```sh
make -C ../.. install  # Install ocamljs_pack once
make deps              # Install local npm dependencies
make build             # Build the Melange site in dist/melange
make serve             # Serve at http://localhost:8000
```

Individual build steps are `make wasm`, `make bindings`, `make frontend`,
`make bundle` and `make assets`. `make clean` removes local build products.

At the SDK root, `make prepare` builds all three compiler variants and native
tests, `make test` runs the bounded SDK checks, and `make test-browser` exercises
real browser interactions. Browser checks cover drawing, erasing, shapes, fill,
keyboard history, PNG download/reimport, invalid PNGs, late import cleanup,
mobile touch input, cached pixel transfers and native ImageBitmap ownership.
All browser variants also check Unicode/BOM preservation, Wasm-load recovery and
worker startup failure notification.

## Source layout

```text
src/ocaml/          App.ml, Ui.ml and Main.ml
src/rescript/      App.res, Ui.res and Main.res
adapters/          Direct browser FFI and native Promise observation
rust/src/          Pixel engine and browser host imports
web/               HTML and CSS
tests/             Compiled checks and image fixture
```

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
