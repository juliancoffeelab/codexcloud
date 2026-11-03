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
  enemy: { policy: "classic", diplomacy: "diplomatic" },
};

export interface OpponentSummary {
  readonly difficulty: string;
  readonly policy: string;
  readonly planner?: string;
  readonly recruiter?: string;
}

/** Describe the active saved configuration, including advanced overrides. */
export function opponentSummary(enemy: AiOptions): OpponentSummary {
  const preset = DIFFICULTIES.find((difficulty) => difficulty.id === enemy.policy);
  const planner = enemy.blocks?.planner ?? preset?.planner;
  const recruiter = enemy.blocks?.recruiter ?? preset?.recruiter;
  const title = (value: string) =>
    value
      .split("-")
      .map((word) => word[0]!.toUpperCase() + word.slice(1))
      .join(" ");
  return Object.freeze({
    difficulty:
      preset && planner === preset.planner && recruiter === preset.recruiter
        ? preset.name
        : "Custom",
    policy: title(enemy.policy),
    planner: planner === undefined ? undefined : title(planner),
    recruiter: recruiter === undefined ? undefined : title(recruiter),
  });
}

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
    ai.diplomacy !== "diplomatic"
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
