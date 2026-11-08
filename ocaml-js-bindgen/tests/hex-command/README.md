# Hex Command integration fixture

The north-star consumer is the complete Hex Command project. The driver takes
its source path explicitly. During this overhaul, Hex uses Cargo paths into the
sibling SDK checkout:

```sh
npm run test:hex -- --source /path/to/hex-command
```

Install that consumer's npm dependencies and its documented toolchains first.
The driver builds the real Rust backend with the sibling SDK sources for all
four crates, compiles its OCaml/ReasonReact frontend, runs preparation and
the complete verification suite, and checks the dependency graph for the retired
binding tools. The resulting playable site is in the consumer's `dist/` folder.
No TypeScript frontend or JSON command adapter is used by the site.

Historical JSON test fixtures are translated by a generated, compiled OCaml
test helper. The application's saves are an explicit versioned JSON boundary;
ordinary application calls carry native variants, options, records and results.
