# Artwork rendering implementation

Pixel sources are character matrices and explicit palettes under `pixel/art/`.
Character sources are always 64×64, stored in
`pixel/art/units/<race>/<unit-id>/`; race trees live in `pixel/art/units/lineups/`.
Each unit folder contains only its matrix and material palette. Generation
history and previews stay in ignored `tmp/`, outside the committed sources.

```sh
npm run art:build
npm run art:check
```

The read-only build, `scripts/build-pixel-art.py`, emits content-hashed native
PNG sprites into ignored `public/assets/sprites/` and URL/layout metadata into
`src/presentation/pixel-art.ts`. It applies faction/floor palettes and flattens
prepared prop layouts at build time. Vite copies the sprites into `dist/assets/`.
The browser fetches individual images as they are rendered. Once an authored map
is displayed, idle work warms one canonical inspection sprite per terrain type,
two at a time with low network priority. Details use the default floor and
connections with the first prebuilt prop arrangement, regardless of the clicked
tile's map variation. It never downloads
an atlas, crops sprites, or encodes PNGs. Source matrices/palettes remain authoritative.
Production Vite builds also render every authored map into a native PNG using
the same terrain layouts and drawing code. Unchanged maps use these sprites
directly. Terrain edits overlay changed cells until a background canvas bake
finishes; missing map sprites also use that fallback. Generated PNGs are not tracked.

Terrain formats, floor palettes and prop layouts are described in
[terrain artwork implementation](../../docs/terrain-artwork.md). Character
creation and editing workflows live in the [generation](../../.agents/skills/pinkbox-generation/SKILL.md),
[enhancement](../../.agents/skills/pinkbox-enhancement/SKILL.md),
[cleaning](../../.agents/skills/pixel-cleaning/SKILL.md) and
[lineup](../../.agents/skills/pixel-lineup/SKILL.md) skills.

The selected unit panel displays attacks and permanent damage modifiers. Terrain
effects show movement and resolved incoming-damage modifiers. Numeric abilities
such as healing come from catalog metadata; descriptions explain roles without
duplicating the values.
