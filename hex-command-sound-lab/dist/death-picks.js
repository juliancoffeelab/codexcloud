// Four shared auditions, with no size, armor, species or variation layers.
export const DEATH_SOUNDS = Object.freeze({
  flesh: "rubberduck/80-cc0-creature-sfx/grunt_01.ogg",
  bones: "rubberduck/80-cc0-creature-sfx/eat_01.ogg",
  spirit: "edits/ghost-grunt-02-trim-v1",
  wood: "impact-sounds/impactPlank_medium_004",
});
export function deathPick(entity, { bone, spirit }) {
  const group = bone ? "bones" : spirit ? "spirit"
    : /^treant(?:-l[23])?$/.test(entity.id) ? "wood" : "flesh";
  return { group, sound: DEATH_SOUNDS[group] };
}
