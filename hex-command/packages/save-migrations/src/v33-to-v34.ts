import type { Save } from "./formats/history";
import { readSave } from "./formats/read";
import { isRecord } from "./formats/envelope-v39";
import frozen from "./legacy/data/objectives-v30.json";
import additions from "./legacy/data/objective-events-v31.json";

type Condition =
  { flag: string } | { all: Condition[] } | { any: Condition[] } | { not: Condition };
interface Objective {
  event: string;
  id: string;
  visibleWhen: Condition;
}
const chapters = frozen as Record<string, Objective[]>;
const stringArray = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every((item) => typeof item === "string");

/** Reconcile objective-branch v31 and saved-progress v31–33 without replaying gameplay. */
export function migrateV33ToV34(input: unknown): Save<34> {
  const value = readSave(input, 33);
  if (value.version !== 33 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-thirty-three save");
  const state = structuredClone(value.checkpoint.state);
  if (state.campaign != null) {
    if (!isRecord(state.campaign) || !isRecord(state.campaign.scripts))
      throw new Error("Invalid campaign objective state");
    const scripts = state.campaign.scripts;
    const hasCompleted = Object.hasOwn(scripts, "completed");
    const hasRevealed = Object.hasOwn(scripts, "revealed");
    if (hasCompleted || hasRevealed) {
      // The objective branch already froze this history. Never recompute it from current flags.
      if (!stringArray(scripts.completed) || !stringArray(scripts.revealed))
        throw new Error("Incomplete saved objective history");
    } else {
      const objectives = chapters[String(state.mapId)];
      if (!objectives || !stringArray(scripts.active) || !stringArray(scripts.failed))
        throw new Error("Invalid historical objective state");
      const flags = isRecord(state.campaign.flags) ? state.campaign.flags : {};
      const matches = (when: Condition): boolean => {
        if ("flag" in when) {
          const history = flags[when.flag];
          const latest = Array.isArray(history) ? history.at(-1) : undefined;
          return isRecord(latest) && latest.state === true;
        }
        if ("all" in when) return when.all.every(matches);
        if ("any" in when) return when.any.some(matches);
        return !matches(when.not);
      };
      const recipe = (additions as Record<string, { added: string[]; order: string[] }>)[
        String(state.mapId)
      ];
      const active = new Set([...scripts.active, ...(recipe?.added ?? [])]);
      if (recipe) {
        const known = new Set(recipe.order);
        scripts.active = [
          ...recipe.order.filter((id) => active.has(id)),
          ...scripts.active.filter((id) => !known.has(id)),
        ];
      }
      const failed = new Set(scripts.failed);
      scripts.completed = objectives
        .filter((o) => !active.has(o.event) && !failed.has(o.id))
        .map((o) => o.id);
      scripts.revealed = objectives
        .filter((o) => failed.has(o.id) || matches(o.visibleWhen))
        .map((o) => o.id);
    }
  }
  // Old undo patches were authored against a different native-state contract.
  const { undo: _discardedUndo, ...save } = value;
  return readSave({ ...save, version: 34, checkpoint: { ...value.checkpoint, state } }, 34);
}
