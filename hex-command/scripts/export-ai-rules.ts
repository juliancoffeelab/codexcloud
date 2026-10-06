import { rustFingerprint } from "./ai-fingerprint";
import { mkdirSync, writeFileSync } from "node:fs";
import { Game, TYPES, TERRAIN } from "../src/game/engine";
import { RACES } from "../src/game/races";
import { MAPS } from "../src/game/maps";
import { CAMPAIGNS } from "../src/game/campaign-data";
import type { TerrainType } from "../src/game/types";
const terrainNames = Object.keys(TERRAIN) as TerrainType[];
const probe = new Game();
const units = Object.entries(TYPES).map(([id, definition]) => ({
  id,
  ...definition,
  costs: terrainNames.map((terrain) => {
    const cost = probe.cost({ type: id as keyof typeof TYPES }, { terrain });
    return Number.isFinite(cost) ? cost : null;
  }),
  environment: terrainNames.map((terrain) =>
    (["melee", "ranged"] as const).map((category) =>
      (["blade", "pierce", "impact"] as const).map((damageType) => {
        const modifiers = [
          TERRAIN[terrain].damageModifiers,
          RACES[definition.race].terrainModifiers[terrain]?.damageModifiers,
          definition.terrainModifiers?.[terrain]?.damageModifiers,
        ];
        return modifiers.reduce(
          (sum, rule) => sum + (rule?.[category] ?? 0) + (rule?.[damageType] ?? 0),
          0,
        );
      }),
    ),
  ),
}));
const maps = Object.keys(MAPS).map((id) => new Game(id).captureState());
maps.push(new Game("river", 1).captureState());
maps.push(new Game("ashen-refuge", 1).captureState());
const data = {
  sourceFingerprint: rustFingerprint(),
  terrainNames,
  terrainDefinitions: terrainNames.map((id) => TERRAIN[id]),
  units,
  maps,
  campaigns: CAMPAIGNS,
};
mkdirSync("rust/data", { recursive: true });
writeFileSync("rust/data/rules.json", JSON.stringify(data) + "\n");
console.log(
  `Exported ${units.length} unit rules, ${maps.length} map layouts and ${Object.keys(CAMPAIGNS).length} campaigns.`,
);
