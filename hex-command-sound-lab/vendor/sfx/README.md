# Vendored Hex Command sound collection

All original audio is tracked here in ordinary Git, organized by creator and
source pack. A normal clone contains the assets: no archive import, network
request, Git LFS, external storage, or other project is needed to use them.
The soundboard in `dist/` indexes every original through smaller listening
copies. Run `python3 scripts/build-soundboard.py` from the project directory to
rebuild those copies. Source originals in this folder remain unmodified.

## Use and verify

From `hex-command-sound-lab/`, with Python 3.11+:

```sh
python3 scripts/sfx.py verify --hash-only
python3 scripts/sfx.py verify --report tmp/sfx-validation.json
```

The first command needs only Python. Full verification also needs FFmpeg and
FFprobe on PATH and decodes every audio stream, checks duration, nonempty files,
SHA-256, file sizes, inventory completeness, duplicate hashes, and coverage
references. No installation or download is needed to access the vendored audio.
Play originals directly in an audio editor/player. Some medieval WAVs contain
multiple takes and silence; keep those originals intact. Put edited audition
copies in a separate directory and record the processing command and input hash.
The soundboard builder writes processed listening copies separately in
`dist/audio/listening/`, with original hashes and processing metadata.

To repair missing originals or reproduce acquisition into a separate directory:

```sh
python3 scripts/sfx.py acquire
python3 scripts/sfx.py acquire --output tmp/sfx-reconstructed --hash-only
```

Acquisition additionally needs 7-Zip (`7zz` or `7z`) for archives and network
access only for missing content. It reuses byte-identical `dist/audio/` files
before downloading, caches downloads under ignored `tmp/sfx-downloads/`, checks
archive SHA-256 and payload CRCs, extracts all locked members, and verifies their
individual hashes before installation. Alternate output contains originals;
provenance metadata remains in this checkout. Acquisition never changes the
lock or accepts a changed upstream download silently. Complete archives are
transient cache inputs, not redundant repository payloads.

Required acquisition hosts are `opengameart.org`, `raw.githubusercontent.com`
(for the pinned Kenney mirror), and `bigsoundbank.com` (horse vocals). GitHub is
needed to clone/push. Kenney source pages were readable, but both signed-link
endpoints returned HTTP 403, so the existing mirror was verified and pinned at
`9662dc1bed3b06ae753045915f9accfac044c7ce`. All 181 existing Kenney originals
were reused byte-for-byte; the missing RPG preview was also included. Kenney's
headline says 50 RPG assets; the pinned complete audio tree has 51 effects and
one preview. No preview is substituted for a required library.

## Licensing and provenance

Every audio file in this folder is CC0-1.0. CC0 permits modification,
redistribution of the standalone files, and commercial use alongside MIT code;
there is no attribution, share-alike, noncommercial, no-derivatives, or
standalone-resale restriction. The audio retains its CC0 provenance rather than
being described as newly authored MIT audio. Creator credits are preserved
voluntarily. See `CC0-1.0.txt`, the unmodified Kenney `License.txt` files, and each
pack's `SOURCE.txt` and `CREDITS.txt`. OpenGameArt pages explicitly associate CC0
with the downloads; BigSoundBank's sound and license pages explicitly permit
editing and redistribution. CC0's own trademark/patent and warranty limitations
still apply.

- `sources.json`: creator, license, source/download URLs, retrieval date, pinned
  revision where available, archive hashes/sizes, extracted-member counts and
  integrity caveats. Source-page text snapshots include creator confirmations.
- `inventory.json`: every original asset and original Kenney license, with exact
  path, archive member, pack/download identity, byte size and SHA-256.
- `validation.json`: full decode results, durations/codecs, counts/sizes and
  duplicate groups. Sizes below are audio bytes, not filesystem allocation.
- `coverage.json`: concrete local examples and limitations for movement, attack,
  impact and vocalization, including magic, fire, undead and animal candidates.
- `roster.json`: a read-only snapshot of 107 units and their attacks from game
  revision `8d4d50c50888ed347a0f427c1eef16c5252b57a2`. No cross-project runtime
  dependency is introduced.

Still North Media's two parts per pack are separate complete archives, not
split-volume segments; both were tested and fully extracted into `part-1/` and
`part-2/`. Their 92 WAVs are actual weapon recordings, not the samples in
PanderMusubi's GitHub repository. The Sorth ZIP has an upstream header warning
in 7-Zip; its 15 payloads pass ZIP CRC checks, extract completely, and all decode.
No audio was omitted or modified to hide that warning.

## Contents

| Creator / pack | Audio files | Bytes | MiB |
| --- | ---: | ---: | ---: |
| kenney/impact-sounds | 130 | 1,034,296 | 0.986 |
| kenney/rpg-audio | 52 | 1,048,639 | 1.000 |
| still-north-media/weapon-textures | 44 | 253,044,648 | 241.322 |
| still-north-media/weapon-impacts | 48 | 255,661,248 | 243.818 |
| rubberduck/80-cc0-rpg-sfx | 80 | 2,005,466 | 1.913 |
| rubberduck/80-cc0-creature-sfx | 80 | 2,033,143 | 1.939 |
| qubodup/dog-snarl-grunt-grumble | 3 | 308,179 | 0.294 |
| sorth/ghost-voice-pack | 15 | 5,904,660 | 5.631 |
| ezduzziteh/horse-trotting | 1 | 138,357 | 0.132 |
| jaggedstone/magic-spell-sfx | 7 | 367,724 | 0.351 |
| antumdeluge/fire-crackling | 2 | 276,016 | 0.263 |
| qubodup/3-dark-magic-spells | 3 | 820,142 | 0.782 |
| denischardonnet/horse-vocals | 2 | 105,625 | 0.101 |
| **Total** | **467** | **522,748,143** | **498.531** |

The largest file is `still-north-media/weapon-textures/part-1/Scythian Recurve Draw.wav` (27191416 bytes), below GitHub's 100 MiB file limit. Two duplicate-hash pairs of Kenney carpet footsteps are retained under their original names. Git deduplicates identical blobs, including the 181 originals also used by the soundboard.

## Coverage and recording types

Kenney supplies recorded foley/material impacts; Still North supplies weapon movement, bow/crossbow releases and metal impacts. Rubberduck supplies designed RPG spells/fire and creatures; the creator explicitly says most creature sounds, including barking, were made with their mouth and filters. JaggedStone supplies filtered tonal spellcasting; qubodup dark magic is upstream synthesis. Sorth supplies performed ghost/lich hurt sounds, lines and laughter.

Natural animal recordings are qubodup's domestic dog at play and DenisChardonnet's outdoor horse neigh/snort. The horse trot is explicitly jacket/water-bottle/hand foley. Generic soft impacts and designed growls/eating are candidates for bites, claws, paws and trampling, not authentic species recordings. Authentic wolf, boar, bear, fox and bat recordings are not included or claimed. Unit-specific sound design and subjective suitability still require auditioning; `coverage.json` records those limits.

The 51 existing synthesized game samples in `dist/audio/current/` remain separate and are not counted or relicensed as CC0 here. No new synthesis or substitute recordings were generated. The acquisition command changes no playable-game, soundboard, hosting or deployment files.
