import { immutable } from "./immutable";
import { COASTAL_CAMPAIGN } from "./coastal-campaign";
import { LEGACY_GREEN_DIALOGUE } from "./legacy-green-dialogue";
import type {
  CampaignDefinition,
  CampaignId,
  CampaignMapId,
  DialogueLine,
  DialoguePortrait,
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
const line = (speaker: string, portrait: DialoguePortrait, text: string): DialogueLine => ({
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
  revision = 1,
): MapDefinition => ({ id, name, description: name, revision, cols, rows, terrain, units });

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
        unit("swordsman", "player", 2, 3),
        unit("swordsman", "player", 2, 4),
        unit("scout", "player", 2, 2, "Ivo", "courier"),
        unit("necromancer", "enemy", 11, 4, "The Tollkeeper", "leader"),
        unit("skeleton", "enemy", 10, 5),
        unit("bonearcher", "enemy", 12, 5),
      ],
    ),
    subtitle: "Carry a message, not a conquest.",
    briefing:
      "The eastern villages have gone dark. Ivo carries their names and a message for the last ferryman; Nera promises to get him across the river. The Tollkeeper’s guards block the road, but this journey is about reaching the living, not conquering the valley.",
    hints: [
      "Ivo travels better in good company.",
      "An arrow finds little purchase in bare bones.",
      "The river has more than one crossing.",
    ],
    goals: [{ kind: "escort", label: "Bring Ivo to the northeastern landing", tile: "11,1" }],
    recruits: ["swordsman", "archer", "scout"],
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
        unit("swordsman", "player", 2, 5),
        unit("necromancer", "enemy", 11, 4, "The Tollkeeper", "leader"),
        unit("skeleton", "enemy", 6, 2),
        unit("skeleton", "enemy", 6, 6),
        unit("bonearcher", "enemy", 8, 1),
        unit("bonearcher", "enemy", 8, 7),
      ],
    ),
    subtitle: "Three crossings. Three homes to reclaim.",
    defendObjectives: true,
    briefing:
      "The ferryman has brought news of three settlements occupied by the dead. Nera’s company is staying to reopen the road and light their windows again. Sella’s wardens will help, but the Tollkeeper is still collecting from the other bank.",
    hints: [
      "One bridge need not carry the whole army.",
      "A welcoming village can keep a march alive. Stone sparks punish bones; shield their fragile adepts with infantry.",
    ],
    goals: [
      { kind: "control", label: "Light all three eastern villages", tiles: ["7,1", "7,4", "7,7"] },
    ],
    recruits: ["swordsman", "archer", "scout", "spearman", "adept"],
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
        ...land("grove", [[5, 1]]),
        ...land("hill", [
          [5, 0],
          [5, 8],
        ]),
      ],
      [
        unit("commander", "player", 3, 4, "Captain Nera", "leader"),
        unit("adept", "player", 4, 3),
        unit("swordsman", "player", 4, 4),
        unit("scout", "player", 4, 5),
        unit("necromancer", "enemy", 13, 4, "The Bellwarden", "leader"),
        unit("skeleton", "enemy", 8, 1),
        unit("skeleton", "enemy", 8, 7),
        unit("skeleton", "enemy", 11, 3),
        unit("skeleton", "enemy", 11, 5),
        unit("bonearcher", "enemy", 10, 2),
        unit("bonearcher", "enemy", 10, 6),
      ],
      2,
    ),
    subtitle: "Silence the graves before silencing the bell.",
    briefing:
      "The villages are lit again, but the bell keeps calling their dead back to war. Nera and the adepts have traced that call to the grave ruins. Breaking those bindings will let the company silence the Bellwarden and leave the valley a road home.",
    hints: [
      "The broken walls still cast a useful shadow.",
      "Listen for where the bell’s answer comes from.",
    ],
    goals: [
      {
        kind: "seal",
        label: "Use an adept to seal the three grave ruins",
        tiles: ["9,1", "11,4", "9,7"],
        unitType: "adept",
      },
      { kind: "defeat", label: "Defeat the Bellwarden" },
    ],
    recruits: ["swordsman", "archer", "scout", "spearman", "adept"],
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
        trigger: { kind: "visit", tile: "5,1" },
        label: "Meet the reed gatherers by the northern bridge",
        gold: 6,
        dialogue: [
          line(
            "Reed gatherer",
            "worker",
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
    requiredCompanions: ["Lark"],
    debrief:
      "Both logging camps are captured and the pay ledger contradicts the Overseer’s report. Dain suspends his patrol’s attacks and agrees to protect Lark’s investigation in the central grove. The Overseer’s other guards have accepted no truce.",
    subtitle: "Take their tools. Find the missing workers.",
    defendObjectives: true,
    briefing:
      "The village council licenses ordinary timber cutting, but its Overseer has sent crews into Alder’s sacred groves. He claims the pack killed the missing woodcutters. His tools and carts are stored in two guarded logging camps. Capture both camps to stop the cutting, and recover the workers’ pay ledger from the northern ruin. Captain Dain commands the patrol protecting those camps; leave him alive if you can.",
    hints: [
      "Capture both logging forts; standing in a marked grove does not stop the operation.",
      "Search the northern ruin for the workers’ ledger.",
      "Wolves cross woods easily. Boars and thorncaller staves fare better against armor.",
    ],
    goals: [
      {
        kind: "control",
        label: "Capture both logging camps",
        tiles: ["7,3", "7,5"],
      },
      {
        kind: "event",
        label: "Recover the workers’ pay ledger",
        tile: "6,1",
        event: "lost-tools",
      },
    ],
    recruits: ["wolf", "boar", "thorncaller"],
    enemyRecruits: ["swordsman", "archer", "spearman"],
    gold: 12,
    enemyGold: 0,
    intro: [
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "The council promised the villages repair timber. Its Overseer marked our sacred groves instead. His crews keep their tools and carts in those two forts.",
      },
      {
        speaker: "Alder",
        portrait: "wildkeeper",
        text: "Take the camps and the cutting stops. The northern post kept the workers’ records. Bring those back too.",
      },
      {
        speaker: "Captain Dain",
        portrait: "warden",
        text: "My missing woodcutters were sent here. The Overseer says your pack killed them. I have orders to protect the crews that remain.",
      },
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "Your orders won’t tell us where they went. Their ledger might.",
      },
      {
        speaker: "Alder",
        portrait: "wildkeeper",
        text: "We need our roots back, not your bones. Keep away from our groves, captain, and help us find your people.",
      },
    ],
    victory: [
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "Both camps are ours. Without their tools and carts, the crews have stopped cutting. Here is the ledger.",
      },
      {
        speaker: "Captain Dain",
        portrait: "warden",
        text: "These men were marked as deserters, but their wages went to the Overseer’s own account. That is his seal. I cannot explain it.",
      },
      {
        speaker: "Alder",
        portrait: "wildkeeper",
        text: "Lark can ask the grove what happened. Come and watch her work.",
      },
      {
        speaker: "Captain Dain",
        portrait: "warden",
        text: "Your pack has hurt my patrol, and mine has hurt yours. I am offering a truce for this investigation, not pretending that never happened.",
      },
    ],
    events: [
      {
        id: "lost-tools",
        trigger: {
          kind: "visit",
          tile: "6,1",
        },
        label: "Recover the workers’ pay ledger",
        dialogue: [
          {
            speaker: "Lark",
            portrait: "thorncaller",
            text: "The missing crew refused to cut the sacred grove until their overdue wages were paid. Then they were ordered to report to the Rootbinder at the Overseer’s river works. Their pay was never collected.",
          },
          {
            speaker: "Alder",
            portrait: "wildkeeper",
            text: "Take the ledger. We can stop the cutting, but the roots must tell us what became of those workers.",
          },
        ],
      },
    ],
    waves: [],
    withoutDain: {
      briefing:
        "Dain has fallen. Capture both logging camps to stop their crews, and recover the workers’ pay ledger from the northern ruin. His death does not answer what happened to the missing woodcutters; Lark must find evidence their families can hear without him.",
      hints: [
        "Capture both logging forts; standing in a marked grove does not stop the operation.",
        "Search the northern ruin for the workers’ ledger.",
        "Wolves cross woods easily. Boars and thorncaller staves fare better against armor.",
      ],
      intro: [
        {
          speaker: "Lark",
          portrait: "thorncaller",
          text: "Dain is dead. His patrol’s orders still do not tell us what happened to the workers.",
        },
        {
          speaker: "Alder",
          portrait: "wildkeeper",
          text: "Take both tool camps and recover their ledger. We still owe the families an answer.",
        },
      ],
      victory: [
        {
          speaker: "Lark",
          portrait: "thorncaller",
          text: "The camps are ours and the cutting has stopped. The missing workers’ wages went to the Overseer, and their last orders sent them to the Rootbinder.",
        },
        {
          speaker: "Alder",
          portrait: "wildkeeper",
          text: "Dain will never hear their story. Lark, bring it back in a living branch. The families must be able to hear it for themselves.",
        },
      ],
      debrief:
        "The cutting operation has lost both camps. Dain died in the fighting; his death remains part of what happened. The ledger leads Lark to question the roots about the Rootbinder and the Overseer.",
      recruits: ["wolf", "boar", "thorncaller"],
      reinforcements: [],
    },
    map: {
      ...map(
        "green-border",
        "The Cutting Camps",
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
          unit("spearman", "enemy", 7, 3),
          unit("spearman", "enemy", 7, 6),
          unit("archer", "enemy", 6, 2),
          unit("archer", "enemy", 7, 5),
        ],
      ),
      revision: 2,
    },
    legacyDialogue: LEGACY_GREEN_DIALOGUE[0],
  },
  {
    requiredCompanions: ["Lark"],
    debrief:
      "The vigil revealed a bargain: the Overseer supplied workers to the Rootbinder, who killed and bound them into unpaid logging crews. Lark carries their memories in a living branch. Dain will help bring that evidence to the village council; the Overseer’s false report can no longer be the only account.",
    subtitle: "A personal truce. An officer’s cover-up.",
    briefing:
      "Dain has suspended his patrol’s attacks and stands with Alder at the central grove. The Overseer, the council’s timber and toll officer, still commands the other paid charter guards. The ledger threatens his office; he calls Dain a mutineer and orders the grove silenced. Keep Lark in the central grove through three enemy turns while she listens to the workers’ memories. Dain is here to defend the investigation, not command the attacking guards.",
    hints: [
      "Move Lark into the central grove and protect her through three full enemy turns.",
      "Leaving the grove or losing ownership resets the vigil.",
      "Dain is your ally; the Overseer’s remaining charter guards are the enemy.",
    ],
    goals: [
      {
        kind: "vigil",
        label: "Protect Lark in the central grove through three enemy turns",
        tile: "6,4",
        turns: 3,
        companion: "Lark",
      },
    ],
    recruits: ["wolf", "boar", "thorncaller"],
    enemyRecruits: ["soldier", "archer"],
    gold: 16,
    enemyGold: 0,
    intro: [
      {
        speaker: "Captain Dain",
        portrait: "warden",
        text: "I suspended my patrol’s orders. These other guards answer to the Overseer. I will stand here with you until we know what happened.",
      },
      {
        speaker: "The Overseer",
        portrait: "commander",
        text: "Dain is dismissed. His truce binds none of you. Destroy the branch and break their circle before the council hears their lies.",
      },
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "Bring me to the central grove. The roots kept the workers’ last memories. If I am driven out, I must begin again.",
      },
      {
        speaker: "Alder",
        portrait: "wildkeeper",
        text: "Then we defend the grove, captain included. The Overseer has given us a reason to keep listening.",
      },
    ],
    victory: [
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "The workers asked for the pay he owed them and refused to cut this grove. The Overseer handed them to the Rootbinder. He killed them and bound their bodies to the logging crews.",
      },
      {
        speaker: "Captain Dain",
        portrait: "warden",
        text: "Unpaid labor for the Overseer. Bodies and protection for the Rootbinder. And my patrol sent to punish you for their murders.",
      },
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "This branch holds what the roots remember. Their families can hear it too.",
      },
      {
        speaker: "Alder",
        portrait: "wildkeeper",
        text: "The council is meeting across the river. We take the branch and the ledger there. Let the people who gave him his office judge what he did with it.",
      },
    ],
    events: [
      {
        id: "owl-cache",
        trigger: {
          kind: "visit",
          tile: "6,0",
        },
        label: "Recover the northern guard-post supplies",
        gold: 6,
        dialogue: [
          {
            speaker: "Alder",
            portrait: "wildkeeper",
            text: "The abandoned guard post still has funds for provisions. Use them to bring another defender to the grove; Lark cannot finish her work if we lose it.",
          },
        ],
      },
    ],
    waves: [
      {
        round: 3,
        label: "A warden and archer arrive from the east on turn 3",
        units: [
          {
            type: "warden",
            side: "enemy",
            col: 11,
            row: 1,
          },
          {
            type: "archer",
            side: "enemy",
            col: 11,
            row: 7,
          },
        ],
      },
      {
        round: 5,
        label: "An adept and soldier arrive from the east on turn 5",
        units: [
          {
            type: "adept",
            side: "enemy",
            col: 10,
            row: 3,
          },
          {
            type: "soldier",
            side: "enemy",
            col: 10,
            row: 5,
          },
        ],
      },
    ],
    withoutDain: {
      briefing:
        "Dain is dead. The ledger still names the Overseer and the Rootbinder, but the council needs more than accusations from the pack. Keep Lark in the central grove through three full enemy turns so she can carry the workers’ memories in a living branch. The Overseer’s remaining charter guards are attacking to destroy that evidence.",
      hints: [
        "Protect Lark in the grove through three enemy turns.",
        "Leaving or losing the grove resets the vigil.",
      ],
      intro: [
        {
          speaker: "Lark",
          portrait: "thorncaller",
          text: "Dain cannot be our witness. The roots can. Keep me in the central grove while I listen.",
        },
        {
          speaker: "The Overseer",
          portrait: "commander",
          text: "The captain is dead. Destroy that branch before the council hears a word of this.",
        },
        {
          speaker: "Alder",
          portrait: "wildkeeper",
          text: "They will hear the workers themselves. We defend the grove until Lark has their memories.",
        },
      ],
      victory: [
        {
          speaker: "Lark",
          portrait: "thorncaller",
          text: "The workers demanded their wages and refused to cut the grove. The Overseer delivered them to the Rootbinder; he killed and bound them into his logging crews.",
        },
        {
          speaker: "Alder",
          portrait: "wildkeeper",
          text: "Dain is among the dead now too. We cannot undo that. Bring the branch and the ledger to the families, and let them judge the whole account.",
        },
      ],
      debrief:
        "The roots revealed the workers’ murder and binding. Dain cannot testify, but the living branch carries their memories. The pack must take it and the ledger to the council across the river.",
      recruits: ["wolf", "boar", "thorncaller"],
      reinforcements: [
        {
          type: "boar",
          side: "player",
          col: 5,
          row: 5,
        },
      ],
    },
    map: {
      ...map(
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
          unit("warden", "player", 5, 5, "Captain Dain"),
          unit("commander", "enemy", 11, 4, "The Overseer", "leader"),
          unit("warden", "enemy", 9, 2),
          unit("archer", "enemy", 8, 4),
          unit("soldier", "enemy", 10, 6),
        ],
      ),
      revision: 2,
    },
    legacyDialogue: LEGACY_GREEN_DIALOGUE[1],
  },
  {
    requiredCompanions: ["Lark"],
    debrief:
      "Lark delivered the memories and the bridge fort remains secure. The council revoked the Overseer’s timber and toll charter. The villages and Alder’s pack agreed to protect the crossing and spare the sacred groves. The Rootbinder may still threaten the valley; this victory delivered the truth and established a road both peoples can defend.",
    subtitle: "A witness, a bridge, and a promise.",
    briefing:
      "The village council is meeting at the northern village across the river. Lark’s branch carries the murdered workers’ memories; the ledger and Dain’s testimony identify the Overseer who supplied them to the Rootbinder. The necromancer has barred the crossing to protect that bargain. Capture the bridge fort so the households can cross safely, and bring Lark to the meeting alive. Tamsin, a village adept, joins the escort against the bound dead.",
    hints: [
      "The northern trees may offer a gentler journey.",
      "A witness need not stand in the front rank.",
    ],
    goals: [
      {
        kind: "control",
        label: "Secure the fort beside the bridge",
        tiles: ["7,4"],
      },
      {
        kind: "escort",
        label: "Bring Lark to the northern village",
        tile: "12,1",
      },
    ],
    recruits: ["wolf", "boar", "thorncaller", "spearman", "adept"],
    enemyRecruits: ["skeleton", "bonearcher", "wolf"],
    gold: 18,
    enemyGold: 0,
    intro: [
      {
        speaker: "Captain Dain",
        portrait: "warden",
        text: "The council gave the Overseer his timber and toll charter. With the ledger and the workers’ memories, they can take it away. I will answer for my patrol’s part as well.",
      },
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "The Rootbinder has put his dead on the crossing. He needs the Overseer’s protection and his supply of bodies. He would rather burn this branch than let the families hear it.",
      },
      {
        speaker: "Alder",
        portrait: "wildkeeper",
        text: "Take the bridge fort so the households can cross. Bring Lark to the northern meeting alive; a dead necromancer alone will not open a shared road.",
      },
      {
        speaker: "Tamsin",
        portrait: "adept",
        text: "Dain sent for me. Some of those workers were my neighbors. Stone sparks crack bone; let the shields hold the dead while I clear your road.",
      },
    ],
    victory: [
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "The families heard the branch. The council matched the orders to the ledger and revoked the Overseer’s charter.",
      },
      {
        speaker: "Captain Dain",
        portrait: "warden",
        text: "I have given them my account too. We will harvest the ordinary woods for repairs. Your groves are outside the charter.",
      },
      {
        speaker: "Alder",
        portrait: "wildkeeper",
        text: "The fort is ours. Village and woodland guards can keep this road together; neither side owns the right to close it.",
      },
      {
        speaker: "Lark",
        portrait: "thorncaller",
        text: "The Rootbinder still has his dead. But he no longer has a silent village to hide them in.",
      },
    ],
    events: [
      {
        id: "old-trail",
        trigger: {
          kind: "visit",
          tile: "6,0",
        },
        label: "Recover provisions from the old crossing post",
        gold: 6,
        dialogue: [
          {
            speaker: "Lark",
            portrait: "thorncaller",
            text: "The old crossing post has provisions for a stranded escort. Bring another shield to the road; the families are waiting on the other bank.",
          },
        ],
      },
    ],
    waves: [],
    withoutDain: {
      briefing:
        "Dain is dead, but Lark’s living branch carries the workers’ memories and the ledger records their last orders. Tamsin, an adept from the villages, has joined the escort to help bring that evidence to the council. The Rootbinder bars the river to protect his supply of bodies. Capture the bridge fort and bring Lark to the northern meeting alive.",
      hints: [
        "The northern trees may offer a gentler journey.",
        "A witness need not stand in the front rank.",
      ],
      intro: [
        {
          speaker: "Tamsin",
          portrait: "adept",
          text: "I came from the villages to help you reach the council. The workers were our neighbors. Dain cannot speak; their families must hear this branch.",
        },
        {
          speaker: "Lark",
          portrait: "thorncaller",
          text: "The Rootbinder wants his bargain kept hidden. The branch and the ledger must reach the northern village.",
        },
        {
          speaker: "Alder",
          portrait: "wildkeeper",
          text: "Take the bridge fort and protect Lark. We owe the families an open road and an honest account, including Dain’s death.",
        },
      ],
      victory: [
        {
          speaker: "Lark",
          portrait: "thorncaller",
          text: "The families heard the workers’ memories. We told them about Dain too. The council has revoked the Overseer’s charter.",
        },
        {
          speaker: "Alder",
          portrait: "wildkeeper",
          text: "The fort is ours. The households will help us guard the crossing and keep the sacred groves outside their timber cutting.",
        },
        {
          speaker: "Lark",
          portrait: "thorncaller",
          text: "Then let the first travelers cross. We have enough graves; we need neighbors.",
        },
      ],
      debrief:
        "The council heard the branch, examined the ledger and learned of Dain’s death. It revoked the Overseer’s charter and agreed to protect the road with the pack. The Rootbinder remains a danger, but the workers’ murder is exposed and the crossing is held.",
      recruits: ["wolf", "boar", "thorncaller", "spearman", "adept"],
      reinforcements: [
        {
          type: "boar",
          side: "player",
          col: 3,
          row: 4,
        },
      ],
    },
    map: {
      ...map(
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
      revision: 2,
    },
    legacyDialogue: LEGACY_GREEN_DIALOGUE[2],
  },
];

export const LEGACY_ASHEN_REFUGE = map(
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
);

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
        unit("swordsman", "enemy", 9, 5),
        unit("archer", "enemy", 10, 4),
      ],
    ),
    subtitle: "The first order you give yourself.",
    briefing:
      "The Master’s binding broke at midnight. For the first time, Orin and the quarry’s dead can choose where to go. Orin leads them toward the northern gate before the guards can return them to servitude. Freedom begins with getting the company out alive.",
    hints: [
      "The guards watch the road more closely than the river.",
      "Bare bones have little to fear from arrows.",
      "Orin need not make the crossing alone.",
    ],
    goals: [{ kind: "escape", label: "Bring Orin to the northern quarry gate", tile: "11,1" }],
    recruits: ["skeleton", "bonearcher"],
    enemyRecruits: ["swordsman", "archer"],
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
    map: {
      ...LEGACY_ASHEN_REFUGE,
      revision: 2,
      terrain: LEGACY_ASHEN_REFUGE.terrain.map((tile) =>
        tile.terrain === "village" && tile.col === 5 ? { ...tile, col: 3 } : tile,
      ),
      units: LEGACY_ASHEN_REFUGE.units.map((unit) =>
        unit.side === "enemy" && unit.col === 6 ? { ...unit, col: 7 } : unit,
      ),
    },
    subtitle: "A refuge needs neighbors, not another war.",
    briefing:
      "Orin’s company has found empty houses beside living villages. Captain Dain has orders to drive the dead away; the villagers want time to judge their new neighbors for themselves. Orin will defend the refuge while they gather. Helping the sick can win their support sooner, but a withdrawing soldier’s death would destroy the chance of peace.",
    hints: [
      "Defend ownership of the refuge through six enemy turns. Troops can leave the ruins to screen it; enemy recapture resets the count. Medicine reduces the defense to five turns.",
      "Skeleton clubs punish the armored patrol. Bone bows answer its unarmored swordsmen, archers and adepts; protect those bows with a melee screen.",
      "Nonlethal defensive attacks are allowed. Killing Dain or a retreating soldier ends mediation; check the combat preview before attacking and leave withdrawing men a way home.",
    ],
    goals: [
      {
        kind: "refuge",
        label: "Defend the refuge until the villagers intervene",
        tile: "5,4",
        turns: 6,
      },
    ],
    recruits: ["skeleton", "bonearcher"],
    enemyRecruits: ["swordsman", "archer"],
    gold: 16,
    enemyGold: 0,
    intro: [
      line(
        "Orin",
        "necromancer",
        "These houses are empty. The people next door are not. We could be neighbors, if they let us stay.",
      ),
      line(
        "Captain Dain",
        "warden",
        "My orders say the dead must leave. My patrol is coming to enforce them.",
      ),
      line(
        "Village elder",
        "worker",
        "Give us six days to gather the households. Defend your shelter if you must, but let men who turn back go home. There are sick families here who need the herbs in the northern grove.",
      ),
      line(
        "Orin",
        "necromancer",
        "Then we hold our ground. We will not hunt your wounded. And we will bring the medicine.",
      ),
    ],
    victory: [
      line(
        "Village elder",
        "worker",
        "Enough, Captain. They held their shelter and let your wounded leave. These houses are ours to offer. The dead may stay as our neighbors.",
      ),
      line(
        "Dain",
        "warden",
        "My orders did not mention people who would ask the dead to stay. Perhaps the man who wrote them left other things out.",
      ),
      line(
        "Orin",
        "necromancer",
        "He used to write my orders too. Come with us, and see what he has been hiding.",
      ),
      line("Dain", "warden", "I will. My soldiers can rest here. I know which road we take next."),
    ],
    refuge: {
      medicineEvent: "medicine",
      medicineTurns: 5,
      warBriefing:
        "The attempt to live beside the villages has become an open battle. After the bloodshed, the households can no longer persuade the patrol to stand down. Orin’s company must break the remaining patrol to leave, and will face the Master without Dain’s help.",
      warHints: [
        "The patrol still has somewhere to run.",
        "The empty houses remain, even after peace is lost.",
      ],
      escalation: [
        line(
          "Village elder",
          "worker",
          "There has been too much bloodshed for the patrol to accept our mediation. We cannot stop this battle for you.",
        ),
        line(
          "Orin",
          "necromancer",
          "Then there is no agreement left to make. We must break the patrol before we can leave.",
        ),
      ],
      warVictory: [
        line(
          "Orin",
          "necromancer",
          "The patrol is broken. But we have lost the chance to leave here with friends. We face the Master with our own company.",
        ),
      ],
    },
    legacyDialogue: [
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
      line(
        "Dain",
        "warden",
        "You put your archers between my patrol and those villagers. My orders did not mention that.",
      ),
      line("Orin", "necromancer", "The man who wrote your orders used to write mine."),
      line("Dain", "warden", "Then I know which road we take next."),
      line(
        "Orin",
        "necromancer",
        "We have little use for the herbs. The people next door do. They insist on paying us. I think that is what neighbors do.",
      ),
    ],
    events: [
      {
        id: "medicine",
        trigger: { kind: "visit", tile: "3,0" },
        label: "Bring medicine to the villagers",
        gold: 6,
        warDialogue: [
          line(
            "Village elder",
            "worker",
            "The herbs will still save lives. Take these six coins. But after the bloodshed, medicine alone cannot stop the patrol’s battle with you.",
          ),
        ],
        dialogue: [
          line(
            "Village elder",
            "worker",
            "The fever is easing. We will send word with the families you helped; the households will gather a day sooner. Take these six coins for your shelter.",
          ),
          line(
            "Orin",
            "necromancer",
            "We did not need the herbs ourselves. I am glad someone did.",
          ),
        ],
      },
    ],
    waves: [
      {
        round: 3,
        label: "An archer and swordsman arrive on turn 3",
        units: [unit("archer", "enemy", 9, 1), unit("swordsman", "enemy", 9, 7)],
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
      "Dain has chosen to stand beside Orin against the man who commanded them both. The Master still binds others to his will through stones along the road. Breaking those chains and confronting him is the company’s chance to secure a freedom he cannot simply revoke.",
    hints: [
      "Dain’s company has more to offer than swords.",
      "The stones are where the Master’s reach begins.",
    ],
    goals: [
      { kind: "seal", label: "Break both binding stones", tiles: ["7,2", "7,6"] },
      { kind: "defeat", label: "Defeat the Master" },
    ],
    recruits: ["skeleton", "bonearcher", "spearman", "adept"],
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
    withoutDain: {
      briefing:
        "The refuge battle ended without an agreement. Dain and his allies will not come with Orin; the company must face the Master alone. His binding stones still hold others in servitude. Breaking them is the first step toward a freedom that can survive him.",
      hints: [
        "The dead will have to look after their own.",
        "The stones are where the Master’s reach begins.",
      ],
      recruits: ["skeleton", "bonearcher"],
      reinforcements: [unit("skeleton", "player", 2, 4), unit("bonearcher", "player", 1, 3)],
      intro: [
        line(
          "The Master",
          "necromancer",
          "You broke the patrol that came to turn you back. You have learned my methods well.",
        ),
        line(
          "Orin",
          "necromancer",
          "We chose that battle. We will carry what it cost us. But you will not choose our lives again.",
        ),
        line(
          "A hollow voice",
          "skeleton",
          "No captain is coming to help us. Break the binding stones. We can still bring the others home.",
        ),
      ],
    },
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
      "Lead Alder’s woodland pack against an illegal logging operation. Capture its camps, uncover the missing workers’ fate, and bring the evidence to the village council across the river.",
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
  "borrowed-tide": COASTAL_CAMPAIGN,
});

export const CAMPAIGN_MAPS = Object.fromEntries(
  Object.values(CAMPAIGNS).flatMap((campaign) =>
    campaign.missions.map((mission) => [mission.map.id, mission.map]),
  ),
) as Record<CampaignMapId, MapDefinition>;
