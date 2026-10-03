import assert from "node:assert/strict";
import { Game, TERRAIN, TYPES } from "../src/game/engine";
import { createGameView } from "../src/game/view";
import type { TerrainType, UnitType } from "../src/game/types";
import type { BattleView } from "../src/controller/view";
import { unitPanelView } from "../src/ui/unit-panel-view";
import { terrainPanelView } from "../src/ui/terrain-view";
import { required } from "./helpers";

function published(game: Game, selected: string): BattleView {
  return {
    game: createGameView(game),
    selected,
    target: null,
    inspectedTile: required(game.unit(selected)).tile,
    focusedTile: "0,0",
    renderRevision: 0,
    status: "",
    artStyle: "normal",
    artLoading: false,
    sound: false,
    canUndo: false,
    recruitment: null,
  };
}

// Every movement stat follows the production cost query without changing published state.
const game = new Game();
game.units = [];
for (const type of Object.keys(TYPES) as UnitType[]) {
  const unit = game.addUnit(type, "player", "0,0");
  const view = published(game, unit.id);
  const before = view.game.snapshot();
  const panel = unitPanelView(view);
  assert.equal(panel.kind, "details");
  if (panel.kind !== "details") throw new Error("Missing unit details");
  assert.equal(panel.details.movementCosts.length, Object.keys(TERRAIN).length);
  for (const movement of panel.details.movementCosts) {
    const actual = game.cost(unit, { terrain: movement.terrain });
    assert.equal(view.game.cost(unit, { terrain: movement.terrain }), actual);
    assert.equal(movement.label, Number.isFinite(actual) ? String(actual) : "Impassable");
    assert.equal(movement.name, TERRAIN[movement.terrain].name);
  }
  const stat = (terrain: TerrainType) =>
    required(panel.details.movementCosts.find((movement) => movement.terrain === terrain));
  assert.equal(stat("water").className, "blocked");
  assert.equal(stat("bridge").className, "easy");
  if (type === "scout") {
    for (const terrain of ["forest", "hill", "grove"] as const) {
      assert.equal(stat(terrain).label, "1");
      assert.equal(stat(terrain).className, "easy");
    }
  }
  if (["soldier", "commander", "warden", "boar"].includes(type)) {
    for (const terrain of ["marsh", "ford"] as const) {
      assert.equal(stat(terrain).label, "3");
      assert.equal(stat(terrain).className, "difficult");
    }
  }
  if (TYPES[type].woodland) {
    assert.equal(stat("forest").label, "1");
    assert.equal(stat("grove").label, "1");
    assert.equal(stat("hill").label, "2");
    assert.equal(stat("hill").className, "slow");
  }
  assert.deepEqual(view.game.snapshot(), before);
}

// Terrain keeps its protection and healing facts; movement costs belong to units.
const unit = required(game.units[0]);
const view = published(game, unit.id);
for (const terrain of Object.keys(TERRAIN) as TerrainType[]) {
  required(game.tile(unit.tile)).terrain = terrain;
  const terrainPanel = required(terrainPanelView({ ...view, game: createGameView(game) }));
  assert(!terrainPanel.facts.some((fact) => fact.startsWith("Movement:")));
  if (terrain === "hill") {
    assert(terrainPanel.facts.includes("Protection: −1 damage from all attacks"));
  }
  if (terrain === "grove") {
    assert(
      terrainPanel.facts.includes("Heals its owner’s occupant by 2 HP at the start of its turn"),
    );
  }
}

console.log("PASS: unit movement stats match engine costs and preserve immutable views");
