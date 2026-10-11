# Dune package metadata overlay

The application uses normal Dune integrated package management. This small overlay
keeps its locked toolchain buildable on cloud environments whose package-manager
allowlist includes GitHub and Debian but omits `opam.ocaml.org`, `erratique.ch`, and
`gitlab.inria.fr`. It changes package metadata, not application/compiler sources.

- Cmdliner 1.3.0 and Menhir 20240715 use Debian's published upstream source archives,
  verified by their archive SHA-256 hashes. Debian's Menhir `+ds` archive omits
  generated manuals. Its package metadata therefore skips that documentation
  installation; the parser, SDK, runtime, and CST source are retained.
- Fix 20230505 comes from the upstream Fix source included in that Menhir archive.
  The build restores the public `Fix` library/module name from `vendored_fix`, then
  installs that library. No algorithm source is changed.
- ReasonReact 0.17.0 uses its unchanged upstream release archive. Its Opam npm
  `depexts` declaration is omitted because Dune 3.21 does not define `npm-version`.
  React and ReactDOM are instead installed and locked through `package-lock.json`.

All other packages use the GitHub-hosted Opam repository. The resulting lock embeds
checksummed source URLs and install/build instructions; consumers do not need the
network-blocked hosts or any absolute workspace paths. Run `dune pkg lock` to
regenerate it. Keep these constraints until replacing them with a verified,
portable toolchain solution.
