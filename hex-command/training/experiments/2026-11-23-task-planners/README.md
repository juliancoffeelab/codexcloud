# Task-specific offline planners

Three stronger teachers are wired through `pretrain_imitator.tactical`. No
training was started. Maps, rosters, deadlines, enemy behavior, visibility,
rewards and sampling allocation are unchanged.

## Fresh matched confirmation

Twenty seeds (493110001–493110020) per map, disjoint from discovery and previous
teacher searches. Each row compares identical conditions and seeds. All games
reached an authored terminal outcome; no round-cap truncations.

| Map / condition | Previous teacher | Hour 10 | Specialist | Configured now |
|---|---:|---:|---:|---|
| Crown Road Relief · clear · Classic | 14/20 | 18/20 | **20/20** | Relief |
| Briar Saddle · clear · Classic | 9/20 | 20/20 | **20/20** | Recovery |
| Lower Gallery Refuge · clear · Classic | 9/20 | 0/20 | **16/20** | Refuge |
| Rally at the Vault · clear · Predictor | **14/20** | 7/20 | 6/20 | Existing Classic |

Previous teachers were objective_progress, Queued, Full and Classic respectively.
“Hour 10” is the final one-hour extension checkpoint; its exact SHA and archived
weights are linked in `protocol.json`. These are success rates, not shaping
scores. The specialist candidates were frozen before fresh confirmation.

## Policies and observed limitations

- **Relief:** stage the forward troops until the authored turn-four relief,
  then advance on a risk-aware route. Preserve the commander. The gain over
  the previous teacher is substantial; 20/20 versus the learner's 18/20 is a
  small sample difference, not proof of general superiority.
- **Recovery:** retreat the wounded protected rider to the healer, let the
  heavy units prioritize the marked gate guard, then return the recovered rider.
  This repairs a weak imitation source; it ties the learner's 20/20.
- **Refuge:** retreat the captive into the western chamber, send the commander
  to its owned keep to recruit, attack observed pursuers, then return to the
  refuge with a healthy screen and hold. Its four fresh losses were **deadlines**,
  not captive deaths. It still needs work on completing the return reliably.
- **Rally:** the new path/race variants failed to beat the existing teacher.
  Keep Classic. Its earlier eight-case deficit to the learner did not survive
  this fresh comparison: Classic won 14/20 versus hour 10's 7/20.
- **Tidegate:** holder/screen/supply and capture-first variants were rejected.
  The configured four clear/both-fog conditions were screened (eight seeds each),
  but no replacement qualified. No blanket claim of solving its teacher gap.

Controllers see the same legal actions and faction-observed spatial inputs as
the learner. Coordinates and friendly roster bindings are explicit authored
teacher parameters, not engine map-ID branches. They do not inspect hidden foes,
combat RNG or private scripts. They choose actions; opponents remain active.

Two prototype errors were diagnosed and corrected before confirmation: confusing
the moved flag with HP fraction, and retaining objective completion across
**tentative** turn-search branches. The final policies recompute from each
current observation. Another prototype scored useful progress below End Turn on
long routes; final route scoring preserves positive progress regardless of total
remaining distance. These were prototype bugs, not claims of engine defects.

## Evidence and reproduction

`summary.json` is the compact fresh comparison. Frozen jobs and unabridged raw
native results are compressed alongside it. `sources` preserves all candidate
versions, including failures; `protocol.json` maps stages to source versions and
records hashes, seeds and scope. `production-parity-*` checks promoted controllers
against the measured prototypes on the same 60 cases and terminal rounds.

After sourcing `/workspace/.setup/env.sh`, run from the repository root:

```sh
python training/experiments/2026-11-23-task-planners/reproduce.py --stage production-parity
```

Other `--stage` choices replay discovery and frozen expert confirmation jobs.
Learner comparison outcomes and configs are preserved separately; replay them
with native `evaluate` and the archived hour-10 model referenced in the protocol.
The original collection scripts in `sources` are historical snapshots: their
live-workload lookup must not replace the frozen teacher declarations in archived
job files when reproducing baselines.

Qualification covers fixed layouts and the listed clear conditions. Fog was
screened for the rejected Tidegate policies, not validated for these three new
teachers. No claim about unseen layouts or a newly trained model follows from
these results. Full verification and recurrent collector smoke results are
preserved in `checks` after completion.
