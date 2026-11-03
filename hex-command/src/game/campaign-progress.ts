import { CAMPAIGNS } from "./campaign-data";
import type { Immutable } from "./immutable";
import type { CampaignState, MissionDefinition, Unit } from "./types";

export function campaignMission(state: Immutable<CampaignState>): Immutable<MissionDefinition> {
  let mission: Immutable<MissionDefinition> = CAMPAIGNS[state.id].missions[state.mission]!;
  if (
    (state.choices?.includes("refuge-war") ||
      state.choices?.includes("green-dain-dead") ||
      state.marks.includes("dain-fell")) &&
    mission.withoutDain
  )
    mission = { ...mission, ...mission.withoutDain };
  const fallen = new Set([
    ...(state.fallenCompanions ?? []),
    ...state.marks.filter((mark) => mark.startsWith("fallen:")).map((mark) => mark.slice(7)),
  ]);
  for (const name of fallen) {
    const changes = mission.companionLosses?.[name];
    if (changes) mission = { ...mission, ...changes };
  }
  if (
    [...mission.intro, ...mission.victory].some(
      (line) => !!line.character && fallen.has(line.character),
    )
  )
    mission = {
      ...mission,
      intro: mission.intro.filter((line) => !line.character || !fallen.has(line.character)),
      victory: mission.victory.filter((line) => !line.character || !fallen.has(line.character)),
    };
  if (state.refuge?.escalated && mission.refuge)
    return { ...mission, briefing: mission.refuge.warBriefing, hints: mission.refuge.warHints };
  return mission;
}

export function dialoguePending(state: Immutable<CampaignState> | undefined): boolean {
  return Boolean(state && state.dialogueIndex < state.dialogue.length);
}

/** Story companions already placed in the next chapter must not become duplicate recruits. */
export function returnsWithStory(state: Immutable<CampaignState>, unit: Readonly<Unit>): boolean {
  const next = CAMPAIGNS[state.id].missions[state.mission + 1];
  return Boolean(
    unit.character &&
    (next?.arrivals?.some((arrival) => arrival.character === unit.character) ||
      next?.map.units.some(
        (incoming) => incoming.side === "player" && incoming.character === unit.character,
      )),
  );
}
