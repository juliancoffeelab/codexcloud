/** Published layouts needed only when reconstructing version-one checkpoints. */
import refuge from "./data/ashen-refuge-v1.json";
import refugeRoster from "./data/ashen-refuge-v1-roster.json";
import type { LegacyStartingUnit } from "./frozen";
import type { MapDefinition } from "./game-types";
export const LEGACY_ASHEN_REFUGE = Object.freeze({
  id: refuge.mapId as MapDefinition["id"],
  revision: refuge.mapRevision,
  name: refuge.mapId,
  description: "",
  cols: refuge.cols,
  rows: refuge.rows,
  terrain: refuge.tiles as MapDefinition["terrain"],
  units: refugeRoster.units as LegacyStartingUnit[],
});
