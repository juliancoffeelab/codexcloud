# Enemy generation: coordinator instructions

For coordinators and evaluators only. Generator agents receive the rendered prompt, not this file or its rating guidance.

## Generate

1. From the project root, run:

   ```sh
   source /workspace/.setup/env.sh
   python prompts/enemy/generate.py --count 10 --output tmp/enemy-generation
   ```

   This saves the sampled ingredients and audit in `sample.json`, and the ready-to-send prompt in `prompt.md`. The seed is recorded; pass `--seed NUMBER` to reproduce a batch. Without `--output`, the script prints the prompt.

2. Spawn a fresh Luna on medium with no conversation history. Give it the generated prompt, including the entire table. Keep prior candidates, ratings and evaluator preferences out of its context.

   **The alternative-generation paragraph in the prompt template is the most important part to give the generator. Include it verbatim, changing only the candidate count.** The script renders that paragraph automatically.

3. Wait for the complete response, then evaluate all candidates. Show the user the candidates, scores and short reasons. The user picks.

## Evaluate

- Probabilities estimate typicality, not quality. Select by design quality, not the highest or lowest probability. A probability above 0.10 is not an automatic violation: the prompt says to aim below it.
- A substantive skill or sampled-row violation scores **0/10**. Distinguish a violation from an ordinary weakness, and identify the specific mismatch. Allow small word-count overruns.
- Judge what is written. Do not invent missing motivations, relationships, changes or disclosures to make a candidate comply.
- Reward threatening enemies, believable ambitions, concrete military undertakings and forces that can produce battles. Familiar premises can work; novelty alone does not make a candidate good.
- Check goals, auxiliary goals and means by their stated purpose. An auxiliary goal is another desired end; something pursued only to enable the primary goal is a means.
- Check organization and presentation against their reference definitions. Repeated raids do not establish changing opposition, and several rivals pursuing one succession struggle are not automatically an unconnected mix.
- Heavily downrate repetitive captive/worker rescues, escorts, ferries, floods, workers’ rights and oppression plots. Downrate clerks or quartermasters replacing threatening enemies, and “gain supplies” as sufficient story development. These preferences are for evaluation, not instructions to leak into generation.
- When seeking independent ratings, give fresh reviewers the skill, sampled table and complete candidates, without your scores or other reviewers’ conclusions.

## Reroll

If none is acceptable, keep the sampled starting table and regenerate from the same template, adding:

```text
None selected. Generate new candidates from the same sampled ingredients.

Issues:
[Specific failures in this set.]

Change the underlying undertakings, not just names and scenery.
```

Retain the alternative-generation paragraph, candidate count and enemy format. Evaluate the replacement set by the same standard.

## Sampling

The weights are inline in `generate.py`; the sampler uses Python’s standard library and reads no research files from `tmp/`.

- Background, primary goal, means, organization and presentation each get an independent weighted bag. Build slightly more cards than needed, round expected counts of at least one, and chance-roll categories below one. Shuffle each bag, cut to the requested size, then zip the columns.
- Auxiliary inclusion is the only per-enemy chance draw. Draw its kind only when included.
- Organization and presentation use separate marginal distributions. Zero cells in the historical joint table do not prohibit new combinations. The output is unfiltered ingredients, not a validated story.
- Background weights cover 16 sole/allied-main enemy records. Goal weights cover 24 primary, three auxiliary and six means occurrences. Organization and presentation each cover 19 campaigns. These are different samples, not a measured joint model.
- Auxiliary inclusion uses three of 19 campaigns as a provisional proxy, not a measured per-enemy frequency. Each skeleton receives one means; its count is a generation choice.
- Raids / pillage is the requested generation-facing adaptation of an earlier acquisition category. Its weight is inherited, not a fresh raid census.

Definitions and research context: [opposition](../../.agents/skills/design-campaign/reference/opposition_distribution.md), [motives](../../.agents/skills/design-campaign/reference/motive_distribution.md), and the [campaign skill](../../.agents/skills/design-campaign/SKILL.md). Background counts themselves are preserved inline in the script.
