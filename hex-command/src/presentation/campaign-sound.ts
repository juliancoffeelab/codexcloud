import type { Immutable } from "../game/immutable";
import type { DialogueLine, MissionDefinition } from "../game/types";

/** Discover all dialogue cues for this mission, including authored story branches. */
export function missionSounds(
  mission: Immutable<MissionDefinition>,
  ending: readonly Immutable<DialogueLine>[] = [],
): string[] {
  const variants: Partial<Immutable<MissionDefinition>>[] = [
    mission,
    ...Object.values(mission.companionLosses ?? {}),
  ];
  if (mission.withoutDain) variants.push(mission.withoutDain);
  const lines = [
    ...ending,
    ...variants.flatMap((story) => [
      ...(story.intro ?? []),
      ...(story.victory ?? []),
      ...(story.events ?? []).flatMap((event) => [...event.dialogue, ...(event.warDialogue ?? [])]),
      ...(story.waves ?? []).flatMap((wave) => wave.dialogue ?? []),
      ...(story.refuge?.escalation ?? []),
      ...(story.refuge?.warVictory ?? []),
    ]),
  ];
  const sounds = lines.flatMap((line) => (line.sound ? [line.sound] : []));
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
  for (const event of mission.scriptedEvents ?? []) actions(event.actions);
  return [...new Set(sounds)];
}
