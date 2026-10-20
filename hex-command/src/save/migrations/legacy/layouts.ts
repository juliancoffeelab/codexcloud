/** Published layouts needed only when reconstructing version-one checkpoints. */
import refuge from "./data/ashen-refuge-v1.json";
import refugeRoster from "./data/ashen-refuge-v1-roster.json";
import type { MapDefinition } from "../../../game/types";
import { immutable } from "../../../game/immutable";
export const LEGACY_ASHEN_REFUGE = immutable({
  id: refuge.mapId as MapDefinition["id"],
  revision: refuge.mapRevision,
  name: refuge.mapId,
  description: "",
  cols: refuge.cols,
  rows: refuge.rows,
  terrain: refuge.tiles as MapDefinition["terrain"],
  units: refugeRoster.units as MapDefinition["units"],
});
