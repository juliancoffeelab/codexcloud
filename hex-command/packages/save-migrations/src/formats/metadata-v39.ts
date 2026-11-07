/** Frozen version-39 progress metadata. */
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
  kind: "flash";
}
export interface TurnEntry extends SaveInfo {
  kind: "turn";
  manual: boolean;
  parentId?: string;
}
