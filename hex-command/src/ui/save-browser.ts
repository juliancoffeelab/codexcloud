import { CAMPAIGNS } from "../game/campaign-data";
import { MAPS } from "../game/maps";
import type { CampaignId, MapId } from "../game/types";
import type { SaveInfo, HistoryEntry } from "../save/metadata";

export function runName(entry: SaveInfo): string {
  return CAMPAIGNS[entry.run as CampaignId]?.name ?? MAPS[entry.run as MapId]?.name ?? "Saved game";
}
export function chapter(entry: SaveInfo): { number: number; name: string } {
  const match = /^Mission (\d+) · (.+)$/.exec(entry.mission);
  const map = (match?.[2] ?? entry.mission) as MapId;
  return { number: match ? Number(match[1]) : 0, name: MAPS[map]?.name ?? runName(entry) };
}
export function roman(value: number): string {
  let remaining = value;
  let result = "";
  for (const [amount, symbol] of [
    [1000, "M"],
    [900, "CM"],
    [500, "D"],
    [400, "CD"],
    [100, "C"],
    [90, "XC"],
    [50, "L"],
    [40, "XL"],
    [10, "X"],
    [9, "IX"],
    [5, "V"],
    [4, "IV"],
    [1, "I"],
  ] as const)
    while (remaining >= amount) {
      result += symbol;
      remaining -= amount;
    }
  return result;
}
export function saveTime(value: number): string {
  const date = new Date(value);
  const today = date.toDateString() === new Date().toDateString();
  const time = date.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
  return `${today ? "Today" : date.toLocaleDateString()}, ${time}`;
}

/** Stable newest-first order, including saves with identical timestamps. */
export function saveOrder(a: SaveInfo, b: SaveInfo): number {
  return b.savedAt - a.savedAt || a.id.localeCompare(b.id);
}
export function savedGames<T extends SaveInfo>(sources: readonly T[]) {
  const games = new Map<string, { id: string; latest: T; earliest: T }>();
  for (const entry of sources) {
    const game = games.get(entry.runId);
    if (!game) games.set(entry.runId, { id: entry.runId, latest: entry, earliest: entry });
    else {
      if (saveOrder(entry, game.latest) < 0) game.latest = entry;
      if (saveOrder(entry, game.earliest) > 0) game.earliest = entry;
    }
  }
  return [...games.values()].sort((a, b) => saveOrder(a.latest, b.latest));
}
/** Group adjacent chapters only; revisiting a chapter creates another heading. */
export function chapterGroups(entries: readonly HistoryEntry[]) {
  const groups: { mission: string; number: number; name: string; entries: HistoryEntry[] }[] = [];
  for (const entry of entries) {
    const previous = groups.at(-1);
    if (previous?.mission === entry.mission) previous.entries.push(entry);
    else groups.push({ mission: entry.mission, ...chapter(entry), entries: [entry] });
  }
  return groups;
}
