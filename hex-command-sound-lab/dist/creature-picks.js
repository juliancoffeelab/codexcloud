// Authored selection palettes; these are textures/proxies, not species recordings.
export function creaturePick(entity) {
  const id = entity.id;
  const grunt = (n) => `rubberduck/80-cc0-creature-sfx/grunt_0${n}.ogg`;
  const cloth = (n) => `rpg-audio/cloth${n}`;
  const pick = (sound, note, move = false, alternatives = []) => ({
    sound,
    note,
    move,
    alternatives,
  });
  if (/^(bone-dragon|dragon|giant-bat)(?:-l[23])?$/.test(id))
    return pick(cloth(2), "Cloth 2 as leathery wing handling.", true, [
      grunt(4),
    ]);
  if (/^(giant-owl|giant-raven|gryphon|corpse-vulture)(?:-l[23])?$/.test(id))
    return pick(cloth(3), "Cloth 3 as feather and wing handling.", true);
  if (/^(giant-snake|sea-serpent)(?:-l[23])?$/.test(id))
    return pick(
      "edits/snake-belt-soft-v1",
      "Quiet, lightly distorted Merfolk belt texture for snake selection.",
      true,
      ["rpg-audio/clothBelt2"],
    );
  if (/^octopus(?:-l[23])?$/.test(id))
    return pick(
      "edits/octopus-gurgle-soft-v1",
      "Soft gurgle for octopus handling.",
      true,
      ["rubberduck/80-cc0-creature-sfx/burble_02.ogg"],
    );
  if (/^ray(?:-l[23])?$/.test(id))
    return pick(
      "rubberduck/80-cc0-creature-sfx/alien_02.ogg",
      "Alien 2 for ray selection.",
      false,
      [grunt(1)],
    );
  if (/^giant-spider(?:-l[23])?$/.test(id))
    return pick(
      "rubberduck/80-cc0-creature-sfx/alien_04.ogg",
      "Alien 4 as the spider selection starter.",
      true,
      [
        "edits/spider-creak-soft-v1",
        "rubberduck/80-cc0-creature-sfx/bug_01.ogg",
        "rubberduck/80-cc0-creature-sfx/bug_02.ogg",
        "rubberduck/80-cc0-creature-sfx/bug_03.ogg",
        "rubberduck/80-cc0-creature-sfx/bug_04.ogg",
        "rpg-audio/creak1",
        "rpg-audio/creak2",
      ],
    );
  if (/^giant-wasp(?:-l[23])?$/.test(id))
    return pick(
      "rubberduck/80-cc0-creature-sfx/alien_03.ogg",
      "Alien 3 as the wasp selection starter.",
      true,
      [
        "rubberduck/80-cc0-creature-sfx/bug_01.ogg",
        "rubberduck/80-cc0-creature-sfx/bug_02.ogg",
        "rubberduck/80-cc0-creature-sfx/bug_03.ogg",
        "rubberduck/80-cc0-creature-sfx/bug_04.ogg",
        "still-north-media/weapon-textures/part-1/Arrow Fletching.wav",
      ],
    );
  if (
    /^(basilisk|crocodile|giant-crab|giant-tortoise|zombie-bear)(?:-l[23])?$/.test(
      id,
    )
  )
    return pick(
      grunt(4),
      "Grunt 4 for heavy, armored or monstrous creatures.",
      false,
      [grunt(1)],
    );
  if (
    /^(bear|lynx|stag|shark|ray)(?:-l[23])?$/.test(id) ||
    /^(boar|greatboar|woodland-ancient-boar)$/.test(id)
  )
    return pick(
      grunt(1),
      "Grunt 1 as a compact creature response proxy.",
      false,
      [grunt(4)],
    );
  return null;
}
