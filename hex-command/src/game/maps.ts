import { immutable } from "./immutable";
import { AUTHORED_SCENARIOS } from "./authored-content";
import { CAMPAIGN_MAPS } from "./campaign-data";
import type { MapDefinition, MapId } from "./types";

export const MAPS = immutable({
  ...CAMPAIGN_MAPS,
  ...Object.fromEntries(AUTHORED_SCENARIOS.map((map) => [map.id, map])),
} as Record<MapId, MapDefinition>);
