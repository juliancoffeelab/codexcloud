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
