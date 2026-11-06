// Explicit semantic families keep mechanical actions separate from animal voices.
// Recommendations are curated source IDs; alternatives include every matching recording.
export function buildActions(entity, catalog, { extras = false } = {}) {
  const find = (pack, pattern) =>
    catalog.filter((x) => x.pack === pack && pattern.test(x.name));
  const union = (...groups) => [
    ...new Map(groups.flat().map((x) => [x.id, x])).values(),
  ];
  const named = (items, pattern) => items.filter((x) => pattern.test(x.name));
  const material = (pattern) => find("impact-sounds", pattern);
  const rpg = (pattern) => find("rpg-audio", pattern);
  const fx = (pattern) => find("rubberduck/80-cc0-rpg-sfx", pattern);
  const creature = (pattern) => find("rubberduck/80-cc0-creature-sfx", pattern);
  const weapon = (pattern) =>
    find("still-north-media/weapon-textures", pattern);
  const contact = (pattern) =>
    find("still-north-media/weapon-impacts", pattern);
  const old = (family) => find("current", new RegExp(`^${family}-[123]$`));
  const row = (
    key,
    title,
    description,
    suggested,
    possible,
    { hypothetical = false, caveat = "" } = {},
  ) => {
    const all = union(suggested, possible);
    const recommended = new Set(suggested.map((x) => x.id));
    return {
      key,
      title,
      description,
      hypothetical,
      caveat,
      suggested: union(suggested).map((x) => x.id),
      alternatives: all.filter((x) => !recommended.has(x.id)).map((x) => x.id),
    };
  };
  const subtle = union(
    rpg(/^(cloth|beltHandle|handleSmallLeather)/),
    old("select"),
  );
  const stone = union(
    material(/^footstep_concrete|^impactMining/),
    fx(/^(stones|item_stone)/),
  );
  const wood = union(
    material(/^impactWood|^impactPlank/),
    rpg(/^creak|^chop/),
    fx(/^wood_|^item_wood/),
  );
  const metal = union(
    material(/^impactMetal|^impactPlate/),
    fx(/^metal_|^chain_/),
    rpg(/^metalClick$|^metalLatch$|^metalPot/),
  );
  const soft = material(/^impactSoft_|^impactPunch_/);
  const magic = union(
    fx(/^spell_[0-9]/),
    find("jaggedstone/magic-spell-sfx", /magical/),
  );
  const dark = find("qubodup/3-dark-magic-spells", /fout/);
  const fire = union(
    fx(/^spell_fire_/),
    find("antumdeluge/fire-crackling", /fire/),
  );
  const body = creature(/^(eat_|grunt_|hurt_|breath$)/);
  const ghostVoice = find("sorth/ghost-voice-pack", /output/);
  const cloth = rpg(/^cloth[1-4]$|^clothBelt/);
  if (entity.kind === "terrain") {
    const id = entity.id,
      water = /water|ford|marsh|landing|ferry/.test(id),
      forest = /forest|grove/.test(id),
      grass = /plains|forest|grove|graveyard/.test(id),
      building =
        /village|house|keep|fort|castle|monastery|church|ruins|cave/.test(id),
      wooden = /bridge|ferry|signpost/.test(id);
    const ground = water
      ? union(old("terrain-water"), creature(/^burble_/))
      : wooden
        ? union(material(/^footstep_wood/), rpg(/^creak/))
        : grass
          ? union(
              material(/^footstep_grass/),
              old(forest ? "terrain-forest" : "terrain-plains"),
            )
          : union(material(/^footstep_concrete/), stone);
    const family = water
      ? "terrain-water"
      : forest
        ? "terrain-forest"
        : /hill|mountain/.test(id)
          ? "terrain-hill"
          : /village|house/.test(id)
            ? "terrain-village"
            : building
              ? "terrain-keep"
              : "terrain-plains";
    let tap = old(family),
      tapPossible = union(tap, old("select"));
    if (wooden) tapPossible = union(tapPossible, wood);
    else if (building)
      tapPossible = union(
        tapPossible,
        material(/^impactWood_light|^impactMetal_light/),
        rpg(/^metalLatch$|^doorOpen|^doorClose/),
      );
    else if (forest) tapPossible = union(tapPossible, cloth);
    if (/ritual/.test(id)) {
      tap = fx(/^spell_01$/);
      tapPossible = union(tap, magic, dark, old("select"));
    }
    if (id === "castle-bell") {
      tap = material(/^impactBell_heavy_000$/);
      tapPossible = union(material(/^impactBell/), old("terrain-keep"));
    }
    const impact = wooden
      ? wood
      : building || /hill|mountain|road/.test(id)
        ? union(stone, wood)
        : water
          ? union(old("terrain-water"), creature(/^burble_/))
          : union(material(/^impactSoft_/), fx(/^stones_/));
    const result = [
      row(
        "tap",
        "Tap / tile selection",
        "A short cue when selecting this tile.",
        tap,
        tapPossible,
      ),
      row(
        "move",
        "Movement / surface contact",
        "The surface beneath a moving unit.",
        named(ground, /^footstep_.*_000$|^terrain-water-1$|^burble_01$/),
        ground,
        {
          caveat: water
            ? "Generated water cue and designed burble proxies; no field-recorded splashes in this collection."
            : "",
        },
      ),
      row(
        "hit",
        "Hit / contact",
        "A hypothetical object or ground impact; not a new game mechanic.",
        named(impact, /light_000$|stones_01$|terrain-water-1$|burble_01$/),
        impact,
        { hypothetical: true },
      ),
    ];
    if (/village|house|keep|fort|castle|monastery|church/.test(id))
      result.push(
        row(
          "capture",
          "Capture / interaction",
          "Coin or mechanism feedback for a capture or building interaction.",
          rpg(/^handleCoins$/),
          union(
            rpg(/^handleCoins|^doorOpen|^metalLatch$/),
            old("recruit"),
            old("terrain-village"),
          ),
          { hypothetical: true },
        ),
      );
    if (extras) {
      const ambience =
        id === "burned-village"
          ? find("antumdeluge/fire-crackling", /fire/)
          : water
            ? old("terrain-water")
            : forest
              ? old("terrain-forest")
              : /hill|mountain/.test(id)
                ? old("terrain-hill")
                : [];
      if (ambience.length)
        result.push(
          row(
            "ambience",
            "Ambience texture",
            "Short source textures to process into ambience; playback does not loop.",
            named(ambience, /fire-1$|-1$/),
            ambience,
            { hypothetical: true },
          ),
        );
    }
    return result.filter((x) => !x.hypothetical || extras);
  }
  const text = `${entity.id} ${entity.name} ${entity.role}`.toLowerCase();
  const faction = entity.faction;
  const mounted = /rider|knight|lancer|horse|deer-archer/.test(text);
  const horse =
    /horse|knight|lancer|skelet.*rider/.test(text) || entity.id === "rider";
  const canine = /wolf|hound/.test(text);
  const undead = /skeletons|undead-animals|zombies/.test(faction);
  const spirit = /ghost|spectre|wraith|lich/.test(text);
  const beast =
    ["beasts", "undead-animals", "cursed"].includes(faction) ||
    /wolf|boar|bear|treant|wildkeeper|grovewarden/.test(text);
  const slither = /snake|serpent|octopus|ray/.test(text);
  const flying =
    /bat|owl|raven|vulture|gryphon|dragon|wasp/.test(text) || spirit;
  const aquatic =
    /shark|ray|octopus|serpent|crocodile/.test(text) || faction === "merfolk";
  const caster = entity.attacks.some((a) =>
    /spell|spark|bolt|surge|light|root|orb|drain|burst|rune/i.test(a.name),
  );
  const armored =
    faction === "dwarves" ||
    /knight|champion|marshal|shield|guard|warden|iron|revenant/.test(text);
  const bone =
    (faction === "skeletons" && !/^(ghost|spectre|wraith)$/.test(entity.id)) ||
    (faction === "undead-animals" && /skeletal|bone-/.test(entity.id));
  const tree = /treant|wildkeeper|grovewarden/.test(text);
  const arrowBow = entity.attacks.some(
    (a) => /bow/i.test(a.name) && !/crossbow/i.test(a.name),
  );
  const skirmisher =
    /scout|ranger|shadowblade|flanker|bladedancer|jumper|slinger/.test(text);
  const frontline =
    !mounted &&
    !beast &&
    !spirit &&
    !caster &&
    !skirmisher &&
    !entity.attacks.some((a) => /bow|arbalest|pistol|arquebus/i.test(a.name)) &&
    entity.attacks.some((a) =>
      /sword|blade|spear|trident|halberd|axe|hammer|mace|club|pick/i.test(
        a.name,
      ),
    );
  const humanoid = !beast && !spirit;
  const materialStarter = bone
    ? fx(/^item_wood_03$/)
    : frontline
      ? entity.level <= 1
        ? rpg(/^beltHandle[12]$/)
        : fx(/^chain_0[123]$/)
      : humanoid
        ? rpg(/^cloth3$/)
        : [];
  const nocking = weapon(/^(English Longbow|Scythian Recurve) Nock Arrow$/);
  const bowHandling = weapon(
    /^(English Longbow|Scythian Recurve) (Nock Arrow|Set Arrow|Draw)$/,
  );

  const movement = mounted
    ? find("ezduzziteh/horse-trotting", /Trot/)
    : flying
      ? union(cloth, spirit ? old("unit-commander") : [])
      : aquatic
        ? union(old("terrain-water"), creature(/^burble_/))
        : slither
          ? union(cloth, old("terrain-forest"), old("terrain-plains"))
          : tree
            ? union(wood, material(/^footstep_wood/))
            : union(material(/^footstep_/), rpg(/^footstep/));
  const recommendedMove = mounted
    ? movement
    : flying
      ? named(movement, /cloth[12]$|unit-commander-1$/)
      : aquatic
        ? named(movement, /terrain-water-1$/)
        : slither
          ? named(movement, /^cloth[12]$/)
          : tree
            ? named(movement, /^creak1$|^impactWood_heavy_000$/)
            : named(
                movement,
                armored
                  ? /^footstep_concrete_00[01]$/
                  : /^footstep_grass_00[01]$/,
              );
  let tapPossible, tap;
  if (bone) {
    tapPossible = union(fx(/^item_wood_|^wood_|^chain_/), rpg(/^cloth/));
    tap = materialStarter;
  } else if (canine) {
    tapPossible = union(
      find("qubodup/dog-snarl-grunt-grumble", /dog/),
      creature(/^breath$|^grunt_/),
    );
    tap = find("qubodup/dog-snarl-grunt-grumble", /^dog-grumble$/);
  } else if (tree) {
    tapPossible = union(wood, old("terrain-forest"));
    tap = rpg(/^creak1$/);
  } else if (beast) {
    tapPossible = union(body, material(/^impactSoft_medium/));
    tap = creature(/^breath$/);
  } else if (spirit) {
    tapPossible = union(cloth, old("unit-commander"));
    tap = rpg(/^cloth1$/);
  } else {
    tapPossible = union(
      subtle,
      armored ? union(metal, old("unit-soldier"), old("unit-archer")) : [],
      caster ? rpg(/^book/) : [],
      weapon(/^.* (Draw|Sheath) (Fast|Slow)$/),
    );
    tap = caster
      ? rpg(/^bookFlip1$/)
      : armored
        ? rpg(/^metalClick$/)
        : rpg(/^handleSmallLeather$/);
  }
  if (!bone && humanoid) {
    tapPossible = union(
      tapPossible,
      fx(/^chain_/),
      rpg(/^beltHandle[12]$|^cloth3$/),
    );
    tap = arrowBow ? nocking : materialStarter;
    if (arrowBow) {
      tapPossible = union(tapPossible, bowHandling);
      if (bone) tap = union(tap, nocking);
    }
  }
  if (arrowBow) {
    tapPossible = union(tapPossible, bowHandling);
    if (bone) tap = union(tap, nocking);
  }
  const result = [
    row(
      "tap",
      "Tap / selection",
      beast
        ? "Brief body or idle feedback when selecting this creature."
        : arrowBow
          ? "Arrow nocking / handling for selection."
          : bone
            ? "Dry item-wood contact for skeletal selection."
            : frontline
              ? entity.level <= 1
                ? "Belt handling for level-one infantry."
                : "Chain movement for armored infantry."
              : "Soft cloth movement for lighter units.",
      tap,
      tapPossible,
      {
        caveat: canine
          ? "Domestic dog reference, not a wolf field recording."
          : "",
      },
    ),
    row(
      "move",
      "Movement",
      mounted
        ? "Mounted movement."
        : flying
          ? "Wing or spectral movement textures."
          : aquatic
            ? "Water movement textures."
            : "Foot contact, with alternatives for different surfaces.",
      recommendedMove,
      movement,
      {
        caveat: mounted
          ? "Trot is Foley; deer/goat uses a hoof-movement proxy."
          : flying
            ? "Cloth is a wing-motion proxy; no authentic wing recordings."
            : aquatic
              ? "Generated water and designed textures; no recorded swim cycle."
              : "",
      },
    ),
  ];
  for (const attack of entity.attacks) {
    const name = attack.name.toLowerCase();
    let suggested = [],
      possible = [],
      note = "",
      caveat = "";
    if (/arquebus|pistol|musket|gunshot/.test(name)) {
      note = "Firearm shot";
      caveat =
        "No actual gunshot recording is present. Enable hypothetical extras for mechanism/reload candidates. No unrelated impacts are substituted for a shot.";
    } else if (/crossbow|arbalest/.test(name)) {
      possible = union(
        weapon(/^Crossbow Shoot$|^Crossbow Bolt (Pass|Passby|Approach)$/),
        old("attack-bow"),
      );
      suggested = weapon(/^Crossbow Shoot$/);
      note = "Recorded crossbow releases, with flight layers as alternatives.";
    } else if (/bow/.test(name)) {
      possible = union(
        weapon(
          /^English Longbow Shoot$|^Scythian Recurve Shoot$|Arrow (Pass|Passby|Approach)$/,
        ),
        old("attack-bow"),
      );
      possible = union(possible, bowHandling);
      suggested = nocking;
      note =
        "Arrow nocking as a quieter attack-feedback candidate. This depicts handling rather than the release; shot and flight recordings remain alternatives.";
    } else if (/fire/.test(name)) {
      possible = fire;
      suggested = fx(/^spell_fire_0[12]$/);
      note = "Designed fire attack, with crackle layers as alternatives.";
    } else if (/grave|corruption|mire|soul/.test(name)) {
      possible = union(dark, magic, fx(/^creature_slime/));
      suggested = named(dark, /fout-01$/);
      note = "Dark casting textures; slime is a wet layer candidate.";
    } else if (
      /spark|light|bolt|surge|orb|burst|rune|spell|crushing (wave|root)/.test(
        name,
      )
    ) {
      possible = union(magic, /spectral/.test(name) ? dark : []);
      suggested = fx(/^spell_0[12]$/);
      note =
        "Designed spell effects; tonal casts are available as alternatives.";
    } else if (/sword|rustblade|knife|blade|cleaver|scythe/.test(name)) {
      possible = union(
        fx(/^blade_/),
        rpg(/^knifeSlice/),
        weapon(/^(Sabre|Katana|Dagger|Seax) Swing$/),
        /scythe|cleaver/.test(name) ? weapon(/^Axe Swing$/) : [],
        contact(
          /Blade on Blade|Sword.*Blade|Sabre.*Blade|Katana.*Blade|Seax.*Blade|Dagger.*Blade/,
        ),
        old("attack-melee"),
      );
      suggested = fx(/^blade_0[123]$/);
      note =
        "Short blade effects first; recorded blade swings remain alternatives.";
    } else if (/poleaxe|halberd/.test(name)) {
      possible = union(
        weapon(/^(Axe|Spear) Swing$/),
        contact(/Axe|Spear/),
        old("attack-melee"),
      );
      suggested = weapon(/^Axe Swing$/);
      note = "Axe / polearm movement recordings.";
    } else if (/spear|lance|trident|harpoon/.test(name)) {
      possible = union(
        weapon(/^Spear Swing$/),
        contact(/Spear/),
        old("attack-melee"),
        /harpoon/.test(name) ? weapon(/^Crossbow Bolt (Pass|Approach)$/) : [],
      );
      suggested = weapon(/^Spear Swing$/);
      note = "Recorded spear movement; harpoon flight is a projectile proxy.";
    } else if (/axe/.test(name)) {
      possible = union(
        weapon(/^(Axe|Sabre) Swing$/),
        contact(/Axe/),
        old("attack-melee"),
      );
      suggested = weapon(/^Axe Swing$/);
      note = "Recorded axe swing.";
    } else if (/mace|hammer|club|pickaxe/.test(name)) {
      possible = union(
        contact(/^Mace /),
        material(/^impactMining|^impactMetal_heavy|^impactWood_heavy/),
      );
      suggested = /pickaxe/.test(name)
        ? material(/^impactMining_000$/)
        : contact(/^Mace Spear Blade$/);
      note =
        "Blunt or mining contacts; these are contact layers, not a complete attack sequence.";
    } else if (/staff|bough/.test(name)) {
      possible = wood;
      suggested = material(/^impactWood_medium_00[01]$/);
      note = "Wood contact for a staff or bough.";
    } else if (/shield/.test(name)) {
      possible = union(material(/^impactPlate|^impactWood/), contact(/Haft/));
      suggested = material(/^impactPlate_medium_00[01]$/);
      note = "Shield contact materials.";
    } else if (/sling|thorn|spike|shard|stone|weighted net/.test(name)) {
      possible = union(
        weapon(/Arrow (Pass|Approach)$|Bolt (Pass|Approach)$/),
        fx(/^stones_|^item_stone/),
      );
      suggested = /stone/.test(name)
        ? fx(/^stones_01$/)
        : weapon(/^Scythian Recurve Arrow Pass$/);
      note =
        "Projectile flight or stone contact; texture proxies for this weapon.";
    } else if (
      /claw|fang|bite|beak|talon|sting|pincer|tusk|grasp|trample|hoof|antler|tail|tentacle/.test(
        name,
      )
    ) {
      possible = union(soft, creature(/^eat_/));
      suggested = material(/^impactSoft_medium_00[01]$/);
      note =
        "Bodily contact layers; designed eating textures are alternatives.";
      caveat = "Not an authentic recording of an animal bite or claw strike.";
    } else {
      note = "Proposed physical contact";
      possible = soft;
      suggested = material(/^impactSoft_medium_000$/);
    }
    result.push(
      row(
        `attack-${attack.id}`,
        `Attack · ${attack.name}`,
        note,
        suggested,
        possible,
        {
          caveat:
            (attack.proposed
              ? "Proposed attack from artwork / role, not game-defined. "
              : "") + caveat,
        },
      ),
    );
  }
  const hitPossible = union(
    materialStarter,
    humanoid ? union(fx(/^chain_/), rpg(/^beltHandle[12]$|^cloth3$/)) : [],
    tree
      ? wood
      : bone
        ? union(wood, fx(/^chain_|^metal_/), soft)
        : armored
          ? union(metal, soft)
          : union(soft, undead ? creature(/^hurt_/) : []),
  );
  const hitSuggested = materialStarter.length
    ? materialStarter
    : tree
      ? material(/^impactWood_light_00[01]$/)
      : armored
        ? material(/^impactMetal_medium_00[01]$/)
        : material(/^impactSoft_medium_00[01]$/);
  result.push(
    row(
      "hit",
      "Hit / taking damage",
      bone
        ? "Item wood 03 as a dry skeletal hit starter."
        : frontline
          ? entity.level <= 1
            ? "Belt handling as the level-one infantry hit starter."
            : "Chain movement as the armored infantry hit starter."
          : humanoid
            ? "Cloth 3 as a soft hit-feedback starter; impacts remain alternatives."
            : "Contact when this unit receives a hit; material follows the target.",
      hitSuggested,
      hitPossible,
      { caveat: bone ? "Wood is a dry bone-contact proxy." : "" },
    ),
  );
  if (extras) {
    let vocals = [];
    if (canine)
      vocals = union(
        find("qubodup/dog-snarl-grunt-grumble", /dog/),
        creature(/^howl$|^barking/),
      );
    else if (horse) vocals = find("denischardonnet/horse-vocals", /028/);
    else if (spirit) vocals = ghostVoice;
    else if (faction === "zombies")
      vocals = union(
        creature(/^grunt_|^cough_|^breath$|^hurt_/),
        fx(/^creature_(hurt|die|monster)/),
      );
    else if (beast) vocals = creature(/^(roar_|grunt_|bug_|breath$|hurt_)/);
    if (vocals.length)
      result.push(
        row(
          "voice",
          "Voice / reaction",
          "Optional idle or reaction voice.",
          named(vocals, /dog-growl$|0284$|output \(1\)$|roar_01$|grunt_01$/),
          vocals,
          {
            hypothetical: true,
            caveat: canine
              ? "Natural dog and designed canine effects; not authentic wolf calls."
              : beast
                ? "Designed creature voices unless explicitly marked natural."
                : "",
          },
        ),
      );
    const deaths = union(
      material(/^impactSoft_heavy/),
      rpg(/^dropLeather$/),
      undead || beast
        ? union(creature(/^hurt_|^scream_/), fx(/^creature_die/))
        : [],
    );
    result.push(
      row(
        "death",
        "Death / fall",
        "Optional fall/contact layer. No human death voice is claimed.",
        material(/^impactSoft_heavy_000$/),
        deaths,
        { hypothetical: true },
      ),
    );
    if (
      entity.attacks.some((a) => /bow|arbalest|arquebus|pistol/i.test(a.name))
    ) {
      const firearm = entity.attacks.some((a) =>
        /arquebus|pistol/i.test(a.name),
      );
      const crossbow = entity.attacks.some((a) =>
        /crossbow|arbalest/i.test(a.name),
      );
      const reload = firearm
        ? union(
            rpg(/^metalClick$|^metalLatch$|^metalPot/),
            weapon(/^Crossbow Lever Trigger$|^Crossbow Set Nut Against Sear$/),
          )
        : crossbow
          ? weapon(/^Crossbow (Lever|Place|Set)/)
          : weapon(
              /^(English Longbow|Scythian Recurve) (Draw|Nock Arrow|Set Arrow)$/,
            );
      const suggestedReload = firearm
        ? rpg(/^metalClick$/)
        : crossbow
          ? weapon(/^Crossbow Place Bolt$/)
          : weapon(/^English Longbow Nock Arrow$/);
      result.push(
        row(
          "reload",
          "Reload / ready weapon",
          "Optional weapon-handling layer; crossbow mechanisms are proxies for firearms.",
          suggestedReload,
          reload,
          { hypothetical: true },
        ),
      );
    }
  }
  return result;
}
