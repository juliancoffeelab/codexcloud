/** Browser metadata comes from Rust's validated, resolved WASM catalog. */
import { loadNativeEngineFactory } from "./native-engine";
import type {
  CampaignDefinition,
  CampaignId,
  MapDefinition,
  Race,
  RaceDefinition,
  TerrainDefinition,
  TerrainType,
  UnitDefinition,
  UnitType,
} from "./types";

const loaded = await loadNativeEngineFactory(
  () => import("../../rust/pkg/hex_command"),
  undefined,
  { responseOwnership: "transferred" },
);
if (!loaded.ok) throw new Error(`Cannot load game catalog: ${JSON.stringify(loaded.error)}`);
/** The browser shares this catalog owner with its battle controller. */
export const GAME_ENGINE_FACTORY = loaded.value;
const response = GAME_ENGINE_FACTORY.catalog();
if (!response.ok) {
  GAME_ENGINE_FACTORY.dispose();
  throw new Error(`Cannot read game catalog: ${JSON.stringify(response.error)}`);
}
const catalog = response.value;

function keyed<T extends { id: string }>(definitions: readonly T[]) {
  return Object.fromEntries(definitions.map((definition) => [definition.id, definition]));
}

// These casts narrow authored identifiers to the UI vocabulary. Rust validates
// the values and inheritance; TypeScript does not parse or resolve game rules.
export const AUTHORED_ITEMS = keyed(catalog.items);
export const AUTHORED_UNITS = keyed(catalog.units) as unknown as Record<UnitType, UnitDefinition>;
export const AUTHORED_RACES = keyed(catalog.races) as unknown as Record<Race, RaceDefinition>;
export const AUTHORED_TERRAIN = keyed(catalog.terrain) as unknown as Record<
  TerrainType,
  TerrainDefinition
>;
export const AUTHORED_SCENARIOS = catalog.maps as unknown as MapDefinition[];
export const TERRAIN_ORDER = catalog.terrain.map((terrain) => terrain.id);
export const AUTHORED_CAMPAIGNS = Object.fromEntries(
  catalog.campaigns.map((campaign) => [
    campaign.id,
    {
      ...campaign,
      missions: campaign.missions.map((mission) => ({
        ...mission,
        map: {
          ...AUTHORED_SCENARIOS.find((map) => map.id === mission.map.id)!,
          units: mission.map.units,
        },
      })),
    },
  ]),
) as unknown as Record<CampaignId, CampaignDefinition>;
