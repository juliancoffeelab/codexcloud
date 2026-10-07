/** Published layouts needed only when reconstructing version-one checkpoints. */
import { AUTHORED_SCENARIOS } from "../../../game/authored-content";
import { immutable } from "../../../game/immutable";
export const LEGACY_RIVER = immutable(
  AUTHORED_SCENARIOS.find((map) => map.id === "river" && map.revision === 1)!,
);
export const LEGACY_ASHEN_REFUGE = immutable(
  AUTHORED_SCENARIOS.find((map) => map.id === "ashen-refuge" && map.revision === 1)!,
);
