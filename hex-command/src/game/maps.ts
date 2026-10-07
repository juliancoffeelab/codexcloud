import { immutable } from "./immutable";
import { AUTHORED_SCENARIOS } from "./authored-content";
import { CAMPAIGN_MAPS } from "./campaign-data";
import type { MapDefinition, MapId } from "./types";

export const MAPS = immutable<Record<MapId, MapDefinition>>({
  ...CAMPAIGN_MAPS,
  // Historical revisions are retained for restore but never replace current maps.
  ...AUTHORED_SCENARIOS.filter((map) => map.id !== "ashen-refuge").reduce(
    (maps, map) => {
      if (!maps[map.id] || (map.revision ?? 1) > (maps[map.id]!.revision ?? 1)) maps[map.id] = map;
      return maps;
    },
    {} as Record<MapId, MapDefinition>,
  ),
});
