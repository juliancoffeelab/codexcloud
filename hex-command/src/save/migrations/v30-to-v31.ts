import { isRecord } from "../schema";
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

/** Freeze the old event-derived outcomes once; runtime never reconstructs them. */
export function migrateV30ToV31(value: Record<string, unknown>): Record<string, unknown> {
  if (value.version !== 30 || !isRecord(value.checkpoint) || !isRecord(value.checkpoint.state))
    throw new Error("Invalid version-thirty save");
  const state = structuredClone(value.checkpoint.state);
  if (isRecord(state.campaign) && isRecord(state.campaign.scripts)) {
    const scripts = state.campaign.scripts;
    const objectives = chapters[String(state.mapId)];
    if (!objectives || !Array.isArray(scripts.active) || !Array.isArray(scripts.failed))
      throw new Error("Invalid version-thirty objective state");
    const flags =
      isRecord(state.campaign) && isRecord(state.campaign.flags) ? state.campaign.flags : {};
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
  return { ...value, version: 31, checkpoint: { ...value.checkpoint, state } };
}
