# Confirmed ordinary-arena starvation

At approximately seven hours, ordinary arenas received 860 decisions in six episodes, last completed at 755.44 trainer seconds. Nine ordinary maps received no decisions. Base ordinary share is 14/38*0.5 = 18.42%; observed share approximately 0.175%. Combined arena sampling pool still has 50%, hiding the internal failure.

Sampler.choose balances flattened condition templates, not maps. Each ordinary map has 288 templates; targeted maps have 2–18. Sampler.finish resets every allocation counter in an affected pool whenever mastery weights change (checked every 20 completed episodes). This repeatedly removes catch-up history, favoring large-weight individual templates and starving ordinary maps despite their aggregate weights.

Isolated reproduction: same sampler and seed, 2000 synthetic 40-decision episodes. Without periodic arena allocation resets: 308 ordinary games / 12320 decisions. With only that history-reset behavior added every 20 completions: zero ordinary games / decisions. Both retain exactly 50% combined arena budget. This is a sampler experiment, not gameplay-strength evaluation.

Existing budget check covers pool totals, not per-map exposure under heterogeneous template counts and repeated adaptive changes. Proposed correction: persistent allocation at map level, then condition selection, with safe adaptation of allocation debt. Validate real map counts under repeated mastery changes and concurrency. No run or trainer modifications made during diagnosis.
