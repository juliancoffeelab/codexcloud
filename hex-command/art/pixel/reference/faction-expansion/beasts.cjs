// All animal anatomy slots use the same three-anchor mask and native extraction.
const fs = require("node:fs"),
  path = require("node:path");
const root = __dirname,
  file = path.join(root, "manifest.json"),
  m = JSON.parse(fs.readFileSync(file));
const palette = m.races.woodland.palette;
const groups = {
  "beast-land": {
    anchors: ["boar", "wolf", "dwarf-goat-rider"],
    vibe: "A natural dangerous animal, not a humanoid. Exact species anatomy, grounded readable pose and complete limbs. Chunky restrained pixel art matching existing wolves/boars. Small teal neck collar or harness identifies faction, no rider. Coat/scales use two flat tones, no noise.",
  },
  "beast-flying": {
    anchors: ["boar", "wolf", "elf-deer-archer"],
    vibe: "A powerful flying creature, not a person. Complete species anatomy and readable wings, side three-quarter view. Chunky native pixel art with flat connected feathers or membranes, no noise. Small teal neck band identifies faction, no rider.",
  },
  "beast-aquatic": {
    anchors: ["merfolk-trident", "merfolk-netter", "reefguard"],
    vibe: "A complete aquatic animal or monster, NOT a merfolk. No human torso or hands. Strong natural swimming silhouette, no water or bubbles. Small teal harness/marking identifies faction. Flat connected colored tissue/scales, no noise.",
  },
  "beast-undead": {
    anchors: ["skeleton-rider", "wolf", "skeleton-axeman"],
    vibe: "Scary undead beast with real animal anatomy, not a human. Exposed bones or rotten flesh as specified, dark sockets and complete limb attachments. Small ragged teal harness identifies faction, no rider. Chunky pixel clusters and flat tissue/bone planes.",
  },
  "beast-fantasy": {
    anchors: ["boar", "wolf", "skeleton-rider"],
    vibe: "A menacing fantasy beast, complete nonhuman anatomy, no rider. Readable species silhouette, deliberate limb attachments and small teal collar/harness for faction. Same native pixel detail as references, flat materials with no noisy textures.",
  },
};
for (const [race, g] of Object.entries(groups)) m.races[race] = { ...g, palette };
const jobs = [];
function add(race, id, name, pose, coat, shadow, colors2, colors3) {
  const p = {
    ...palette,
    H: [...coat, 255],
    h: [...shadow, 255],
    V: [...coat, 255],
    v: [...shadow, 255],
  };
  jobs.push({
    race,
    id,
    name,
    pose,
    size: 64,
    status: "pending",
    palette: p,
    tierColors: [null, colors2, colors3],
  });
}
const brown = [133, 88, 51],
  brownShadow = [73, 46, 32],
  gray = [132, 145, 151],
  grayShadow = [68, 78, 87],
  green = [99, 141, 67],
  greenShadow = [49, 83, 43];
const colors = (H, h) => ({ H: [...H, 255], h: [...h, 255], V: [...H, 255], v: [...h, 255] });
add(
  "beast-land",
  "bear",
  "Bear",
  "Large adult brown bear in low powerful walking stance, FOUR distinct strong legs and paws, round ears, short tail, heavy shoulders and broad short muzzle. Mouth slightly open with attached ivory fangs.",
  brown,
  brownShadow,
  colors([53, 56, 59], [28, 32, 36]),
  colors([196, 199, 188], [109, 121, 120]),
);
add(
  "beast-land",
  "lynx",
  "Lynx",
  "Athletic wild CAT, tufted pointed ears, broad cheek ruff, short feline muzzle and SHORT stub tail, four paws, stalking stance. Warm tawny fur with only a few large deliberate dark cheek marks; NOT wolf, not long dog muzzle.",
  [166, 116, 63],
  [99, 67, 39],
  colors(gray, grayShadow),
  colors([225, 222, 207], [143, 149, 143]),
);
add(
  "beast-land",
  "stag",
  "Stag",
  "Adult stag with branching antlers, upright proud slender neck, short deer muzzle, FOUR long slim legs with split hooves, short tail, brown coat and cream belly. Antlers attach correctly above ears.",
  brown,
  brownShadow,
  colors([161, 136, 97], [92, 79, 57]),
  colors([229, 224, 202], [155, 162, 138]),
);
add(
  "beast-land",
  "giant-spider",
  "Giant spider",
  "Huge low spider, EIGHT articulated legs clearly arranged four per side, two body segments abdomen/cephalothorax, small connected fangs and compact eye cluster. Gray-violet chitin, no fur, no human face.",
  [108, 91, 132],
  [57, 47, 76],
  colors([136, 51, 68], [76, 31, 45]),
  colors([42, 48, 57], [22, 26, 34]),
);
add(
  "beast-land",
  "giant-tortoise",
  "Giant tortoise",
  "Heavy tortoise, broad domed olive shell with a few large polygon scutes, FOUR short stout reptilian legs, small beaked head on short neck, small tail. Shell and neck connect naturally, no teeth.",
  green,
  greenShadow,
  colors([79, 139, 115], [38, 79, 68]),
  colors([198, 196, 143], [109, 126, 82]),
);
add(
  "beast-land",
  "giant-snake",
  "Giant snake",
  "Long thick coiled snake with raised menacing head, connected looping body and tapered tail, NO legs. Broad reptile head, small attached forked tongue, two tiny fixed fangs, olive scales and cream belly. One coil supports the raised neck.",
  green,
  greenShadow,
  colors([174, 116, 68], [105, 63, 39]),
  colors([227, 224, 204], [143, 153, 135]),
);
add(
  "beast-flying",
  "giant-raven",
  "Giant raven",
  "Large black raven, hooked heavy beak, broad partly raised wings, TWO clawed feet, long wedge tail and feathered body. Wings connect at shoulders, not free floating. Flat blue-black feather planes, no speckling.",
  [47, 55, 65],
  [23, 29, 36],
  colors([41, 62, 89], [22, 35, 53]),
  colors([68, 47, 91], [37, 26, 56]),
);
add(
  "beast-flying",
  "giant-owl",
  "Giant owl",
  "Large owl, TWO broad partly spread wings, TWO clawed feet, rounded head with deliberate facial disk and big stern eyes, tiny hooked beak, short fan tail. Brown/cream feathers, only large feather groups.",
  brown,
  brownShadow,
  colors(gray, grayShadow),
  colors([227, 226, 210], [144, 152, 143]),
);
add(
  "beast-flying",
  "giant-wasp",
  "Giant wasp",
  "Huge wasp with THREE body segments, SIX insect legs, TWO pairs of transparent ivory/cyan wings visibly attached to thorax, antennae and a connected pointed sting. Yellow-and-black broad abdominal stripes, NOT bee fur.",
  [191, 155, 41],
  [104, 77, 27],
  colors([220, 150, 35], [115, 65, 25]),
  colors([236, 191, 69], [140, 103, 30]),
);
add(
  "beast-flying",
  "giant-bat",
  "Giant bat",
  "Large bat with TWO spread leathery wings attached to forelimb fingers, TWO small clawed hind feet, large pointed ears, short snout with tiny attached fangs. Dark brown fur and purple-brown wing membranes, no extra limbs.",
  [114, 78, 86],
  [61, 42, 52],
  colors([100, 74, 132], [51, 37, 76]),
  colors([190, 168, 166], [105, 85, 108]),
);
add(
  "beast-aquatic",
  "giant-crab",
  "Giant crab",
  "Huge crab, TWO large front pincers and EIGHT smaller walking legs, flat broad coral shell, two eye stalks. Both claws have a real connected hinge, no floating claw tips. Three-quarter low threatening stance.",
  [196, 94, 59],
  [112, 46, 36],
  colors([123, 84, 157], [66, 44, 89]),
  colors([177, 47, 49], [96, 27, 37]),
);
add(
  "beast-aquatic",
  "crocodile",
  "Crocodile",
  "Long low crocodile, FOUR splayed reptile legs, broad long snout with open jaw and teeth attached to upper/lower gums, thick tapering tail and large dorsal scutes. Olive skin, cream belly. Not wolf or dragon.",
  green,
  greenShadow,
  colors(gray, grayShadow),
  colors([213, 216, 185], [128, 145, 114]),
);
add(
  "beast-aquatic",
  "shark",
  "Shark",
  "Complete shark swimming side-on, pointed shark snout with mouth/attached teeth, ONE dorsal fin, TWO pectoral fins, small pelvic fins and correct vertical crescent tail with upper/lower lobes. Gray-blue upper body, cream underside, visible gill lines, NO legs.",
  [100, 135, 153],
  [50, 76, 96],
  colors([60, 76, 92], [29, 44, 59]),
  colors([211, 221, 216], [128, 153, 161]),
);
add(
  "beast-aquatic",
  "ray",
  "Ray",
  "Large stingray swimming in three-quarter top view, ONE continuous broad diamond body with TWO attached wing-like pectoral fins, two eyes on top and ONE long thin whip tail with attached barb. No separate bird wings or legs.",
  [94, 138, 153],
  [44, 76, 98],
  colors([117, 83, 150], [61, 45, 89]),
  colors([175, 130, 74], [100, 75, 41]),
);
add(
  "beast-aquatic",
  "octopus",
  "Octopus",
  "Large octopus with ONE rounded mantle, stern eye cluster and EIGHT separately readable curling arms all joined below head. Several arms coil into loops, ivory suckers in intentional broad runs. No extra disconnected tentacles.",
  [173, 84, 68],
  [101, 44, 43],
  colors([135, 80, 164], [74, 41, 97]),
  colors([93, 169, 171], [42, 94, 111]),
);
add(
  "beast-undead",
  "skeletal-hound",
  "Skeletal hound",
  "Scary skeletal dog, FOUR distinct articulated bone legs/paws, exposed ribs and spine, canine skull with long muzzle and open black sockets, attached jaws and teeth, bony tail. No fur or fleshy ears.",
  [224, 215, 182],
  [153, 146, 118],
  colors([227, 229, 212], [138, 158, 146]),
  colors([241, 230, 189], [174, 150, 96]),
);
add(
  "beast-undead",
  "corpse-vulture",
  "Corpse vulture",
  "Rotten large vulture, hunched ragged dark wings, TWO clawed legs, long partly bare crooked neck with decayed olive tissue, hooked beak and sunken eyes. No skeleton human face. Thick connected ragged feather groups.",
  [111, 108, 72],
  [60, 62, 45],
  colors([84, 96, 68], [42, 56, 39]),
  colors([128, 105, 129], [65, 51, 76]),
);
add(
  "beast-undead",
  "skeletal-horse",
  "Skeletal horse",
  "Complete RIDERLESS skeletal horse, FOUR articulated bony legs with hooves, exposed ribcage and spine, long horse skull, attached jaw and dark sockets, bony tail and small ragged teal saddlecloth. No person seated on it.",
  [224, 215, 182],
  [153, 146, 118],
  colors([227, 229, 212], [138, 158, 146]),
  colors([241, 230, 189], [174, 150, 96]),
);
add(
  "beast-undead",
  "zombie-bear",
  "Zombie bear",
  "Huge hideous rotten FLESHY bear, four heavy decaying paws, round ears, broad bear muzzle, sunken eyes and two exposed patches of ribs/limb bone, sick olive dead flesh and ragged remaining brown fur. Not a clean skeleton and no human torso.",
  [137, 146, 83],
  [73, 89, 48],
  colors([113, 128, 75], [59, 77, 41]),
  colors([159, 149, 104], [86, 82, 52]),
);
add(
  "beast-fantasy",
  "gryphon",
  "Gryphon",
  "Complete gryphon: lion hindquarters and tail, eagle chest/head with hooked beak, FOUR legs (eagle front talons and lion hind paws), TWO folded raised feathered wings attached above forelegs. Ivory eagle feathers and tawny lion coat. No rider.",
  [168, 118, 59],
  [94, 61, 35],
  colors([148, 153, 154], [78, 85, 99]),
  colors([231, 222, 190], [155, 142, 108]),
);
add(
  "beast-fantasy",
  "dragon",
  "Dragon",
  "Complete young dragon in side three-quarter grounded stance, FOUR separate reptilian legs PLUS TWO folded partially raised bat-like wings, long neck, clear dragon head with attached jaw/fangs, long tapering tail, olive scales and muted red membranes. No rider.",
  green,
  greenShadow,
  colors([110, 86, 151], [55, 43, 88]),
  colors([170, 55, 62], [95, 29, 43]),
);
add(
  "beast-fantasy",
  "basilisk",
  "Basilisk",
  "Low menacing basilisk reptile, SIX clearly separated stout legs, broad reptile head with a royal crest, long tapering tail and a few large dorsal scales. Amber-olive scales, cream belly, attached jaws and fangs. Not generic four-legged lizard.",
  [154, 136, 68],
  [86, 79, 39],
  colors(green, greenShadow),
  colors([216, 211, 172], [132, 142, 102]),
);
add(
  "beast-fantasy",
  "treant",
  "Treant",
  "Walking living tree, thick trunk torso, TWO branch arms ending in branch fingers and TWO distinct root feet supporting it. Stern face integrated into bark, connected leafy canopy above and on shoulders. Brown bark and green leaves, no human clothes or metal.",
  brown,
  brownShadow,
  colors([111, 95, 72], [65, 53, 42]),
  colors([155, 143, 102], [91, 84, 59]),
);
add(
  "beast-aquatic",
  "sea-serpent",
  "Sea serpent",
  "Complete sea serpent, ONE long thick curling/coiled aquatic body, raised dragonlike head with attached jaw, fin ears and TWO paired small flippers, long tapering tail and dorsal fins. No human upper torso, no walking legs. Cyan sea-green scales, cream belly.",
  [90, 163, 145],
  [43, 94, 87],
  colors([133, 100, 175], [71, 49, 111]),
  colors([181, 160, 77], [102, 94, 39]),
);
add(
  "beast-undead",
  "bone-dragon",
  "Bone dragon",
  "Complete skeletal dragon, FOUR separate articulated bone legs PLUS TWO raised folded wings composed of connected long wing finger bones, ribcage/spine, long bony neck, dragon skull with attached jaw and fangs, bony tapering tail. NO skin or membrane, no rider.",
  [224, 215, 182],
  [153, 146, 118],
  colors([227, 229, 212], [138, 158, 146]),
  colors([241, 230, 189], [174, 150, 96]),
);
for (const j of jobs) {
  if (j.id === "treant") {
    Object.assign(j.palette, { L: [72, 125, 49, 255], l: [39, 74, 35, 255] });
    Object.assign(j.tierColors[1], { L: [96, 146, 56, 255], l: [50, 84, 36, 255] });
    Object.assign(j.tierColors[2], { L: [158, 173, 81, 255], l: [81, 103, 47, 255] });
  }
  if (j.id === "dragon") Object.assign(j.palette, { M: [158, 65, 49, 255], m: [92, 36, 38, 255] });
  if (j.id === "giant-wasp") {
    Object.assign(j.palette, { V: [29, 28, 22, 255], v: [16, 17, 15, 255] });
    for (let l = 1; l <= 2; l++) Object.assign(j.tierColors[l], { V: j.palette.V, v: j.palette.v });
  }
  if (j.id === "giant-crab") {
    Object.assign(j.palette, { V: [58, 63, 40, 255], v: [31, 36, 28, 255] });
    for (let l = 1; l <= 2; l++) Object.assign(j.tierColors[l], { V: j.palette.V, v: j.palette.v });
    j.pose =
      "Top-view coral crab, two large forward pincers plus eight walking legs, four fanned on each side with separated tips. Dark natural leg hinges, ivory claw interiors and tiny eye stalks, teal central harness.";
  }
  if (j.id === "giant-spider")
    j.pose =
      "Elevated top-view gray-violet spider with eight long walking legs, four separated tips on each side. Two connected body segments, small dark eyes and tiny ivory fangs. Teal body band.";
  if (j.id === "basilisk") {
    j.anchors = ["giant-spider", "giant-crab", "crocodile"];
    j.pose =
      "Top-view six-legged basilisk, head toward bottom and connected tail toward top. Three stout articulated walking legs fan out on each side, all six feet exposed with gaps. Amber-olive scutes, royal head crest, ivory fangs, small teal neck band.";
  }
  if (["skeletal-hound", "skeletal-horse", "bone-dragon"].includes(j.id))
    for (let l = 1; l <= 2; l++)
      Object.assign(j.tierColors[l], { I: j.tierColors[l].H, i: j.tierColors[l].h });
  if (j.id === "crocodile") {
    j.anchors = groups["beast-land"].anchors;
    j.pose =
      "Four-legged crocodile seen diagonally from above, head lower right and thick tapering tail upper left. Four separate splayed legs, two on each side, all joined to the torso. Olive scutes and ivory belly.";
  }
  if (j.id === "octopus")
    j.pose =
      "Frontal coral octopus, one mantle and eight curled arms, four on each side, all joined to the mantle base. Ivory suckers in deliberate connected runs.";
  const i = m.jobs.findIndex((x) => x.id === j.id);
  if (i < 0) m.jobs.push(j);
  else m.jobs[i] = { ...j, status: m.jobs[i].status, canvas: m.jobs[i].canvas };
}
fs.writeFileSync(file, JSON.stringify(m, null, 2) + "\n");
require("./templates.cjs")(jobs, m.races);
console.log(jobs.length + " complete creature templates prepared.");
