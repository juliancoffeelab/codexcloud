---
name: live-play
description: Use only for explicitly user-authorized browser gameplay with the actual game. Ordinary implementation, review, simulation or deployment requests do not trigger it.
---

# Live play

Use only when the user explicitly authorizes real browser gameplay. Ordinary
implementation, review, simulation or deployment requests do not authorize it.
A natural-language gameplay request can supply authorization; the skill name
alone still needs a gameplay scope. Use a local Vite server; the harness does not
support production bundles.

1. Establish the campaign/scenario, assertions and execution limits from the
   request and record them in the disposable probe. Read
   [testing](../testing/SKILL.md). For loading investigations, also read
   [save-migrations](../save-migrations/SKILL.md) and the
   [loading-probe reference](references/harness.md#saved-progress-and-loading-probes).
2. Start Vite using the project build/toolchain guidance on a separate port from
   concurrent work. Copy [campaign.mjs](scripts/campaign.mjs) into ignored `tmp/`
   and adapt that copy. Use an isolated context and generated progress; identify
   synthetic historical fixtures when relevant. For custom policies, recalls,
   hooks or adapter changes, read the [harness reference](references/harness.md).
3. Run from the game root, setting `CHROMIUM_PATH` for an installed system browser
   when necessary.
   Give every run a fresh artifact directory. Defaults are bounded, not a victory
   guarantee; configure the requested limits and assertions before execution.
4. Stop when the assertions observe their target transitions or a bounded failure
   captures enough evidence to identify the blocker. Always close the harness in
   `finally`. An operation timeout does not cancel browser evaluation: capture
   available diagnostics and close the browser instead of continuing that session.
5. Distinguish campaign completion, AI defeat, application/harness failure and
   exhausted limits. Report findings and validation limits. Fix application code
   only when authorized; preserve current-run evidence under ignored `tmp/`.

```sh
# From hex-command; use a fresh artifacts directory for every run.
CHROMIUM_PATH=/usr/bin/chromium \
  LIVE_PLAY_URL=http://127.0.0.1:5191 \
  LIVE_PLAY_ARTIFACTS=tmp/live-play/run-1 \
  ENEMY_PAUSE_SCALE=0 PLAYER_AI_POLICY=classic CAMPAIGNS=mirror-night \
  node tmp/live-play/campaign.mjs
```

Use normal game commands; never manufacture units, resources, objectives or wins.
Use UI interactions for UI assertions and controller/engine inspection for state
assertions. A visible turn number or clean console alone does not prove a loading
or gameplay claim. Loading checks compare full native checkpoints including seed,
UI and supported historical identities before and after restoration/reload.

Artifacts contain decisions/revisions, terminal states, screenshots, checkpoints,
supported progress metadata, console errors and isolated storage. The total
deadline closes the browser even if AI computation hangs. Renderer failure may
prevent a final screenshot; preserve earlier checkpoints and logs in that case.
