import type { AiOptions, PlannerKind, RecruiterKind } from "../../rust/pkg/hex_command";
import type { Side } from "./types";
const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export interface SessionSetup {
  first: Side;
  enemy: AiOptions;
}
export const PLANNERS: PlannerKind[] = ["classic", "queued", "full"];
export const RECRUITERS: RecruiterKind[] = [
  "classic",
  "balanced",
  "frontline",
  "rotation",
  "situational",
  "predictor",
  "aggressive-predictor",
];
export const DIFFICULTIES = [
  { id: "classic", name: "Relaxed", planner: "classic", recruiter: "classic" },
  { id: "situational", name: "Standard", planner: "queued", recruiter: "situational" },
  { id: "predictor", name: "Veteran", planner: "queued", recruiter: "predictor" },
] as const;
export const DEFAULT_SETUP: SessionSetup = {
  first: "player",
  enemy: { policy: "classic", diplomacy: "restrained" },
};

/** Validate browser-owned preferences before resuming a native checkpoint. */
export function validSetup(value: unknown): value is SessionSetup {
  if (
    !isRecord(value) ||
    Object.keys(value).some((k) => !["first", "enemy"].includes(k)) ||
    !["player", "enemy"].includes(String(value.first)) ||
    !isRecord(value.enemy)
  )
    return false;
  const ai = value.enemy;
  if (
    Object.keys(ai).some((k) => !["policy", "diplomacy", "blocks"].includes(k)) ||
    !DIFFICULTIES.some((d) => d.id === ai.policy) ||
    ai.diplomacy !== "restrained"
  )
    return false;
  return (
    ai.blocks === undefined ||
    (isRecord(ai.blocks) &&
      Object.keys(ai.blocks).length === 3 &&
      ai.blocks.campaign === "mission" &&
      PLANNERS.includes(ai.blocks.planner as PlannerKind) &&
      RECRUITERS.includes(ai.blocks.recruiter as RecruiterKind))
  );
}
