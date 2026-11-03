import type { Immutable } from "../game/immutable";
import type { DialogueLine, MissionDefinition } from "../game/types";

/** Discover all dialogue cues for this mission, including authored story branches. */
export function missionSounds(
  mission: Immutable<MissionDefinition>,
  ending: readonly Immutable<import("../../rust/pkg/hex_command").Speech>[] = [],
): string[] {
  const sounds: string[] = [];
  const speech = (
    nodes: readonly import("../game/immutable").Immutable<
      import("../../rust/pkg/hex_command").Speech
    >[],
  ): void => {
    for (const node of nodes) {
      if ("with" in node) {
        speech(node.dialogue);
        speech(node.alternative ?? []);
      } else if (node.sound) sounds.push(node.sound);
    }
  };
  const actions = (
    values: readonly import("../game/immutable").Immutable<
      import("../../rust/pkg/hex_command").EventAction
    >[],
  ): void => {
    for (const action of values) {
      if ("sound" in action) sounds.push(action.sound);
      else if ("say" in action) speech(action.say);
      else if ("forEachUnit" in action) actions(action.forEachUnit.actions);
      else if ("forEachMark" in action) actions(action.forEachMark.actions);
    }
  };
  speech(mission.intro);
  speech(mission.victory);
  speech(ending);
  for (const event of mission.scriptedEvents ?? []) actions(event.actions);
  return [...new Set(sounds)];
}
