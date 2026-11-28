# Better targeted teachers: existing bots, specialists and fresh checks

Investigated eight maps whose configured teachers trailed the learner or had weak completion. Reused both existing objective-progress and path-to-objective teachers before adding a visible-target path specialist. The old targeted bots were already active on the deadline fixtures and Siltbar; Rally was using a standard Classic planner.

## Fresh20-seed confirmation

Each row uses the same fresh seeds493070001–020 for the old teacher, selected candidate and exact hour10 model. Selection used separate eight-seed screens. Opponents/visibility/forces/objectives/recruitment/deadlines unchanged; Situational player recruiter for this comparison. All games reached authored outcomes, zero caps.

| Encounter | Visibility | Enemy | Old teacher | Candidate | Hour10 |
|---|---|---|---:|---:|---:|
|rl-urgent-crossing|authored|full|12/20|19/20|18/20|
|rl-vault-rally|authored|predictor|13/20|12/20|12/20|
|rl-bellmarket-courts|both-fog|full|17/20|20/20|20/20|
|rl-thorn-verge|authored|predictor|17/20|20/20|20/20|
|rl-thorn-verge|authored|full|17/20|20/20|20/20|
|rl-thorn-verge|both-fog|full|17/20|20/20|20/20|
|rl-defend-tidegate-stand-110|authored|classic|14/20|13/20|20/20|
|rl-defend-tidegate-stand-110|authored|predictor|10/20|17/20|19/20|
|rl-defend-tidegate-stand-110|both-fog|predictor|5/20|17/20|18/20|
|rl-defend-lower-gallery-0|authored|classic|5/20|5/20|0/20|

## Wired replacements

- Force the Crossing, clear againstFull: Situational recruiter / Mission Campaigner / Full planner. Classic-enemy condition retains objective_progress.
- Bellmarket Courts, both-fog againstFull: Situational / Mission / Queued. Other conditions retain their existing teachers.
- Thorn Verge, clear againstPredictor/Full and both-fog againstFull: `visible_target_path`. Other conditions retain Classic.

The new specialist extends the existing path teacher. It selects among observed hostile positions, prioritizes observed defeat markers, then uses the existing actor-specific terrain-path and attack scoring. It does not inspect hidden enemies, battle seeds or future combat draws. It is an offline imitator; PPO still uses the learned policy. Production implementation exactly matched all100 selected candidate terminal outcomes and rounds. The Thorn specialist also won60/60 with the other configured Predictor recruiter. Ordered native teacher collection passed in all three new specialist scopes with zero missing labels.

The PPO configuration sets, training/validation/test seed partitions, group/map/condition masses and reward profiles remain unchanged. Only teacher selectors and the new offline controller changed. No pretraining/PPO run launched.

## Rejected or unresolved

Reusing existing path controllers onRally gave5/8, tied with hour10; objective_progress2/8. Full screened6/8 but failed fresh confirmation:12/20 against oldClassic13/20 and hour10 12/20. Its original one-win deficit was not stable evidence that the old teacher was worse. No Rally replacement.

Tidegate’s new Classic alternatives improve some conditions but still trail the learner; clearClassic13/20 against learner20/20 is a clear unresolved deficit. Lower Gallery stays weak, both tested ordinary teachers5/20 against learner0/20. Prototype hold heuristics failed all forty screened matches in each of two versions and were rejected. The visible-target gank variation did not improve Bellmarket or Briar and was rejected. Crown/Briar did not gain a better candidate in the screen. This work does not make blanket imitation safe on every map.

## Coverage

1472 screening games,600 fresh confirmation games,100 production parity replays and60 additional recruiter checks:2232 games total; collection smokes separate. Complete raw results and candidate configurations are retained. Native rules/data remain current; historical frozen experiment catalogs and source weights are unchanged. `make full-verify` passed, including272 frontend tests and all Rust/workspace checks. The41-test training suite passed with one optional dependency skip. Outputs are archived compressed.
