# Coastal damage and healing trade

This follow-up to [the initial balance pass](combat-balance.md) lowers macer damage,
adds a small health buffer and makes healing support cheaper. The healer's damage
is reduced to compensate for the lower price; healing itself remains unchanged.

| Unit | Before | Selected stats |
| --- | --- | --- |
| Macer | 20 HP, 7 Impact | 22 HP, 6 Impact; move 4, cost 8 |
| Veteran macer | 26 HP, 9 Impact | 28 HP, 8 Impact; move 4, authored cost 12 |
| Wavemender | Cost 10, Surge 6 | Cost 9, Surge 5; 16 HP, move 4, healing 2 |
| Tide sage | Surge 8 | Surge 7; 22 HP, move 4, authored cost 15, healing 4 |

Mace attacks are melee with reach 1; Surge is ranged Impact with reach 2.
Staff damage remains 2 for both healers. Macer and healer lines still promote at
10 XP and stop at L2. All other unit stats and racial modifiers are unchanged.
Reef guard remains 22 HP, with a 6-Pierce spear, 4-Impact tail and 1 Blade protection.

## Paired validation

The selected source is `28e8b97`; the baseline report records `844a407`.
All four arena factions and four maps use Mixed versus Mixed, seeds **61–110**,
both seats and initiatives, budget 36, one recruit per phase and a 30-round cap.
Each version contains **6,400 physical matches**, including self-play controls;
each cross-faction pair/map has **200 games**. Faction/map totals below contain
**600 games** against the other three factions. Cells show **win percentage
(wins/losses/caps)**; caps stay in the denominator.

| Faction | open | wooded | crossings | islands |
| --- | --- | --- | --- | --- |
| human | 63.2% (379/219/2) | 52.8% (317/266/17) | 54.7% (328/266/6) | 49.5% (297/286/17) |
| woodland | 45.0% (270/328/2) | 73.2% (439/155/6) | 42.3% (254/337/9) | 20.2% (121/463/16) |
| undead | 64.2% (385/212/3) | 49.8% (299/286/15) | 52.2% (313/281/6) | 48.5% (291/281/28) |
| coastal | 27.0% (162/437/1) | 20.7% (124/472/4) | 48.8% (293/304/3) | 76.5% (459/138/3) |

Merfolk's overall terrain profile stays close to the prior version:

| Map | Before | After | Delta |
| --- | --- | --- | --- |
| open | 26.3% | 27.0% | +0.7 pp |
| wooded | 23.2% | 20.7% | -2.5 pp |
| crossings | 50.0% | 48.8% | -1.2 pp |
| islands | 78.8% | 76.5% | -2.3 pp |

Woodland still wins woods, and merfolk still wins islands. The human-versus-undead
matches are unchanged: human wins 43.5%, 53%, 56.5% and 52% on the four maps.
This is a controlled change in unit damage, durability and support price, not a
claim that every faction's win rate improved or every opponent is equally matched.
The remaining extremes are explicit: merfolk wins 90% against humans on islands
(180 wins, 19 losses, 1 cap), versus 85.5% before; it wins 60% against undead
on islands and 79.5% against woodland. Its plains win rate against undead is only
11%. Those differences remain visible in the complete matchup tables.

## Experiments and rejected alternatives

Each trial uses all four factions/maps with complete native simulation and analysis.
Initial trials use seeds 1–10 (1,280 matches); larger comparisons use seeds 61–110
(6,400 matches). This comparison reuses the prior baseline seeds; it is a paired
follow-up, rather than a new untouched-seed validation.

- Macer 24 HP/6 Impact, veteran 30/8, healer cost 10 kept overall faction results
  almost unchanged but raised merfolk's human island matchup to 91%.
- Adding a 9-gold healer without reducing its damage raised the merfolk island
  aggregate to 83.2% with 24-HP macers, and 81.2% with 22-HP macers. The human
  island matchup reached 92% and 91% respectively.
- Smaller 21/27-HP macers and a veteran-only healer damage reduction were also
  tested on the ten-seed panel. Neither provided enough evidence to prefer it.
- The selected 5/7-damage healer line compensates for the discount. Its preliminary
  crossings result was weak, so it was checked on the larger panel: 48.8% merfolk
  wins versus 50% before. Island aggregate drops to 76.5%, preserving the advantage.

All trial reports, catalogs, per-match outcomes and detailed gameplay remain in
`artifacts/coastal-smoothing-*/`; rejected trials are preserved too.

## Every selected opponent matchup

Mirrored cross-faction cells describe the same physical games. Self-play diagonal
cells count both participants and serve as controls.

### open

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 50.0% (100/100/0) | 66.5% (133/67/0) | 43.5% (87/111/2) | 79.5% (159/41/0) |
| woodland | 33.5% (67/133/0) | 48.0% (96/96/8) | 51.5% (103/96/1) | 50.0% (100/99/1) |
| undead | 55.5% (111/87/2) | 48.0% (96/103/1) | 50.0% (100/100/0) | 89.0% (178/22/0) |
| coastal | 20.5% (41/159/0) | 49.5% (99/100/1) | 11.0% (22/178/0) | 50.0% (100/100/0) |

### wooded

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 48.0% (96/96/8) | 32.5% (65/129/6) | 53.0% (106/83/11) | 73.0% (146/54/0) |
| woodland | 64.5% (129/65/6) | 48.5% (97/97/6) | 80.0% (160/40/0) | 75.0% (150/50/0) |
| undead | 41.5% (83/106/11) | 20.0% (40/160/0) | 47.0% (94/94/12) | 88.0% (176/20/4) |
| coastal | 27.0% (54/146/0) | 25.0% (50/150/0) | 10.0% (20/176/4) | 49.0% (98/98/4) |

### crossings

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 50.0% (100/100/0) | 61.5% (123/74/3) | 56.5% (113/85/2) | 46.0% (92/107/1) |
| woodland | 37.0% (74/123/3) | 47.0% (94/94/12) | 56.5% (113/83/4) | 33.5% (67/131/2) |
| undead | 42.5% (85/113/2) | 41.5% (83/113/4) | 49.0% (98/98/4) | 72.5% (145/55/0) |
| coastal | 53.5% (107/92/1) | 65.5% (131/67/2) | 27.5% (55/145/0) | 50.0% (100/100/0) |

### islands

| Faction | human | woodland | undead | coastal |
| --- | --- | --- | --- | --- |
| human | 47.0% (94/94/12) | 87.0% (174/23/3) | 52.0% (104/83/13) | 9.5% (19/180/1) |
| woodland | 11.5% (23/174/3) | 43.0% (86/86/28) | 28.5% (57/130/13) | 20.5% (41/159/0) |
| undead | 41.5% (83/104/13) | 65.0% (130/57/13) | 42.0% (84/84/32) | 39.0% (78/120/2) |
| coastal | 90.0% (180/19/1) | 79.5% (159/41/0) | 60.0% (120/78/2) | 50.0% (100/100/0) |

## Complete data and reproduction

- [Searchable report](../artifacts/coastal-smoothing-support-trade50/analytics/index.html)
- [Every matrix and unit-event row](../artifacts/coastal-smoothing-support-trade50/analytics/summary.md)
- [Full CSV/catalog/outcome archive](../artifacts/coastal-smoothing-support-trade50/analytics/full-data.zip)
- [Every individual match](../artifacts/coastal-smoothing-support-trade50/analytics/matches.csv)
- [Current raw gameplay](../artifacts/coastal-smoothing-support-trade50/report.details.jsonl)
- [Baseline raw gameplay](../artifacts/balance-final-mixed/report.details.jsonl)

```sh
npm run ai:combat -- --policies mixed --seed 61 --seeds 50 --out artifacts/coastal-run
```

Add the saved baseline with `--baseline artifacts/balance-final-mixed/report.json`
for the identical paired comparison. This follow-up measures Mixed-policy arena
armies. The all-policy results in the initial report retain their original stats;
they are not reruns of these coastal changes. Campaign pools share the new stats,
but campaign balance was not measured by this arena panel.

## Verification

All nine `npm run verify` groups passed: style, content, WASM bindings, UI,
types, Rust, artwork, browser tests and production build. Browser tests took
10.32 seconds, within the unchanged 20-second budget. All 6,400 selected matches
completed. Current and baseline catalog/engine fingerprints, matchup denominators,
native recruitment/promotion/casualty totals and ZIP integrity were checked.
Every one of the 3,600 matches without merfolk retained the same winner, reason,
round count, state hash, recruitment, promotions and casualties.
