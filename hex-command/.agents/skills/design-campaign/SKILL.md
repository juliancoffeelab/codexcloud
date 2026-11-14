---
name: design-campaign
description: Design or revise playable campaigns, or audit their story, branching, disclosure and consequences.
---

# How to design a campaign

Use this workflow for playable campaigns. Simulation fixtures and debug/test scenarios do not inherit its story, branching or design-document requirements. For simulation work, use [combat simulation](../combat-simulation/SKILL.md); rendering and tooling tests follow their relevant task workflows.

## Rules

- **Progressive disclosure.** Give the player only information the characters already know. Revealing information they do not yet know is forbidden in every channel, including briefings, hints, objectives, markers, counters and logs.
- **Consistency.** Preserve established facts across the campaign. Changes must follow from events in the story.
- **Design document.** Every campaign has a `design.md` beside `campaign.json`, at most 25 lines and 450 words. Establish its intent, mood and why its story and gameplay take this form. Keep implementation details in the authored content and code.
- If you accidentally looked at other campaigns, make sure your design has nothing in common with them.

## Develop the story

Ground campaigns in medieval fantasy: its powers, beliefs, ambitions and conflicts, unless asked otherwise.

Strictly follow the step sequence laid out below.

### 1. Enemy

For examples of how opposition is organized and presented across campaigns, read the [opposition distribution](reference/opposition_distribution.md). For the kinds of goals enemies pursue and their roles, read the [motive distribution](reference/motive_distribution.md).

Enemies need not be tragic, complicated or justified. They may want crowns, territory, artifacts, magical power, revenge, victims or dominance. Some are straightforward predators. Others believe in their undertaking.

Describe every enemy candidate using these fields:

- **Enemy faction:** who they are.
- **Motives:** what the faction wants and why.
- **Actions:** what it does to achieve its aims.
- **Leader:** their desires, reasons, plans and how they command the faction.
- **Forces:** the fighting force the faction can field.
- **Probability:** estimated likelihood under ordinary prompting.

Use the alternative-generation process in the [appendix](#appendix-generate-alternatives) to compare enemy factions and their undertakings. Select the enemy before developing the protagonist. Changing names, races or locations does not make alternatives materially different.

### 2. Protagonist

For player-character backgrounds, faction types and pursuits, read the [protagonist distribution](reference/protagonist_distribution.md).

Protagonists themselves can be dangerous aggressors.

Describe every protagonist candidate using these fields:

- **Protagonist faction:** who they are.
- **Motives:** what the faction wants and why.
- **Leader:** their desires, reasons, plans and how they command the faction.
- **Forces:** the fighting force the faction can field.
- **Probability:** estimated likelihood under ordinary prompting.

Use the alternative-generation process in the [appendix](#appendix-generate-alternatives) to compare protagonist factions and their pursuits against the selected enemy. Find issues in how their goals and actions bring the factions into conflict.

### 3. How to write a chapter

Present each chapter candidate using this format:

- **Premise:** what is happening that creates this battle; maximum 15 words.
- **Our characters**
- **Their characters**
- **Terrain:** rivers, forests, hills, swamps, plains, or a mix; include both factions' recruitment tiles, rarely omitted.
- **Objective(s) to finish:** maximum ten words total.
- **What the battle achieves, and what's in it for the player:** maximum ten words.
- **Kind in the skill**
- **Probability**

Each chapter is a battle. The characters enter it pursuing a goal and confronting a problem. Its outcome changes their situation and moves the story forward.

Useful chapter shapes:

- **Breakthrough:** overcome the current problem; the battle's outcome reveals a new one.
- **Progress:** attempt to overcome a problem, discover it is larger or harder than expected, and struggle further. The battle's outcome brings the characters closer to solving it.
- **Reversal:** attempt to overcome a problem, then encounter something that changes the situation. The battle's outcome irreversibly and radically changes the protagonist faction's loyalties or original purpose.

A chapter can be a fork: an explicit combination of the kinds above, rather than another kind. A fork must follow the same format, sharing premise, terrain and characters and having different objectives that lead to different achieved results. Choose the point during the chapter when it becomes a fork. A campaign can have no fork.

A forced "pick one, lose another" is not a fork; it is bullshit.

Use **forks** for story decisions enacted through events. Each option changes how the story unfolds and what the player faces or can do afterward; branches may reunite. Each branch must change what the player actually plays in one or more chapters, beginning in the current or immediately following chapter. They may also lead to entirely different chapters.

Before developing a fork, use the alternative-generation process in the [appendix](#appendix-generate-alternatives) to compare different designs for the decision and its branches. For each candidate, establish what the player chooses and what they actually play along each path. Find issues in the decision and its playable consequences before selecting a design.

For each chapter, use the alternative-generation process in the [appendix](#appendix-generate-alternatives) to compare developments from the same starting situation: what the characters attempt, why the opposing sides fight, what changes during the battle, and what its outcome means for the story. Find issues in each development before selecting one and expanding its scenes and objectives. These shapes are possibilities, not a required rotation.

Establish what the player commands and what makes the battle a particular tactical problem: the opposing forces, terrain, objectives, available gold, recruitment and veterans. Track how the outcome changes the factions' capabilities and opportunities in subsequent chapters.

Develop events from the characters' actions and decisions. Cause and effect must make internal sense, even when the explanation is initially withheld from the player. Both sides need clear motivations to kill or seriously injure each other. Dialogue must express those intentions rather than grant permission for the desired gameplay.

#### Evolve the campaign

The first chapter hooks the player through something happening that needs resolution, without explaining everything. Establish the mood, a believable immediate goal, and a real problem.

Derive each subsequent chapter from the previous chapter's achievement: what it enables, how the factions respond, and what the characters now pursue. Establish that starting situation before generating alternatives.

#### Finale

The finale fulfills the established goal: kill the villain. If a Reversal changed that goal, the finale fulfills the replacement goal established by that Reversal.

Use these kinds instead of the middle-chapter kinds:

- **Resolution:** the battle achieves the campaign's goal and settles its central conflict.
- **New story:** the battle achieves the campaign's goal, but that achievement creates a different pursuit. This campaign ends; another can begin.

The finale can be planned at the outset or developed alongside the preceding chapters. Use the same chapter format and alternative-generation process to compare final battles and their achievements.

This step is complete when every chapter has a goal, a motivated battle, a developing situation and an outcome that establishes its place in the campaign.

## Points of interest

A **point of interest** is a place on the map with small optional content: a structure, a hidden unique enemy, or another encounter. Completing or exploring it gives a gameplay reward: an item, effect, unit or discovered clue that helps with the main task. Establish the reward’s actual use in play; an item needs a gameplay effect, not merely a name.

## Player-facing elements

**Dialogue** drives and motivates the other elements. Give each exchange a clear subject and purpose. Ask: what would we lose if this dialogue were removed? Characters say things that matter to them, from their knowledge, role and circumstances. They speak to each other; the player may not fully understand them yet. Prior experience and expertise are valid sources of knowledge; a character need not discover everything on screen or explain their credentials.

**Briefing** gives short, current information about what is happening and what to do, in prose. Update it when the characters gain context. Put mechanical objectives and numbers in objectives instead.

**Hints** help a stuck player find an approach that progresses the story without revealing what it will unlock. They can, and usually should, be empty when the briefing is sufficient. Use them sparingly to guide early campaigns or chapters: “You never know what lurks in hidden places.”

### Objectives

For the relative frequency of objective kinds and their roles, read the [objective distribution](reference/objective_distribution.md).

A gameplay objective puts the story into actions the player can perform with their units:

- Kill someone (ends a chapter, except for _rare_ exceptions). There can be no objectives after defeating an enemy.
- Move to or hold a place (usually for optional quests).
- Move to pick up or drop an item (not a story device, just something a unit can do).

These actions enact the story and shape how the player experiences it. Choose the action, target and completion condition to express what the characters are trying to accomplish.

Translate a task established by dialogue into these mechanics, such as “Reach X by turn N.” Reveal the objective after that dialogue; precision must still respect progressive disclosure. Objectives describe intended goals, not guaranteed facts about the world or literal accounts of their resolution. Their mechanics need not be things characters state or know: a turn limit can quantify urgency.

This step is complete when every disclosed story fact has a source in character knowledge, dialogue establishes each task before its objective appears, and each channel serves its own purpose.

## Events and authored content

Events develop and move the story forward. For implementation, read [authoring references](authoring.md); for static map previews, read [map drafting](map-drafting.md).

## Reconcile or audit

Check proposed gameplay mechanics against the current engine and authored content. Verify support for abilities, unit actions, objectives and rewards; explicitly flag anything that requires new engine support.

Compare the delivered content with the design: chapter progression, fork consequences, points of interest, player-facing elements and persistent outcomes. Trace disclosure through actual encounter conditions. Check each path against the events and facts established along it.

Report each mismatch with its location and consequence. An audit is complete when every campaign and chapter in scope is accounted for, with confirmed defects distinguished from design judgements and unverified behavior. Change only what the user has authorized.

## Appendix: Generate alternatives

Coordinators and evaluators: use the [enemy generation workflow](../../../prompts/enemy/instructions.md) for enemy candidates and the [prompting guide](prompting_guide.md) for other campaign-design evaluations. Generator agents must not read either coordinator document or receive its rating guidance.

When designing alternatives, generate five materially different candidates, each with an estimated probability of producing it under ordinary prompting. Sample beyond the most likely answers, aiming for candidates below 0.10. Develop them to comparable detail before evaluating them.

Find issues with each candidate against the relevant design criteria. Discard candidates whose problems undermine the design. If all candidates are discarded, start again from a different angle.

These probabilities describe typicality, not quality. Select among the surviving candidates by the relevant design criteria, not the highest or lowest probability.

In the proposal, report every candidate, its probability and the issues found beside the relevant design section, from enemy and protagonist selection through each chapter and fork. Identify the selected candidate and its remaining issues.

When asked only to generate candidates, return these fields without evaluation, ratings or selection.

### Unit references

When choosing units, consult:

- [Game definitions](../../../rust/data/game/units/) for implemented unit rules.
- [Visual lineups](../../../pixel/art/units/lineups/) for rosters and promotion branches.
- [Unit artwork](../../../pixel/art/units/) for existing sprites and palettes.
