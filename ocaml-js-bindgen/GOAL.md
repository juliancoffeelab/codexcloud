# ocaml-js-bindgen

An experimental binding generator for calling Rust libraries compiled to
WebAssembly from OCaml applications in JavaScript environments.

The goal is an OCaml API that feels authored in OCaml: native records, ordinary
variants, `option`, and `result`, with abstract types for Rust objects that retain
identity and state. Rust declarations should supply the binding information
directly, without generating TypeScript declarations and parsing them back into
OCaml.

This document starts the experiment. There is no implementation yet. The first
prototype should test the design before we commit to a transport or toolchain.

## What using it should feel like

A Rust developer marks supported types and exported functions with derives or
attributes. The generator produces documented `.ml` and `.mli` files, Rust export
wrappers, and the required transport glue. Attribute names and syntax remain to
be designed.

For example, a stateful Rust API might become:

```ocaml
type position = { x : int32; y : int32 }

type action = Move of position | Wait

type error = Invalid_action | Finished

type snapshot = { position : position; finished : bool }

type simulation

val create : position -> simulation
val apply : simulation -> action -> (snapshot, error) result
val dispose : simulation -> unit
```

Callers can pattern-match on returned data and use normal OCaml libraries.
The interface hides the simulation's foreign handle and transport details.
Disposal and operations on disposed objects have documented behavior.

Data and opaque objects can coexist, including in the same record or function
signature. Supported structs and enums become native values; explicit attributes
select opaque treatment where copying would lose identity or expose internals.
Unsupported types produce actionable generation errors.

## Portability

Share the API schema and public OCaml types across two initial targets:

- **Melange**, including applications using Reason syntax and ReasonReact.
- **js_of_ocaml**, including compatible Bonsai and Eliom client applications.

The project should have no dependency on a UI framework. Native OCaml values must
be constructed according to each backend's representation; generated runtime
bindings are backend-specific even when their public signatures are identical.
Compatibility with particular libraries and compiler versions must be tested,
not inferred from the interface alone.

Bindings to unrelated JavaScript libraries remain the application's choice.
Native OCaml execution and wasm_of_ocaml are possible later targets, requiring
their own adapters and validation.

## Architecture to investigate

Keep three responsibilities separate:

1. **Rust API description.** Derives and attributes describe supported data,
   exported operations, documentation, and ownership. Rust's compiler checks
   generated wrappers. Define explicit mappings for integer widths, strings,
   collections, records, enum payloads, `Option`, and `Result`.
2. **Transport contract.** Specify how calls, values, buffers, and handles cross
   the WASM boundary, including allocation, release, and failure behavior.
   Detect incompatible generated bindings and runtime versions.
3. **OCaml generation.** Emit native types and deep modules that own conversion
   and lifetime management behind small public interfaces. Generate the
   backend-specific glue rather than requiring handwritten application adapters.

A first-principles implementation is the working direction. A wasm-bindgen
extension or fork remains an option if investigation shows it removes useful
work without imposing unwanted semantics. A binary transport through WASM memory
is a candidate, not a settled ABI. Compare it with reusing existing transport
machinery before making a commitment.

In every approach, expected Rust `Result` errors must arrive as OCaml `Error`
values. They must not accidentally inherit wasm-bindgen's exception convention.
Unexpected traps and boundary failures need a separately documented contract.

## Ownership, reliability, and performance

Opaque types hide implementation details; they do not enforce Rust lifetimes.
Start with owned values and explicit object disposal. Reject unsupported borrowed
references rather than exposing handles whose validity cannot be guaranteed.
Check invalid and stale handles, including reuse after disposal. GC finalizers
must not be the only mechanism for releasing Rust resources.

Document and test cleanup on every error path, bounds and lengths at the boundary,
integer conversion, UTF-8, and WASM memory growth. Generated transport code must
refresh memory views when necessary. Treat callbacks and reentrancy as later
features requiring explicit contracts.

Native records and variants require materializing OCaml values. Measure that
cost, copies, allocations, and crossings on representative workloads. Favor
coarse operations over a crossing for every field access. Opaque handles can
preserve identity without copying an object; that does not make returned native
data zero-copy. Borrowed views need a future design of their own.

## First experiment and acceptance criteria

Begin with synchronous functions, primitives, strings, small collections, records,
payload variants, `option`, `result`, and an owned opaque object. Defer async,
callbacks, arbitrary borrowing, and a universal Rust type mapping.

The first demonstration should cover:

- The same generated public API working under Melange and js_of_ocaml.
- Native record and variant round trips, including nested values and typed errors.
- An opaque object that keeps identity across calls and releases resources safely.
- Clear compile-time or generation errors for unsupported signatures.
- Negative cases for disposal, invalid handles, malformed data, and conversion.
- A small performance comparison reporting payload sizes, copying, allocations,
  and boundary-call counts where measurable.

Start with a small standalone fixture crate, then try independent third-party
libraries with different API shapes, using thin export wrappers where required.
Record which signatures work, what needs annotation, and what cannot be supported.
Do not claim general ecosystem support from one successful example.

Keep fixtures, dependencies, tooling, and documentation within this project.
Do not depend on other projects in this monorepo. The complete test suite and each
verification command must finish in under 20 seconds; explicit toolchain and
artifact preparation should be separate from those checks. If access or setup
blocks validation, report it without introducing package or network workarounds.

The next step is to choose a small fixture and write down its generated API,
ownership rules, and transport experiment before implementing the generator.
