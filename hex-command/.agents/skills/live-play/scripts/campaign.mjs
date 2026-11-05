import { chromium } from "playwright";
import { mkdir, appendFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

// Copy this file into ignored tmp/, then adapt hooks/assertions for the authorized task.
export async function openHarness(options = {}) {
  if (process.env.LIVE_PLAY_AUTHORIZED !== "1")
    throw new Error("Explicit user authorization required before opening live gameplay");
  const config = {
    url: process.env.LIVE_PLAY_URL || "http://127.0.0.1:5191",
    artifacts: process.env.LIVE_PLAY_ARTIFACTS || "tmp/live-play/artifacts",
    enemyPauseScale: Number(process.env.ENEMY_PAUSE_SCALE ?? 0),
    operationTimeout: 60_000,
    totalTimeout: 15 * 60_000,
    playerAI: { policy: process.env.PLAYER_AI_POLICY || "predictor", diplomacy: "unrestricted" },
    ...options,
  };
  const address = new URL(config.url);
  if (!["localhost", "127.0.0.1", "[::1]"].includes(address.hostname))
    throw new Error("Use an isolated local Vite application for this adapter.");
  if (!Number.isFinite(config.enemyPauseScale) || config.enemyPauseScale < 0)
    throw new Error("enemyPauseScale must be finite and nonnegative");
  const directory = resolve(config.artifacts);
  await mkdir(directory, { recursive: true });
  const log = (event, data = {}) =>
    appendFile(
      `${directory}/events.jsonl`,
      JSON.stringify({ at: Date.now(), event, ...data }) + "\n",
    );
  const browser = await chromium.launch({
    headless: true,
    ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
  });
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  page.setDefaultTimeout(config.operationTimeout);
  page.setDefaultNavigationTimeout(config.operationTimeout);
  const errors = [];
  page.on("pageerror", (error) => {
    errors.push(error.stack || String(error));
    void log("page-error", { error: error.stack || String(error) });
  });
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(`console: ${message.text()}`);
  });
  const deadline = setTimeout(() => {
    errors.push("Total wall-clock deadline exceeded");
    void browser.close();
  }, config.totalTimeout);
  const bounded = async (promise, label) => {
    let timer;
    try {
      return await Promise.race([
        promise,
        new Promise((_, reject) => {
          timer = setTimeout(
            () => reject(new Error(`${label}: operation deadline exceeded`)),
            config.operationTimeout,
          );
        }),
      ]);
    } finally {
      clearTimeout(timer);
    }
  };
  // Browser response instrumentation only: fail if the controller seam changes.
  // The closure follows #game across restoration and campaign transitions.
  await page.route("**/src/controller/battle.ts*", async (route) => {
    try {
      const response = await route.fetch();
      const source = await response.text();
      const marker = "const game = restored.game;\n    this.#game = game;";
      if (source.split(marker).length !== 2) throw new Error("Controller adapter marker changed");
      await route.fulfill({
        response,
        body: source.replace(
          marker,
          `${marker}
        globalThis.__livePlay = { controller: this, game: () => this.#game };
        this.pause = ms => new Promise(resolve => setTimeout(resolve, ms * ${config.enemyPauseScale}));
      `,
        ),
      });
    } catch (error) {
      errors.push(String(error));
      await route.abort();
    }
  });
  const evaluate = (fn, arg) => bounded(page.evaluate(fn, arg), "browser evaluation");
  const snapshot = () => evaluate(() => globalThis.__livePlay.controller.snapshot());
  const entries = () => evaluate(() => globalThis.__livePlay.controller.savedProgress());
  const command = (name, args = []) =>
    evaluate(
      async ({ name, args }) => {
        const controller = globalThis.__livePlay.controller;
        if (typeof controller[name] !== "function")
          throw new Error(`Unknown controller command ${name}`);
        return await controller[name](...args);
      },
      { name, args },
    );
  const checkpoint = async (name) => {
    const data = await evaluate(() => {
      const { controller, game } = globalThis.__livePlay;
      return {
        snapshot: controller.snapshot(),
        native: game().checkpoint(),
        entries: controller.savedProgress(),
        storage: Object.fromEntries(
          Object.entries(localStorage).filter(([key]) => key.startsWith("hex-command")),
        ),
      };
    });
    await writeFile(`${directory}/${name}.json`, JSON.stringify(data, null, 2));
    await page.screenshot({ path: `${directory}/${name}.png`, fullPage: true, timeout: 5000 });
    return data;
  };
  const step = async () => {
    const result = await evaluate(async (ai) => {
      const { controller: c, game } = globalThis.__livePlay;
      const g = game();
      const before = g.engine.view();
      if (!before.ok) throw new Error(JSON.stringify(before));
      const decision = g.engine.chooseAiCommand(ai);
      if (!decision.ok) throw new Error(JSON.stringify(decision));
      const { command: a, revision } = decision.value;
      const id = (number) => `u${number}`;
      switch (a.kind) {
        case "move":
          c.move(id(a.unit), a.tile);
          break;
        case "attack":
          c.attack(id(a.unit), id(a.target), a.weapon);
          break;
        case "recruit":
          c.recruit(id(a.unit), a.type, a.tile);
          break;
        case "advance":
          c.advance(id(a.unit), a.type);
          break;
        case "dropItem":
          c.dropItem(id(a.unit), a.item);
          break;
        case "endTurn":
          await c.endTurn();
          break;
        case "setUnitIntent":
          // AI bookkeeping has no UI command. Apply the native decision, then use
          // the usual controller commit so publication and persistence still run.
          g.dispatch(a, revision);
          c.commitAction();
          break;
        default:
          throw new Error(`Unhandled AI command: ${JSON.stringify(a)}`);
      }
      const after = game().engine.view();
      if (!after.ok || after.value.revision <= before.value.revision)
        throw new Error("AI action did not advance the live battle revision");
      return { command: a, revision, snapshot: c.snapshot() };
    }, config.playerAI);
    await log("action", result);
    return result;
  };
  const close = async () => {
    clearTimeout(deadline);
    try {
      if (!page.isClosed()) {
        await bounded(page.content(), "capture document")
          .then((content) => writeFile(`${directory}/document.txt`, content))
          .catch(() => {});
        await page
          .screenshot({ path: `${directory}/browser-final.png`, fullPage: true, timeout: 5000 })
          .catch(() => {});
        await checkpoint("final").catch((error) => errors.push(`capture: ${error}`));
        await context.storageState({ path: `${directory}/storage.json` }).catch(() => {});
      }
      await writeFile(`${directory}/errors.json`, JSON.stringify(errors, null, 2));
    } finally {
      await browser.close();
    }
  };
  try {
    await page.goto(config.url);
    await page.waitForFunction(
      () =>
        !!globalThis.__livePlay?.controller?.commands &&
        globalThis.__livePlay.controller.initialized,
    );
    await log("ready", { config });
    return {
      page,
      config,
      directory,
      errors,
      evaluate,
      snapshot,
      entries,
      command,
      checkpoint,
      step,
      log,
      close,
    };
  } catch (error) {
    await log("startup-failure", { error: String(error), errors });
    await close();
    throw error;
  }
}

// Hooks run between real controller actions. Put task-specific assertions here
// or wrap these exported functions in a disposable probe.
export async function runCampaigns(harness, options = {}) {
  const {
    campaigns = ["lantern-road"],
    maxActions = 20_000,
    maxRetries = 0,
    enemyAI = { policy: "classic", diplomacy: "diplomatic" },
    onStep = async () => {},
    recall = async () => [],
  } = options;
  const results = [];
  for (const campaign of campaigns) {
    await harness.command("startCampaign", [campaign, { first: "player", enemy: enemyAI }]);
    let retries = 0;
    let terminal;
    for (let action = 0; action < maxActions; action++) {
      const state = await harness.snapshot();
      if (state.campaign?.dialogueIndex < state.campaign?.dialogue.length) {
        await harness.command("advanceDialogue", [true]);
      } else if (state.winner === "enemy") {
        if (retries >= maxRetries) {
          terminal = "ai-defeat";
          break;
        }
        retries++;
        await harness.command("retryMission");
      } else if (state.campaign?.completed && state.winner === "player") {
        await harness.page
          .getByRole("button", { name: "Choose another campaign", exact: true })
          .click();
        await harness.page
          .getByRole("dialog", { name: "New game", exact: true })
          .waitFor({ state: "visible" });
        await harness.page.getByRole("button", { name: "Close new game", exact: true }).click();
        terminal = "campaign-complete-next-game-ready";
        break;
      } else if (state.winner === "player") {
        // Default empty recall is legal. Override for the requested workload.
        await harness.command("nextMission", [await recall(state)]);
      } else if (state.turn === "player") {
        await harness.step();
      } else {
        throw new Error("Unexpected enemy phase outside awaited controller endTurn");
      }
      await onStep(harness, { campaign, action });
    }
    terminal ??= "action-limit";
    const result = { campaign, terminal, retries, state: await harness.snapshot() };
    results.push(result);
    await harness.log("campaign-result", result);
    await harness.checkpoint(`campaign-${results.length}`);
  }
  return results;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  if (process.env.LIVE_PLAY_AUTHORIZED !== "1")
    throw new Error(
      "Explicit user authorization required; set LIVE_PLAY_AUTHORIZED=1 only after receiving it.",
    );
  const harness = await openHarness();
  try {
    const results = await runCampaigns(harness, {
      campaigns: (process.env.CAMPAIGNS || "lantern-road").split(","),
    });
    console.log(
      JSON.stringify(
        results.map(({ campaign, terminal, retries }) => ({ campaign, terminal, retries })),
        null,
        2,
      ),
    );
    if (
      harness.errors.length ||
      results.some((result) => result.terminal !== "campaign-complete-next-game-ready")
    )
      process.exitCode = 1;
  } finally {
    await harness.close();
  }
}
