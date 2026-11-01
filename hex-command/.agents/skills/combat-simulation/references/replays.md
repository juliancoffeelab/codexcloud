# Counterfactual replays

Read for sampled thorn or aura interventions in previously recorded phases.

For sampled aura deterrence, replay observed enemy-phase positions with aura
enabled and disabled, leaving intrinsic thorns unchanged:

```sh
python3 .agents/skills/combat-simulation/scripts/measure-thorn-deterrence.py \
  --report artifacts/saved/report.json --data-dir rust/data \
  --out tmp/thorn-deterrence --limit 50
```

Use `--effect intrinsic` or `--effect all` to disable personal thorns or both
effects instead of aura alone. `--policy situational` filters the acting tactic;
`--per-seed 10` spreads observations across seeds. `--engaged-only` requires
an affected recipient within an enemy melee unit's geometric movement reach;
this conservative filter does not prove a legal path or an actual attack.
Compare reflection with target/weapon switches and casualties. Low reflection
alone does not establish that thorns lack defensive value. Saved-phase replays
restore combat RNG from recorded attack and retaliation draws and verify the
final RNG. Strict replays must reproduce the original actions, units and gold
before interpreting an intervention; unsupported draw histories fail explicitly.

Use game data and a simulator matching the saved report. If deliberately testing
a different version, state that limitation. Paired replays measure one-phase
changes in attacks, targets, reflection and casualties; they do not establish
long-term win effects. Spread samples across battles and seeds.
