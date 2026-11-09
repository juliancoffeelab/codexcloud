/** Run the existing Rust planner on a disposable copy; return a UI action. */
export function suggestAction(factory, checkpoint, options) {
  const value = (result) => {
    if (!result.ok) throw new Error(JSON.stringify(result.error));
    return result.value;
  };
  const forecast = value(factory.restore(structuredClone(checkpoint)));
  try {
    const intents = new Set();
    while (true) {
      const decision = value(forecast.chooseAiCommand(options));
      if (decision.command.kind !== "setUnitIntent") return structuredClone(decision.command);
      // Planner bookkeeping has no player control. Apply it only to the copy.
      const key = JSON.stringify(decision.command);
      if (intents.has(key)) throw new Error("Planner repeated an intent without a UI action");
      intents.add(key);
      value(forecast.dispatch(decision.command, decision.revision));
    }
  } finally {
    value(forecast.dispose());
  }
}

/** Install before navigating to a local Vite game. No live mutation handles escape. */
export async function installAiAdvisor(page) {
  await page.route("**/src/controller/battle.ts*", async (route) => {
    const response = await route.fetch();
    const source = await response.text();
    const marker = "const game = restored.game;\n    this.#game = game;";
    if (source.split(marker).length !== 2)
      throw new Error("AI advisor: controller observation seam changed");
    await route.fulfill({
      response,
      body: source.replace(
        marker,
        `${marker}
        const suggestOnCopy = (${suggestAction.toString()});
        globalThis.__livePlayAdvisor = Object.freeze({
          observe: () => structuredClone({
            revision: nativeValue(this.#game.engine.view()).revision,
            state: this.snapshot(),
            checkpoint: nativeValue(this.#game.checkpoint()),
            selected: this.lastPublishedView?.selected ?? null,
            target: this.lastPublishedView?.target ?? null,
            canAct: this.initialized && this.active(),
          }),
          suggest: options => {
            if (!this.initialized || !this.active())
              throw new Error("AI advisor: finish the visible turn or dialog before planning");
            return {
              revision: nativeValue(this.#game.engine.view()).revision,
              command: suggestOnCopy(this.nativeFactory, nativeValue(this.#game.checkpoint()), options),
            };
          },
        });`,
      ),
    });
  });
}

export async function observe(page) {
  return page.evaluate(() => {
    if (!globalThis.__livePlayAdvisor)
      throw new Error("AI advisor unavailable: install before navigating to the local Vite game");
    return globalThis.__livePlayAdvisor.observe();
  });
}

export async function recommend(page, options = { policy: "classic", diplomacy: "diplomatic" }) {
  return page.evaluate((options) => {
    if (!globalThis.__livePlayAdvisor)
      throw new Error("AI advisor unavailable: install before navigating to the local Vite game");
    return globalThis.__livePlayAdvisor.suggest(options);
  }, options);
}
