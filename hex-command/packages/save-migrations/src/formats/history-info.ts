import type { SaveInfo } from "./metadata-v39";
/** Strip retired/unknown envelope metadata while preserving every identity field. */
export function entryInfo(entry: SaveInfo): SaveInfo {
  const {
    id,
    runId,
    branchId,
    checkpointId,
    ordinal,
    savedAt,
    run,
    mission,
    round,
    side,
    missionStart,
  } = entry;
  return {
    id,
    runId,
    branchId,
    checkpointId,
    ordinal,
    savedAt,
    run,
    mission,
    round,
    side,
    missionStart,
  };
}
