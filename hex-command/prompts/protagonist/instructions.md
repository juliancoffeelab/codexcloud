# Protagonist generation: author instructions

For the campaign author, including development evals when requested. Give generator agents the rendered prompt with its sampled table; keep this guide and selection guidance out of their context.

## Generate

1. Use the selected enemy's description-only JSON: `faction`, `motives`, `means`, `actions`, `leader`, `forces`. Preserve its undertaking and relevant user requirements. Supply the player race when specified. Selection logs remain author-only.
2. From the project root, run:

   ```sh
   source /workspace/.setup/env.sh
   python prompts/protagonist/generate.py --enemy tmp/selected-enemy.json --count 10 --output tmp/protagonist-generation
   ```

   Add `--race elves` when the player race is specified. Use `--seed NUMBER` to reproduce a batch. The script saves `sample.json` and a ready-to-send `prompt.md` in a new directory; without `--output`, it prints the prompt.

3. For the initial batch, spawn a fresh Luna on medium with no conversation history. Give it the entire rendered prompt. It includes the selected enemy, sampled rows and explicit protagonist format.

   **The alternative-generation paragraph is the most important part to give the generator.** The template includes it verbatim, changing only the candidate count.

4. Wait for the complete batch; ask Luna to finish missing rows. Apply the rating scheme and choose the strongest eligible protagonist. Save the enemy and protagonist descriptions for chapter generation. Store candidate number, seed, scores and selection rationale separately in the author-only selection log. Continue the campaign workflow; human selection is used only for an explicitly requested interactive eval.

## Select

Rate every candidate starting at **5**. Discard candidates scoring **6 or below**; only **7** is eligible for selection. Choose the strongest eligible candidate. If none is eligible, reroll. Presenting all candidates and scores to the user is for requested evals.

- Adds an ingredient beyond the sampled row: **−1 per additional ingredient**.
- Apply the [shared rating deductions](../../.agents/skills/design-campaign/SKILL.md#shared-rating-deductions).
- Assumes a working legal or administrative system: **−3**.
- Has an altruistic motive: **−1**, no matter what.
- Has traits contradicting the race's grounding: **−1 per trait**.
- Introduces a new battle mechanic: **+2**. Implement in campaign JSONs and/or state Rust code modifications and identify what differs from existing mechanics. A fictional explanation for an existing effect, or a tactic using existing authored elements, earns no bonus.
- Requires modifying the Rust engine: **−1** for extending an existing effect or resolution path; **−3** for a mechanic requiring an independent subsystem.

Record battlefield novelty and implementation path/cost separately. A new interaction assembled from supported engine primitives can earn **+2** without an engine-modification deduction.

Record each candidate's score and the concrete evidence for every bonus or deduction. Plausibility and completed fields alone earn no points. Judge what is written; do not invent missing motives, developments or battle mechanics to award a bonus. Reject substantive violations of the shared design rules regardless of score.

- Probabilities describe typicality, not quality. A probability above 0.10 is not an automatic violation.
- Check ingredients by their stated purpose. A primary or auxiliary goal is a desired end; means enable those ends. Possessing troops, wealth or expertise does not itself establish a pursuit to obtain them.
- Check that the protagonist preserves the selected enemy's established identity and undertaking.

Use the [authoring guidance](../../.agents/skills/design-campaign/authoring.md#implementation-costs) to establish the Rust-engine modification deduction; new authored data using supported mechanics incurs no engine-modification deduction.

Investigate specific uncertainties when they affect selection; give Luna focused feedback rather than rewriting every candidate.

## Reroll

Follow the [shared three-reroll limit and manual-repair fallback](../../.agents/skills/design-campaign/SKILL.md#reroll-limit).

If no candidate is eligible, retain the selected enemy and sampled table. Send the rendered prompt again, adding:

```text
None selected. Generate new candidates from the same enemy and sampled ingredients.

Issues:
[Specific failures in this set.]

Change the underlying factions and pursuits, not just names and scenery.
```

If the sampled ingredients themselves cannot fit the established conflict, record the incompatibility and sample a new batch. Preserve the selected enemy and campaign brief.

Retain the alternative-generation paragraph, count and protagonist format. Select from the replacement set by the same criteria.
