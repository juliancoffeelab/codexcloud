---
name: testing
description: Use for Hex Command bug fixes, test changes and implementation validation.
---

# Hex Command testing

Run `npm run format` before validating changes.

Tests must be small, fast, and lightweight.

- Data-only changes do not justify new behavioral tests or extra named-content
  cases in existing test matrices.
- Validate content through generic schema and consistency checks. Measure
  balance through simulations.
- A test belongs in the codebase only if the agent can promise its value is
  greater than its total burden, including runtime, maintenance, brittleness and
  duplication. If that promise cannot be made, use a disposable probe. Reproducing
  a bug alone does not satisfy this rule.
- For bug fixes, use a disposable test or probe: reproduce the failure, fix the
  code, verify that the probe passes, then delete it. Run the relevant existing
  suites and remove obsolete expectations rather than preserving incorrect behavior.
- Use a small, fixed set of representative fixtures. Do not freeze current
  unit stats, promotion choices, recruitment lists or roster sizes.
- Do not duplicate existing coverage merely because new content uses it.
  Remove redundant cases when found.

- Target 20 seconds for all test suites together, including Rust, Python
  and Vitest. Use the shared `npm test` runner; exceeding the target warns
  without terminating the run or failing successful tests. Use
  controlled promises or an injected clock instead of real delays.
- Keep browser automation out of the default test suite. Playwright is available
  for explicitly requested browser spot checks; keep scratch probes in ignored
  `tmp/`. Do not add mandatory browser installs, extra services, large migration
  suites, or Git-history fixtures. Use focused checks for meaningful rules,
  state transitions, publication, and cancellation behavior.

AI, harness and analyzer changes also require the simulation validation described
in [combat-simulation](../combat-simulation/SKILL.md). Validation for an explicit
deploy request is owned by the deploy operation; follow [deploy](../deploy/SKILL.md).
