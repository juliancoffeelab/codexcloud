// Unattended focused UI example, not a complete battle. From hex-command:
// node .agents/skills/live-play/scripts/presentation-sample-example.mjs GAME_URL
// Write actions/assertions into the script; no terminal prompts or human gates.
import { chromium } from "playwright";
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";

const gameUrl = process.argv[2];
if (!gameUrl) throw new Error("Supply the URL of a running game.");
const evidence = `tmp/live-play-presentation-${Date.now()}`;
await mkdir(evidence, { recursive: true });
const browser = await chromium.launch({ headless: true });
const deadline = setTimeout(() => void browser.close(), 2 * 60_000);
let page;
try {
  const context = await browser.newContext({
    reducedMotion: "reduce",
    recordVideo: { dir: evidence },
  });
  page = await context.newPage();
  page.setDefaultTimeout(20_000);
  page.setDefaultNavigationTimeout(45_000);
  await page.goto(gameUrl);
  // Fresh context: enter a deterministic skirmish through the normal setup UI.
  const setup = page.getByRole("dialog", { name: "New game", exact: true });
  await setup.getByRole("button", { name: "Skirmish", exact: true }).click();
  await setup.getByRole("button", { name: "Battlefield", exact: true }).click();
  await page
    .getByRole("dialog", { name: "Battlefield", exact: true })
    .getByRole("option", { name: "Classic mirror", exact: true })
    .click();
  await setup.getByRole("button", { name: "First turn", exact: true }).click();
  await page
    .getByRole("dialog", { name: "First turn", exact: true })
    .getByRole("option", { name: "You", exact: true })
    .click();
  await setup.getByRole("button", { name: "Start skirmish", exact: true }).click();
  await setup.waitFor({ state: "hidden" });
  await page.locator("#map.map-ready canvas").waitFor({ state: "visible" });
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  const menu = page.getByRole("dialog", { name: "Menu", exact: true });
  const sound = menu.getByRole("checkbox", { name: "Sound effects", exact: true });
  await sound.uncheck();
  await menu.getByRole("button", { name: "Save game", exact: true }).click();
  await menu.getByRole("button", { name: "Saved", exact: true }).waitFor();
  await menu.getByRole("button", { name: "Close menu", exact: true }).click();

  // Fast approach ends here. Keep normal settings for the rest of this segment:
  // controls/canAct allow input but cannot safely gate switching back to fast mode.
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  await sound.check();
  await menu.getByRole("button", { name: "Close menu", exact: true }).click();
  const before = await page.locator("#round").innerText();
  await page.locator("#end-turn").click();
  await page.waitForFunction((previous) => {
    const turn = document.querySelector("#end-turn");
    return document.querySelector("#round")?.textContent !== previous && turn && !turn.disabled;
  }, before);
  // This asserts return to gameplay, not completion of every animation or sound.
  // Add action-specific read-only visual observations for the behavior under test.
  // When that evidence is unavailable, report the gap; never insert operator input.
  await page.screenshot({ path: `${evidence}/next-turn.png` });
  console.log({
    endpoint: "enemy turn returned control at a later round",
    evidence,
    settings: "normal motion and sound retained until context close",
    limitations: "video contains no audio capture; visual/audio synchronization unverified",
  });
} catch (error) {
  if (page && !page.isClosed()) {
    await page.screenshot({ path: `${evidence}/blocker.png`, timeout: 5000 }).catch(() => {});
  }
  throw error;
} finally {
  clearTimeout(deadline);
  await browser.close();
}
