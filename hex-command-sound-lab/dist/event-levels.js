// Event context matters: the same recording can serve a weapon and a quiet tile.
export function eventLevelClass(entity, action, reference) {
  if (entity.kind === "terrain")
    return (reference.terrainExceptionIds || []).includes(entity.id)
      ? null
      : "terrain";
  return action.key.startsWith("attack-") ? "attack" : null;
}
export function playbackGainForRef(profile, ref) {
  const scoped = profile?.contexts?.[ref.levelClass] || profile;
  return ref.take === null ? scoped?.full : scoped?.takes?.[ref.take || 0];
}
