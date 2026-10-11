# Sequential OCaml ports

This rewrite started again from the main commit preceding PR #13. The rejected
hybrid implementation is not used as the baseline. Each incoming PR was read
with its description, rebased in order, and given a separate OCaml checkpoint.
The final ledger records original and port changes separately, excluding generated
sources, lockfiles and assets.

- PR #13: native artwork, shared terrain rendering/prebuilds, startup ownership,
  hidden maps and version-four save expansion. Checkpoint `d83199c`.
- PR #15: bounded after-paint/idle terrain warming with map-generation cancellation.
  Checkpoint `5a865b9`; all 204 frontend tests passed in 7.34 seconds.
- PR #16: repository PR workflow instructions carried unchanged. This is documentation
  only, so the frontend runtime and interface port deltas are both zero. No pushes
  or commits to main, merge or Sites publication are part of this rewrite.
