import { CAMPAIGNS } from "./campaign-data";
import type { Immutable } from "./immutable";
import type { CampaignState, MissionGoal, Side, Tile, Unit } from "./types";

export interface CampaignBoard {
  campaign?: Immutable<CampaignState>;
  units: readonly Readonly<Unit>[];
  round: number;
  turn: Side;
  tile(id: string): Readonly<Tile> | undefined;
}

export function campaignMission(state: Immutable<CampaignState>) {
  return CAMPAIGNS[state.id].missions[state.mission]!;
}

export function goalTiles(goal: Immutable<MissionGoal>): readonly string[] {
  if (goal.kind === "defeat") return [];
  return "tiles" in goal ? goal.tiles : [goal.tile];
}

export function goalProgress(board: CampaignBoard, goal: Immutable<MissionGoal>) {
  let current = 0;
  let total = 1;
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
        ),
      );
      break;
    case "escape":
      current = Number(
        board.units.some(
          (unit) => unit.side === "player" && unit.role === "leader" && unit.tile === goal.tile,
        ),
      );
      break;
    case "control":
      total = goal.tiles.length;
      current = goal.tiles.filter((id) => board.tile(id)?.owner === "player").length;
      break;
    case "seal":
      total = goal.tiles.length;
      current = goal.tiles.filter((id) => board.campaign?.marks.includes(`sealed:${id}`)).length;
      break;
    case "hold":
      total = goal.round;
      current = board.tile(goal.tile)?.owner === "player" ? Math.min(board.round, total) : 0;
      break;
  }
  const done =
    current === total &&
    (goal.kind !== "hold" ||
      (board.turn === "player" && board.campaign?.marks.includes(`held:${goal.tile}`)));
  return { label: goal.label, current, total, done, tiles: goalTiles(goal) };
}

export function dialoguePending(state: Immutable<CampaignState> | undefined): boolean {
  return Boolean(state && state.dialogueIndex < state.dialogue.length);
}
