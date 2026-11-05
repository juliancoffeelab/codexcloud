import { CAMPAIGNS } from "../game/campaign-data";
import { MAPS } from "../game/maps";
import type { CampaignId, MapId } from "../game/types";
import type { ProgressEntry } from "../save/progress";

export function runName(entry: ProgressEntry): string {
  return CAMPAIGNS[entry.run as CampaignId]?.name ?? MAPS[entry.run as MapId]?.name ?? "Saved game";
}
export function chapter(entry: ProgressEntry): { number: number; name: string } {
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
