const fs = require("../../../../scripts/pixel-fs.cjs");
const path = require("node:path");
const { PNG } = require("pngjs");
const { execFileSync } = require("node:child_process");
const root = __dirname;
const art = path.resolve(root, "../..");
const common = {
  ".": [0, 0, 0, 0],
  O: [25, 22, 20, 255],
  B: [111, 72, 43, 255],
  b: [64, 40, 28, 255],
  T: [5, 100, 111, 255],
  t: [4, 57, 65, 255],
  V: [121, 111, 82, 255],
  v: [72, 65, 48, 255],
  H: [247, 190, 140, 255],
  h: [177, 118, 81, 255],
  A: [203, 215, 218, 255],
  a: [101, 119, 127, 255],
  G: [238, 184, 52, 255],
  W: [132, 88, 51, 255],
  w: [76, 49, 31, 255],
  I: [240, 229, 198, 255],
  i: [180, 163, 127, 255],
  E: [99, 207, 170, 255],
};
const races = {
  skeleton: {
    anchors: ["skeleton", "bonearcher", "revenant"],
    vibe: "Scary skeletal undead. Anatomically readable ivory skull, deep black eye sockets, exposed ribs, elbows and knee bones, no living flesh. Lean roughly human-height bodies; rough rusty equipment and ragged teal faction cloth. Bias strongly toward the skeleton WARRIOR proportions. Skull width about 7-9 logical pixels and height 8-10; never the oversized lich head. Avoid oversized cartoon skulls.",
    palette: {
      H: [181, 171, 148, 255],
      h: [118, 108, 91, 255],
      V: [98, 82, 107, 255],
      v: [54, 42, 65, 255],
    },
  },
  zombie: {
    anchors: ["skeleton", "leather-swordsman", "necromancer"],
    vibe: "Hideous walking corpses, NOT skeletons. Heavy or hunched decaying fleshy bodies, asymmetric torn clothes, sick yellow-green dead skin, sunken faces and missing chunks. Ugly, unsettling, grounded pixel silhouettes, no cute faces. Teal scraps identify faction.",
    palette: {
      H: [143, 154, 92, 255],
      h: [86, 102, 58, 255],
      V: [117, 95, 74, 255],
      v: [69, 54, 43, 255],
      E: [188, 217, 70, 255],
    },
  },
  elf: {
    anchors: ["archer", "ranger", "healer"],
    vibe: "Slim tall woodland elves: long slender limbs, small heads, visible pointed ears, graceful but practical athletic poses. Slightly taller than human anchors, not stretched. Leaf-green garments, light cream details, warm brown hair and leather, teal faction cloaks. Elegant and restrained, not bulky human armor.",
    palette: { V: [118, 149, 64, 255], v: [65, 93, 43, 255], E: [165, 211, 116, 255] },
  },
  dwarf: {
    anchors: ["leather-swordsman", "hammerman", "crossbowman"],
    vibe: "Short, broad, tough dwarves, about three quarters human height. Chunky arms, squat strong legs, big noses and bushy beards. Slightly funny stubborn faces, not baby proportions. Practical leather workwear, iron equipment and teal faction cloth. Do not draw full-height humans.",
    palette: { V: [148, 106, 57, 255], v: [88, 61, 36, 255], E: [122, 198, 227, 255] },
  },
  woodland: {
    anchors: ["thorncaller", "grovewarden", "boar"],
    vibe: "Anthropomorphic woodland fox with orange fur, cream muzzle, upright pointed ears, digitigrade-looking boots and visible bushy tail. Same scale/detail as existing fox casters, natural bark/wicker equipment, teal faction cloak.",
    palette: {
      H: [241, 112, 26, 255],
      h: [158, 64, 22, 255],
      V: [130, 147, 60, 255],
      v: [72, 89, 36, 255],
      E: [159, 196, 64, 255],
    },
  },
  merfolk: {
    anchors: ["tidehunter", "reefguard", "tidesage"],
    vibe: "Merfolk with a human-like upper torso and ONE long curling fish tail instead of legs. Sea-green skin, fin ears, coral/bronze equipment, teal faction cloth. Strong readable shoulders and hands, flowing tail planted as support. Match the existing merfolk anchors.",
    palette: {
      H: [92, 167, 138, 255],
      h: [46, 103, 88, 255],
      V: [121, 138, 164, 255],
      v: [61, 79, 105, 255],
      E: [130, 215, 218, 255],
      C: [207, 112, 69, 255],
      c: [143, 70, 48, 255],
    },
  },
};
const jobs = [];
function add(race, id, name, pose, size = 48) {
  jobs.push({ race, id, name, pose, size, status: "pending" });
}
add(
  "skeleton",
  "skeleton-flanker",
  "Skeleton dagger flanker",
  "Low forward-leaning athletic stance. TWO separate steel daggers, each visibly gripped by an ivory skeletal hand beside the body. Short torn teal cloak, exposed ribs, minimal rusty leather. Both elbows and knees readable.",
);
add(
  "skeleton",
  "skeleton-impact-caster",
  "Skeleton impact caster",
  "Standing skeletal caster. New proportionate angular skull and hood opening. One hand grips a staff capped with a chunky blunt stone cube, other skeletal hand opens outward casting a small pale cyan force orb. Simple muted violet tunic, teal hood/cloak, bare bone hands.",
);
add(
  "skeleton",
  "skeleton-scythe-caster",
  "Skeleton scythe caster",
  "Skeletal reaper caster. A long wooden scythe shaft held visibly in one bony hand, real large curved steel scythe blade attached above and beside its skull, other hand casting a small green soul flame. Modest ragged violet robe and teal hood/cloak, readable skull with black eye sockets.",
);
add(
  "skeleton",
  "skeleton-axeman",
  "Skeleton two-handed axeman",
  "Exposed-rib skeleton in a wide grounded stance, BOTH hands visibly grip ONE diagonal wooden shaft, broad double-edged rusty steel axe head at its upper free end. Simple belt and short teal mantle, no full plate armor.",
);
add(
  "skeleton",
  "skeleton-rider",
  "Skeleton mounted spearman",
  "A skeletal warrior seated correctly on a SKELETAL HORSE, not a living horse. Four separate bony legs, ribcage, horse skull with long muzzle, dark sockets. Rider holds a long wooden spear with steel tip in one hand and reins in other. Teal small cloak, beginner light equipment. Full horse, spear tip and tail inside slot.",
  64,
);
add(
  "zombie",
  "zombie-shambler",
  "Zombie shambler",
  "Hunched upright corpse with TWO reaching open hands, torn brown civilian tunic, teal scarf scrap, yellow-green decaying flesh, exposed wounds but no red background. Two awkward planted feet. No weapon, no door or prop.",
);
add(
  "zombie",
  "zombie-wretch",
  "Zombie weakling wretch",
  "Weak emaciated civilian corpse, only 28-32 logical pixels tall and 16-20 wide: visibly MUCH THINNER and smaller than the soldier anchors. Narrow shoulders, skinny dangling arms, ragged short brown shirt, one bare foot and one worn boot, tiny teal cloth scrap. Small sick gray-green fleshy face and arms, slack jaw, hunched spine. No armor, no weapon, no huge muscles. Full two-legged standing anatomy. Keep extra empty pink space around this small weak unit, do not enlarge it to fill the slot.",
);
add(
  "zombie",
  "zombie-crawler",
  "Zombie weakling crawler",
  "Weak decayed fleshy zombie crawling low on the ground, only 14-18 logical pixels high and 28-32 wide. Two thin hands planted in front pulling its body, two wasted legs dragging behind. Small head and shoulders low, torn brown peasant shirt, tiny teal cloth scrap, gray-green flesh. No muscles, no armor, no weapon. Complete crawling silhouette with hands and dragging legs. Keep lots of empty pink above it, do not fill the slot.",
);
add(
  "zombie",
  "zombie-light-cleaver",
  "Zombie weakling cleaver",
  "Small skinny low-tier civilian zombie only 28-32 logical pixels tall, narrow chest and wasted arms, small head. One hand grips a small chipped kitchen cleaver, other hand empty and drooping. Torn plain brown shirt, tiny teal scarf scrap, no shield, no armor, no bulky equipment, sick gray-green skin. Two thin planted legs. Extra pink padding: deliberately much smaller and squishier than anchors.",
);
add(
  "zombie",
  "zombie-novice-caster",
  "Zombie weakling corruption caster",
  "Small frail decayed novice caster only 28-32 logical pixels tall, narrow hunched shoulders, tiny skull-like FLESHY green face. Ragged short brown robe and little teal cowl, wasted thin arms. One hand holds a thin plain dead branch staff with a tiny green bud, other hand open casting a tiny lime spark. No huge staff head, no heavy armor, no enormous robe or cloak. Much smaller and squishier than anchors, extra pink padding.",
);
add(
  "zombie",
  "zombie-walker",
  "Zombie skeleton-sized walker",
  "Thin adult corpse at EXACTLY the skeleton WARRIOR anchor height, about 36-40 logical pixels head to feet, narrow torso, normal small head and wasted limbs, not a bulky giant and not a miniature. Two drooping reaching fleshy hands, torn brown civilian shirt and trousers, tiny teal scarf scrap, gray-green corpse skin. No weapons or armor. Match warrior body scale but with flesh and ragged clothes.",
);
add(
  "zombie",
  "zombie-fighter",
  "Zombie skeleton-sized fighter",
  "Thin adult corpse at EXACTLY skeleton WARRIOR anchor height, 36-40 logical pixels head to feet, narrow torso, small head, wasted arms and legs. One hand visibly grips a SMALL chipped cleaver and other a SMALL battered wooden shield. Ragged brown shirt and trousers, teal scarf scrap, gray-green corpse skin. No huge armor or muscles, all equipment proportional.",
);
add(
  "zombie",
  "zombie-decay-caster",
  "Zombie skeleton-sized caster",
  "Thin adult decayed corpse at EXACTLY skeleton WARRIOR body scale, 36-40 logical pixels head to feet, small head and narrow shoulders. Simple ragged brown robe and short teal hood/cloak, wasted fleshy gray-green hands. One hand grips a thin dead-branch staff with small green gem, other casts a compact lime corruption flame. Staff may extend above head but body must match warrior height. No enormous shoulders, hood, head or robe.",
);
add(
  "zombie",
  "zombie-cleaver",
  "Zombie cleaver fighter",
  "Ugly upright zombie corpse visibly gripping a chipped broad cleaver in one hand and a battered small wooden shield in the other. Torn brown work clothes, teal cloth scrap, sick yellow-green flesh. Low-level equipment and exposed dead face.",
);
add(
  "zombie",
  "zombie-ghoul",
  "Zombie ghoul",
  "Low crouched FLESHY ghoul, long arms with broad three-clawed hands, bent legs and hunched spine. Sick yellow-green flesh, ugly sunken face, torn shorts and small teal cloth scrap. Clear hand/foot separation, no animal quadruped, no weapons.",
);
add(
  "zombie",
  "zombie-plague-carrier",
  "Zombie plague carrier",
  "Bulky swollen walking corpse, big crooked basket of bones and remains strapped to its back, ragged brown apron, teal sash. One hand holds a simple short walking club, other open. Yellow-green decayed face and thick arms, two distinct feet.",
);
add(
  "zombie",
  "zombie-corruption-mage",
  "Zombie corruption mage",
  "Hideous decayed fleshy zombie caster, sick greenish skin, ragged dirty robe, teal faction hood/cloak. One hand visibly grips a gnarled rotten staff with crooked dead branches and a sick lime-green gem, other hand casts a compact corruption flame. Not a clean skeleton, not cute.",
);
add(
  "elf",
  "elf-spearman",
  "Elf spear and shield",
  "Tall slim elf gripping a long wooden spear with slender steel leaf tip and a narrow leaf-shaped wooden shield. Pointed ears, simple green tunic, light leather protection, small teal cloak. Both hands and slim legs distinct.",
);
add(
  "elf",
  "elf-bladedancer",
  "Elf two-handed sword",
  "Tall slim elf in graceful wide stance, BOTH hands grip ONE long gently curved steel sword diagonally in front of body. Pointed ears, green tunic, light leather protection, short teal cloak. Blade clear of face, correct connected hilt and hands.",
);
add(
  "elf",
  "elf-archer",
  "Elf archer",
  "Tall slender elf with pointed ears, drawing a wooden longbow, one hand on bow grip and other drawing string beside cheek. Arrow properly seated along bow. Green simple tunic, brown boots, teal cloak, small quiver behind shoulder with arrows up.",
);
add(
  "elf",
  "elf-flanker",
  "Elf dual knife flanker",
  "Tall slim elf in low mobile stance with TWO short steel knives, each held visibly in a separate hand outside torso. Pointed ears, green fitted tunic, brown boots and short teal cloak. No knives stuck in body or belt.",
);
add(
  "elf",
  "elf-caster",
  "Elf branch staff caster",
  "Tall slim elf with pointed ears, plain green robe, teal faction cloak. One hand visibly grips a slender branch staff topped with a small leaf-shaped green crystal, other open casting hand. Clear feet, face and staff head.",
);
add(
  "elf",
  "elf-deer-archer",
  "Elf deer-mounted archer",
  "Tall slim elf seated naturally on a lean DEER with small branching antlers, four slim legs and short tail, drawing a bow. Pointed ears, green tunic, teal small cloak. Hands on bow and string, no horse anatomy. Keep antlers bow and all four legs inside frame.",
  64,
);
add(
  "dwarf",
  "dwarf-axeguard",
  "Dwarf axe and shield",
  "Short broad dwarf with huge brown beard and stout leather boots, one hand grips a short broad axe and the other holds a round wooden shield. Leather jerkin, teal scarf, beginner iron accents, stern funny big nose.",
);
add(
  "dwarf",
  "dwarf-hammerer",
  "Dwarf two-handed hammer",
  "Short squat dwarf, wide stance, BOTH chunky hands gripping ONE diagonal shaft. Large rectangular steel HAMMER head centered across its upper end, broad striking face points downward, short opposite poll; NOT mace, not axe. Leather apron, teal scarf, big beard.",
);
add(
  "dwarf",
  "dwarf-pickaxe",
  "Dwarf pickaxe",
  "Short broad bearded dwarf with leather mining apron and teal scarf, BOTH hands gripping one diagonal wooden pickaxe shaft. A curved pointed steel pick and a short flat counterweight clearly attached across upper shaft. Chunky arms, distinct feet.",
);
add(
  "dwarf",
  "dwarf-poleaxe",
  "Dwarf poleaxe",
  "Short broad bearded dwarf gripping a long poleaxe with both hands. Head has a steel axe blade perpendicular to shaft, spear point aligned with shaft, and rear hook. Entire weapon visible, blade clear of face, leather work clothes and teal scarf.",
);
add(
  "dwarf",
  "dwarf-crossbow",
  "Dwarf crossbow",
  "Short broad bearded dwarf holding a compact CROSSBOW across the front of body, stock against shoulder, trigger hand at grip and supporting hand under stock. Horizontal bow limbs and seated bolt visible, not ordinary bow. Leather jerkin, teal scarf.",
);
add(
  "dwarf",
  "dwarf-arquebus",
  "Dwarf arquebus",
  "Short tough dwarf with big beard and nose aiming an early ARQUEBUS: wooden butt against shoulder, ONE long straight metal barrel points diagonally outward, trigger hand on grip, other hand under barrel. Small muzzle opening, no modern rifle magazine. Leather clothes, teal scarf.",
);
add(
  "dwarf",
  "dwarf-runesmith",
  "Dwarf rune caster",
  "Short broad bushy-bearded dwarf holding a short rectangular smith hammer in one hand and a glowing small cyan RUNE STONE in the other. Simple leather apron, teal scarf, clear hammer grip and stone resting in hand, sturdy boots.",
);
add(
  "dwarf",
  "dwarf-goat-rider",
  "Dwarf goat-mounted hammerman",
  "Short broad bearded dwarf seated on a stocky MOUNTAIN GOAT, curved horns, four sturdy legs and short tail. Rider holds a rectangular steel hammer in one hand and reins in other, leather clothes and teal cloak. Clearly goat not horse, complete hammer head and all legs.",
  64,
);
add(
  "woodland",
  "fox-spearguard",
  "Fox spear and shield",
  "Orange anthropomorphic fox warrior holding a spear and wicker/bark round shield. Both hands visibly grip equipment. Cream muzzle, upright pointed ears, bushy tail, simple brown tunic, teal faction cloak. Match scale and fox anatomy of anchors.",
);
add(
  "merfolk",
  "merfolk-trident",
  "Merfolk two-handed trident",
  "Merfolk with teal-green skin and a single curling fish tail, BOTH hands grip one long bronze/wooden TRIDENT with three clearly connected pointed prongs. Forward attacking reach, head clear of trident. Simple coral chest straps and teal cloth.",
);
add(
  "merfolk",
  "merfolk-netter",
  "Merfolk net and spear",
  "Merfolk with one curling tail, one hand gripping a short spear and other hand visibly holding a GATHERED fishing NET with connected diamond mesh and a loop of rope. Keep mesh simple readable pixels, no loose noise. Simple coral straps and teal cloth.",
);
add(
  "merfolk",
  "merfolk-maceguard",
  "Merfolk mace and shield",
  "Broad-shouldered merfolk with one curling tail, one hand grips a short blunt iron MACE with chunky flanged head, other holds a large round coral shield ready to bash. Simple bronze chest straps, fin ears, teal cloth. Not spear shield variant.",
);
for (const [race, def] of Object.entries(races)) {
  def.palette = { ...common, ...def.palette };
}
const previousPath = path.join(root, "manifest.json");
const previous = fs.existsSync(previousPath) ? JSON.parse(fs.readFileSync(previousPath)) : null;
for (const job of jobs) {
  const old = previous?.jobs.find((j) => j.id === job.id);
  if (old) {
    job.status = old.status;
    if (old.canvas) job.canvas = old.canvas;
  }
}
for (const old of previous?.jobs || []) if (!jobs.some((j) => j.id === old.id)) jobs.push(old);
for (const [id, def] of Object.entries(previous?.races || {})) if (!races[id]) races[id] = def;
fs.writeFileSync(
  previousPath,
  JSON.stringify({ deadline: "2026-10-08T08:00:00Z", races, jobs }, null, 2) + "\n",
);
require("./templates.cjs")(jobs, races);
console.log(`Prepared ${jobs.length} equipped anatomy templates.`);
