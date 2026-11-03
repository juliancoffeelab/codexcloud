/** Historical mission/story shapes used only by frozen save conversion. */
import type {
  UnitType,
  StartingUnit,
  MapDefinition,
  DialogueLine,
  CampaignId,
  Race,
} from "../../../game/types";
export type MissionGoal = (
  | { kind: "escort"; label: string; tile: string }
  | { kind: "escape"; label: string; tile: string }
  | { kind: "control"; label: string; tiles: string[] }
  | {
      kind: "hold";
      label: string;
      tile: string;
      turns: number;
      earliestRound: number;
      startEvent?: string;
      companion?: string;
      unitTypes?: string[];
      once?: boolean;
    }
  | { kind: "vigil"; label: string; tile: string; turns: number; companion: string }
  | { kind: "refuge"; label: string; tile: string; turns: number }
  | { kind: "seal"; label: string; tiles: string[]; unitType?: UnitType }
  | { kind: "event"; label: string; tile?: string; event: string }
  | { kind: "defeat"; label: string }
) & { visibleWhen?: import("../../../../rust/pkg/hex_command").Condition };
export interface MissionEvent {
  id: string;
  recruitment?: Record<string, import("../../../../rust/pkg/hex_command").FactionRecruit[]>;
  spawn?: import("../../../../rust/pkg/hex_command").SpawnGroup;
  trigger:
    | { kind: "condition"; condition: import("../../../../rust/pkg/hex_command").Condition }
    | { kind: "visit"; tile: string }
    | { kind: "round"; round: number }
    | { kind: "held"; tile: string; turns: number }
    | { kind: "control"; tile: string; round?: number };
  requiresEvent?: string;
  unlessEvent?: string;
  beforeRound?: number;
  label?: string;
  dialogue: DialogueLine[];
  gold?: number;
  warDialogue?: DialogueLine[];
}
export interface ReinforcementWave {
  dialogue?: DialogueLine[];
  /** Stable within its mission; saved deployment must never depend on array order. */
  id: string;
  round: number;
  label: string;
  units: StartingUnit[];
  requiresUnsealed?: string;
}
export interface MissionDefinition {
  itemIds?: string[];
  map: MapDefinition;
  subtitle: string;
  briefing: string;
  debrief?: string;
  requiredCompanions?: string[];
  arrivals?: { character: string; tile: string }[];
  hints: string[];
  goals: MissionGoal[];
  optionalGoals?: MissionGoal[];
  recruitment: Record<string, import("../../../../rust/pkg/hex_command").FactionRecruit[]>;
  gold: import("../../../../rust/pkg/hex_command").Gold;
  /** Occupying guards patrol their control objectives instead of raiding the starting camp. */
  defendObjectives?: boolean;
  /** An enemy leader guards this post and its immediate approaches; retreat remains free. */
  enemyLeaderPost?: string;
  intro: DialogueLine[];
  victory: DialogueLine[];
  events: MissionEvent[];
  waves: ReinforcementWave[];
  /** Authored story changes when a named ally has fallen, without resurrecting them. */
  companionLosses?: Record<
    string,
    Partial<
      Pick<
        MissionDefinition,
        "briefing" | "debrief" | "hints" | "intro" | "victory" | "recruitment"
      >
    >
  >;
  refuge?: {
    medicineEvent: string;
    medicineTurns: number;
    escalation: DialogueLine[];
    warVictory: DialogueLine[];
    warBriefing: string;
    warHints: string[];
  };
  withoutDain?: {
    briefing: string;
    hints: string[];
    intro: DialogueLine[];
    recruitment: Record<string, import("../../../../rust/pkg/hex_command").FactionRecruit[]>;
    reinforcements: StartingUnit[];
    victory?: DialogueLine[];
    debrief?: string;
  };
}
export interface CampaignDefinition {
  characters: { id: string; name: string }[];
  playerDiplomacy: ("pragmatic" | "restrained")[];
  id: CampaignId;
  name: string;
  tagline: string;
  description: string;
  shortDescription: string;
  race: Race;
  leader: UnitType;
  missions: MissionDefinition[];
  ending: DialogueLine[];
}
