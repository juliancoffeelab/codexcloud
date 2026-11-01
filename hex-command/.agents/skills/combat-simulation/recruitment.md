# Adaptive recruitment coefficients

`rust/data/game/recruitment.json` contains `standard` and
`aggressivePredictor` profiles. Read effective values from that file and report metadata. Each accepts the camelCase fields below;
omitted fields use the defaults in `rust/src/data/recruitment.rs`. Unknown
fields, negative coefficients and nonpositive endurance normalization fail
catalog validation. Catalog fingerprints include this document. Reports also record the effective
profiles in `recruitmentProfiles`, retained in analysis metadata.

Situational and Predictor use `standard`. Their difference is observed enemies
versus observed enemies plus a fading legal-roster forecast. The optional
`aggressive-predictor` recruiter uses `aggressivePredictor`, with the same
forecast and tactical planner. Classic and Balanced do not use these weights.
Earned promotions retain the standard mechanical evaluation; the aggressive
movement terms apply only when buying recruits.

| Field                | Meaning                                                                                       |
| -------------------- | --------------------------------------------------------------------------------------------- |
| offense              | Sustained outgoing exchange value                                                             |
| enduranceReferenceHp | Endurance normalization: effective HP / sqrt(reference HP)                                    |
| retaliation          | Value of damage returned when attacked                                                        |
| exchangeRetaliation  | Cost of the enemy's response during our attack                                                |
| drain                | Value of healing from our attack                                                              |
| kill                 | Bonus for lethal attack probability                                                           |
| healing              | Marginal healing support value                                                                |
| auraThorns           | Marginal aura reflection value                                                                |
| intrinsicThorns      | Intrinsic reflection value under expected melee pressure                                      |
| futureWounds         | Fraction of ally maximum HP anticipated as future healing demand                              |
| supportRecipients    | Expected simultaneous recipients per provider                                                 |
| healingOverlap       | Multiplier on remote healing providers' estimated overlap probability                         |
| auraOverlap          | Equivalent multiplier for remote thorn aura providers                                         |
| futureEnemies        | Forecast troop mass, reduced by visible nonleader count                                       |
| villageCapture       | Marginal village reach, reduced by existing allies' coverage                                  |
| speed                | Intrinsic speed on the map's average traversable terrain cost                                 |
| leaderPressure       | Expected fraction of an actual leader's HP delivered, discounted by attack-route travel turns |
| movementCap          | Maximum combined village/speed/leader multiplier bonus; null is uncapped                      |
| duplicatePenalty     | Denominator increment for each living copy of the same family                                 |
| waitPenalty          | Denominator increment per income turn needed to afford a recruit                              |

For recruitment:

```
combat = offenseWeight * exchange * effectiveHP / sqrt(enduranceReferenceHp)
         + retaliationWeight * retaliation
movementBonus = villageWeight * marginalVillageReach
                + max(speedWeight * terrainSpeed,
                      leaderWeight * leaderDelivery)
score = [combat * (1 + min(movementCap, movementBonus))
         + healingWeight * marginalHealing
         + auraWeight * marginalAura
         + thornWeight * intrinsicThorns]
        / [cost * (1 + duplicatePenalty * copies)
                * (1 + waitPenalty * incomeTurns)]
```

A null movement cap omits the `min` operation. Terrain speed
is `clamp((move / meanTraversableTileCost - 3) / 3, 0, 1)`.
Leader delivery uses terrain-weighted routes to legal attack origins and is
zero for unreachable leaders. Speed and leader delivery use **max**, so they
do not stack as separate movement multipliers. Village value is a distinct
objective contribution; the combined bonus has one explicit cap.

All ordinary combat and support estimates remain distance-independent. Leader
travel affects only the optional leader-pressure bonus. The mechanical enemy
weapon-choice model remains unchanged. Overlap controls expected redundancy,
not a distance gate: exact adjacent coverage remains exact, while remote
providers use a probability clamped to [0,1].

To compare a profile without changing normal recruiters:

```sh
npm run ai:simulate -- --mode arena --policies predictor --with aggressive-predictor,-,queued --vs predictor --seeds 50 --unit-usage
```

Tune in an ignored catalog copy with `--data-dir`. Change one field at a time;
compare speed-only, leader-only and combined profiles against the same fixed
opponent. Keep unit stats, rosters, maps, budgets and seeds fixed. Validate on
fresh seeds, all five arena maps and 100 campaign seeds before changing default
behavior. Use presence, contributions, leader deaths and mirror initiative;
policy self-play alone cannot establish improvement against a fixed opponent.
