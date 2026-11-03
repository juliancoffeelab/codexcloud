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
  return [...new Set(lines.flatMap((line) => (line.sound ? [line.sound] : [])))];
}
