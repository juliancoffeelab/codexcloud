import type { MissionDefinition } from "./mission-types";
/** Historical save metadata stays private to migration; live definitions come from WASM. */
import { FROZEN_CAMPAIGNS } from "./frozen";
import type { CampaignId, DialogueLine } from "../../../game/types";
import stages from "./data/stages.json";
import type { Immutable } from "../../../game/immutable";

interface WaveHistory {
  id: string;
  legacyIndex: number;
  previouslyDeployedBeforeRevision?: number;
}
interface StageHistory {
  dialogue: DialogueLine[];
  waves: WaveHistory[];
}
type MigrationMission = Omit<Immutable<MissionDefinition>, "waves" | "map"> & {
  map: Immutable<(typeof FROZEN_CAMPAIGNS)[CampaignId]["missions"][number]["map"]>;
  waves: (Immutable<MissionDefinition["waves"][number]> & Partial<WaveHistory>)[];
  legacyDialogue: DialogueLine[];
};
const history = stages as unknown as Record<string, StageHistory>;
/** Add the old marker and dialogue vocabulary only at the importer boundary. */
export const CAMPAIGNS = Object.fromEntries(
  Object.entries(FROZEN_CAMPAIGNS).map(([id, campaign]) => [
    id,
    {
      ...campaign,
      missions: campaign.missions.map((mission): MigrationMission => ({
        ...mission,
        legacyDialogue: history[mission.map.id]?.dialogue ?? [],
        waves: mission.waves.map((wave) => ({
          ...wave,
          ...history[mission.map.id]?.waves.find((old) => old.id === wave.id),
        })),
      })),
    },
  ]),
) as Record<
  CampaignId,
  Omit<(typeof FROZEN_CAMPAIGNS)[CampaignId], "missions"> & {
    missions: MigrationMission[];
  }
>;
