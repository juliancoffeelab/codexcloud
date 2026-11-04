# Hexcommand Sound Lab

A standalone, buildless soundboard for comparing unit movement, attacks, impacts,
and equipment sounds. Includes ten representative units, numbered variations,
action sequences, a searchable library, favorites, and JSON export.

Live site: https://hexcommand-sound-lab.lightdarkdaughter.chatgpt.site

The site is private; open it while signed into the owning ChatGPT account.
The layout adapts to phone screens. Playback starts after a tap or click.

## Run locally

From this project directory:

```sh
python3 -m http.server 4173 --directory dist
```

Open http://localhost:4173. Any static HTTP server works; Python is just a
convenience. No npm installation, build step, API key, database, or backend is
required. Serve over HTTP or HTTPS instead of opening `index.html` as a file.

To test on a phone on the same local network, open the host computer's LAN
address on port 4173; the local firewall must allow that connection.

All application source and bundled assets are tracked in `dist/`; that directory
is authored source, despite its name. Nothing depends on another project in this
repository or on files left in an execution environment.

## Offline source collection

The complete source packs and supplemental CC0 audio are now tracked in
[`vendor/sfx/`](vendor/sfx/README.md), with licenses, acquisition script,
SHA-256 inventory and validation. A normal clone contains all 467 originals;
future asset work needs no browser import or external audio URLs. The existing
`dist/` soundboard remains unchanged and uses its original catalog.

## Included sounds and provenance

- 130 Kenney Impact Sounds recordings, licensed CC0.
- 51 Kenney RPG Audio effects, licensed CC0. The pack preview is excluded.
- 51 existing Hexcommand synthesized samples, labeled separately from CC0 audio.
- Original 64×64 Hexcommand unit sprites, copied without resizing or editing.

Original Kenney license texts are in `dist/licenses/`. The files were obtained
from https://github.com/GeorgeQLe/assets-sfx-combat with those license texts.
`dist/catalog.json` records sound paths, sources, licenses, and SHA-256 hashes;
`dist/provenance.json` records pack origins. `dist/sound-library.zip` contains the
bundled audio, licenses, and manifests for download.

The archive reader is libarchive.js 2.0.2, bundled locally with its worker and
WebAssembly binary. Its MIT license and libarchive's BSD license are preserved
in `dist/licenses/`. It loads only when importing an archive.

Unit assignments are audition suggestions, not changes to the playable game.
Material substitutes and missing bow/magic candidates are labeled explicitly.
The live game remains unchanged.

## Medieval source packs and legacy imports

Both archive parts of Still North Media's weapon textures and weapon impacts
are fully extracted in `vendor/sfx/still-north-media/`. See the vendored
collection's README for offline access, licensing, counts and provenance.

The preserved soundboard's fetch/import buttons are legacy audition controls.
They still import files into tab memory and are not needed to obtain the
vendored collection. The source collection is not automatically indexed by
`dist/catalog.json`; future integration is separate work. Arbitrary user imports
still require their own license verification.

Favorites persist in browser storage on that device. Export JSON to keep a
portable shortlist; importing audio itself is not persistent storage.

## Hosting and recovery

`.openai/hosting.json` preserves the existing Sites project ID and declares
`dist/` as its static directory. Reuse that ID when updating the existing site;
do not create a replacement site. Publication needs the owning Sites account
and a fresh short-lived source repository credential from the Sites tools.
Credentials are not stored in this repository.

The existing deployment survives execution environment reloads independently.
A fresh repository checkout contains everything needed to serve the bundled
soundboard again. Opening a repository PR does not update the hosted site.
