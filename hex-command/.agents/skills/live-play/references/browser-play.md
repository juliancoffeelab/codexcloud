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

Choose between two standalone, human-guided Playwright examples:

- [Fast journey](../scripts/fast-journey-example.mjs): reduced motion and sound off
  throughout, for gameplay and save/load workflows.
- [Presentation sample](../scripts/presentation-sample-example.mjs): fast play to
  reach a useful position, then normal motion and sound for a short visual/audio
  sample. Explicit human confirmation of completed presentation gates restoration
  of fast settings. Uncertainty or timeout ends the session without restoring them.

Run either from `hex-command` with `node SCRIPT_PATH GAME_URL`, an existing server,
installed Playwright/Chromium, and a desktop display. Use audible output for the
second example. Both require a human playing through the UI; neither is an
autonomous playthrough or an audio recording. Adapt the journey to the requested
scenario and endpoint, and keep operator evidence identified as such.

For automated scripts, enabled controls determine which player input is available.
Advisor `canAct` can gate a request for advice; neither establishes that movement
has finished. Keep normal settings enabled until you have action-specific visual
completion evidence. If that evidence is unavailable, report timing as unverified
and keep normal settings for the remaining segment rather than toggling per action.

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
