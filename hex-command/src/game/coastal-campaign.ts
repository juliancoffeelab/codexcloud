import type {
  CampaignDefinition,
  CoastalMapId,
  DialogueLine,
  MapDefinition,
  MissionDefinition,
  StartingUnit,
  TerrainType,
  UnitType,
  Side,
} from "./types";

const line = (speaker: string, portrait: UnitType | "worker", text: string): DialogueLine => ({
  speaker,
  portrait,
  text,
});
const unit = (
  type: UnitType,
  side: Side,
  col: number,
  row: number,
  name?: string,
  role?: StartingUnit["role"],
): StartingUnit => ({ type, side, col, row, ...(name ? { name } : {}), ...(role ? { role } : {}) });
const legend: Record<string, TerrainType> = {
  "~": "water",
  s: "ford",
  ".": "plains",
  F: "forest",
  H: "hill",
  V: "village",
  K: "keep",
  T: "fort",
  R: "ruins",
  M: "marsh",
  "=": "bridge",
  G: "grove",
};
/** Authored coastlines: each row is a complete, fixed map, not generated at reset. */
function map(
  id: CoastalMapId,
  name: string,
  description: string,
  rows: string[],
  units: StartingUnit[],
): MapDefinition {
  return {
    id,
    name,
    description,
    cols: 12,
    rows: rows.length,
    terrain: rows.flatMap((row, r) =>
      [...row].map((symbol, col) => ({
        col,
        row: r,
        terrain: legend[symbol]!,
        ...(symbol === "K" ? { owner: col < 4 ? ("player" as const) : ("enemy" as const) } : {}),
      })),
    ),
    units,
  };
}
const keepCompany = (col: number, row: number) => [
  unit("tidekeeper", "player", col, row, "Mira", "leader"),
  unit("reefguard", "player", col + 1, row),
  unit("harpooner", "player", col, row + 1),
  unit("wavemender", "player", col + 1, row + 1),
];
const goalsOne: MissionDefinition["goals"] = [
  { kind: "control", label: "Open both sounding outworks", tiles: ["6,2", "8,5"] },
  { kind: "escort", label: "Bring pilot Edda to the fishing village", tile: "9,3" },
];
const missions: MissionDefinition[] = [
  {
    map: map(
      "tide-nets",
      "The Empty Nets",
      "A tidal estuary, offshore sandbars and a wooded mainland. Swim around the causeway or contest it.",
      [
        "~~~~~~~~ssHH",
        "~~~~~ss..FFH",
        "~~K~ssT..FF.",
        "~~s~~ss..V..",
        "~ss~sV.=....",
        "~~~~~~ssT.K.",
        "~~~ss~sM.FF.",
        "~~~ssV.MMFF.",
        "~~~~~~~ss...",
      ],
      [
        ...keepCompany(2, 2),
        unit("harpooner", "player", 4, 4, "Edda", "courier"),
        unit("commander", "enemy", 10, 5, "Postmaster Harl", "leader"),
        unit("spearman", "enemy", 6, 2),
        unit("swordsman", "enemy", 8, 5),
        unit("archer", "enemy", 9, 4),
      ],
    ),
    subtitle: "Someone has borrowed tomorrow's tide.",
    briefing:
      "Rescue pilot Edda from the blocked estuary. Capture both sounding outworks and escort her to the fishing village, where she can show the townspeople why their boats are stranded.",
    debrief:
      "The outworks held a forged sluice order. Edda's soundings show water diverted into an abandoned quarry, exposing the wreck of a debt ship. The salvage factor is making workers recover its ledgers before anyone can compare the books.",
    hints: [
      "Merfolk can cross deep water and move normally on plains. Forests, groves and hills slow them and increase incoming damage.",
      "Shallows are neutral for merfolk; swamps protect them. Land troops are exposed in both.",
      "Keep Edda alive. A harpooner can skirt the outworks by sea; reef guards and wavemenders counter the armored shore guards.",
    ],
    goals: goalsOne,
    recruits: ["reefguard", "harpooner", "wavemender"],
    enemyRecruits: ["spearman", "archer", "swordsman"],
    gold: 18,
    enemyGold: 8,
    intro: [
      line(
        "Mira",
        "tidekeeper",
        "The tide should have returned by dawn. Our nets are dry; their boats are on the mud. This is no quarrel either shore can afford.",
      ),
      line(
        "Edda",
        "harpooner",
        "Vale's men took the sounding posts. I saw a sluice order bearing the council seal, but the date was cut from an older sheet.",
      ),
      line(
        "Postmaster Harl",
        "commander",
        "Captain Vale sent us to hold these posts for the factor's salvage crew. Bring your accusation to the council; you will not cross here.",
      ),
      line(
        "Mira",
        "tidekeeper",
        "Then we take the posts and bring the pilot. Leave the fishing houses standing.",
      ),
    ],
    legacyDialogue: [
      line(
        "Captain Vale",
        "commander",
        "The factor pays us to keep this channel clear for salvage. Bring your accusation to the council; you will not cross my posts.",
      ),
    ],
    victory: [
      line(
        "Edda",
        "harpooner",
        "Here are my soundings, and his order. The quarry gate is open. The water did not disappear; someone moved it.",
      ),
      line(
        "Mira",
        "tidekeeper",
        "We cannot just close that gate. There are workers inside the wreck, below the next high tide.",
      ),
    ],
    events: [
      {
        id: "nets",
        trigger: { kind: "visit", tile: "5,7" },
        label: "Recover provisions from the southern fishing hamlet",
        gold: 4,
        dialogue: [
          line(
            "Fisher",
            "worker",
            "The factor bought our food on credit too. Take these nets and rations. Bring back the people he sent into that ship.",
          ),
        ],
      },
    ],
    waves: [],
  },
  {
    map: map(
      "tide-wreck",
      "The Drowned Ledger",
      "A wreck on an exposed sandbar; two sea channels flank a marshy shore and forested quarry road.",
      [
        "~~~~~ssHHFFF",
        "~~~ss..HFF..",
        "~K~~sV...FK.",
        "~ss~~sM..F..",
        "~~~~sRss=...",
        "~~~ssMsV.F..",
        "~~~~ssM..T..",
        "~~~~~ss..FF.",
        "~~~~~~~ss...",
      ],
      [
        ...keepCompany(1, 2),
        unit("harpooner", "player", 3, 3, "Edda", "courier"),
        unit("spearman", "player", 4, 2, "Tomas"),
        unit("commander", "enemy", 10, 2, "Foreman Cale", "leader"),
        unit("soldier", "enemy", 8, 4),
        unit("archer", "enemy", 8, 5),
        unit("spearman", "enemy", 7, 2),
      ],
    ),
    subtitle: "The books can wait. The workers cannot.",
    briefing:
      "Enter the wreck to recover the original ledger, then defend its evacuation landing for three ferry crossings. The final boat leaves at high tide, no earlier than turn 6. Edda guides workers through the exposed hull while your company holds the channel.",
    debrief:
      "The last worker leaves with the ledger. Senn had paid Vale to seize the posts, then charged the villagers for salvage they never requested. Tomas deserted when ordered to burn the hull with workers inside. He knows the wooded route to the upper gate.",
    requiredCompanions: ["Edda"],
    companionLosses: {
      Tomas: {
        debrief:
          "The workers leave with the ledger, but Tomas dies defending their escape. The rescued clerk explains that Senn's upper gate must open with the lower one to avoid flooding the village. His fellow deserters offer to guide the company along the wooded bank.",
        victory: [
          line(
            "Edda",
            "harpooner",
            "Last boat away. The clerk has the books. Tomas did not make it back.",
          ),
          line(
            "Mira",
            "tidekeeper",
            "We carry his testimony too. The clerk says we need both gates and the beacon; nobody else will die in a release we could have prevented.",
          ),
        ],
      },
    },
    hints: [
      "Reach the wreck at 5,4 with any unit to recover the ledger. Defend its ownership through three enemy turns, then wait for high tide on turn 6. Your troops can leave it to screen the workers; enemy recapture resets the crossings.",
      "Tomas and other land recruits can use forest cover. Merfolk are exposed there; use the parallel sea and marsh routes.",
      "Promotions preserve wounds, add abilities and consume a turn. Ready veterans can advance before a difficult fight.",
    ],
    goals: [
      {
        kind: "event",
        label: "Recover the original ledger from the wreck",
        tile: "5,4",
        event: "ledger",
      },
      {
        kind: "hold",
        label: "Defend three ferry crossings; high tide begins on turn 6",
        tile: "5,4",
        turns: 3,
        earliestRound: 6,
      },
    ],
    recruits: ["reefguard", "harpooner", "wavemender", "spearman", "adept"],
    enemyRecruits: ["swordsman", "spearman", "archer"],
    gold: 14,
    enemyGold: 10,
    intro: [
      line(
        "Tomas",
        "spearman",
        "I was hired to guard a salvage crew. Today Senn ordered us to set the ship alight. There are still people down there.",
      ),
      line(
        "Edda",
        "harpooner",
        "I need three ferry crossings to bring everyone out along the keel. Hold the landing. The last boat leaves with the sixth bell's high tide, or later if we are delayed; nobody touches the quarry gate before then.",
      ),
      line(
        "Foreman Cale",
        "commander",
        "The factor's ledger is company property. Clear the landing and burn the books.",
      ),
      line(
        "Mira",
        "tidekeeper",
        "A debt cannot give you ownership of a person. We take the books after we get everyone out.",
      ),
    ],
    legacyDialogue: [
      line("Factor Senn", "commander", "That ledger is company property. Vale, clear the landing."),
      line(
        "Edda",
        "harpooner",
        "I can bring them out along the keel. Hold this landing until the sixth bell; nobody touches the quarry gate before then.",
      ),
    ],
    victory: [
      line(
        "Edda",
        "harpooner",
        "Last boat away. The clerk brought the books with her. Every order, every payment.",
      ),
      line(
        "Tomas",
        "spearman",
        "Senn has shut the upper gate now. If we open only the lower one, the quarry water will flood the village. We need both controls and the beacon.",
      ),
    ],
    events: [
      {
        id: "ledger",
        trigger: { kind: "visit", tile: "5,4" },
        label: "Recover the unaltered salvage ledger",
        dialogue: [
          line(
            "Clerk",
            "worker",
            "These are the original accounts. He rewrote the council's order, then made each village pay for the delay. Keep them dry; I still have two colleagues below.",
          ),
        ],
      },
    ],
    waves: [
      {
        round: 4,
        label: "Senn's hired guard arrives along the wooded shore",
        id: "senns-guards",
        legacyIndex: 0,
        units: [unit("swordsman", "enemy", 10, 1), unit("archer", "enemy", 11, 5)],
      },
    ],
  },
  {
    map: map(
      "tide-sluice",
      "A Tide for Both",
      "Twin sluice channels with a wooded high bank, a marsh bypass and an offshore beacon.",
      [
        "~~~~ssHHFFF.",
        "~~~sV.HFF..K",
        "~K~~s.MFT...",
        "~ss~~sM=....",
        "~~~~sV~~sR..",
        "~~~ssMMsT...",
        "~~~~ssM.FF..",
        "~~V~~ss.FF..",
        "~~~~~~~~ss..",
      ],
      [
        ...keepCompany(1, 2),
        unit("harpooner", "player", 3, 3, "Edda", "courier"),
        unit("spearman", "player", 4, 1, "Tomas"),
        unit("commander", "enemy", 11, 1, "Captain Vale", "leader"),
        unit("soldier", "enemy", 8, 2),
        unit("warden", "enemy", 8, 5),
        unit("archer", "enemy", 9, 3),
        unit("scout", "enemy", 8, 6),
      ],
    ),
    subtitle: "One gate for the town, one for the sea.",
    briefing:
      "Capture the two sluice controls and escort Edda to the beacon. She will signal a safe release only after both gates are in your hands. Senn's guards defend the gates to prevent the council from discovering the diversion.",
    debrief:
      "At Edda's signal both controls release together. The estuary fills without a surge through the village. The recovered books reach the council; Senn loses his lease. The fishing towns and the reef appoint a shared tide crew, with neither shore allowed to operate a gate alone.",
    hints: [
      "All three objectives are required: controls at 8,2 and 8,5, and Edda at the beacon at 9,4.",
      "Use land allies through the wooded high bank and swimmers through the lower marsh channel. Neither route requires crossing the other army's strongest terrain.",
      "Impact attacks break the armored gate guards; harpoons punish scouts and cover advancing allies. Wavemenders heal adjacent allies at turn start.",
    ],
    goals: [
      { kind: "control", label: "Control both sluice gates", tiles: ["8,2", "8,5"] },
      { kind: "escort", label: "Bring Edda to the release beacon", tile: "9,4" },
    ],
    companionLosses: {
      Tomas: {
        intro: [
          line(
            "Mira",
            "tidekeeper",
            "Tomas's companions showed us the high bank through the woods. They will lead our land fighters. We take the lower wheel through the reeds.",
          ),
          line(
            "Captain Vale",
            "commander",
            "Senn says the council will hang deserters first. Your testimony will not save you.",
          ),
          line(
            "Mira",
            "tidekeeper",
            "He forged the council's order. Hear the clerk, or answer for holding this flood above the village.",
          ),
          line(
            "Edda",
            "harpooner",
            "Bring me to the beacon. I will signal only when both wheels are ours.",
          ),
        ],
        victory: [
          line("Edda", "harpooner", "Both flags up. Open together—now."),
          line("Mira", "tidekeeper", "The tide is home. Tomas should have seen it."),
          line(
            "Edda",
            "harpooner",
            "Put someone from each shore beside each wheel. Keep this from happening again.",
          ),
        ],
      },
    },
    recruits: ["reefguard", "harpooner", "wavemender", "swordsman", "spearman", "adept"],
    enemyRecruits: ["swordsman", "spearman", "archer"],
    gold: 16,
    enemyGold: 12,
    intro: [
      line(
        "Tomas",
        "spearman",
        "The high gate is reached through the woods. I can take that path. Your swimmers can reach the lower wheel through the reeds.",
      ),
      line(
        "Captain Vale",
        "commander",
        "You have taken a company's books and attacked its guards. Senn says the council will hang deserters first.",
      ),
      line(
        "Mira",
        "tidekeeper",
        "He forged the council's order. The clerk will testify. Stand aside and hear her, or answer for keeping this flood above the village.",
      ),
      line(
        "Edda",
        "harpooner",
        "They are staying. Bring me to the beacon. I will not signal until both wheels are ours.",
      ),
    ],
    victory: [
      line("Edda", "harpooner", "Both flags up. Open together—now."),
      line("Mira", "tidekeeper", "Watch the mudflats. There. The tide is home."),
      line("Tomas", "spearman", "And tomorrow?"),
      line("Mira", "tidekeeper", "Tomorrow we put someone from each shore beside each wheel."),
    ],
    events: [
      {
        id: "boat",
        trigger: { kind: "visit", tile: "2,7" },
        label: "Recover the southern ferry's supplies",
        gold: 4,
        dialogue: [
          line(
            "Boatkeeper",
            "worker",
            "We could not leave our homes, but we saved food for your company. Make sure that beacon means a safe tide.",
          ),
        ],
      },
    ],
    waves: [],
  },
];

export const COASTAL_CAMPAIGN: CampaignDefinition = {
  id: "borrowed-tide",
  name: "The Borrowed Tide",
  tagline: "Neither shore can keep the tide alone.",
  description:
    "Lead Mira's merfolk through a drained estuary, rescue a salvage crew and coordinate two sluice gates. Open sea routes, sandbars, wooded shores and marsh channels reward different troops.",
  race: "merfolk",
  leader: "tidekeeper",
  missions,
  ending: [
    line(
      "Edda",
      "harpooner",
      "The next sounding chart has two signatures. One from the quay, one from the reef.",
    ),
    line(
      "Mira",
      "tidekeeper",
      "And if the numbers disagree, we go out together and measure again.",
    ),
  ],
};
