# Browser play

Use these branches for the selected tooling and requested workflows. The
[skill](../SKILL.md) owns the play session and completion criteria.

## Playwright

Use the repository's installed Playwright and available Chromium executable.
Choose a context matching the requested viewport and input capabilities. Keep
cookies and saves isolated from the user's session; a requested saved-game
fixture belongs in a separate, identified context.

Use role/label locators for controls, scrolling them into view as needed, and
browser mouse, touch, wheel or keyboard input for gestures. Let normal visibility,
hit testing and enabled-state checks apply. Forced clicks, DOM handler invocation
and synthetic event dispatch can skip the player's obstruction; record that
obstruction instead.

Wait for the expected user-visible transition after each action. Bound waits and
the run's lifetime. On timeout, preserve available evidence and close the browser
in `finally`; a timed-out evaluation can still be executing.

## Presentation settings

Start long journeys with the supported browser preference. Set sound through the
actual Menu control after entering the game:

```js
const context = await browser.newContext({ reducedMotion: "reduce" });
const page = await context.newPage();
await page.goto(gameUrl);
await page.getByRole("button", { name: "Menu", exact: true }).click();
const menu = page.getByRole("dialog", { name: "Menu", exact: true });
await menu.getByRole("checkbox", { name: "Sound effects", exact: true }).uncheck();
await menu.getByRole("button", { name: "Close menu", exact: true }).click();
// Continue through normal UI controls, waiting for each visible result.
```

To exercise normal timing, first wait for the current action to settle, then:

```js
await page.emulateMedia({ reducedMotion: "no-preference" });
// Enable Sound effects through the same Menu checkbox when audio is in scope.
// Perform representative movement/combat and observe its complete presentation.
// Once settled, return to the long-journey setting:
await page.emulateMedia({ reducedMotion: "reduce" });
// Turn sound off through Menu before continuing without audio timing checks.
```

Reduced motion is an actual player setting, not permission to force completion,
alter timers or skip controller barriers. Sound can retain combat pacing even
with reduced motion. Report the selected settings; ordinary frame readiness or
a requested audio cue alone does not prove timing or audible output.

## Canvas and gestures

Locate units and terrain from the rendered battlefield. Read-only coordinate
mapping may use observed camera geometry and visible tile positions; expose plain
snapshots rather than controller or scene handles. Pan and zoom through actual
input to reach offscreen targets, then target their current screen positions.

For touch, configure touch support and use touch input. Exercise the requested
pan, pinch, details scrolling and gesture transitions on the actual surfaces.
Desktop mouse input does not establish touch or trackpad behavior. Describe what
the available tooling can reproduce and leave physical-device claims to evidence
from those devices.

## Diagnostics

Prefer existing in-game diagnostics and exports. Additional local instrumentation
may observe immutable state and timing snapshots without changing the session.
Keep it separate from the input driver. An AI recommendation or legal-action
query cannot replace finding and operating the corresponding UI controls.

Capture a screenshot or recording around a blocker;
console silence and an advancing engine revision do not establish usability.

If audio is in scope, distinguish an audible recording or output capture from a
log showing that playback was requested. Ordinary browser video may omit audio.
Keep recordings within any requested size limit and report missing capture
capabilities honestly.

## Saves and recovery

Create progress through UI play and use the save, load and recovery controls.
Use read-only exports or snapshots to compare native checkpoints, seed and save
identity when required; a screenshot of the turn number is insufficient. Keep
storage migration fixtures and injected failures separate from the ordinary
playthrough, and follow the save-migrations skill for those investigations.
