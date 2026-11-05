# Local saved progress

Saved progress is local to a browser profile. After the initial online visit
finishes preparing offline play, reopening and playing saved runs requires no
account, network request, or cloud service. Clearing site data removes it. Cloud synchronization
and backup/export tools are postponed.

The game exposes the result of the latest recovery write alongside the Saved
progress button. A failed write leaves the previous committed recovery available;
play can continue, and Saved progress provides deletion, thinning, and retry tools.
Storage refusal is never reported as a successful save.

## Recovery and history

`hex-command-battle` is the atomic current recovery envelope. Completed player
actions write the complete native checkpoint together with the available move
undo states. Combat, recruitment, promotion, item drops and turn changes retain
their existing undo barriers. UI selection and AI planner workspaces are not
saved. An enemy turn retains the pre-turn recovery with `pendingEnemy: true`
throughout its actions, then writes the fully resolved result. Enemy-first games
retain the untouched opening enemy state. Reload resumes from these safe points;
updates may migrate the underlying state and discard undo.

Historical payloads are immutable turn-start checkpoints, including opening
mission dialogue and campaign progress. Completed battles remain recoverable as
the current continuation; they do not create a mid-turn historical checkpoint.
Each inactive continuation also retains its latest recovery, including undo. This
is one recovery per continuation, not an automatic history of mid-turn actions.
Starting another run or loading earlier progress parks the outgoing recovery
before replacing the current one. Loading creates a distinct continuation, so
playing differently cannot overwrite a previously reached continuation.

## Storage and retention

`src/save/progress.ts` owns the local repository operations. Its versioned
metadata index is separate from checkpoint payloads, so browsing reads labels and
timestamps without deserializing every battlefield. The dialog renders thirty
entries at a time. Recovery writes do not scan or rewrite historical payloads.
A history insert writes its payload before publishing its index entry; deletions
remove index entries before removing payloads. Interrupted operations can leave
unreferenced bytes but never publish an absent payload. Each payload replacement
and current recovery replacement is an atomic browser storage write.

Automatic thinning runs at turn boundaries. It retains every one of the latest
24 turn checkpoints in each continuation. Older checkpoints are selected by a
stable turn ordinal at intervals of 2, 4, 8 and so on in progressively older age
bands. Mission starts are always exempt from automatic thinning. Both automatic
and manual whole-run operations protect the currently played checkpoint and its
recovery. Other individual checkpoints and inactive continuation recoveries can
be deleted explicitly. Retained payloads are complete native snapshots, so a
missing ancestor does not prevent loading them.

Each record carries stable run, continuation, checkpoint and parent identities.
These identities and self-contained payloads leave room for a future player-private
sync transport with divergent continuations. No sync, account credentials, planner
state or network-dependent code is part of local persistence today.

## Compatibility

The battle envelope is version 31. The pure v30-to-v31 conversion adds support for
optional progress metadata and move undo without changing native game state.
Earlier versions continue through the existing migration chain. Current recovery
and selected historical payloads pass through that same loading entry point:
parse, migrate, validate in Rust, then atomically commit the validated conversion
before using it. Migration code remains isolated under `src/save/migrations/`.

## Offline reopening

Production builds generate a content-versioned service worker manifest from the
complete built release, including the HTML shell, dynamic JavaScript chunks,
styles, map-rendering worker, WASM, all unit/terrain artwork, and audio. Hosting
configuration files are excluded. The release downloads in the background with
bounded concurrency; it does not delay mounting or gameplay.

The UI shows preparation, readiness, unsupported-browser and failure states.
Wait for **Ready for offline play** before disconnecting or closing the first
online visit. Failed preparation provides error details and a retry button.
An incomplete release never becomes active. An installed worker can repair a
missing cache after reconnecting. Once ready, navigations reopen the cached shell
and local saves remain available without a connection, including other maps and
campaigns that were not opened during the initial visit.

Each worker serves one coherent release. A downloaded update waits until the
previous release's tabs close; activation then removes obsolete release caches.
Static release files ignore cache-busting query parameters and HTTP `Vary`
headers when matching locally cached files. Offline installation needs HTTPS
(or localhost), service-worker support, and enough browser cache storage.
Browsers can evict site data; local saves and offline app files have separate
storage status, and neither survives an explicit clearing of site data.
