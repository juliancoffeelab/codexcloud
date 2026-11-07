# AI-assisted UI play

Use the existing Rust algorithms to choose actions during long playthroughs.
The [AI advisor](../scripts/ai-advisor.mjs) demonstrates the current engine API;
it supplies recommendations and read-only observations. The browser still carries
out each action through the player UI, following [live play](../SKILL.md).

## Obtain a recommendation

For a local Vite session, import the advisor from a disposable Playwright script
and install it before navigation:

```js
import { installAiAdvisor, observe, recommend } from "./ai-advisor.mjs";

await installAiAdvisor(page);
await page.goto(localGameUrl);
// Choose the campaign and dismiss its briefing through visible controls.
const before = await observe(page);
const decision = await recommend(page, { policy: "classic", diplomacy: "diplomatic" });
console.log({ revision: decision.revision, command: decision.command });
// Translate this command into player input, then inspect the screen and observe again.
```

Resolve the import to the skill script or copy it alongside the disposable probe.
The adapter is specific to served Vite modules; it fails explicitly if the
observation seam changes. The main play workflow also supports deployed previews,
where this source adapter is unavailable. Report the selected planner and whether
AI assistance was available; retain the same UI execution boundary either way.

`suggestAction()` restores a disposable Rust battle from the current checkpoint,
asks `chooseAiCommand()` for the selected policy's recommendation, then disposes
that copy. Internal `setUnitIntent` commands apply only to this copy until the
planner produces a player action. Copy revisions are internal; the returned
recommendation is tagged with the live revision observed before planning.
The live battle, saves and controller are never dispatched by the advisor.

The AI may use the full game state to plan. This workflow tests whether the UI
can carry out those actions; game design and balance are outside its scope.

## Translate the plan into UI input

Read current units and tile coordinates from the observation, then locate them
on the rendered board. Use the corresponding control for the returned action:

| Recommendation | Player workflow                                                       |
| -------------- | --------------------------------------------------------------------- |
| `move`         | Select the unit, then press the destination tile.                     |
| `attack`       | Select the attacker and target, then activate the recommended weapon. |
| `recruit`      | Open the destination's recruitment UI and choose the unit type.       |
| `advance`      | Inspect the unit and choose its promotion.                            |
| `dropItem`     | Inspect the unit's equipment and use the item's drop control.         |
| `endTurn`      | Activate End turn and wait through the AI's visible actions.          |

Dialogue, recalls, mission changes and new-game selection use their visible UI
rather than planner dispatch. For unfamiliar command variants, inspect the current
Rust command definitions and the matching player workflow; report unsupported
translation rather than guessing or applying it internally.

Before input, confirm the recommendation's revision still matches the live
observation. After input, verify both the intended visible result and the Rust
state change; a revision increment alone does not identify which action occurred.
Keep the selected unit, target and available next controls in these checks.
When a visible control is absent, disabled or produces the wrong result, preserve
that attempted workflow as a blocker. Request another recommendation after a
successful action instead of applying a stale sequence.

Use ordinary presentation timing. Advisor computation is additional diagnostic
work; separate it from measurements of the game's own interaction latency.
