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

## Medieval packs: what is still needed

The four OpenGameArt archives are not bundled. The build environment's network
policy blocked their host. Browser fetch buttons request these original files
directly, without a server proxy:

| Pack | Archive | Approximate size |
| --- | --- | --- |
| [Weapon textures](https://opengameart.org/content/medieval-sound-effects-weapon-textures) | `medieval_sfx_textures_1_of_2.7z` | 89.9 MB |
| Weapon textures | `medieval_sfx_textures_2_of_2.7z` | 86.1 MB |
| [Weapon impacts](https://opengameart.org/node/146863) | `medieval_sfx_weapon_on_weapon_1_of_2.7z` | 82.1 MB |
| Weapon impacts | `medieval_sfx_weapon_on_weapon_2_of_2.7z` | 73.8 MB |

These packs are credited to Ben Jaszczak and Brian Nelson and listed as CC0 by
OpenGameArt. Their download URLs use
`https://opengameart.org/sites/default/files/<archive>`.

Browser fetching needs the host to permit cross-origin reads (CORS). If that
fails, use the download link, then import the `.7z` file. `.zip` archives and
loose WAV, OGG, MP3, FLAC, or M4A files can also be imported. Audio decoding
support depends on the browser. Extracting these large archives is easier on a
desktop; import one archive at a time on devices with limited memory.

Imports stay in memory for the current tab; reloading closes that session.
Nothing is uploaded. Matching imported filenames automatically supply weapon
candidates to unit buttons, and all imported recordings appear in Full library.
Imported arbitrary local files require their own license verification.

To bundle medieval recordings permanently, a maintainer needs copies of these
archives (or direct download access to OpenGameArt), their source/license
records, and a reviewed selection of useful takes. Add those recordings to the
tracked audio and catalog, update unit mappings and the downloadable ZIP, then
publish an updated site. A CORS bypass or environment reset is not required for
local auditioning.

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
