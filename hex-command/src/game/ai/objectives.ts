import { activeGoalTiles, campaignMission, goalProgress } from "../campaign-progress";
import { Game, TYPES } from "../engine";
import type { Side, Unit } from "../types";

export interface AiObjective {
  id: string;
  tiles: string[];
  weight: number;
  kind: "capture" | "visit" | "escort" | "escape" | "seal" | "vigil" | "defeat";
  actor?: string;
  family?: string;
}

export function objectivesFor(game: Game, side: Side): AiObjective[] {
  const objectives: AiObjective[] = [];
  if (game.campaign) {
    const mission = campaignMission(game.campaign);
    if (side === "player") {
      for (const [index, goal] of mission.goals.entries()) {
        const progress = goalProgress(game, goal);
        if (progress.done && ["seal", "event", "defeat"].includes(goal.kind)) continue;
        const tiles = [...activeGoalTiles(game, goal)];
        let kind: AiObjective["kind"] = "capture";
        let actor: string | undefined;
        if (goal.kind === "escort" || goal.kind === "escape" || goal.kind === "vigil") {
          kind = goal.kind;
          actor = game.units.find(
            (unit) =>
              unit.side === side &&
              (goal.kind === "vigil"
                ? unit.name === goal.companion
                : unit.role === (goal.kind === "escort" ? "courier" : "leader")),
          )?.id;
        } else if (goal.kind === "seal") kind = "seal";
        else if (goal.kind === "event") kind = "visit";
        else if (
          goal.kind === "defeat" ||
          (goal.kind === "refuge" && game.campaign.refuge?.escalated)
        ) {
          kind = "defeat";
          tiles.push(
            ...game.units
              .filter(
                (unit) => unit.side !== side && (goal.kind === "refuge" || TYPES[unit.type].leader),
              )
              .map((unit) => unit.tile),
          );
        }
        if (tiles.length)
          objectives.push({
            id: `required:${index}`,
            tiles,
            kind,
            weight: 24,
            actor,
            family: goal.kind === "seal" ? goal.unitType : undefined,
          });
      }
      for (const event of mission.events) {
        if (
          event.trigger.kind === "visit" &&
          !game.campaign.marks.includes(`event:${event.id}`) &&
          !mission.goals.some((goal) => goal.kind === "event" && goal.event === event.id)
        )
          objectives.push({
            id: `optional:${event.id}`,
            tiles: [event.trigger.tile],
            kind: "visit",
            weight: 4,
          });
      }
    } else {
      for (const [index, goal] of mission.goals.entries()) {
        if (["control", "hold", "refuge", "vigil"].includes(goal.kind)) {
          objectives.push({
            id: `deny:${index}`,
            tiles: [...activeGoalTiles(game, goal)],
            kind: "capture",
            weight: 24,
          });
        }
      }
    }
  }
  for (const tile of game.tiles) {
    if (tile.terrain === "village" && tile.owner !== side)
      objectives.push({ id: `village:${tile.id}`, tiles: [tile.id], kind: "capture", weight: 8 });
  }
  if (!objectives.some((goal) => goal.kind === "defeat")) {
    const leaders = game.units.filter((unit) => unit.side !== side && TYPES[unit.type].leader);
    objectives.push({
      id: "opposing-leader",
      tiles: leaders.map((unit) => unit.tile),
      kind: "defeat",
      weight: 12,
    });
  }
  return objectives.filter((goal) => goal.tiles.length);
}

export function canPursue(unit: Unit, goal: AiObjective): boolean {
  return (
    (!goal.actor || goal.actor === unit.id) &&
    (!goal.family || (TYPES[unit.type].family ?? unit.type) === goal.family)
  );
}

export function individualObjectives(objectives: AiObjective[]): AiObjective[] {
  return objectives.flatMap((goal) =>
    goal.kind === "capture"
      ? goal.tiles.map((tile) => ({ ...goal, id: `${goal.id}:${tile}`, tiles: [tile] }))
      : [goal],
  );
}

export function essential(game: Game, unit: Unit): boolean {
  return Boolean(
    TYPES[unit.type].leader ||
    unit.role === "courier" ||
    (unit.side === "player" &&
      game.campaign &&
      campaignMission(game.campaign).requiredCompanions?.includes(unit.name ?? "")),
  );
}
