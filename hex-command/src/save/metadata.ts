import type { LiveGame } from "../game/live-game";

export interface ProgressIdentity {
  runId: string;
  branchId: string;
  checkpointId: string;
  ordinal: number;
}
export interface SaveInfo extends ProgressIdentity {
  id: string;
  savedAt: number;
  run: string;
  mission: string;
  round: number;
  side: string;
  missionStart: boolean;
}
export interface CurrentEntry extends SaveInfo {
  kind: "recovery";
}
export interface TurnEntry extends SaveInfo {
  kind: "turn";
  manual: boolean;
  parentId?: string;
}
export const progressId = (): string => crypto.randomUUID();
export function progressIdentity(entry: ProgressIdentity): ProgressIdentity {
  return {
    runId: entry.runId,
    branchId: entry.branchId,
    checkpointId: entry.checkpointId,
    ordinal: entry.ordinal,
  };
}
export function newRun(): ProgressIdentity {
  return { runId: progressId(), branchId: progressId(), checkpointId: progressId(), ordinal: 0 };
}
function describe(
  game: LiveGame,
  identity: ProgressIdentity,
): Omit<SaveInfo, "id" | "missionStart"> {
  return {
    ...progressIdentity(identity),
    savedAt: Date.now(),
    run: game.campaign?.id ?? game.mapId,
    mission: game.campaign ? `Mission ${game.campaign.mission + 1} · ${game.mapId}` : game.mapId,
    round: game.round,
    side: game.turn,
  };
}
export function describeRecovery(game: LiveGame, identity: ProgressIdentity): CurrentEntry {
  return {
    ...describe(game, identity),
    id: identity.branchId,
    kind: "recovery",
    missionStart: false,
  };
}
export function describeTurn(
  game: LiveGame,
  identity: ProgressIdentity,
  missionStart: boolean,
  parentId: string,
  manual: boolean,
): TurnEntry {
  return {
    ...describe(game, identity),
    id: identity.checkpointId,
    kind: "turn",
    manual,
    missionStart,
    parentId,
  };
}
