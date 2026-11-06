// One shared recording per death/timbre group; no runtime layers or random variants.
export const DEATH_SOUNDS = Object.freeze({
  merfolk: "edits/merfolk-death-underwater-v1",
  flesh: "rubberduck/80-cc0-creature-sfx/cough_02.ogg",
  bones: "rubberduck/80-cc0-creature-sfx/eat_01.ogg",
  spirit: "edits/ghost-scream-02-no-cough-v1",
  wood: "rpg-audio/creak1",
  horse: "edits/horse-0284-death-start-v1",
  wolf: "rubberduck/80-cc0-creature-sfx/howl.ogg",
  zombie: "rubberduck/80-cc0-creature-sfx/hurt_05.ogg",
  armor: "rpg-audio/metalPot3",
  dwarf: "edits/death-cough-dwarf-v1",
  elf: "edits/death-cough-elf-v1",
});
// Species-specific, deliberately not a generic mounted/rider/selection-sound test.
const LIVING_HORSE_UNITS = new Set([
  "rider", "lancer", "knight", "horse-archer", "horse-bowman", "horse-marksman",
]);
// Visible heavy metal infantry, rather than every shield/guard/mage/helmet wearer.
const HEAVY_ARMOR_UNITS = new Set([
  "champion", "warhammer", "sentinel", "halberd-master", "marshal", "grand-marshal",
  "dwarf-axeguard-l2", "dwarf-axeguard-l3", "dwarf-axe-pick-l2", "dwarf-axe-pick-l3",
  "dwarf-hammerer-l2", "dwarf-hammerer-l3", "dwarf-poleaxe-l2", "dwarf-poleaxe-l3",
]);
export function beastDeathId(id, take = 0) {
  return `edits/beast-death/${id}/take-${take + 1}`;
}
export function deathPick(entity, { bone, spirit }, selection) {
  const group = bone ? "bones" : spirit ? "spirit"
    : entity.faction === "zombies" || entity.faction === "undead-animals" ? "zombie"
    : /^treant(?:-l[23])?$/.test(entity.id) ? "wood"
    : /wolf/.test(entity.id) ? "wolf"
    : entity.faction === "humans" && LIVING_HORSE_UNITS.has(entity.id) ? "horse"
    : HEAVY_ARMOR_UNITS.has(entity.id) ? "armor"
    : entity.faction === "merfolk" ? "merfolk"
    : entity.faction === "dwarves" ? "dwarf"
    : entity.faction === "elves" ? "elf"
    : entity.faction === "beasts" || /^(boar|greatboar|woodland-ancient-boar)$/.test(entity.id) ? "beast"
    : "flesh";
  return { group, sound: group === "beast" ? beastDeathId(selection.id, selection.take) : DEATH_SOUNDS[group] };
}
