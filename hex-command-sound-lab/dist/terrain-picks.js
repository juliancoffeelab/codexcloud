// Authored terrain preferences. Existing raw sounds remain available for comparison.
const original = (family) => [1, 2, 3].map((n) => `current/${family}-${n}`);
const edit = (name) => `edits/${name}`;
const cue = (suggested, note, families = []) => ({ suggested, note, families });
const stone = "rubberduck/80-cc0-rpg-sfx/stones_01.ogg";
const door = cue(
  [edit("house-door-soft-v1")],
  "Quiet door contact for this building.",
  ["doors"],
);
const water = cue(original("terrain-water"), "The existing water cue.", [
  "water",
]);
export const terrainPicks = {
  forest: {
    tap: cue(
      [edit("forest-book-03-soft-v1")],
      "Quiet distorted Book 03 forest texture.",
      ["wood"],
    ),
  },
  road: {
    tap: cue([stone], "Stones 01 for the road.", ["stone"]),
    move: cue([stone], "Stones 01 for road contact.", ["stone"]),
  },
  plains: {
    tap: cue(
      [edit("plains-grass-003-soft-v1")],
      "Grass 003 with less bass and a lower level.",
    ),
    move: cue(
      [edit("plains-grass-003-soft-v1")],
      "Soft, bass-cut grass contact.",
    ),
  },
  bridge: {
    tap: cue([stone], "Road-like stone contact for the bridge.", [
      "stone",
      "wood",
    ]),
  },
  keep: {
    tap: cue(
      [edit("keep-metal-light-v1")],
      "Light metal contact, mixed to a restrained tile-cue level.",
      ["metal"],
    ),
  },
  marsh: {
    tap: cue(
      [...original("terrain-water"), edit("marsh-gurgle-soft-v1")],
      "Choose the existing water cue or a heavily softened burble.",
      ["water", "burble"],
    ),
    move: cue(
      [original("terrain-water")[0], edit("marsh-gurgle-soft-v1")],
      "Water or very quiet burble contact.",
      ["water", "burble"],
    ),
  },
  ruins: {
    tap: cue(original("terrain-hill"), "The existing hill wind for ruins."),
  },
  fort: {
    tap: cue(
      [edit("fort-wood-v1")],
      "Wood contact matched approximately to the keep metal cue.",
      ["wood"],
    ),
  },
  grove: {
    tap: cue(
      [edit("ruins-magic-soft-v1")],
      "The former quiet ruins magic for the sacred grove.",
      ["magic"],
    ),
  },
  ferry: { tap: water },
  landing: { tap: water },
  water: { tap: water },
  house: { tap: door },
  monastery: { tap: door },
  church: { tap: door },
  village: {
    tap: cue(original("terrain-village"), "The existing village cue."),
  },
  hill: {
    tap: cue(
      original("terrain-hill"),
      "The existing wind-through-the-hills cue.",
    ),
  },
  mountain: {
    tap: cue(
      [edit("mountain-mining-soft-v1")],
      "A heavily quieted mining texture.",
      ["mining", "stone"],
    ),
  },
  cave: {
    tap: cue(
      [edit("quarry-mining-v1")],
      "A more present mining cue for the quarry / mine.",
      ["mining", "stone"],
    ),
  },
  "burned-village": {
    tap: cue(
      [edit("burned-fire-soft-v1")],
      "Soft crackling fire for the burned settlement.",
      ["fire"],
    ),
    ambience: cue(
      [edit("burned-fire-soft-v1")],
      "A short quiet crackle texture, not an automatic loop.",
      ["fire"],
    ),
  },
  signpost: {
    tap: cue(original("terrain-plains"), "Keep the existing signpost cue."),
  },
  "castle-bell": {
    tap: cue(
      ["impact-sounds/impactBell_heavy_000"],
      "A bell for the bell keep.",
      ["bell"],
    ),
  },
  "castle-wall": {
    tap: cue(
      original("terrain-keep"),
      "Keep the existing castle courtyard cue.",
    ),
  },
  graveyard: {
    tap: cue(
      ["rubberduck/80-cc0-rpg-sfx/item_wood_03.ogg"],
      "The same item-wood starter used for skeletons.",
      ["wood"],
    ),
  },
  "ritual-star": {
    tap: cue(
      [edit("ritual-fout-03-soft-v1")],
      "A short quiet fout-03 ritual cue; the full original is an alternative.",
      ["dark", "magic"],
    ),
  },
};
