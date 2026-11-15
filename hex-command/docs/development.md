# Development

From `hex-command/`, run:

```sh
source /workspace/.setup/env.sh
make dev
```

Sourcing `env.sh` is mandatory in every new shell. Development, builds and checks
require its tool paths and cache variables and will not work correctly without
them. `make dev` starts Dune watching and Vite. The Makefile's `help` target lists
its entry points.

- [Project instructions](../AGENTS.md): ownership rules and task skills.
- [Verification](../.agents/skills/verification/SKILL.md): targeted checks, test policy and final verification.
- [WASM preparation](../scripts/build-wasm.sh): development/release WASM and bindings.
- [Deployment](../.agents/skills/deploy/SKILL.md): playable hosted previews and publication.
- [Deployment operation](../.agents/skills/deploy/operations.md): pinned builds and hosting transport.
- [Benchmarks](benchmarkjs.md): existing benchmark commands and measurement boundaries.

## Browser text

Live frontend `string` values contain ordinary JavaScript text. Use `{js|…|js}`
literals, `{j|…$(value)…|j}` interpolation and `Js.String` operations. Match text
with guards comparing JS literals; Melange rejects JS literals in patterns. Rust models
use the `Text` alias, selected with `js_value("string")`, so nested fields and map
keys keep JS text without changing the surrounding OCaml representation.

Save versions 1–48 retain their frozen byte-string implementation. The final
V48 → V49 conversion changes interpreted strings and keys to JS text; V49's JSON
codec reads and writes JS text directly. The current history index uses the same
convention at its V4 → V5 conversion, including storage-import and repair
identities. `migrate_typed` therefore returns JS text,
while older wire representations remain inside their versioned conversions. Opaque JSON payloads
are already JS values and pass through unchanged.

`Binding_text` is only for the SDK-generated loader input and failure diagnostics,
which still use native OCaml byte strings. Keep those conversions in the engine
adapter; application text and game-data bindings do not use them.

## Issue labels

When creating an issue, apply the relevant labels below. Combine labels for the
affected area, kind of work and status as appropriate; preserve their exact names,
including the capitalization of `Neural`.

Use `follow-up` for issues that complete work left unfinished by a PR. Link the
originating PR in the issue and add the relevant area and work labels too.

| Label              | Use for                                             |
| ------------------ | --------------------------------------------------- |
| `ai`               | Our AI bots.                                        |
| `art`              | Media assets, shaders, images and related problems. |
| `bug`              | Something that is not working.                      |
| `build`            | Tooling, builds and deployments.                    |
| `campaign`         | Creating or tuning campaigns.                       |
| `documentation`    | Documentation improvements or additions.            |
| `duplicate`        | Work already covered by an existing issue or PR.    |
| `enhancement`      | A new feature or request.                           |
| `follow-up`        | Completing work left unfinished by a PR.            |
| `gameplay`         | Features and game rules to add or fix.              |
| `good first issue` | Work suitable for newcomers.                        |
| `help wanted`      | Work needing extra attention or help.               |
| `invalid`          | An issue that does not appear valid.                |
| `investigate`      | Work blocked on investigation.                      |
| `Neural`           | The CNN-based planner, reinforcement learning, etc. |
| `ocaml`            | OCaml implementation issues.                        |
| `performance`      | Code that is slower than it should be.              |
| `question`         | A request for further information.                  |
| `saves`            | Persistence, migrations and related work.           |
| `sdk`              | OCaml–WASM bindings.                                |
| `setup`            | Problems that prevent agents from working.          |
| `ux`               | User-facing game bugs or usability problems.        |
| `wontfix`          | Work that will not be undertaken.                   |

## Save metadata

Current saves use format 49 and history index 5. Entries contain a separate
`mapId` and numeric `mission` (one-based for campaigns, `null` for scenarios).
The obsolete `side` metadata field is absent; turn phase comes from the native
checkpoint. Chapter names and numerals are presentation output. Legacy formatted
mission labels are interpreted only by explicit save/history migrations.
