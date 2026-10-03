import { CAMPAIGNS } from "./campaign-data";
import type { Immutable } from "./immutable";
import type { CampaignState, MissionDefinition, MissionGoal, Side, Tile, Unit } from "./types";

export interface CampaignBoard {
  campaign?: Immutable<CampaignState>;
  units: readonly Readonly<Unit>[];
  round: number;
  turn: Side;
  tile(id: string): Readonly<Tile> | undefined;
}

export function campaignMission(state: Immutable<CampaignState>): Immutable<MissionDefinition> {
  const mission = CAMPAIGNS[state.id].missions[state.mission]!;
  if (
    (state.choices?.includes("refuge-war") ||
      state.choices?.includes("green-dain-dead") ||
      state.marks.includes("dain-fell")) &&
    mission.withoutDain
  )
    return { ...mission, ...mission.withoutDain };
  if (state.refuge?.escalated && mission.refuge)
    return { ...mission, briefing: mission.refuge.warBriefing, hints: mission.refuge.warHints };
  return mission;
}

export function goalTiles(goal: Immutable<MissionGoal>): readonly string[] {
  if (goal.kind === "defeat") return [];
  return "tiles" in goal ? goal.tiles : [goal.tile];
}

export function goalProgress(board: CampaignBoard, goal: Immutable<MissionGoal>) {
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
      (board.turn === "player" && board.campaign?.marks.includes(`held:${goal.tile}`)));
  return { label, current, total, done, tiles };
}

export function dialoguePending(state: Immutable<CampaignState> | undefined): boolean {
  return Boolean(state && state.dialogueIndex < state.dialogue.length);
}

export function endsRefugeMediation(board: CampaignBoard, target: Readonly<Unit>): boolean {
  return Boolean(
    board.campaign &&
    campaignMission(board.campaign).refuge &&
    !board.campaign.refuge?.escalated &&
    target.side === "enemy" &&
    (target.retreating || target.role === "leader"),
  );
}
