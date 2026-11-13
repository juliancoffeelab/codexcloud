# Campaign-design prompting guide

For coordinators and evaluators only. Generator agents must not read this file. Do not include it or its rating guidance in their prompts.

For enemy candidates, use the [enemy generation workflow](../../../prompts/enemy/instructions.md). This guide covers protagonist and chapter generation.

Give fresh Luna agents, on medium, the skill path, established context and an explicit format. Keep ratings and evaluator preferences out of the generation prompt.

## Generation prompt

**The alternative-generation paragraph below is the most important part to give the generator. Include it verbatim in every generation prompt, changing only the candidate count when requested.**

```text
Read [absolute campaign skill path]. Generate candidates for
[protagonist faction / chapter N].

When designing alternatives, generate five materially different candidates, each with an estimated probability of producing it under ordinary prompting. Sample beyond the most likely answers, aiming for candidates below 0.10. Develop them to comparable detail before evaluating them.

[Player race, when selecting factions.]
[Selected factions, when already established.]
[Previous chapters and their achievements, when writing a chapter.]
[Say explicitly if this is the finale.]

[Paste the relevant format below.]

Candidates only: no critique, ratings or selection.
Reply here; no files.
```

For protagonist candidates, use the Protagonist format in the skill.

### Chapter format

```text
- Premise: maximum 15 words.
- Our characters:
- Their characters:
- Terrain: rivers, forests, hills, swamps, plains, or a mix;
  include both factions' recruitment tiles.
- Objective(s) to finish: maximum ten words total.
- What the battle achieves, and what's in it for the player:
  maximum ten words.
- Kind in the skill:
- Probability:
```

## Rating

These probabilities describe typicality, not quality. Select among the surviving candidates by the relevant design criteria, not the highest or lowest probability.

- **Substantive skill violation = 0/10.** Allow small word-count overruns.
- Judge what is written. Do not supply missing motivations or repair central claims while scoring.
- Reward threatening enemies, believable motives, concrete fighting objectives, worthwhile achievements and distinct battles.
- Check that chapters build on previous achievements.
- Heavily downrate recurring escorts, captive/worker rescues, ferries, floods, workers' rights and oppression plots.
- Downrate "gain supplies" as sufficient story development; clerks and quartermasters replacing threatening villains; vague "secure/protect/help" objectives; exposition presented as an achievement.
- Reject forced sacrifices disguised as forks, cosmetic branches and reversals that merely change tactics.
- A straightforward effective battle can score highly. Killing the villain is sufficient for the finale unless a Reversal established another goal.

Show the candidates with scores and short reasons. The user picks.

## Reroll

If none is acceptable, send:

```text
None selected. Generate five new candidates from the same starting situation.

Issues:
[Specific failures in this set.]

Change the underlying developments, not just names and scenery.

[Repeat the exact relevant format.]

Candidates only: no critique, ratings or selection.
Reply here; no files.
```

Rate the new set by the same standard.
