import { applyCuratedPicks } from "./curation-picks.js?v=tree-selection-1";
import { weaponSelection } from "./weapon-selection.js?v=curation-1";
import { creaturePick } from "./creature-picks.js?v=terrain-recut-2";
import { terrainPicks } from "./terrain-picks.js?v=terrain-recut-2";
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
  const edited = (name) =>
    catalog.filter((sound) => sound.id === `edits/${name}`);
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
  const cloth = rpg(/^cloth[2-4]$|^clothBelt/);
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
    const surfaceSuggestion = named(
      ground,
      /^footstep_.*_000$|^terrain-water-1$|^burble_01$/,
    );
    const surfacePossible = ground;
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
        surfaceSuggestion,
        surfacePossible,
        {
          caveat:
            id === "plains"
              ? "Processed audition: less bass and intentionally quieter. Match levels does not boost it back up; the unprocessed grass 003 stays in alternatives."
              : water
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
    const preference = terrainPicks[id] || {};
    const families = {
      stone,
      wood,
      metal,
      magic,
      dark,
      fire,
      water: old("terrain-water"),
      burble: creature(/^burble_/),
      doors: rpg(/^doorOpen|^doorClose/),
      mining: material(/^impactMining/),
      bell: material(/^impactBell/),
    };
    for (const action of result) {
      const pick = preference[action.key];
      if (!pick) continue;
      const suggestions = pick.suggested
        .map((id) => catalog.find((item) => item.id === id))
        .filter(Boolean);
      const originalVersions = suggestions
        .filter((item) => item.derivedFrom)
        .map((item) => catalog.find((source) => source.id === item.derivedFrom))
        .filter(Boolean);
      const matches = union(
        [...action.suggested, ...action.alternatives].map((id) =>
          catalog.find((item) => item.id === id),
        ),
        ...pick.families.map((name) => families[name] || []),
        suggestions,
        originalVersions,
      );
      const suggestedIds = new Set(suggestions.map((x) => x.id));
      action.suggested = suggestions.map((x) => x.id);
      action.alternatives = matches
        .filter((x) => !suggestedIds.has(x.id))
        .map((x) => x.id);
      action.description = pick.note;
      if (suggestions.some((x) => x.derivedFrom))
        action.caveat =
          "Quiet processed audition. Match levels keeps its intended level. Expand alternatives to compare with the unprocessed source.";
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
  const bear = /^bear(?:-l[23])?$/.test(entity.id);
  const heavyHorror =
    tree ||
    (faction === "zombies" &&
      /^zombie-(cleaver|plague-carrier|shambler|ghoul)$/.test(entity.id));
  const vocalStarter = heavyHorror
    ? creature(/^grunt_04$/)
    : bear
      ? creature(/^grunt_01$/)
      : [];
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
    tap = rpg(/^cloth2$/);
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
  if (faction === "merfolk") {
    tap = rpg(/^clothBelt2$/);
    tapPossible = union(tapPossible, cloth);
  }
  if (vocalStarter.length) {
    tapPossible = union(tapPossible, tap, vocalStarter);
    tap = vocalStarter;
  }
  const result = [
    row(
      "tap",
      "Tap / selection",
      vocalStarter.length
        ? heavyHorror
          ? "Grunt 4 for large tree creatures and zombie horrors."
          : "Grunt 1 for bear selection."
        : faction === "merfolk"
          ? "Cloth belt handling for Merfolk selection."
          : beast
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
      "A single board-piece click when the figurine finishes moving.",
      old("select"),
      [],
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
      suggested = weapon(/^Crossbow Shoot$|^Crossbow Bolt Passby$/);
      note = "Recorded crossbow releases, with flight layers as alternatives.";
    } else if (/bow/.test(name)) {
      possible = union(
        weapon(
          /^English Longbow Shoot$|^Scythian Recurve Shoot$|Arrow (Pass|Passby|Approach)$/,
        ),
        old("attack-bow"),
      );
      possible = union(possible, bowHandling);
      suggested = union(
        weapon(/^Scythian Recurve Shoot$/),
        weapon(/^English Longbow Shoot$/),
      );
      note =
        "Recorded bow release; arrow flight and nocking remain alternatives.";
    } else if (/fire/.test(name)) {
      possible = fire;
      suggested = fx(/^spell_fire_07$/);
      note = "Designed fire attack, with crackle layers as alternatives.";
    } else if (/grave|corruption|mire|soul/.test(name)) {
      possible = union(dark, magic, fx(/^creature_slime/));
      suggested = edited("fout-01-short-v1");
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
      suggested = edited("axe-katana-one-shot-v1");
      note =
        "A single approved axe/katana blade contact; swing sessions are excluded.";
    } else if (/spear|lance|trident|harpoon/.test(name)) {
      possible = union(
        weapon(/^Spear Swing$/),
        contact(/Spear/),
        old("attack-melee"),
        /harpoon/.test(name) ? weapon(/^Crossbow Bolt (Pass|Approach)$/) : [],
      );
      suggested = edited("spear-sabre-one-shot-v1");
      note =
        "A single approved spear/sabre contact; harpoon flight stays a proxy alternative.";
    } else if (/axe/.test(name)) {
      possible = union(
        weapon(/^(Axe|Sabre) Swing$/),
        contact(/Axe/),
        old("attack-melee"),
      );
      suggested = edited("axe-katana-one-shot-v1");
      note =
        "A single approved axe/katana contact instead of the near-silent swing.";
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
      suggested = /staff/.test(name)
        ? fx(/^wood_05$/)
        : material(/^impactWood_medium_00[01]$/);
      note = /staff/.test(name)
        ? "Wood 05 for the physical staff contact, separate from casting."
        : "Wood contact for a bough.";
    } else if (/shield/.test(name)) {
      possible = union(material(/^impactPlate|^impactWood/), contact(/Haft/));
      suggested = material(/^impactPlate_medium_00[01]$/);
      note = "Shield contact materials.";
    } else if (/sling|thorn|spike|shard|stone|weighted net/.test(name)) {
      possible = union(
        weapon(/Arrow (Pass|Approach)$|Bolt (Pass|Approach)$/),
        fx(/^stones_|^item_stone/),
      );
      suggested = /sling/.test(name)
        ? rpg(/^bookClose$/)
        : /stone/.test(name)
          ? fx(/^stones_01$/)
          : weapon(/^Scythian Recurve Arrow Pass$/);
      note = /sling/.test(name)
        ? "Book close as a sling-release texture proxy."
        : "Projectile flight or stone contact; texture proxies for this weapon.";
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
    else if (horse || mounted)
      vocals = find("denischardonnet/horse-vocals", /028/);
    else if (spirit) vocals = ghostVoice;
    else if (faction === "zombies")
      vocals = union(
        creature(/^grunt_|^cough_|^breath$|^hurt_/),
        fx(/^creature_(hurt|die|monster)/),
      );
    else if (beast) vocals = creature(/^(roar_|grunt_|bug_|breath$|hurt_)/);
    if (!heavyHorror) vocals = vocals.filter((x) => x.name !== "grunt_04");
    vocals = union(vocalStarter, vocals);
    if (vocals.length)
      result.push(
        row(
          "voice",
          "Voice / reaction",
          "Optional idle or reaction voice.",
          vocalStarter.length
            ? vocalStarter
            : named(
                vocals,
                /dog-growl$|0284$|output \(1\)$|roar_01$|grunt_01$/,
              ),
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
  const palette = creaturePick(entity);
  const ids = (values) => catalog.filter((x) => values.includes(x.id));
  for (const action of result) {
    const excluded = (id) => {
      const sound = catalog.find((x) => x.id === id);
      return (
        sound?.name === "cloth1" ||
        (faction !== "zombies" && sound?.name === "breath")
      );
    };
    action.suggested = action.suggested.filter((id) => !excluded(id));
    action.alternatives = action.alternatives.filter((id) => !excluded(id));
    if (!palette) continue;
    if (action.key === "tap") {
      const previous = [...action.suggested, ...action.alternatives];
      action.suggested = ids([palette.sound]).map((x) => x.id);
      action.alternatives = [
        ...new Set([...previous, ...palette.alternatives]),
      ].filter((id) => id !== palette.sound && !excluded(id));
      action.description = palette.note;
      action.caveat = "Audition texture, not an authentic species recording.";
    }
    if (action.key === "voice" && !/(grunt_|alien_)/.test(palette.sound)) {
      action.suggested = [];
      action.caveat =
        "No species-specific voice selected; selection uses a handling texture.";
    } else if (action.key === "voice") {
      action.alternatives = [
        ...new Set([...action.suggested, ...action.alternatives]),
      ].filter((id) => id !== palette.sound && !excluded(id));
      action.suggested = [palette.sound];
    }
  }
  const selection = weaponSelection(entity);
  if (selection && !mounted && !horse && !bone && faction !== "zombies") {
    const action = result.find((row) => row.key === "tap");
    const previous = [...action.suggested, ...action.alternatives];
    action.suggested = find(selection.pack, selection.suggested).map(
      (item) => item.id,
    );
    action.alternatives = [
      ...new Set([
        ...previous,
        ...find(selection.pack, selection.alternatives).map((item) => item.id),
      ]),
    ].filter((id) => !action.suggested.includes(id));
    action.description = selection.note;
    action.caveat =
      "Weapon/equipment handling for selection; attack and hit cues stay separate.";
  }
  if ((mounted || horse) && !bone) {
    const mountVoice = edited("horse-0286-trim-v1");
    for (const action of result) {
      if (!["tap", "voice"].includes(action.key)) continue;
      const previous = [...action.suggested, ...action.alternatives];
      action.suggested = mountVoice.map((item) => item.id);
      action.alternatives = [
        ...new Set([
          ...previous,
          ...find("denischardonnet/horse-vocals", /028/).map((item) => item.id),
        ]),
      ].filter((id) => !action.suggested.includes(id));
      action.description = "0286 horse burst with its quiet lead-in trimmed.";
      action.caveat = horse
        ? "Natural horse recording; undead mounts use it as a proxy."
        : "Natural horse vocal used as a mount proxy for deer/goat riders.";
    }
  }
  if (bone || faction === "zombies") {
    const starter = bone
      ? fx(/^item_wood_03$/)
      : heavyHorror
        ? creature(/^grunt_04$/)
        : creature(/^breath$/);
    for (const action of result) {
      if (action.key !== "tap" && action.key !== "voice") continue;
      const previous = [...action.suggested, ...action.alternatives];
      action.suggested =
        bone && action.key === "voice" ? [] : starter.map((item) => item.id);
      action.alternatives = [
        ...new Set([...previous, ...starter.map((item) => item.id)]),
      ].filter((id) => !action.suggested.includes(id));
      action.description = bone
        ? "Dry skeletal item-wood contact for selection."
        : heavyHorror
          ? "Grunt 4 for bulky zombie melee/carrier bodies."
          : "Breath for ordinary zombies and fire-holding casters.";
      action.caveat = bone
        ? "Wood is a bone-contact proxy, not a creature voice."
        : "Designed creature vocal; fire-holding casters keep breath regardless of size.";
    }
  }
  applyCuratedPicks(entity, result, catalog, { bone, spirit });
  return result;
}
