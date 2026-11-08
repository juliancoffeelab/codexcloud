# Binding API rules

- Only bindings corresponding to explicit Rust declarations may enter the public Bindings top level.
- Bindings.Wasm contains only glue actually necessary for the binding runtime. Do not move application helpers there to conceal an API violation.
- Application JSON/save decoding, specialized equality, and enum/string conversions belong in the application frontend unless explicitly exported from Rust.
- Audit collection modules as representation dependencies of bound Rust map types; document why each public item is necessary.
- Native mappings are fixed: Vec<T> and [T; N] become OCaml arrays (fixed arrays check length); LinkedList<T> becomes an OCaml list. Do not add native representation switches.
- js_value selects JS representation only. Use the supported selectors and representation contracts in [JavaScript representations](README.md#javascript-representations).
- Named types have one global representation, never a different representation at top level versus nested. Prefer explicit local type/field annotations; js_value_rec is an escape hatch to avoid in ordinary usage.
- Defaults preserve option distinctions. Explicit nullable/optional permit the requested absence-layer collapse: null/undefined decodes to outer None; present payloads reconstruct Some layers. Generated types describe the flattened JS representation, without pretending round-trip identity is preserved.
- Use compiler-native JS object/array/dictionary APIs. Do not add public proxy/accessor modules. Check actual emitted consumer code.
- Production generated codecs trust the matching Rust/OCaml value contract. Emit heavy range/shape/UTF-8/framing/budget validation in the build-time paranoid mode, independently of runtime diagnostics counters. Preserve generation-time checks, checked public byte parsers, ownership and lifetimes. Selecting a typed array does not authorize dangling Wasm-memory views.
- Keep full verification in one command under the shared advisory clock, including stale builds. Do not hide prerequisites outside the measurement.
- Benchmark ordinary processing without forcing GC between batches. If GC stress is measured, label it separately. Decoder-only improvements do not establish application parity.

- Follow the API and representation contracts in [README.md](README.md).
- Keep generated reports, profiles and benchmark artifacts under ignored `tmp/`; commit runnable drivers and permanent API documentation.
- Implementation must be from first principles: no reuse of wasm-bindgen/tsify/wasm-pack implementation code. A separately built pinned upstream is a comparison reference only.
