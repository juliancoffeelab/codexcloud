import * as Native from "./native-adapter.mjs";
import * as Story from "@frontend/Story";
import { fromJSON as from, toJSON as to, listToArray as toArray } from "./json-fixture.mjs";
export * from "@frontend/Story";
import * as Catalog from "./catalog-adapter.mjs";
export const mission = (catalog, state) => ({
  source: Catalog.campaign(catalog, state.id).missions[state.mission],
});
export const progress = (catalog, view, goal) => {
  const campaign = Native.state(view).campaign;
  const goals = mission(catalog, campaign).source.goals;
  const index = goals.findIndex((value) => JSON.stringify(value) === JSON.stringify(goal));
  const status = Native.inspection(view).objectives.goals[index];
  if (!status) throw new Error("Unknown inspected objective");
  return { ...status.progress, done_: status.progress.done };
};
export const goal_statuses = (view) =>
  toArray(Story.goal_statuses(view)).map((value) => to("ObjectiveStatus", value));
export const optional_statuses = (view) =>
  toArray(Story.optional_statuses(view)).map((value) => to("ObjectiveStatus", value));
export const objective_labels = (c, v) =>
  Object.fromEntries(
    Object.entries(Story.objective_labels(c, v)).map(([key, value]) => [key, toArray(value)]),
  );
export const dialogue_pending = (s) => Story.dialogue_pending(from("CampaignState", s));
export const returns_with_story = (c, s, u) =>
  Story.returns_with_story(c, from("CampaignState", s), from("PublicUnit", u));
