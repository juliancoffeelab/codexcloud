/** Published layouts needed only when reconstructing version-one checkpoints. */
import river from "./data/river-v1.json";
import refuge from "./data/ashen-refuge-v1.json";
import riverRoster from "./data/river-v1-roster.json";
import refugeRoster from "./data/ashen-refuge-v1-roster.json";
import type { MapDefinition } from "../../../game/types";
import { immutable } from "../../../game/immutable";
function layout(map: typeof river, roster: typeof riverRoster): MapDefinition {
  return {
    id: map.mapId as MapDefinition["id"],
    revision: map.mapRevision,
    name: map.mapId,
    description: "",
    cols: map.cols,
    rows: map.rows,
    terrain: map.tiles as MapDefinition["terrain"],
    units: roster.units as MapDefinition["units"],
  };
}
export const LEGACY_RIVER = immutable(layout(river, riverRoster));
export const LEGACY_ASHEN_REFUGE = immutable(layout(refuge, refugeRoster));
