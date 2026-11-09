// Unattended focused UI example, not a complete battle. From hex-command:
// node .agents/skills/live-play/scripts/fast-journey-example.mjs GAME_URL
// Write all gameplay actions/assertions into the script; never wait for an operator.
import { chromium } from "playwright";
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";

const gameUrl = process.argv[2];
if (!gameUrl) throw new Error("Supply the URL of a running game.");
const evidence = `tmp/live-play-fast-${Date.now()}`;
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

  const before = await page.locator("#round").innerText();
  await page.reload();
  await page.locator("#map.map-ready canvas").waitFor({ state: "visible" });
  assert.equal(await page.locator("#round").innerText(), before);
  await page.getByRole("button", { name: "Menu", exact: true }).click();
  assert.equal(await sound.isChecked(), false);
  await menu.getByRole("button", { name: "Close menu", exact: true }).click();
  await page.screenshot({ path: `${evidence}/restored.png` });
  console.log({
    endpoint: "UI save, reload, same visible turn and muted preference",
    evidence,
    limitations: "HUD equality is not native checkpoint identity; no animation/audio timing claim",
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
