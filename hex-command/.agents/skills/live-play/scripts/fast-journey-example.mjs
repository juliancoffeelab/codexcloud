// Human-guided example. From hex-command:
// node .agents/skills/live-play/scripts/fast-journey-example.mjs GAME_URL
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
  await menu.getByRole("checkbox", { name: "Sound effects", exact: true }).uncheck();
  await menu.getByRole("button", { name: "Close menu", exact: true }).click();
  console.log(
    "Play using the visible controls. " +
      "Keep reduced motion and sound off throughout. Reach the requested end screen " +
      "or reproduce the requested workflow; otherwise describe the blocker. " +
      "This example does not verify normal animation timing or audible output.",
  );
  const result = await terminal.question("Record endpoint and observations: ", {
    signal: AbortSignal.timeout(30 * 60_000),
  });
  console.log({ evidence: "operator report", result });
} finally {
  terminal.close();
  await browser.close();
}
