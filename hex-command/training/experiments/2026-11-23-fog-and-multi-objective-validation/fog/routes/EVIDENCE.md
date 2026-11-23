# Scree Relay returning-fog validation

**The blocked-road challenge qualifies under both returning-fog conditions.** Hour8 and hour10 failed all twenty matched seeds in each scope. Full and Classic completed all twenty. Keep the bypass-threat start as an evaluation control. [Exact fixtures, hashes and data](final-summary.json)

The confirmed 18×11 v8 terrain, forces, objective and deadline are identical to the clear fixture. Only vision-group fog penetration and the returning-fog overlay change. Both variants use active Classic enemies with zero recruitment. The named captain must stand on (16,4) by the end of turn 7. Discovery used seeds 56181101–56181108. Confirmation used a different cohort, 56181201–56181220, matched across all policies and visibility scopes. [Clear baseline and maps](../../new-env-validation/routes/EVIDENCE.md) · [Build and evaluation script](fog_eval.py)

| Start | Visibility | Hour8 | Hour10 | Full | Classic |
| --- | --- | ---: | ---: | ---: | ---: |
| Blocked road | Clear | 0/20 | 0/20 | 20/20 | 20/20 |
| Blocked road | Player-only fog | 0/20 | 0/20 | 20/20 | 20/20 |
| Blocked road | Both-sides fog | 0/20 | 0/20 | 20/20 | 20/20 |
| Bypass threat | Clear | 0/20 | 20/20 | 20/20 | 20/20 |
| Bypass threat | Player-only fog | 20/20 | 20/20 | 20/20 | 20/20 |
| Bypass threat | Both-sides fog | 14/20 | 14/20 | 20/20 | 19/20 |

All results are native single-stage campaign completions. Every failure was an authored defeat. In the blocked-road challenge the two fog scopes each produced nineteen deadline defeats and one commander loss for hour10. No simulation caps occurred in the 480 confirmation games. [Clear rows](clear/confirmation/results.jsonl) · [Player-only rows](player-fog/confirmation/results.jsonl) · [Both-sides rows](both-fog/confirmation/results.jsonl)

## Effective visibility

The selected fog route fires at chapter start and applies one returning-fog effect with reveal distance three to each of the 198 tiles. The `player` faction uses the `player` vision group and the `enemy` faction uses `enemy`. In player-only fog the enemy group retains `penetrates: ["fog"]` and sees all 198 tiles at its first sampled decision. The player group loses fog penetration and sees 65 tiles. In both-sides fog neither group penetrates fog. The first sampled enemy decision sees 37 tiles and the player sees 65. These counts come from the native faction views of Full vs Classic on seed 56181201. In the blocked-road start, the enemy can see the ranger and captain at its first decision with player-only fog. With both-sides fog it sees the ranger alone. By turn 3, tiles have returned to fog for each restricted observer in that match. The clear runs have zero fog effects and both sides see all 198 tiles. [Scope audit](player-fog/confirmation/evidence/fog-scope-audit.json) · [Both-sides audit](both-fog/confirmation/evidence/fog-scope-audit.json) · [Player-only native observations](player-fog/confirmation/evidence/native-visible-information.json) · [Both-sides native observations](both-fog/confirmation/evidence/native-visible-information.json)

Across 72 phase replays, the effective groups and tile effects at the first player phase and in final state matched the intended scope. Each replay reproduced the original winner and terminal round. On the blocked road, Full and Classic sent the captain through the bypass in all three sampled games per planner and scope. Hour8 and hour10 each kept the captain off the bypass in all three sampled games per scope. In the bypass-threat control, the dreadguard is hidden from the player at first under either fog setting. All four policies' captains stayed off the bypass in the three sampled games per fog scope. [Player-only phase records](player-fog/confirmation/evidence) · [Both-sides phase records](both-fog/confirmation/evidence)

The comparison is scoped to this layout, clear or three-hex returning fog and Classic opponents. The original clear catalogs, playable content, sampling, and training are untouched.
