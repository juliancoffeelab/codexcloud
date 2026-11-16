# Chapter generation: author instructions

For the campaign author, including development evals when requested. Give generator agents the rendered prompt; keep this guide and selection guidance out of their context.

## Generate

1. Prepare a context file for the fresh Luna:
   - Campaign scope, established goal, selected enemy/protagonist descriptions and this chapter's starting situation.
   - The selected previous chapter and its achievement: what it enables, faction responses and what the characters now pursue.
   - Surviving characters, faction relationships, current capabilities, items and decisions relevant to the chapter. Preserve optional and route-dependent consequences as conditions, not guaranteed acquisitions or discoveries.
   - What the viewpoint characters know, distinguished from author-known facts. Include unresolved goals and promises relevant to the next chapter.
   - The previous battle's sampled position and story kind; identify an opening or finale explicitly.

   Supply selected context, not rejected proposals, earlier ratings or the full author conversation.

2. Run the unified chapter sampler from the project root. For an opening:

   ```sh
   source /workspace/.setup/env.sh
   python prompts/chapter/generate.py --position opening --context tmp/chapter-context.md --count 5 --output tmp/chapter-generation
   ```

   For a middle chapter, supply the selected previous battle's position and sampled story-kind name:

   ```sh
   source /workspace/.setup/env.sh
   python prompts/chapter/generate.py --position middle --previous-position opening --previous-kind "Success, Then Trouble" --context tmp/chapter-context.md --count 5 --output tmp/chapter-generation
   ```

   For the finale, use `--position finale` with the previous position and kind. Keep the established goal in the context. Use a new output directory for each batch and `--seed NUMBER` for replay.

3. The output contains `sample.json`, readable ingredients in `table.md`, and ready-to-send `prompt.md`. For this chapter's initial batch, give a fresh Luna on medium with no conversation history the entire rendered prompt. Each candidate develops its corresponding ingredient set.

   **The alternative-generation paragraph is the most important part to give the generator.** The template includes it verbatim, changing only the candidate count. It also supplies the chapter format and sampled story-kind names and definitions.

4. Wait for the complete batch; ask Luna to finish missing candidates. Apply the rating scheme and choose the strongest eligible chapter. Record the selected candidate, seed, story kind, achievements and consequences, and update the next handoff context. Continue until the author-selected finale completes the campaign. Human selection is used only for an explicitly requested interactive eval.

## Interpret ingredients

- The sampled **story kind and its definition** describe the chapter's development. Use those names and definitions consistently. Finales also receive a separately sampled ending kind.
- Objective arrows require order; AND requires both tasks; OR gives alternative finishing routes. Preserve their separate results. Alternative victory conditions are not automatically campaign forks.
- Linked events establish, enact or resolve objectives. They are not necessarily events after completion. Keep connected effects together; several event kinds can describe one scene.
- Independent developments are outside objective execution/resolution. Their list has no prescribed chronology.
- Optional encounters connect an action to its reward/consequences. Keep encounter alternatives distinct from chapter endings.
- Conditional source events need conditions grounded in the new story. A recorded casualty does not require inventing a forced sacrifice. Missing mapped events can mean ordinary automatic resolution or incomplete annotation.
- Ground sampled ingredients in the selected factions and current situation. Require explicit acknowledgement of necessary adaptations rather than silent replacement of objectives, results or factions.

## Select

Rate every candidate starting at **5**. Discard candidates scoring **6 or below**. Choose the strongest eligible candidate; if none qualifies, follow the bounded reroll procedure below.

### General

- Apply the [shared rating deductions](../../.agents/skills/design-campaign/SKILL.md#shared-rating-deductions).
- Terrain forces the player to adapt how they fight: **+1**.
- The player's racial roster forces the player to adapt how they fight: **+1**.
- Introduces a new battle mechanic: **+2**. Implement in campaign JSONs and/or state Rust code modifications and identify what differs from existing mechanics. A fictional explanation for an existing effect, or a tactic using existing authored elements, earns no bonus.
- Requires modifying the Rust engine: **−1** for extending an existing effect or resolution path; **−3** for a mechanic requiring an independent subsystem.
- Advances the protagonist's established goal: **+2**. A replacement goal counts only when established by a chapter with a **Fundamental realignment** ingredient.
- Related to social justice themes: **−2**.
- Changes the protagonist's goal without a **Fundamental realignment** ingredient: **−2**.

Record battlefield novelty and implementation path/cost separately. A new interaction assembled from supported engine primitives can earn **+2** without an engine-modification deduction.

To award either terrain or racial-roster point, identify what ordinary approach becomes ineffective, why, and what the player must do differently. “Archers use hills” or “walkers advance behind cover” is not sufficient by itself.

### Continuation

Skip this section for openings.

- Continuing characters receive no progression: **−2**. Progression can develop their plans, relationships, circumstances or actions; it need not be personal growth or involve every character from the previous chapter.
- The previous battle's result receives no progression: **−2**. Its achievement must affect the new situation, rather than merely be mentioned.
- Repeats a materially similar battle among the previous three chapters: **−1 per similar battle**. Compare tactical problems and player decisions, not objective names alone.

### Finale

Apply these additional deductions to finales:

- Lacks ironclad fulfillment of the protagonist's original or fundamentally realigned goal: **−3**.
- Does not tie unfinished developments together: **−2**.
- Does not resolve the preceding chapters' buildup: **−2**.

### Causality

- Broken chapter causality—why this encounter happens: **−1**.
- Weak objective-to-result causality—why completing the task produces the claimed achievement: **−1**.

Record each score and concrete evidence for every bonus or deduction. Judge what is written; do not supply missing explanations or imagine gameplay possibilities to improve the score.

Probabilities describe typicality, not quality. Allow small premise word-count overruns; objectives and achievements have no word limit. Use the [authoring guidance](../../.agents/skills/design-campaign/authoring.md#implementation-costs) to establish the Rust-engine modification deduction; new authored data using supported mechanics incurs no engine-modification deduction.

Investigate specific uncertainties when they affect selection; requests for candidate rewrites count toward the reroll limit below. Presenting all candidates and scores to the user is for requested evals.

## Reroll

Follow the [shared three-reroll limit and manual-repair fallback](../../.agents/skills/design-campaign/SKILL.md#reroll-limit).

If none is eligible, keep the same context and sampled ingredients. Send the rendered prompt again, adding:

```text
None selected. Generate new candidates from the same starting situation and ingredient sets.

Issues:
[Specific failures in this set.]

Change the underlying developments, not just names and scenery.
```

If the ingredients themselves cannot fit the campaign state, engine capabilities or intended resolution, record the incompatibility and sample a new batch within the same reroll limit. Keep the selected factions, achieved facts and current goals intact.

Retain the alternative-generation paragraph, candidate count and chapter format. Select from the replacement set by the same criteria.
