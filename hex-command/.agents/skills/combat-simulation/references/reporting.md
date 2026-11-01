# Present simulation results

## Arena and campaign summaries

1. State completed coverage, seeds, round/recruit limits and pooled dimensions.
2. For an arena workload, default to **one faction-versus-faction matrix per map**, with row factions
   winning against column factions. Show every recorded opposing faction;
   pooled race averages must not replace the matchup breakdown. Order maps as
   Plains (open), Forests (wooded), Crossing (crossings), Islands (islands).
   Include Wood Edge (wood-edge) after Islands. All five maps belong to the
   default arena roster; a playable-map picker does not define arena coverage.
   Order factions as Humans, Skeletons, Woodland, Merfolk (recorded as Coastal).
   Put comparable policies in each cell in the order **Balanced → Situational →
   Predictor**, using the available subset (for example Situational → Predictor
   when Balanced is absent). State the order and that values are win percentages
   with cap counts in parentheses, for example `55.0 (2) → 49.8 (9) → 70.8 (14)`.
   With one policy, show its single value. Keep caps in the denominator and omit
   win counts.
   Name the row and opponent tactics for each panel. Pair only comparable scopes:
   Balanced-versus-Balanced → Situational-versus-Situational →
   Predictor-versus-Predictor for faction-balance self-play, or all selected row
   tactics against the same fixed opponent for a fixed-opponent panel. Explain when both sides change policy; this does not
   measure head-to-head recruiter strength. Present other tactic panels separately,
   including Classic when recorded. Mark missing configurations as unrecorded;
   do not fabricate a full matrix or broaden a scoped request.
   Mark same-faction diagonals as **Mirror**. Follow with a separate mirror table
   of first-to-act win percentages and cap counts, using physical-game
   denominators, in the same Balanced → Situational → Predictor order using
   the available subset. Do not
   present the mechanically symmetric faction win rate as mirror balance evidence.
3. For a campaign workload, include **the campaign table**, campaigns as rows and simulation-player
   tactics as columns. Cells are completion counts such as `16/20`, without
   percentages. Split campaigns with multiple player diplomacy modes into
   separate labeled rows, such as Ashen Oath (pragmatic) and Ashen Oath
   (restrained); never pool these modes. Also split named objective routes such as flight and fight
   into separate labeled rows. Name the fixed Classic opponent.
4. Add important findings and limitations. For scoped requests, show that scope.

Keep caps in arena win-rate denominators. Campaign completion covers the whole
campaign; stage results count only attempts reaching that stage. Keep wins,
defeats and caps distinct. Summarize both workloads for full runs.

### Optional elements on request

Collect and read the evidence required for the claim even when its detailed
table was not requested. Present optional tables only within the requested scope.

For optional damage, recruitment, presence, support, casualty and leader-audit
outputs, use the definitions and export names in [metrics](metrics.md). Show
requested damage tables per exact unit/tier as Presence %, weapon Damage/turn,
D/P and separate Thorns/turn. The replay branch is in [replays](replays.md).

For damage or contribution rates, show the exposure denominator, such as living
unit-turns. Keep faction, unit/tier, tactic, opposing tactic and terrain scope
explicit; identify pooled dimensions. Collect the necessary telemetry when it
is absent rather than substituting another table or treating missing data as zero.

Report files and exports are internal records in ignored `artifacts/`; present
results directly, without local file links. Deliver files only when requested.
