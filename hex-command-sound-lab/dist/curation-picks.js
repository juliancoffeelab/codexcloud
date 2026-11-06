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
  if (bone) {
    const death = choose(
      "death",
      ["rubberduck/80-cc0-creature-sfx/eat_01.ogg"],
      "Eat 1 as a proposed skeletal breakup/death texture.",
    );
    death.caveat =
      "Designed eating texture used as a skeletal proxy; the live game has no separate death cue.";
  } else if (entity.id === "knight") {
    const death = choose(
      "death",
      ["rpg-audio/metalPot3"],
      "Metal pot 3 as a proposed armored knight fall/death contact.",
    );
    death.caveat =
      "Equipment Foley proxy, not a recorded armored fall; no live-game death integration.";
  }
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
  if (/^(?:fox-spearguard|treant)(?:-l[23])?$|^(wildkeeper|grovewarden)$/.test(entity.id))
    choose("tap", [roar], "Roar 3 for Thornshield and living tree selection.");
  for (const attack of entity.attacks || []) {
    const key = `attack-${attack.id}`;
    if (/arquebus|pistol|musket|gunshot/i.test(attack.name)) {
      const row = choose(key, ["impact-sounds/impactPlate_medium_000"],
        "Impact Plate medium 000 as the chosen firearm shot proxy.");
      row.caveat = "Metal impact proxy selected for this attack.";
    } else if (/^treant(?:-l[23])?$/.test(entity.id)) {
      choose(key, ["impact-sounds/impactPlank_medium_004"],
        "Impact Plank medium 004 for treant attacks.");
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
