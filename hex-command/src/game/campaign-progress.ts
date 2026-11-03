import type { Faction } from "../../rust/pkg/hex_command";
import { CAMPAIGNS } from "./campaign-data";
import type { Immutable } from "./immutable";
import type { CampaignState, MissionDefinition, MissionGoal, Side, Tile, Unit } from "./types";

export interface CampaignBoard {
  campaign?: Immutable<CampaignState>;
  units: readonly Readonly<Unit>[];
  factions?: readonly Immutable<Faction>[];
  round: number;
  turn: Side;
  winner?: Side | null;
  tile(id: string): Immutable<Tile> | undefined;
}

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
  if ([...mission.intro, ...mission.victory].some((line) => fallen.has(line.speaker)))
    mission = {
      ...mission,
      intro: mission.intro.filter((line) => !fallen.has(line.speaker)),
      victory: mission.victory.filter((line) => !fallen.has(line.speaker)),
    };
  if (state.refuge?.escalated && mission.refuge)
    return { ...mission, briefing: mission.refuge.warBriefing, hints: mission.refuge.warHints };
  return mission;
}

export function goalTiles(goal: Immutable<MissionGoal>): readonly string[] {
  if (goal.kind === "defeat") return [];
  return "tiles" in goal ? goal.tiles : goal.tile ? [goal.tile] : [];
}

export function goalProgress(board: CampaignBoard, goal: Immutable<MissionGoal>) {
  const owned = (id: string) => {
    const owner = board.tile(id)?.owner;
    return (
      owner === "player" ||
      board.factions?.some((f) => f.id === owner && f.controller.kind === "human") === true
    );
  };
  let current = 0;
  let total = 1;
  let label = goal.label;
  let tiles = goalTiles(goal);
  switch (goal.kind) {
    case "defeat":
      current = Number(
        !board.units.some((unit) => unit.side === "enemy" && unit.role === "leader"),
      );
      break;
    case "escort":
      current = Number(
        board.units.some(
          (unit) => unit.side === "player" && unit.role === "courier" && unit.tile === goal.tile,
        ) ||
          board.campaign?.evacuated?.some(
            ({ unit }) =>
              unit.side === "player" && unit.role === "courier" && unit.tile === goal.tile,
          ),
      );
      break;
    case "escape":
      current = Number(
        board.units.some(
          (unit) => unit.side === "player" && unit.role === "leader" && unit.tile === goal.tile,
        ) ||
          board.campaign?.evacuated?.some(
            ({ unit }) =>
              unit.side === "player" && unit.role === "leader" && unit.tile === goal.tile,
          ),
      );
      break;
    case "control":
      total = goal.tiles.length;
      current = goal.tiles.filter((id) => owned(id)).length;
      break;
    case "seal":
      total = goal.tiles.length;
      current = goal.tiles.filter((id) => board.campaign?.marks.includes(`sealed:${id}`)).length;
      break;
    case "event":
      current = Number(board.campaign?.marks.includes(`event:${goal.event}`));
      break;
    case "hold":
      total = goal.turns;
      if (!goal.startEvent || board.campaign?.marks.includes(`event:${goal.startEvent}`))
        current = Math.min(board.campaign?.holds?.[goal.tile]?.heldTurns ?? 0, total);
      break;
    case "vigil":
      total = goal.turns;
      current = Math.min(board.campaign?.vigil?.heldTurns ?? 0, total);
      break;
    case "refuge": {
      if (board.campaign?.refuge?.escalated) {
        label = "Defeat the remaining patrol";
        current = Number(!board.units.some((unit) => unit.side === "enemy"));
        tiles = [];
      } else {
        const mission = board.campaign && campaignMission(board.campaign);
        const medicine =
          mission?.refuge &&
          board.campaign?.marks.includes(`event:${mission.refuge.medicineEvent}`);
        total = medicine ? mission!.refuge!.medicineTurns : goal.turns;
        current = Math.min(board.campaign?.refuge?.heldTurns ?? 0, total);
      }
      break;
    }
  }
  const done =
    current === total &&
    (goal.kind !== "hold" ||
      ((goal.once || board.turn === "player") &&
        board.round >= goal.earliestRound &&
        (goal.once || owned(goal.tile))));
  return { label, current, total, done, tiles };
}

/** Permanent completions leave the map; reversible objectives still need defending. */
export function activeGoalTiles(board: CampaignBoard, goal: Immutable<MissionGoal>) {
  if (board.winner) return [];
  if (goal.kind === "seal")
    return goal.tiles.filter((id) => !board.campaign?.marks.includes(`sealed:${id}`));
  if (goal.kind === "event" && goalProgress(board, goal).done) return [];
  return goalTiles(goal);
}

export function dialoguePending(state: Immutable<CampaignState> | undefined): boolean {
  return Boolean(state && state.dialogueIndex < state.dialogue.length);
}

/** Story companions already placed in the next chapter must not become duplicate recruits. */
export function returnsWithStory(state: Immutable<CampaignState>, unit: Readonly<Unit>): boolean {
  const next = CAMPAIGNS[state.id].missions[state.mission + 1];
  return Boolean(
    unit.name &&
    (next?.arrivals?.some((arrival) => arrival.name === unit.name) ||
      next?.map.units.some(
        (incoming) => incoming.side === "player" && incoming.name === unit.name,
      )),
  );
}
