import { deathPick } from "./death-picks.js?v=death-four-1";
// Authored audition choices. Death rows are candidates, not live-game events.
export function applyCuratedPicks(entity, actions, catalog, { bone, spirit }) {
  const byId = new Map(catalog.map((sound) => [sound.id, sound]));
  const choose = (key, ids, note) => {
    let action = actions.find((row) => row.key === key);
    if (!action) {
      action = {
        key,
        title: key === "death" ? "Death candidate" : key,
        suggested: [],
        alternatives: [],
        description: note,
        caveat: "",
        hypothetical: true,
      };
      actions.push(action);
    }
    const previous = [...action.suggested, ...action.alternatives];
    action.suggested = ids.filter((id) => byId.has(id));
    action.alternatives = [...new Set(previous)].filter(
      (id) => !action.suggested.includes(id),
    );
    action.description = note;
    return action;
  };
  if (spirit && !bone) {
    for (const key of ["tap", "voice"]) {
      if (key === "voice" && !actions.some((row) => row.key === key)) continue;
      const row = choose(
        key,
        ["edits/ghost-grunt-02-trim-v1"],
        "The tonal body of Grunt 2, with its scratchy ending removed.",
      );
      row.caveat = "Trimmed designed creature voice as a ghost proxy.";
    }
  }
  const deathChoice = deathPick(entity, { bone, spirit });
  const death = choose("death", [deathChoice.sound], `Shared ${deathChoice.group} death audition.`);
  death.title = "Death";
  death.deathGroup = deathChoice.group;
  death.alternatives = [];
  death.hypothetical = true;
  death.caveat = "Shared soundboard death audition; no live-game death integration.";
  if (/^merfolk-maceguard(?:-l[23])?$/.test(entity.id)) {
    const id = "still-north-media/weapon-impacts/part-1/Mace Spear Haft.wav";
    const row = choose("attack-mace", [id],
      "Mace Spear Haft, take 12: a dull contact for the stone-headed Merfolk mace.");
    row.preferredTakes = { [id]: 11 };
  }
  const creatureSound = (name) => `rubberduck/80-cc0-creature-sfx/${name}.ogg`;
  const roar = creatureSound("roar_03");
  const tap = actions.find((row) => row.key === "tap");
  const isBeast = entity.key?.startsWith("beasts/") || /^(boar|greatboar|woodland-ancient-boar)$/.test(entity.id);
  const originalGrunt = tap?.suggested.find((id) => /\/grunt_0[14]\.ogg$/.test(id));
  if (isBeast && originalGrunt) {
    choose("tap", [roar], "Roar 3 for creature selection; its former grunt moves to attacks.");
    if (actions.some(row => row.key === "voice"))
      choose("voice", [roar], "Roar 3 creature response.");
  }
  if (/^(?:fox-spearguard|treant)(?:-l[23])?$/.test(entity.id))
    choose("tap", [roar], "Roar 3 for Thornshield and living tree selection.");
  if (/^(wildkeeper|grovewarden)(?:-l[23])?$/.test(entity.id))
    choose("tap", ["rpg-audio/cloth3"], "Cloth 3 for Wildkeeper-line mage selection.");
  for (const attack of entity.attacks || []) {
    const key = `attack-${attack.id}`;
    const name = attack.name.toLowerCase();
    if (/tail.*strike/i.test(attack.name)) {
      choose(key, ["kyles/fish-slap/fish_drop1.ogg"], "Short recorded fish slap for Tail Strike.");
    } else if ((entity.faction === "zombies" || /^zombie-bear/.test(entity.id)) && /grasp|hands/.test(name)) {
      choose(key, [creatureSound("burp_02")], "Burp 2 for the zombie grasping-hands attack.");
    } else if ((entity.faction === "zombies" || /^zombie-bear/.test(entity.id)) && /bite/.test(name)) {
      choose(key, [creatureSound("grunt_05")], "Grunt 5 for the zombie bite.");
    } else if (/claw/.test(name)) {
      choose(key, ["edits/knife-slice-2-no-hit-v1"], "Knife Slice 2 swish with its opening hit removed.");
    } else if (/wolf/.test(entity.id) && /bite|fang/.test(name)) {
      choose(key, ["qubodup/dog-snarl-grunt-grumble/dog/dog-snarl.flac"], "Snarl for wolf and werewolf bites.");
    } else if (entity.faction === "elves" && /^(elf-flanker(?:-l[23])?|elf-jumper-l[23])$/.test(entity.id) && attack.id === "blade") {
      const jumper = /^elf-jumper/.test(entity.id);
      const row = choose(key, [1,2,3].map(n => `rubberduck/80-cc0-rpg-sfx/blade_0${n}.ogg`),
        jumper ? "Jumper spear attack using the current spear palette." : "Knife attack; excluded from the sword/spear swap.");
      row.title = `Attack · ${jumper ? "Spear" : "Knives"}`;
      row.attackName = jumper ? "Spear" : "Knives";
      row.damageType = jumper ? "pierce" : "blade";
    } else if (attack.id === "spectral-touch") {
      choose(key, ["rubberduck/80-cc0-rpg-sfx/spell_02.ogg"], "Spell 02 for magical Spectral Touch.");
    } else if (attack.id === "root-crush") {
      choose(key, ["rubberduck/80-cc0-rpg-sfx/spell_02.ogg"], "Spell 02 for Crushing Roots.");
    } else if (/axe|halberd/.test(name) && !/pickaxe/.test(name)) {
      choose(key, ["edits/knife-slice-2-no-hit-v1"],
        "The werewolf claw swish for axes, poleaxes and halberds; pickaxes keep mining.");
    } else if (["humans", "elves"].includes(entity.faction) && /^(sword|blade)$/.test(name)) {
      choose(key, ["edits/spear-sabre-one-shot-v1"], "Sword attack using the former spear contact.");
    } else if (["humans", "elves"].includes(entity.faction) && /^(spear|lance)$/.test(name)) {
      choose(key, [1,2,3].map(n => `rubberduck/80-cc0-rpg-sfx/blade_0${n}.ogg`),
        "Spear attack using the former sword blade sounds.");
    } else if (/^skeleton-rider(?:-l[23])?$/.test(entity.id) && /spear/.test(name)) {
      choose(key, [1,2,3].map(n => `rubberduck/80-cc0-rpg-sfx/blade_0${n}.ogg`),
        "Skeleton rider spear attack using the current spear palette.");
    } else if (entity.faction === "humans" && /mace|hammer/.test(name)) {
      const id = "still-north-media/weapon-impacts/part-1/Mace Spear Haft.wav";
      const row = choose(key, [id], "Mace Spear Haft, take 12, shared with Merfolk macers.");
      row.preferredTakes = { [id]: 11 };
    } else if (entity.faction === "dwarves" && /axe|hammer|spear|shield/.test(name)) {
      choose(key, ["impact-sounds/impactMining_001"], "Mining 001 as the dwarven physical melee contact.");
    } else if (/arquebus|pistol|musket|gunshot/i.test(attack.name)) {
      const row = choose(key, ["impact-sounds/impactPlate_medium_000"],
        "Impact Plate medium 000 as the chosen firearm shot proxy.");
      row.caveat = "Metal impact proxy selected for this attack.";
    } else if (/^treant(?:-l[23])?$/.test(entity.id)) {
      choose(key, ["impact-sounds/impactPlank_medium_004"],
        "Impact Plank medium 004 for treant attacks.");
    } else if (entity.faction === "undead-animals" && !/fire|magic|spell/i.test(attack.name)) {
      const sound = tap?.suggested[0];
      if (sound) {
        const row = choose(key, [sound], "Undead animal physical attack reuses its skeletal, zombie or wing selection texture.");
        row.caveat = "Audition proxy for the undead animal attack.";
      }
    } else if (isBeast && !/fire|magic|spell/i.test(attack.name)) {
      const sound = /^giant-(wasp|spider)(?:-l[23])?$/.test(entity.id)
        ? creatureSound("spit_01")
        : originalGrunt || tap?.suggested[0];
      if (sound) {
        const row = choose(key, [sound], originalGrunt
          ? "Former selection grunt used for the creature attack."
          : "Creature attack placeholder using its selection texture; bugs use Spit 01.");
        row.caveat = "Audition proxy for a creature attack.";
      }
    }
  }
  const movement = actions.find((row) => row.key === "move");
  if (
    movement &&
    !movement.alternatives.includes("rubberduck/80-cc0-rpg-sfx/wood_04.ogg")
  )
    movement.alternatives.push("rubberduck/80-cc0-rpg-sfx/wood_04.ogg");
  // Rejected near-silent swing sessions remain in the library, not action suggestions.
  for (const action of actions) {
    const usable = (id) =>
      !(
        byId.get(id)?.pack === "still-north-media/weapon-textures" &&
        / Swing$/.test(byId.get(id).name)
      );
    action.suggested = action.suggested.filter(usable);
    action.alternatives = action.alternatives.filter(usable);
  }
}
