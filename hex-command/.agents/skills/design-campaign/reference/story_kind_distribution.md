# Story-development kind distribution

A story-development kind describes how the chapter changes its problem, purpose or situation. It is distinct from a gameplay objective, event or achieved asset. The names below were selected by the user after the classification.

## Counts by position

Working semantic classification of 339 annotated templates. The ordinary comparison covers nineteen story families: 19 opening battles, 209 middle battles, 21 alternative-route finale templates, and 19 closing sequences. Two story openings were source-tagged tutorial; three explicit bonus battles and the classic duplicate campaign are separate.

| Kind                    | Opening |  Middle | Final battle | Ending |
| ----------------------- | ------: | ------: | -----------: | -----: |
| Planned progress        |       5 |      90 |            — |      — |
| Hard-won progress       |       1 |      50 |            — |      — |
| Success, Then Trouble   |       8 |      22 |            — |      — |
| Resolved Detour         |       — |      19 |            — |      — |
| Redirection             |       2 |      11 |            — |      — |
| Fundamental realignment |       — |       7 |            1 |      1 |
| Setback                 |       2 |       6 |            — |      — |
| Costly victory          |       — |       2 |            1 |      1 |
| False victory           |       — |       2 |            — |      — |
| Launch                  |       1 |       — |            — |      — |
| Resolution              |       — |       — |           16 |      — |
| Cycle continues         |       — |       — |            1 |      — |
| Irreversible loss       |       — |       — |            1 |      — |
| Aftermath               |       — |       — |            — |     17 |
| Step Forward            |       — |       — |            1 |      — |
| **Total**               |  **19** | **209** |       **21** | **19** |

Planned progress and Step Forward name the same pattern in regular and finishing contexts respectively. An opening is the first story battle, a finale is the last regular combat chapter on its route, and an ending is post-combat closing material whether separate or embedded.

Shared epilogues count once per campaign even when multiple battles lead to them. The conditional book route supplies the closing sequence for the recurring lich ending; the repeating battle itself remains Cycle continues. The exceptional early ending of the South Guard delaying action is preserved in the research, without inventing a route probability.

## Meanings

- **Planned progress:** Complete a planned step toward the existing goal.
- **Hard-won progress:** The problem proves harder than expected; struggle brings the protagonists closer to solving it.
- **Success, Then Trouble:** Solve the current problem, but uncover another one.
- **Resolved Detour:** Encounter and resolve a local problem, then resume the larger pursuit.
- **Redirection:** New circumstances change the plan or immediate target while the larger purpose remains.
- **Fundamental realignment:** Established loyalties or purpose change radically and durably.
- **Setback:** Fail to achieve the intended step or lose ground, though something may be salvaged.
- **Costly victory:** Genuinely succeed, but the cost or consequences undermine something important.
- **False victory:** Think the goal was achieved, then discover deception or that the success served the enemy's plan.
- **Launch:** Establish the situation, commitment or plan that launches the undertaking.
- **Resolution:** Achieve the central goal and settle its conflict.
- **Cycle continues:** Actions perpetuate the same unresolved situation.
- **Irreversible loss:** The undertaking ends in irreversible loss rather than its intended fulfillment.
- **Aftermath:** Show what the completed conflict leaves behind: consequences, changed lives and the world afterward.
- **Step Forward:** Complete a planned step toward the existing goal.

## Sampling and interpretation

A batch represents alternative designs for one next battle, not a generated sequence of campaign chapters. Use the [chapter workflow](../../../../prompts/chapter/instructions.md) to supply campaign position and previous battle context; interpret the supplied story and ending kinds using the meanings above.

These counts are interpretations of source-backed chapter records, not author-written labels or measured playthrough probabilities. One dominant pattern per template supports the table; conditional paths and overlapping secondary patterns remain in the research. Neighboring patterns can require judgment.

The sampler preserves recorded dependencies between ingredients, but those relationships do not establish a coherent causal arc or engine support. The designer must ground the ingredients in the established factions and situation, or flag an incompatible combination. Frequencies are descriptive, not a required rotation of twists or losses.

## Source checks and examples

The classifications retain entry situations, pursuits, developments, outcomes and citations from the archived source at revision `39f64458e811dbf98f1fdaf03ea2b30a4269cfda`. All 339 IDs were accounted for. Phase splitting distinguishes the Deceiver finale's achieved battle goal from its undermining closing narration.

- [The troll journey's ending](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Dusk_of_Dawn/scenarios/04_A_Beautiful_World.cfg#L939): the last battle advances the journey, while the subsequent ending fundamentally realigns the tribe.
- [The king's death and the rebels' repulse](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_Deceivers_Gambit/scenarios/14_Long_Live_the_Queen.cfg#L1559): battle Resolution followed by a Costly victory ending.
- [The lich's book epilogue](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Descent_Into_Darkness/scenarios/10_Endless_Night.cfg#L1306): a conditional closing sequence within an otherwise recurring final scenario.
