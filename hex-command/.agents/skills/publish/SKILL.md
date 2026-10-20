---
name: publish
description: Publish Hex Command when the user authorizes deployment or updating the live game. Simulation, rendering, committing and pushing alone do not authorize publication.
---

# Publish

## Scope

Use only with a user instruction to publish, deploy or update the live game.
Existing authorization in the session counts; do not ask for it again.
An instruction to push authorizes Git, not deployment. Visual changes, a
successful build and elapsed working time do not authorize deployment.

Simulation and balance experiments stay in ignored `tmp/` or `artifacts/`
data snapshots. Run them with `--data-dir`; render their terrain directly.
Do not register an experimental map in the playable catalog, update the hosted
site or invoke Sites merely because the experiment produces a visual result.
Promoting an experiment to playable content requires a user request for that
product change. Publication requires its own authorization.

If publication is outside the requested scope, finish the requested local work
and report it. Do not introduce a permission question just to offer deployment.

## Prepare

Read the installed Sites skill for the current native API and access rules.
Reuse `.openai/hosting.json`'s project ID and preserve its audience. Use the
existing feature branch and PR; never push directly to the default branch.

Verify that the selected product changes are authorized and that ignored
experiment data has not entered the playable catalog. Run checks appropriate
to those changes. `dist/` is generated and Git-ignored; commit source and public
assets, not build output.

## Package and deploy

Use `python3 scripts/publish-site.py --message 'Describe the product change'`
with the current Site source credential through its hidden stdin prompt.
Keep credentials out of files, shell arguments and logs.

The helper commits the project source, builds that exact source with
`npm run build` in the isolated Site checkout, pushes the Site source repository
and packages the output with `scripts/package-site.py`. Retain its returned
project ID, source commit, Site commit and archive path. Never use `git archive`
to package generated output; the packager verifies client references and bytes.

Publish the returned archive and Site commit through the appropriate native
Sites operation. Follow an existing nonterminal deployment to completion;
retain saved-version and deployment IDs on failure rather than saving twice.

For authorized visual publication, deploy before pushing the feature branch to
GitHub. If deployment fails, report it and still perform an authorized or
repository-required GitHub push. A GitHub push alone must never invoke deployment.

Report the verified production URL and any verification limits. Do not describe
a packaged build, pushed source or pending deployment as live.
