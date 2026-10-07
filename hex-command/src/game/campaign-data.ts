import { immutable } from "./immutable";
import { AUTHORED_CAMPAIGNS, AUTHORED_SCENARIOS } from "./authored-content";
import type { CampaignMapId, MapDefinition } from "./types";

export const CAMPAIGNS = immutable(AUTHORED_CAMPAIGNS);
export const LEGACY_ASHEN_REFUGE = immutable(
  AUTHORED_SCENARIOS.find((map) => map.id === "ashen-refuge" && map.revision === 1)!,
);
export const CAMPAIGN_MAPS = Object.fromEntries(
  Object.values(CAMPAIGNS).flatMap((campaign) =>
    campaign.missions.map((mission) => [mission.map.id, mission.map]),
  ),
) as Record<CampaignMapId, MapDefinition>;
