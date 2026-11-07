/** Historical flat-choice campaign shape; later save fields must not change old importers. */
import type { CampaignId, CampaignVeteran, DialogueLine, RecruitType } from "./game-types";
export interface LegacyCampaignState {
  id: CampaignId;
  mission: number;
  recall: RecruitType[];
  veterans?: CampaignVeteran[];
  storyVeterans?: CampaignVeteran[];
  fallenCompanions?: string[];
  carryGold: number;
  marks: string[];
  dialogue: DialogueLine[];
  dialogueIndex: number;
  outcome: string | null;
  completed: boolean;
  refuge?: { heldTurns: number; lastCountedRound: number; escalated: boolean };
  vigil?: { heldTurns: number; lastCountedRound: number };
  holds?: Record<string, { heldTurns: number; lastCountedRound: number }>;
  choices?: string[];
}
