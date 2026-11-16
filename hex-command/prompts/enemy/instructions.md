# Enemy generation: author instructions

For the campaign author, including development evals when requested. Generator agents receive the rendered prompt, not this guide or its selection guidance.

## Generate

1. Keep the user-specified campaign requirements available. Include relevant user requirements with the rendered prompt. From the project root, run:

   ```sh
   source /workspace/.setup/env.sh
   python prompts/enemy/generate.py --count 10 --output tmp/enemy-generation
   ```

   Add `--race zombies` when the player race is zombies, or supply the specified player race. This saves the sampled ingredients and audit in `sample.json`, and the ready-to-send prompt in `prompt.md`. The seed is recorded; pass `--seed NUMBER` to reproduce a batch. Without `--output`, the script prints the prompt.

2. For the initial batch, spawn a fresh Luna on medium with no conversation history. Give it the generated prompt, including the entire table. Keep prior selections' candidates, ratings and evaluator preferences out of its context.

   **The alternative-generation paragraph in the prompt template is the most important part to give the generator. Include it verbatim, changing only the candidate count.** The script renders that paragraph automatically.

3. Wait for the complete batch; ask Luna to finish any missing rows. Apply the rating scheme and choose the strongest eligible enemy. Save its description in `selected-enemy.json`, containing exactly six text fields: `faction`, `motives`, `means`, `actions`, `leader`, `forces`. Store candidate number, seed, scores and selection rationale separately in the author-only selection log. Pass only the description to protagonist generation. Continue the campaign workflow; human selection is used only for an explicitly requested interactive eval.

## Select

Rate every candidate starting at **5**. Discard candidates scoring **7 or below**; only **8** is eligible for selection. Choose the strongest eligible candidate. If none is eligible, reroll. Presenting all candidates and scores to the user is for requested evals.

- Adds an ingredient beyond the sampled row: **−1 per additional ingredient**.
- Apply the [shared rating deductions](../../.agents/skills/design-campaign/SKILL.md#shared-rating-deductions).
- Assumes a working legal or administrative system: **−3**.
- Introduces a new battle mechanic: **+2**. Implement in campaign JSONs and/or state Rust code modifications and identify what differs from existing mechanics. A fictional explanation for an existing effect, or a tactic using existing authored elements, earns no bonus.
- Requires modifying the Rust engine: **−1** for extending an existing effect or resolution path; **−3** for a mechanic requiring an independent subsystem.
- Makes the player want to encounter, challenge, hunt or uncover the enemy: **+1**.

Record battlefield novelty and implementation path/cost separately. A new interaction assembled from supported engine primitives can earn **+2** without an engine-modification deduction.

Record each candidate's score and the concrete evidence for every bonus or deduction. Plausibility and completed fields alone earn no points. Judge what is written; do not invent missing motivations, relationships, developments or battle mechanics to award a bonus. Reject substantive violations of the shared design rules regardless of score.

- Probabilities estimate typicality, not quality. Do not select by the highest or lowest probability. A probability above 0.10 is not an automatic violation: the prompt says to aim below it.
- Check ingredients by their stated purpose. An auxiliary goal is another desired end; something pursued only to enable the primary goal is a means. Check organization and presentation against their reference definitions. Repeated raids do not establish changing opposition, and several rivals pursuing one succession struggle are not automatically an unconnected mix.
- When comparing eligible candidates, disfavor repetitive captive/worker rescues, escorts, ferries, floods, workers’ rights and oppression plots; clerks or quartermasters replacing threatening enemies; and “gain supplies” as sufficient story development. Keep these author preferences out of generation prompts.

Use the [authoring guidance](../../.agents/skills/design-campaign/authoring.md#implementation-costs) to establish the Rust-engine modification deduction; new authored data using supported mechanics incurs no engine-modification deduction.

Investigate specific uncertainties when they affect selection; give Luna focused feedback rather than rewriting every candidate.

## Reroll

Follow the [shared three-reroll limit and manual-repair fallback](../../.agents/skills/design-campaign/SKILL.md#reroll-limit).

If none is eligible, keep the sampled starting table and regenerate from the same template, adding:

```text
None selected. Generate new candidates from the same sampled ingredients.

Issues:
[Specific failures in this set.]

Change the underlying undertakings, not just names and scenery.
```

If the sampled ingredients themselves prevent a usable enemy, record the incompatibility and sample a new batch. Preserve the campaign brief.

Retain the alternative-generation paragraph, candidate count and enemy format. Select from the replacement set by the same criteria.
