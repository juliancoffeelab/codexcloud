import type { UnitDefinition, UnitType, Weapon } from "./types";

const weapon = (
  id: string,
  name: string,
  damage: number,
  damageType: Weapon["damageType"],
  category: Weapon["category"] = "melee",
  range = 1,
): Weapon => ({
  id,
  name,
  damage,
  damageType,
  category,
  range,
  sound: category === "ranged" ? "attack-bow" : "attack-melee",
});
const neutral = { blade: 0, pierce: 0, impact: 0 };

/** Promotion recipes are finite and retain weapon identities for saved combat logs. */
export function progressionUnits(
  base: Partial<Record<UnitType, UnitDefinition>>,
): Record<UnitType, UnitDefinition> {
  const units = { ...base } as Record<UnitType, UnitDefinition>;
  units.swordsman = {
    ...units.soldier,
    name: "Swordsman",
    hp: 10,
    damage: 3,
    cost: 6,
    terrainModifiers: {},
    damageModifiers: neutral,
    attacks: [weapon("sword", "Sword", 3, "blade")],
    ability: "Line infantry",
    description: "Affordable sword infantry. Earn armor and a mace by advancing to heavy infantry.",
  };
  units.spearman = {
    ...units.warden,
    name: "Spearman",
    hp: 10,
    damage: 3,
    cost: 6,
    terrainModifiers: {},
    damageModifiers: neutral,
    attacks: [weapon("spear", "Spear", 3, "pierce")],
    ability: "Spear line",
    description:
      "Affordable spear infantry. Effective against woodland beasts; advances to a shield warden with a hammer.",
  };
  const sea = (
    name: string,
    hp: number,
    move: number,
    cost: number,
    attacks: [Weapon, ...Weapon[]],
    description: string,
  ): UnitDefinition => ({
    name,
    race: "merfolk",
    hp,
    move,
    cost,
    attacks,
    damage: attacks[0].damage,
    range: Math.max(...attacks.map((attack) => attack.range)),
    damageModifiers: neutral,
    ability: "Amphibious",
    description,
  });
  units.tidekeeper = {
    ...sea(
      "Tidekeeper",
      14,
      4,
      0,
      [
        weapon("trident", "Trident", 3, "pierce"),
        weapon("tidebolt", "Tide bolt", 2, "impact", "ranged", 2),
      ],
      "Merfolk leader. Swims through deep water; recruits on an owned keep. Forests and groves slow and expose merfolk.",
    ),
    leader: true,
  };
  units.reefguard = {
    ...sea(
      "Reef guard",
      11,
      4,
      8,
      [weapon("spear", "Reef spear", 3, "pierce")],
      "Shell armor stops 1 Blade damage. A durable spear screen in shallow water; needs Impact support against armor and bones, and lacks a ranged reply.",
    ),
    damageModifiers: { ...neutral, blade: -1 },
  };
  units.harpooner = sea(
    "Harpooner",
    8,
    5,
    7,
    [weapon("harpoon", "Harpoon", 3, "pierce", "ranged", 2), weapon("knife", "Knife", 2, "blade")],
    "Fast swimmer with ranged harpoons. Vulnerable to armor and bones; use it to take offshore routes and attack unarmored foes.",
  );
  units.wavemender = {
    ...sea(
      "Wavemender",
      8,
      4,
      9,
      [weapon("surge", "Surge", 3, "impact", "ranged", 2), weapon("staff", "Staff", 1, "impact")],
      "Ranged Impact against armor. Heals adjacent allies by 1 HP at turn start; multiple healers do not stack. Fragile in melee.",
    ),
    healing: 1,
  };
  units.soldier = { ...units.soldier, name: "Heavy infantry" };
  const advances: [UnitType, UnitType, string, Partial<UnitDefinition>][] = [
    ["swordsman", "soldier", "Heavy infantry", {}],
    ["spearman", "warden", "Shield warden", {}],
    ["archer", "marksman", "Marksman", { hp: 11 }],
    ["scout", "ranger", "Ranger", { hp: 11 }],
    ["adept", "mage", "Stone mage", { hp: 10 }],
    ["skeleton", "revenant", "Revenant", { hp: 12 }],
    ["bonearcher", "deatharcher", "Death archer", { hp: 10 }],
    ["wolf", "direwolf", "Dire wolf", { hp: 11 }],
    ["boar", "greatboar", "Great boar", { hp: 16 }],
    ["thorncaller", "briarweaver", "Briarweaver", { hp: 10 }],
    ["reefguard", "tridentguard", "Trident guard", { hp: 14 }],
    ["harpooner", "tidehunter", "Tide hunter", { hp: 11 }],
    ["wavemender", "tidesage", "Tide sage", { hp: 11, healing: 2 }],
    ["commander", "marshal", "Marshal", { hp: 19 }],
    ["wildkeeper", "grovewarden", "Grove warden", { hp: 17 }],
    ["necromancer", "lich", "Lich", { hp: 15 }],
    ["tidekeeper", "tideseer", "Tide seer", { hp: 17 }],
  ];
  for (const [from, to, name, override] of advances) {
    const source = units[from];
    const earnedArmor = from === "swordsman" || from === "spearman";
    if (!earnedArmor) {
      const attacks = source.attacks.map((attack, index) => ({
        ...attack,
        damage: attack.damage + (index === 0 ? 1 : 0),
      })) as [Weapon, ...Weapon[]];
      units[to] = {
        ...source,
        ...override,
        name,
        attacks,
        damage: attacks[0].damage,
        cost: Math.ceil(source.cost * 1.5),
        description: `${name}: an experienced ${source.name.toLowerCase()}. ${source.description}`,
      };
    }
    units[from] = {
      ...source,
      level: 1,
      family: from,
      experienceNeeded: source.leader ? 18 : 10,
      advancesTo: [to],
    };
    units[to] = { ...units[to], level: 2, family: from, experienceNeeded: 0, advancesTo: [] };
  }
  // Veteran specialists gain a role change as well as durability.
  units.marksman = {
    ...units.marksman,
    terrainModifiers: {
      forest: { damageModifiers: { ranged: -1 } },
      grove: { damageModifiers: { ranged: -1 } },
    },
    ability: "Concealment",
    description:
      "An experienced bow specialist: takes 1 additional point less ranged damage in forests and groves. Keep it screened from melee.",
  };
  units.ranger = {
    ...units.ranger,
    move: 6,
    attacks: [
      weapon("knife", "Knife", 3, "blade"),
      weapon("sling", "Sling", 2, "impact", "ranged", 2),
    ],
    ability: "Trail runner",
    description:
      "Faster capture and flanking specialist with a useful Impact sling. Still lacks armor.",
  };
  units.mage = {
    ...units.mage,
    attacks: [
      weapon("spark", "Spark", 4, "impact", "ranged", 2),
      weapon("staff", "Staff", 2, "impact"),
    ],
    ability: "Stone mastery",
    description:
      "Strong ranged Impact and a firmer staff reply. Still fragile; preserve cover and a melee screen.",
  };
  units.revenant = {
    ...units.revenant,
    attacks: [
      weapon("rustblade", "Rust blade", 4, "blade"),
      weapon("boneclub", "Bone club", 3, "impact"),
    ],
    damageModifiers: { blade: -1, pierce: -1, impact: 0 },
    ability: "Bone shield",
    description:
      "A shielded skeleton veteran. Gains Blade protection while retaining resistance to Pierce and vulnerability to Impact.",
  };
  units.deatharcher = {
    ...units.deatharcher,
    attacks: [
      weapon("bonebow", "Bone bow", 4, "pierce", "ranged", 2),
      weapon("shard", "Shard", 2, "blade"),
    ],
    description:
      "A bone-bow veteran with a usable melee shard. Armor and Impact enemies still demand support.",
  };
  units.direwolf = {
    ...units.direwolf,
    terrainModifiers: { hill: { cost: -1 } },
    ability: "Ridge runner",
    description:
      "A wolf veteran that crosses hills as easily as forest. Fast pursuit, but spears still exploit its Pierce weakness.",
  };
  units.greatboar = {
    ...units.greatboar,
    attacks: [weapon("tusks", "Tusks", 5, "pierce"), weapon("trample", "Trample", 3, "impact")],
    ability: "Trample",
    description:
      "A boar veteran with an Impact trample against bones and armor. Remains slow in wetlands and has no ranged reply.",
  };
  units.briarweaver = {
    ...units.briarweaver,
    healing: 1,
    ability: "Living boughs",
    description:
      "Thorncaller veteran. Retains extra ranged protection in forests and groves, and heals adjacent allies by 1 at turn start while providing ranged Pierce and a light Impact staff.",
  };
  units.tridentguard = {
    ...units.tridentguard,
    attacks: [
      weapon("spear", "Trident", 4, "pierce"),
      weapon("shellclub", "Shell club", 2, "impact"),
    ],
    ability: "Reef bulwark",
    description:
      "A shell-armored merfolk veteran that gains an Impact club. Controls the shallows; woodland remains dangerous and ranged support still matters.",
  };
  units.tidehunter = {
    ...units.tidehunter,
    attacks: [
      weapon("harpoon", "Harpoon", 4, "pierce", "ranged", 2),
      weapon("knife", "Knife", 3, "blade"),
    ],
    ability: "Sea skirmisher",
    description:
      "A harpoon veteran with a stronger knife reply. Exploits sea routes, but cannot replace Impact support against armor.",
  };
  return units;
}
