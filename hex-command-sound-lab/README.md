# Hexcommand Sound Lab

A standalone soundboard for the complete vendored collection: 467 CC0 recordings
and 51 archived original-game samples. Browse every recording and its detected
takes, or use suggested movement, attack, impact, equipment and voice assignments
for all 107 units in the bundled roster snapshot.

Live site: https://hexcommand-sound-lab.lightdarkdaughter.chatgpt.site

The site is private; sign in with its owning ChatGPT account. It adapts to phone
screens, and playback starts only after a tap. No audio downloads at startup.
No imports, external audio hosts, API keys, database or backend are needed.

## Run locally

```sh
python3 -m http.server 4173 --directory dist
```

Open http://localhost:4173. Any static HTTP server works. To use a phone on the
same network, open the host computer's LAN address on port 4173; its firewall
must permit the connection. Serve through HTTP or HTTPS, not a file URL.

All application source and listening assets are tracked in `dist/`. That folder
is authored static-site source. The project has no runtime dependency on other
repository projects, an execution environment, or external downloads.

## Auditioning

- Unit buttons are quick picks; the roster dropdown includes every snapshotted
  unit. Each named attack has a separate row. Numbered buttons play suggested
  takes. Each row has an explicit sound selector; All plays every take in the
  selected group. Long groups have a Show all control. Blade effects, legacy
  knife slices, sabre, katana, dagger and seax stay separate choices. These are
  proposed assignments, not game
  changes or claims of authentic species-specific recordings.
- Full library includes every recording, including the Kenney pack preview.
  Filter by source, category, name or recording type. Results load in batches.
  Tap a title to expose all detected takes and the complete recording. Show
  every recording removes the initial result-page limit in one tap.
- Multi-take weapon recordings have approximate silence-based boundaries. Full
  recordings always preserve the entire session. A detected take may contain
  multiple contacts, quiet sounds or room noise; audition before final selection.
- Stop or Escape cancels playback, including pending downloads and sequences.
  Scrub the position slider to seek. Match levels applies optional peak gain
  during playback; it is not a loudness mastering pass. Volume defaults to 55%.
- Stars persist favorites in this browser. Export produces a JSON shortlist
  with original paths/hashes, source/creator/license, and take boundaries.
- Natural dog and horse recordings, designed creature sounds, performed ghost
  voices, synthesized spells and recorded weapon Foley are labeled separately.
  Domestic dog is a canine reference, not a wolf field recording. See the
  vendored collection's coverage notes for species limitations.

## Originals and reproducible listening copies

[`vendor/sfx/`](vendor/sfx/README.md) contains the 467 unmodified originals,
CC0 licenses, source evidence, acquisition lock, hashes and validation. Normal
Git tracks them all; no LFS or external storage is required. The 51 archived game
samples remain separate and are not represented as CC0.

To rebuild listening assets with Python 3.11+, FFmpeg and FFprobe:

```sh
python3 scripts/build-soundboard.py
```

The builder reads only this project's locked originals and metadata. It converts
CC0 recordings to mono 44.1 kHz / 96 kbps MP3 for broad browser compatibility,
records optional peak gain, and suggests weapon takes from silence detection.
Original audio is never rewritten. A clean rebuild may differ bytewise between
FFmpeg versions; source hashes identify unchanged originals.

`dist/catalog.json` indexes all recordings with source hashes, durations,
categories, take boundaries and listening paths. `sources.json`, `coverage.json`,
`provenance.json` and original license/source texts ship with the site. The
listening-library ZIP contains every listening file plus those records. It is a
portable audition collection, not a lossless copy of the large original WAVs.
Only six decoded recordings are cached; playback requests are made on demand.

Unit artwork is a copied snapshot of native 64×64 game sprites, without edits or
resizing. Roster and artwork snapshots introduce no cross-project runtime imports.

## Hosting and recovery

`.openai/hosting.json` preserves the existing Sites project ID and declares
`dist/` as its static directory. Reuse that ID for updates. Publication needs
the owning Sites account and a fresh short-lived Sites source credential;
credentials are never committed. Push the exact static source before publishing
an archive containing `.openai/hosting.json` and `dist/`.

The deployed site survives environment reloads, and a fresh repository checkout
can serve the complete soundboard without rebuilding or downloading assets.
Source originals under `vendor/` need not be included in the hosted static output.
