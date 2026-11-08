# Hello, Rust

The smallest formula example: enter two side lengths and Rust computes their
hypotenuse as you type. The initial inputs `3` and `4` produce `5`. No application state, UI helpers or handwritten JavaScript.

Each compiler has its own short entry point: `src/ocaml/Main.ml` (Melange),
`src/js_of_ocaml/Main.ml`, and `src/rescript/Main.res`.

```sh
make -C ../.. install
make deps
make build
make serve
```

`make build` builds only this project's Melange site, without running tests.
Use the SDK root's `make matrix` for all three compilers. The Rust implementation
is `rust/src/lib.rs`; generated bindings and glue go into `generated/`.
