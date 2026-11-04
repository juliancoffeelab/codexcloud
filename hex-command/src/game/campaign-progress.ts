import { CAMPAIGNS } from "./campaign-data";
import type { Immutable } from "./immutable";
import type { CampaignState, MissionDefinition, Unit } from "./types";

export function campaignMission(state: Immutable<CampaignState>): Immutable<MissionDefinition> {
  return CAMPAIGNS[state.id].missions[state.mission]!;
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
