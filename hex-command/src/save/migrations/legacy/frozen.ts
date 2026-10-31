/** Frozen v6 conversion metadata; never consult the live gameplay catalog. */
import snapshot from "./data/catalog-v6.json";
import type {
  CampaignDefinition,
  CampaignId,
  Coordinates,
  MapDefinition,
  MapId,
  UnitDefinition,
  UnitType,
} from "../../../game/types";

export const MAPS = Object.fromEntries(
  Object.entries(snapshot.maps).map(([id, map]) => [
    id,
    {
      ...map,
      terrain: map.terrain.map(([col, row, terrain, owner, floor, connections, deadEnds]) => ({
        id: `${col},${row}`,
        col,
        row,
        terrain,
        owner,
        ...(floor === null ? {} : { floor }),
        ...(connections === null ? {} : { connections }),
        ...(deadEnds === null ? {} : { deadEnds }),
      })),
    },
  ]),
) as unknown as Record<MapId, MapDefinition>;
export const TYPES = snapshot.units as unknown as Record<
  UnitType,
  Pick<UnitDefinition, "hp" | "experienceNeeded" | "family" | "costs">
>;
export const TERRAIN_ORDER = snapshot.terrainOrder;
export const FROZEN_CAMPAIGNS = Object.fromEntries(
  snapshot.campaigns.map((campaign) => [
    campaign.id,
    {
      ...campaign,
      missions: campaign.missions.map((mission) => ({
        ...mission,
        map: { ...MAPS[mission.map.id as MapId], units: mission.map.units },
      })),
    },
  ]),
) as unknown as Record<CampaignId, CampaignDefinition>;
export const CAMPAIGN_MAPS = Object.fromEntries(
  Object.values(FROZEN_CAMPAIGNS).flatMap((campaign) =>
    campaign.missions.map((mission) => [mission.map.id, mission.map]),
  ),
);

/** Historical offset-hex geometry used only by the pure data converter. */
export function distance(first: Coordinates, second: Coordinates) {
  const q = first.col - Math.floor(first.row / 2) - second.col + Math.floor(second.row / 2);
  const r = first.row - second.row;
  return (Math.abs(q) + Math.abs(r) + Math.abs(q + r)) / 2;
}
