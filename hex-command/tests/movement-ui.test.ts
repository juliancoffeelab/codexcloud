import { test } from "vitest";
import assert from "node:assert/strict";
import { currentArtwork } from "../src/presentation/art";
import { Game, TERRAIN } from "./game-fixture";
import type { BattleView } from "../src/controller/view";
import { unitPanelView } from "../src/ui/unit-panel-view";
import { terrainPanelView } from "../src/ui/terrain-view";
import { required } from "./helpers";

function published(game: Game, selected: string): BattleView {
  return {
    game: game.publishedView(),
    selected,
    target: null,
    inspectedTile: required(game.unit(selected)).tile,
    focusedTile: "0,0",
    renderRevision: 0,
    status: "",
    artwork: currentArtwork(),
    sound: false,
    canUndo: false,
    completedTurns: 0,
    recruitment: null,
  };
}

test("Terrain rows show native movement costs without mutating the view", () => {
  const game = new Game();
  game.units = [];
  const unit = game.addUnit("soldier", "player", "0,0");
  const view = published(game, unit.id);
  const before = view.game.snapshot();
  const panel = unitPanelView(view);
  if (panel.kind !== "details") throw new Error("Missing unit details");
  for (const row of panel.details.terrainEffects) {
    const cost = view.game.cost(unit, { terrain: row.terrain });
    assert.equal(
      row.label,
      Number.isFinite(cost) ? `${cost} ${cost === 1 ? "Step" : "Steps"}` : "-",
    );
    assert.equal(row.name, TERRAIN[row.terrain].name);
  }
  assert.deepEqual(view.game.snapshot(), before);
});

// Terrain keeps its protection and healing facts; movement costs belong to units.

for (const terrain of ["plains", "hill", "grove"] as const) {
  test(
    "Terrain panel shows protection and healing rather than generic movement — " +
      JSON.stringify([terrain]),
    async () => {
      const game = new Game();
      const unit = required(game.units[0]);
      const view = published(game, unit.id);
      required(game.tile(unit.tile)).terrain = terrain;
      const terrainPanel = required(terrainPanelView({ ...view, game: game.publishedView() }));
      assert(!terrainPanel.facts.some((fact) => fact.startsWith("Movement:")));
      if (terrain === "hill") {
        assert(
          terrainPanel.facts.includes(
            "Land troops: −1 damage received. Merfolk: +1 damage received.",
          ),
        );
      }
      if (terrain === "grove") {
        assert(
          terrainPanel.facts.includes(
            "Heals its owner’s occupant by 4 HP at the start of its turn",
          ),
        );
      }
    },
  );
}

test("Terrain facts distinguish category-specific and damage-type-specific effects", async () => {
  const { terrainDamageFacts } = await import("../src/ui/terrain-effects");
  assert.deepEqual(
    terrainDamageFacts([
      [0, 0, 0],
      [0, 0, 0],
    ]),
    [{ label: "", value: 0 }],
  );
  assert.deepEqual(
    terrainDamageFacts([
      [0, -2, 1],
      [0, 0, 0],
    ]),
    [
      { label: "Melee Pierce", value: -2 },
      { label: "Melee Impact", value: 1 },
    ],
  );
});
