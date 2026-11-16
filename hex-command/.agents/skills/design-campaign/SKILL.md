---
name: design-campaign
description: Author complete campaigns, revise playable campaigns, or audit their story, branching, disclosure and consequences.
---

# How to design a campaign

Use this workflow for playable campaigns. Simulation fixtures and debug/test scenarios do not inherit its story, branching or design-document requirements. For simulation work, use [combat simulation](../combat-simulation/SKILL.md); rendering and tooling tests follow their relevant task workflows.

Read [world and campaign design](world-design.md) before generating or selecting content. Both the author and Luna use those rules. The author uses [authoring](authoring.md) to turn the selected design into dialogue, briefings, in-game objectives and authored content, and to assess implementation costs.

## Author workflow

The author owns the complete campaign and controls its development through the finale. Work within the user's requested scope when revising a campaign or running a partial eval.

1. Establish the intended campaign length. It can develop alongside the story; keep the campaign's scope explicit.
2. Use the [enemy workflow](../../../prompts/enemy/instructions.md) to sample ingredients, delegate candidate generation to a fresh Luna context, and select an enemy.
3. Use the [protagonist workflow](../../../prompts/protagonist/instructions.md) against that enemy and select the player faction, leader and their goal.
4. Use the [chapter workflow](../../../prompts/chapter/instructions.md) to develop and select the opening, then subsequent chapters. Decide when to enter the finale. The prompting guides govern context handoffs, selection and rerolling.
5. Reconcile the complete selected design against the [shared design rules](world-design.md). This step is complete when the campaign has selected factions, a coherent chapter sequence through its resolution, concrete battles and achievements, and accounted-for story consequences.

The author makes selections using the guides' criteria. Human choice, comparative ratings and inspection of individual stages are development evals when requested. The guides and their selection guidance are for the author; Luna receives the rendered generation prompt and relevant context.

Every campaign has a `design.md` beside `campaign.json`, at most 25 lines and 450 words. Describe the campaign and why its story and gameplay take this form. Keep implementation details in the authored content and code.

## Shared rating deductions

Apply these deductions when rating enemies, protagonists and chapters. Keep them out of generator prompts.

- Mentions bells, floods, ferries, ledgers, records or the literal word "names": **−1**. Ledgers and records include equivalent paperwork under other labels, such as "papers"; judge their role, not their wording.

## Reroll limit

Start each selection with a fresh Luna context. Send its rerolls and focused
issue feedback to that same Luna, preserving its candidate-generation context.
Start a new context for the next selection: enemy, protagonist or next chapter.

For enemy selection, protagonist selection and each chapter, allow at most **three rerolls after the initial batch**. Replacement batches, resampling ingredients and requests for candidate rewrites all count toward this limit. Stop as soon as an eligible candidate is selected.

Only if the third reroll still produces no eligible candidate, choose the most interesting candidate generated for that selection and fix its issues manually. Do not request further generation for that selection.

## Implementation: TODO

Add the workflow for turning a selected campaign design into authored, validated game content. The current generation workflow produces the complete campaign design.

Use [authoring](authoring.md) for implementation guidance and [map drafting](map-drafting.md) for previews. A campaign design or static map preview does not establish implementation or playability.

## Reconcile or audit

Compare the delivered content with the design: chapter progression, fork consequences, points of interest, player-facing elements and persistent outcomes. Trace disclosure through actual encounter conditions. Check each path against the events and facts established along it.

Report each mismatch with its location and consequence. An audit is complete when every campaign and chapter in scope is accounted for, with confirmed defects distinguished from design judgements and unverified behavior. Change only what the user has authorized.
