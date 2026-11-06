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
