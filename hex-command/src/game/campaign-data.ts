import { immutable } from "./immutable";
import { AUTHORED_CAMPAIGNS } from "./authored-content";
import type { CampaignMapId, MapDefinition } from "./types";

export const CAMPAIGNS = immutable(AUTHORED_CAMPAIGNS);
export const CAMPAIGN_MAPS = Object.fromEntries(
  Object.values(CAMPAIGNS).flatMap((campaign) =>
    campaign.missions.map((mission) => [mission.map.id, mission.map]),
  ),
) as Record<CampaignMapId, MapDefinition>;
