// Human-guided example. From hex-command:
// node .agents/skills/live-play/scripts/presentation-sample-example.mjs GAME_URL
import { chromium } from "playwright";
import { createInterface } from "node:readline/promises";
import { stdin, stdout } from "node:process";

const gameUrl = process.argv[2];
if (!gameUrl) throw new Error("Supply the URL of a running game.");
const browser = await chromium.launch({ headless: false });
const terminal = createInterface({ input: stdin, output: stdout });
try {
  const context = await browser.newContext({ reducedMotion: "reduce" });
  const page = await context.newPage();
  await page.goto(gameUrl);
  const entered = await terminal.question(
    "Choose the scenario and dismiss its briefing through the UI; type entered: ",
    {
      signal: AbortSignal.timeout(5 * 60_000),
    },
  );
  if (entered.trim() !== "entered") throw new Error("Game entry not confirmed.");
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  const menu = page.getByRole("dialog", { name: "Menu", exact: true });
  const sound = menu.getByRole("checkbox", { name: "Sound effects", exact: true });
  const closeMenu = menu.getByRole("button", { name: "Close menu", exact: true });
  await sound.uncheck();
  await closeMenu.click();
  console.log("Play through the UI to a position suitable for a movement/combat sample.");
  const before = await terminal.question(
    "When the preceding action has visibly finished, type settled: ",
    { signal: AbortSignal.timeout(20 * 60_000) },
  );
  if (before.trim() !== "settled") throw new Error("Sample not started: completion uncertain.");

  await page.emulateMedia({ reducedMotion: "no-preference" });
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  await sound.check();
  await closeMenu.click();
  console.log(
    "Perform a short movement/combat segment through player controls. Watch the " +
      "full movement, strikes and effects, and listen to impact cues. Enabled controls " +
      "or advisor canAct allow gameplay; they do not prove animation completion.",
  );
  const completion = await terminal.question(
    "After observing the entire sample finish, type finished; otherwise describe uncertainty: ",
    { signal: AbortSignal.timeout(5 * 60_000) },
  );
  if (completion.trim() !== "finished") {
    console.log({ evidence: "operator report", completion, restoredFastSettings: false });
    throw new Error(
      "Sample incomplete or uncertain; normal settings retained until browser close.",
    );
  }
  const observations = await terminal.question(
    "Describe the actions and visual/audio observations: ",
    {
      signal: AbortSignal.timeout(5 * 60_000),
    },
  );
  // Restore only after explicit visual confirmation, never from gameplay readiness.
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  await sound.uncheck();
  await closeMenu.click();
  await page.emulateMedia({ reducedMotion: "reduce" });
  console.log({ evidence: "human-observed sample; no audio capture", observations });
  const result = await terminal.question(
    "Continue the UI journey; record its endpoint or blocker: ",
    {
      signal: AbortSignal.timeout(20 * 60_000),
    },
  );
  console.log({ evidence: "operator report", result });
} finally {
  // Errors/timeouts skip restoration; closing the owned browser ends the session.
  terminal.close();
  await browser.close();
}
