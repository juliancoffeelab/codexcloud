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
