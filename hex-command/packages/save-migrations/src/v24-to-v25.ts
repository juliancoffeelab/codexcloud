import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";

/** Freeze the formerly implicit player/enemy hostility; preserve authored diplomacy. */
export function migrateV24ToV25(input: unknown): Save<25> {
  const value = readSave(input, 24);
  if (value.version !== 24 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-twenty-four save");
  const old = value.checkpoint.state;
  if (
    !Array.isArray(old.factions) ||
    !old.factions.every(
      (entry) => isRecord(entry) && (entry.relations === undefined || isRecord(entry.relations)),
    )
  )
    throw new Error("Invalid historical factions");
  const player = old.factions.find((faction) => faction.id === "player");
  const enemy = old.factions.find((faction) => faction.id === "enemy");
  const explicit =
    (isRecord(player?.relations) && player.relations.enemy !== undefined) ||
    (isRecord(enemy?.relations) && enemy.relations.player !== undefined);
  const factions =
    !player || !enemy || explicit
      ? old.factions
      : old.factions.map((faction) => {
          if (faction.id !== "player" && faction.id !== "enemy") return faction;
          return {
            ...faction,
            relations: {
              ...(isRecord(faction.relations) ? faction.relations : {}),
              [faction.id === "player" ? "enemy" : "player"]: "hostile" as const,
            },
          };
        });
  return {
    ...value,
    version: 25,
    checkpoint: { ...value.checkpoint, state: { ...old, factions } },
  };
}
