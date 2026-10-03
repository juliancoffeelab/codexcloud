import { immutable } from "./immutable";
import type {
  CampaignDefinition,
  CampaignId,
  CampaignMapId,
  DialogueLine,
  MapDefinition,
  MissionDefinition,
  StartingUnit,
  TerrainOverride,
  TerrainType,
  UnitType,
} from "./types";

const land = (
  terrain: TerrainType,
  positions: [number, number][],
  owner: TerrainOverride["owner"] = null,
): TerrainOverride[] => positions.map(([col, row]) => ({ col, row, terrain, owner }));
const river = (
  col: number,
  rows: number,
  crossings: Record<number, "bridge" | "ford">,
): TerrainOverride[] =>
  Array.from({ length: rows }, (_, row) => ({
    col,
    row,
    terrain: crossings[row] ?? "water",
    owner: null,
  }));
const keep = (col: number, row: number, owner: "player" | "enemy"): TerrainOverride => ({
  col,
  row,
  terrain: "keep",
  owner,
});
const unit = (
  type: UnitType,
  side: "player" | "enemy",
  col: number,
  row: number,
  name?: string,
  role?: StartingUnit["role"],
): StartingUnit => ({ type, side, col, row, ...(name ? { name } : {}), ...(role ? { role } : {}) });
const line = (speaker: string, portrait: UnitType, text: string): DialogueLine => ({
  speaker,
  portrait,
  text,
});
const map = (
  id: CampaignMapId,
  name: string,
  cols: number,
  rows: number,
  terrain: TerrainOverride[],
  units: StartingUnit[],
): MapDefinition => ({ id, name, description: name, revision: 1, cols, rows, terrain, units });

const lantern: MissionDefinition[] = [
  {
    map: map(
      "lantern-ferry",
      "The Last Ferry",
      13,
      7,
      [
        ...river(6, 7, { 1: "ford", 3: "bridge" }),
        keep(1, 3, "player"),
        keep(11, 4, "enemy"),
        ...land("village", [
          [3, 1],
          [3, 5],
          [8, 1],
        ]),
        ...land("village", [[10, 2]], "enemy"),
        ...land("forest", [
          [4, 0],
          [4, 1],
          [8, 4],
          [9, 5],
        ]),
        ...land("hill", [
          [5, 4],
          [8, 2],
        ]),
        ...land("marsh", [[7, 5]]),
        ...land("ruins", [
          [9, 1],
          [7, 1],
          [7, 2],
        ]),
        ...land("fort", [[9, 3]]),
      ],
      [
        unit("commander", "player", 1, 3, "Captain Nera", "leader"),
        unit("soldier", "player", 2, 3),
        unit("soldier", "player", 2, 4),
        unit("scout", "player", 2, 2, "Ivo", "courier"),
        unit("necromancer", "enemy", 11, 4, "The Tollkeeper", "leader"),
        unit("skeleton", "enemy", 10, 5),
        unit("bonearcher", "enemy", 12, 5),
      ],
    ),
    subtitle: "Carry a message, not a conquest.",
    briefing:
      "Get Ivo to the northeastern landing, beyond the guards rather than into their keep. Two shields can screen the courier while your recruits arrive. Skeletons resist arrows; maces crack their bones. The bridge leads toward the guards; the northern ford leads to a healing village and a route around them. A southern patrol arrives on turn 6. Nera and Ivo must survive.",
    goals: [{ kind: "escort", label: "Bring Ivo to the northeastern landing", tile: "11,1" }],
    recruits: ["soldier", "archer", "scout"],
    enemyRecruits: ["skeleton", "bonearcher"],
    gold: 16,
    enemyGold: 0,
    intro: [
      line(
        "Ivo",
        "scout",
        "The eastern villages have not seen a lantern in three nights. I have their names here. Someone has to read them.",
      ),
      line(
        "Nera",
        "commander",
        "Then we get you across. Stay behind the shields. We are not here to win the river; we are here to reach the other bank.",
      ),
      line(
        "Ivo",
        "scout",
        "The ford north of the bridge is still open. And the ferryman hid his purse in the old riverside ruin, if we need supplies.",
      ),
    ],
    victory: [
      line(
        "Ivo",
        "scout",
        "The names are safe. But the ferryman says the dead have taken three settlements farther up the road.",
      ),
      line(
        "Nera",
        "commander",
        "We will light each one. Let the people see that the road still belongs to the living.",
      ),
    ],
    events: [
      {
        id: "ferryman",
        trigger: { kind: "visit", tile: "9,1" },
        label: "Find the ferryman’s purse in the riverside ruin",
        gold: 6,
        dialogue: [
          line(
            "Ferryman",
            "scout",
            "Take it. Six coins will not buy my ferry back, but perhaps they will buy someone else a morning.",
          ),
        ],
      },
    ],
    waves: [
      {
        round: 6,
        label: "A skeleton patrol arrives from the southern road on turn 6",
        units: [unit("skeleton", "enemy", 11, 6)],
      },
    ],
  },
  {
    map: map(
      "lantern-villages",
      "Three Unlit Windows",
      13,
      9,
      [
        ...river(5, 9, { 1: "ford", 4: "bridge", 7: "ford" }),
        keep(1, 4, "player"),
        keep(11, 4, "enemy"),
        ...land(
          "village",
          [
            [7, 1],
            [7, 4],
            [7, 7],
          ],
          "enemy",
        ),
        ...land("village", [
          [3, 3],
          [3, 5],
        ]),
        ...land("forest", [
          [4, 2],
          [4, 6],
          [9, 2],
          [9, 6],
        ]),
        ...land("marsh", [
          [6, 0],
          [6, 1],
          [6, 7],
          [6, 8],
        ]),
        ...land("hill", [
          [8, 3],
          [8, 5],
        ]),
        ...land("fort", [[10, 4]]),
        ...land("grove", [[2, 0]]),
        ...land("ruins", [[4, 8]]),
      ],
      [
        unit("commander", "player", 1, 4, "Captain Nera", "leader"),
        unit("warden", "player", 2, 4),
        unit("scout", "player", 2, 3),
        unit("soldier", "player", 2, 5),
        unit("necromancer", "enemy", 11, 4, "The Tollkeeper", "leader"),
        unit("skeleton", "enemy", 6, 2),
        unit("skeleton", "enemy", 6, 6),
        unit("bonearcher", "enemy", 8, 1),
        unit("bonearcher", "enemy", 8, 7),
      ],
    ),
    subtitle: "Three crossings. Three homes to reclaim.",
    briefing:
      "Capture all three eastern villages. Split your force or secure the central bridge first. Enemy reinforcements arrive on turn 5; the villages fund whoever controls them.",
    goals: [
      { kind: "control", label: "Light all three eastern villages", tiles: ["7,1", "7,4", "7,7"] },
    ],
    recruits: ["soldier", "archer", "scout", "warden"],
    enemyRecruits: ["skeleton", "bonearcher"],
    gold: 18,
    enemyGold: 0,
    intro: [
      line(
        "Warden Sella",
        "warden",
        "Their bowmen are watching the bridge. My spears can hold the banks, but the skeletons will barely feel their points.",
      ),
      line(
        "Nera",
        "commander",
        "Keep the hammer close. Ivo, find a ford. We need those villages earning for us, not for the tollkeeper.",
      ),
    ],
    victory: [
      line(
        "Warden Sella",
        "warden",
        "All three windows are lit. One family even lent us their kettle.",
      ),
      line(
        "Ivo",
        "scout",
        "There is still a bell sounding in the marsh. Every time it rings, another grave opens.",
      ),
      line("Nera", "commander", "Then we go to the bell."),
    ],
    events: [
      {
        id: "road-cache",
        trigger: { kind: "visit", tile: "4,8" },
        label: "Recover the southern road cache",
        gold: 6,
        dialogue: [
          line(
            "Ivo",
            "scout",
            "Dry blankets, clean bandages, and six coins. Someone expected the road to open again. I would hate to disappoint them.",
          ),
        ],
      },
    ],
    waves: [
      {
        round: 5,
        label: "Two skeletons enter from the eastern road on turn 5",
        units: [unit("skeleton", "enemy", 11, 1), unit("skeleton", "enemy", 11, 7)],
      },
    ],
  },
  {
    map: map(
      "lantern-bell",
      "The Bell in the Reeds",
      15,
      9,
      [
        ...river(6, 9, { 2: "bridge", 4: "ford", 6: "bridge" }),
        keep(3, 4, "player"),
        keep(13, 4, "enemy"),
        ...land("village", [
          [3, 2],
          [3, 6],
        ]),
        ...land("village", [[13, 1]], "enemy"),
        ...land("ruins", [
          [9, 1],
          [11, 4],
          [9, 7],
          [7, 1],
          [7, 7],
        ]),
        ...land("marsh", [
          [7, 3],
          [7, 4],
          [7, 5],
          [12, 2],
          [12, 6],
        ]),
        ...land("fort", [
          [10, 3],
          [10, 5],
        ]),
        ...land("forest", [
          [8, 2],
          [8, 6],
        ]),
        ...land("grove", [[4, 3]]),
        ...land("hill", [
          [5, 0],
          [5, 8],
        ]),
      ],
      [
        unit("commander", "player", 3, 4, "Captain Nera", "leader"),
        unit("adept", "player", 4, 3),
        unit("soldier", "player", 4, 4),
        unit("scout", "player", 4, 5),
        unit("necromancer", "enemy", 13, 4, "The Bellwarden", "leader"),
        unit("skeleton", "enemy", 8, 1),
        unit("skeleton", "enemy", 8, 7),
        unit("skeleton", "enemy", 11, 3),
        unit("skeleton", "enemy", 11, 5),
        unit("bonearcher", "enemy", 10, 2),
        unit("bonearcher", "enemy", 10, 6),
      ],
    ),
    subtitle: "Silence the graves before silencing the bell.",
    briefing:
      "Seal the three grave ruins by entering them, then defeat the Bellwarden. An unsealed grave raises a skeleton on turns 8 and 12. Your forward camp is near the crossings. Ruins beyond each flank bridge offer cover; your scout can reach the graves once the shields clear a crossing. A sealed grave stays quiet.",
    goals: [
      { kind: "seal", label: "Seal the three grave ruins", tiles: ["9,1", "11,4", "9,7"] },
      { kind: "defeat", label: "Defeat the Bellwarden" },
    ],
    recruits: ["soldier", "archer", "scout", "warden", "adept"],
    enemyRecruits: ["skeleton", "bonearcher"],
    gold: 20,
    enemyGold: 0,
    intro: [
      line(
        "Adept Meri",
        "adept",
        "The bell does not make the dead. It calls them from these graves. Stand on a grave and I can close the binding.",
      ),
      line(
        "Nera",
        "commander",
        "We can hit the walls all day and still lose the road. Seal the flanks, protect Meri, then take the bell.",
      ),
      line("Meri", "adept", "My stone sparks hurt bones. Do keep their swords away from me."),
    ],
    victory: [
      line("Ivo", "scout", "It stopped. Listen. Just frogs."),
      line("Nera", "commander", "That is a better bell than any I have heard."),
    ],
    events: [
      {
        id: "reeds",
        trigger: { kind: "visit", tile: "4,3" },
        label: "Meet the reed gatherers at the grove",
        gold: 6,
        dialogue: [
          line(
            "Reed gatherer",
            "scout",
            "We saved what we could. Please make sure the road is still open when our children come home.",
          ),
        ],
      },
    ],
    waves: [8, 12].flatMap((round) =>
      ["9,1", "11,4", "9,7"].map((tile, index) => ({
        round,
        label: `An unsealed grave raises a skeleton on turn ${round}`,
        requiresUnsealed: tile,
        units: [unit("skeleton", "enemy", [9, 11, 9][index]!, [1, 4, 7][index]!)],
      })),
    ),
  },
];

const green: MissionDefinition[] = [
  {
    map: map(
      "green-border",
      "Roots at the Border",
      11,
      9,
      [
        keep(1, 4, "player"),
        keep(9, 4, "enemy"),
        ...land(
          "grove",
          [
            [3, 1],
            [5, 4],
            [3, 7],
          ],
          "enemy",
        ),
        ...land("forest", [
          [2, 0],
          [2, 1],
          [3, 2],
          [4, 3],
          [4, 5],
          [3, 6],
          [2, 7],
          [2, 8],
          [6, 2],
          [6, 6],
        ]),
        ...land("fort", [
          [7, 3],
          [7, 5],
        ]),
        ...land("hill", [
          [4, 2],
          [4, 6],
        ]),
        ...land("marsh", [
          [5, 0],
          [5, 8],
        ]),
        ...land("ruins", [[6, 1]]),
        ...land("village", [
          [2, 3],
          [2, 5],
          [8, 4],
        ]),
      ],
      [
        unit("wildkeeper", "player", 1, 4, "Alder", "leader"),
        unit("wolf", "player", 2, 3),
        unit("boar", "player", 2, 4),
        unit("thorncaller", "player", 2, 5, "Lark"),
        unit("commander", "enemy", 9, 4, "Captain Dain", "leader"),
        unit("warden", "enemy", 5, 1),
        unit("warden", "enemy", 6, 6),
        unit("archer", "enemy", 5, 3),
        unit("archer", "enemy", 7, 5),
      ],
    ),
    subtitle: "Take back the roots without burning the branches.",
    briefing:
      "Reclaim the three sacred groves. Wolves cross woods quickly, but enemy spears and arrows hurt them. Use boars to hold lanes and Alder’s Impact attacks against armored wardens.",
    goals: [
      { kind: "control", label: "Reclaim the three sacred groves", tiles: ["3,1", "5,4", "3,7"] },
    ],
    recruits: ["wolf", "boar", "thorncaller"],
    enemyRecruits: ["soldier", "archer", "warden"],
    gold: 12,
    enemyGold: 0,
    intro: [
      line(
        "Lark",
        "thorncaller",
        "They call the trees an obstacle. They have marked three groves for cutting.",
      ),
      line(
        "Alder",
        "wildkeeper",
        "Then we stand where their marks are. No chase into their fort. We need our roots back, not their bones.",
      ),
      line(
        "Lark",
        "thorncaller",
        "The wolves know the narrow paths. Their hunters know where to point a spear. Let the boars go first when the woods run out.",
      ),
    ],
    victory: [
      line(
        "Captain Dain",
        "warden",
        "We came for timber. Something in the old grove has been killing our woodcutters. Was that you?",
      ),
      line(
        "Alder",
        "wildkeeper",
        "No. Come back at dusk, with empty hands, and I will show you what wakes beneath the stones.",
      ),
    ],
    events: [
      {
        id: "lost-tools",
        trigger: { kind: "visit", tile: "6,1" },
        label: "Return the lost woodcutters’ tools",
        gold: 6,
        dialogue: [
          line(
            "Lark",
            "thorncaller",
            "Tools, not trophies. Leave them where their owners will find them. There are six coins tucked in the handle; we can use those for the vigil.",
          ),
        ],
      },
    ],
    waves: [],
  },
  {
    map: map(
      "green-vigil",
      "A Vigil Under Leaves",
      13,
      9,
      [
        keep(2, 4, "player"),
        keep(11, 4, "enemy"),
        ...land("grove", [[6, 4]], "player"),
        ...land("forest", [
          [4, 2],
          [4, 3],
          [4, 5],
          [4, 6],
          [5, 2],
          [5, 6],
          [7, 2],
          [7, 6],
        ]),
        ...land("fort", [
          [7, 3],
          [7, 5],
        ]),
        ...land("marsh", [
          [5, 0],
          [5, 1],
          [5, 7],
          [5, 8],
        ]),
        ...land("village", [
          [3, 1],
          [3, 7],
        ]),
        ...land("village", [[10, 4]], "enemy"),
        ...land("ruins", [[6, 0]]),
        ...land("hill", [
          [9, 2],
          [9, 6],
        ]),
      ],
      [
        unit("wildkeeper", "player", 2, 4, "Alder", "leader"),
        unit("thorncaller", "player", 2, 3, "Lark"),
        unit("boar", "player", 6, 4),
        unit("commander", "enemy", 11, 4, "The Overseer", "leader"),
        unit("warden", "enemy", 7, 2),
        unit("archer", "enemy", 8, 4),
        unit("soldier", "enemy", 8, 6),
      ],
    ),
    subtitle: "The grove must still be yours at dawn.",
    briefing:
      "Control the central grove at the start of your turn, from turn 6 onward. Its healing helps whoever owns it. Eastern patrols arrive on turns 3 and 5; you can take their fortifications and use their cover.",
    goals: [{ kind: "hold", label: "Hold the central grove until turn 6", tile: "6,4", round: 6 }],
    recruits: ["wolf", "boar", "thorncaller"],
    enemyRecruits: ["soldier", "archer"],
    gold: 16,
    enemyGold: 0,
    intro: [
      line("Alder", "wildkeeper", "The grove needs one night to recover. We hold it until dawn."),
      line(
        "Lark",
        "thorncaller",
        "The overseer has sent patrols along both roads. One on the third turn, another on the fifth. The wooden forts are theirs only while they stand in them.",
      ),
    ],
    victory: [
      line(
        "Captain Dain",
        "warden",
        "I saw the thing under the stones. It wore a dead man’s face and my lord’s seal.",
      ),
      line(
        "Alder",
        "wildkeeper",
        "Then take our witness to your people. A forest and a village can share a road.",
      ),
    ],
    events: [
      {
        id: "owl-cache",
        trigger: { kind: "visit", tile: "6,0" },
        label: "Recover supplies from the northern ruin",
        gold: 6,
        dialogue: [
          line(
            "Lark",
            "thorncaller",
            "The owls found a forgotten patrol cache. Better six coins for a living grove than for another axe.",
          ),
        ],
      },
    ],
    waves: [
      {
        round: 3,
        label: "A warden and archer arrive from the east on turn 3",
        units: [unit("warden", "enemy", 11, 1), unit("archer", "enemy", 11, 7)],
      },
      {
        round: 5,
        label: "An adept and soldier arrive from the east on turn 5",
        units: [unit("adept", "enemy", 10, 3), unit("soldier", "enemy", 10, 5)],
      },
    ],
  },
  {
    map: map(
      "green-pact",
      "A Road for Both",
      15,
      9,
      [
        keep(1, 4, "player"),
        keep(13, 5, "enemy"),
        ...river(8, 9, { 1: "ford", 4: "bridge", 7: "ford" }),
        ...land("fort", [[7, 4]]),
        ...land("village", [
          [3, 2],
          [3, 6],
          [12, 1],
        ]),
        ...land("village", [[12, 7]], "enemy"),
        ...land("grove", [
          [5, 2],
          [5, 6],
          [7, 1],
          [10, 1],
        ]),
        ...land("forest", [
          [4, 1],
          [4, 3],
          [4, 5],
          [4, 7],
          [10, 3],
          [10, 5],
        ]),
        ...land("marsh", [
          [7, 7],
          [9, 7],
        ]),
        ...land("ruins", [
          [6, 0],
          [9, 1],
        ]),
        ...land("hill", [
          [11, 2],
          [11, 6],
        ]),
      ],
      [
        unit("wildkeeper", "player", 1, 4, "Alder", "leader"),
        unit("thorncaller", "player", 2, 4, "Lark", "courier"),
        unit("warden", "player", 3, 4, "Captain Dain"),
        unit("adept", "player", 2, 5, "Adept Tamsin"),
        unit("necromancer", "enemy", 13, 5, "The Rootbinder", "leader"),
        unit("skeleton", "enemy", 10, 2),
        unit("skeleton", "enemy", 10, 6),
        unit("bonearcher", "enemy", 12, 3),
        unit("bonearcher", "enemy", 11, 7),
        unit("wolf", "enemy", 11, 4),
      ],
    ),
    subtitle: "A witness, a bridge, and a promise.",
    briefing:
      "Claim the bridge fort and bring Lark to the northern village beyond the river. Groves along the northern route offer healing and ranged cover. Dain brings wardens and adepts: spears counter wolves, while stone sparks hurt skeletal armor. Keep the witness behind the fighting line.",
    goals: [
      { kind: "control", label: "Secure the fort beside the bridge", tiles: ["7,4"] },
      { kind: "escort", label: "Bring Lark to the northern village", tile: "12,1" },
    ],
    recruits: ["wolf", "boar", "thorncaller", "warden", "adept"],
    enemyRecruits: ["skeleton", "bonearcher", "wolf"],
    gold: 18,
    enemyGold: 0,
    intro: [
      line(
        "Dain",
        "warden",
        "Lark carries a living branch. When it flowers, no one can pretend the grove was only lumber.",
      ),
      line(
        "Lark",
        "thorncaller",
        "It will flower. But I would rather not test that while a skeleton is shooting at me.",
      ),
      line(
        "Alder",
        "wildkeeper",
        "Secure the bridge. The witness comes first. Dain, your spears and our boars hold the open road together.",
      ),
      line(
        "Tamsin",
        "adept",
        "The dead hardly feel arrows, but stone sparks crack bone. Let the shields hold them while I work. Lark can rest in the groves beyond the ford.",
      ),
    ],
    victory: [
      line("Lark", "thorncaller", "It flowered."),
      line(
        "Dain",
        "warden",
        "Then I will say what I saw. And I will keep saying it until they hear.",
      ),
      line("Alder", "wildkeeper", "Leave the bridge open. That is what a pact is for."),
    ],
    events: [
      {
        id: "old-trail",
        trigger: { kind: "visit", tile: "6,0" },
        label: "Find the old trail supplies",
        gold: 6,
        dialogue: [
          line(
            "Lark",
            "thorncaller",
            "Someone left these for a traveler who never came. Today, someone will.",
          ),
        ],
      },
    ],
    waves: [],
  },
];

const ashen: MissionDefinition[] = [
  {
    map: map(
      "ashen-quarry",
      "No Name in the Ledger",
      13,
      7,
      [
        keep(1, 3, "player"),
        keep(11, 5, "enemy"),
        ...river(4, 7, { 1: "ford", 4: "bridge" }),
        ...land("hill", [
          [6, 2],
          [6, 3],
          [7, 4],
        ]),
        ...land("ruins", [
          [5, 5],
          [7, 0],
          [10, 0],
          [5, 1],
        ]),
        ...land("fort", [[8, 5]]),
        ...land("marsh", [[5, 2]]),
        ...land("village", [
          [2, 5],
          [6, 1],
          [8, 0],
        ]),
        ...land("village", [[10, 5]], "enemy"),
      ],
      [
        unit("necromancer", "player", 1, 3, "Orin", "leader"),
        unit("skeleton", "player", 2, 3),
        unit("bonearcher", "player", 2, 2),
        unit("skeleton", "player", 2, 4),
        unit("skeleton", "player", 1, 2),
        unit("commander", "enemy", 11, 5, "The Ledgerkeeper", "leader"),
        unit("soldier", "enemy", 9, 5),
        unit("archer", "enemy", 10, 4),
      ],
    ),
    subtitle: "The first order you give yourself.",
    briefing:
      "Escape with Orin to the northern quarry gate. Recruit before leaving your keep. The guards hold the southern road; the northern ford offers a way around them. Three skeletons can screen Orin while he crosses. Bones resist arrows; northern ruins provide ranged cover, and owned villages heal their occupants. An adept and warden arrive from the southern road on turn 6. You do not need to defeat every guard.",
    goals: [{ kind: "escape", label: "Bring Orin to the northern quarry gate", tile: "11,1" }],
    recruits: ["skeleton", "bonearcher"],
    enemyRecruits: ["soldier", "archer"],
    gold: 14,
    enemyGold: 0,
    intro: [
      line(
        "Orin",
        "necromancer",
        "The binding broke at midnight. I waited for another order. None came.",
      ),
      line("A hollow voice", "skeleton", "Then give us one of our own."),
      line(
        "Orin",
        "necromancer",
        "We leave the quarry. Keep the archers behind the bones. If the mage finds me, there may not be a second midnight.",
      ),
    ],
    victory: [
      line("Orin", "necromancer", "The gate is open. I thought freedom would sound louder."),
      line("A hollow voice", "skeleton", "It sounds like no one telling us to go back."),
      line(
        "Orin",
        "necromancer",
        "There are empty houses west of here. Let us give them neighbors.",
      ),
    ],
    events: [
      {
        id: "ledger",
        trigger: { kind: "visit", tile: "5,5" },
        label: "Recover the forgotten workers’ ledger",
        gold: 6,
        dialogue: [
          line(
            "Orin",
            "necromancer",
            "They wrote down our wages. They did not write down our names. Take the six coins. We can begin with that.",
          ),
        ],
      },
    ],
    waves: [
      {
        round: 6,
        label: "An adept and warden arrive from the southern road on turn 6",
        units: [unit("adept", "enemy", 11, 6), unit("warden", "enemy", 10, 6)],
      },
    ],
  },
  {
    map: map(
      "ashen-refuge",
      "The Quiet Refuge",
      11,
      9,
      [
        keep(1, 4, "player"),
        keep(9, 4, "enemy"),
        ...land("village", [
          [5, 1],
          [5, 7],
          [2, 6],
        ]),
        ...land("village", [[9, 2]], "enemy"),
        ...land("ruins", [[5, 4]]),
        ...land("forest", [
          [4, 2],
          [4, 6],
          [7, 1],
          [7, 7],
        ]),
        ...land("marsh", [
          [6, 3],
          [6, 5],
        ]),
        ...land("grove", [[3, 0]]),
        ...land("fort", [
          [7, 3],
          [7, 5],
        ]),
        ...land("hill", [
          [3, 3],
          [3, 5],
        ]),
      ],
      [
        unit("necromancer", "player", 1, 4, "Orin", "leader"),
        unit("skeleton", "player", 5, 4),
        unit("skeleton", "player", 2, 5),
        unit("bonearcher", "player", 2, 3),
        unit("commander", "enemy", 9, 4, "Captain Dain", "leader"),
        unit("soldier", "enemy", 6, 3),
        unit("warden", "enemy", 6, 5),
        unit("adept", "enemy", 8, 4),
      ],
    ),
    subtitle: "Empty houses need not stay empty.",
    briefing:
      "Claim both outer villages and control the central refuge at the start of your turn, from turn 6 onward. Supply patrols arrive on turns 3 and 5. Your units heal in owned villages and groves just like the living.",
    goals: [
      { kind: "control", label: "Claim the two outer villages", tiles: ["5,1", "5,7"] },
      { kind: "hold", label: "Keep the central refuge until turn 6", tile: "5,4", round: 6 },
    ],
    recruits: ["skeleton", "bonearcher"],
    enemyRecruits: ["soldier", "archer"],
    gold: 16,
    enemyGold: 0,
    intro: [
      line(
        "Orin",
        "necromancer",
        "The houses are empty, but the wells are clean. We can stop running here.",
      ),
      line("Captain Dain", "warden", "My orders say you are dangerous."),
      line(
        "Orin",
        "necromancer",
        "Then watch what we do. We will hold the houses, not march on yours.",
      ),
    ],
    victory: [
      line(
        "Dain",
        "warden",
        "You put your archers between my patrol and those villagers. My orders did not mention that.",
      ),
      line("Orin", "necromancer", "The man who wrote your orders used to write mine."),
      line("Dain", "warden", "Then I know which road we take next."),
    ],
    events: [
      {
        id: "medicine",
        trigger: { kind: "visit", tile: "3,0" },
        label: "Find medicine for the village neighbors",
        gold: 6,
        dialogue: [
          line(
            "Orin",
            "necromancer",
            "We have little use for the herbs. The people next door do. They insist on paying us. I think that is what neighbors do.",
          ),
        ],
      },
    ],
    waves: [
      {
        round: 3,
        label: "An archer and soldier arrive on turn 3",
        units: [unit("archer", "enemy", 9, 1), unit("soldier", "enemy", 9, 7)],
      },
      { round: 5, label: "An adept arrives on turn 5", units: [unit("adept", "enemy", 9, 3)] },
    ],
  },
  {
    map: map(
      "ashen-dawn",
      "A Dawn of Our Own",
      15,
      9,
      [
        keep(1, 4, "player"),
        keep(13, 4, "enemy"),
        ...land("ruins", [
          [7, 2],
          [7, 6],
          [12, 0],
        ]),
        ...land("marsh", [
          [5, 1],
          [5, 2],
          [5, 6],
          [5, 7],
          [8, 3],
          [8, 5],
        ]),
        ...land("grove", [[4, 4]]),
        ...land("fort", [
          [10, 3],
          [10, 5],
        ]),
        ...land("forest", [
          [6, 0],
          [6, 1],
          [6, 7],
          [6, 8],
          [11, 2],
          [11, 6],
        ]),
        ...land("hill", [
          [9, 2],
          [9, 6],
        ]),
        ...land("village", [
          [3, 1],
          [3, 7],
        ]),
        ...land("village", [[13, 1]], "enemy"),
        ...land("water", [
          [8, 0],
          [8, 1],
          [8, 7],
          [8, 8],
        ]),
      ],
      [
        unit("necromancer", "player", 1, 4, "Orin", "leader"),
        unit("skeleton", "player", 2, 3),
        unit("warden", "player", 2, 4, "Captain Dain"),
        unit("necromancer", "enemy", 13, 4, "The Master", "leader"),
        unit("skeleton", "enemy", 10, 2),
        unit("skeleton", "enemy", 10, 6),
        unit("boar", "enemy", 9, 4),
        unit("bonearcher", "enemy", 11, 1),
        unit("bonearcher", "enemy", 11, 7),
        unit("thorncaller", "enemy", 12, 4),
      ],
    ),
    subtitle: "Break the chains. Leave the names.",
    briefing:
      "Seal both binding stones and defeat the Master. Dain brings wardens and adepts into your recruitment pool. Unsealed stones raise a skeleton on turn 5; enter them first to stop it.",
    goals: [
      { kind: "seal", label: "Break both binding stones", tiles: ["7,2", "7,6"] },
      { kind: "defeat", label: "Defeat the Master" },
    ],
    recruits: ["skeleton", "bonearcher", "warden", "adept"],
    enemyRecruits: ["skeleton", "bonearcher", "boar"],
    gold: 20,
    enemyGold: 0,
    intro: [
      line("The Master", "necromancer", "I gave you a second life. You owe me the use of it."),
      line("Orin", "necromancer", "A life is not a ledger. These people choose whom they follow."),
      line(
        "Dain",
        "warden",
        "My wardens chose the same road. Break his binding stones. The adepts can answer his bones; my spears can keep the beasts off them.",
      ),
    ],
    victory: [
      line("The Master", "necromancer", "Without me, what are you?"),
      line("Orin", "necromancer", "We have not decided yet. That is the point."),
    ],
    events: [
      {
        id: "names",
        trigger: { kind: "visit", tile: "12,0" },
        label: "Recover the stolen names from the old archive",
        gold: 6,
        dialogue: [
          line(
            "Orin",
            "necromancer",
            "Here they are. All the names he took. We will return them, one house at a time.",
          ),
        ],
      },
    ],
    waves: ["7,2", "7,6"].map((tile, index) => ({
      round: 5,
      label: "An unsealed binding stone raises a skeleton on turn 5",
      requiresUnsealed: tile,
      units: [unit("skeleton", "enemy", 7, [2, 6][index]!)],
    })),
  },
];

export const CAMPAIGNS = immutable<Record<CampaignId, CampaignDefinition>>({
  "lantern-road": {
    id: "lantern-road",
    name: "The Lantern Road",
    tagline: "A message becomes a road home.",
    description:
      "Lead Captain Nera through a haunted river valley. Escort a courier, reclaim villages, and silence the graves. Human infantry, wardens, and adepts face the undead.",
    race: "human",
    leader: "commander",
    missions: lantern,
    ending: [
      line(
        "Nera",
        "commander",
        "We will leave a lantern at each crossing. Not as a warning. As a way home.",
      ),
      line("Ivo", "scout", "I have a new list. This one has everyone who made it."),
    ],
  },
  "green-accord": {
    id: "green-accord",
    name: "The Green Accord",
    tagline: "A forest and a village can share a road.",
    description:
      "Play as Alder’s woodland pack. Reclaim sacred groves, hold a vigil, and escort a witness to peace. Wolves, boars, and thorncallers each have a different job.",
    race: "beast",
    leader: "wildkeeper",
    missions: green,
    ending: [
      line("Lark", "thorncaller", "The first cart crossed today. It carried apples, not axes."),
      line("Alder", "wildkeeper", "Keep the road open. The roots will do the rest."),
    ],
  },
  "ashen-oath": {
    id: "ashen-oath",
    name: "The Ashen Oath",
    tagline: "Freedom is an order you give yourself.",
    description:
      "Lead Orin’s unbound dead out of servitude. Escape a quarry, defend a refuge, and break the Master’s bindings with unexpected living allies.",
    race: "undead",
    leader: "necromancer",
    missions: ashen,
    ending: [
      line("Dain", "warden", "What should I put in the new ledger?"),
      line(
        "Orin",
        "necromancer",
        "Names first. And leave a blank page. We may become more than you expected.",
      ),
    ],
  },
});

export const CAMPAIGN_MAPS = Object.fromEntries(
  Object.values(CAMPAIGNS).flatMap((campaign) =>
    campaign.missions.map((mission) => [mission.map.id, mission.map]),
  ),
) as Record<CampaignMapId, MapDefinition>;
