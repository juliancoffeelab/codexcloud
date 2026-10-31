---
name: publish
description: Prepare checked game implementation batches as unpublished Site releases; deploy an already prepared release only when the user asks.
---

# Publish

## Two separate operations

**Prepare while implementing. Deploy when asked.** Checked and pushed game changes
must have an archive-backed saved Site version ready before reporting the batch
finished. A deploy request publishes that version in one native hosting operation.
Do not run Git, tests, formatting, builds, packaging or source synchronization at
deployment time. Do not use deployment as a reason to recommit or push GitHub.

A deployment hold still permits source pushes and unpublished release preparation.
Simulation experiments, visual drafts, reviews and unfinished work do not trigger
preparation. Publishing-tool or documentation-only changes do not require a new
game build. Explicit local-only or no-build instructions take precedence.
Publishing to the live game still requires the user's instruction; preparing or
saving a release never authorizes deployment.

## Prepare a completed implementation batch

Read the installed Sites skill for its native API and access rules. Reuse
`.openai/hosting.json`'s project ID and preserve the existing audience. Continue on
the existing feature branch and PR. Perform appropriate checks, commit and push
as part of implementation, once. Do not repeat already successful checks.

Obtain a fresh source credential and run:

```sh
npm run release:prepare -- --message 'Describe the checked changes'
```

Pass the credential through hidden stdin, never arguments, files or logs. The
helper exports the committed source into the Site checkout, preserves ignored
compiler/build caches, builds once, pushes the Site source and packages generated
output. It does not format, test or commit the working tree. Retain its returned
project ID, source commit, Site commit and archive path.

Call `save_site_version` with that exact Site commit and archive **without
calling any deployment operation**. Retain the returned saved version ID and
version number. Sites stores the version and archive remotely; release readiness
must not depend on an ignored local file surviving this cloud environment.
Finish the batch with that unpublished version ready. If preparation fails,
report the failure rather than claiming the batch is ready to deploy.

The named `prepareSite` export in `scripts/publish-sites-call.js` orchestrates
this preparation and save operation; it never deploys or pushes GitHub. Its
returned receipt contains the exact version ID and audience for deployment.

## Deploy: one action

The default export in `scripts/publish-sites-call.js` accepts that receipt and
calls only native hosting operations. With its version ID and known audience,
it makes exactly one deployment call. Use this adapter or the native call
directly; never repeat the preparation workflow.

With an archive-backed version already prepared, call the native operation:

- Confirmed owner-private Site: `deploy_private_site_version`.
- Other supported audiences: `deploy_site_version` with the existing audience.

Use the exact saved version ID from preparation. For a fresh session, retrieve
it through `list_site_versions` and select the newest archive-backed release;
perform discovery and deployment in the same orchestration call. Do not inspect
Git or rebuild to rediscover an already saved release.

Use an initial execution yield long enough for the hosting operation to finish
(for example 120 seconds) instead of repeatedly returning to poll a running cell.
A terminal successful result and its URL complete the deployment. Poll only when
the hosting API itself returns a nonterminal deployment; retain deployment and
saved-version IDs on failure. Never save or upload the same release again merely
to deploy it.

If no prepared release exists, state that preparation was missed. Do not silently
fall back to the former multi-step build/Git workflow during a deploy request.

Report the literal verified production URL. Never describe a prepared or saved
version as live. Do not redeploy an unchanged version just to demonstrate speed.
