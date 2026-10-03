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

// Fixed rule expectations: forest, hills, grove, then shallow water/marsh.
const expectedMovement: Record<UnitType, readonly [number, number, number, number]> = {
  commander: [2, 2, 2, 3],
  soldier: [2, 2, 2, 3],
  archer: [2, 2, 2, 2],
  scout: [1, 1, 1, 2],
  warden: [2, 2, 2, 3],
  adept: [2, 2, 2, 2],
  skeleton: [2, 2, 2, 2],
  bonearcher: [2, 2, 2, 2],
  wolf: [1, 2, 1, 2],
  boar: [1, 2, 1, 3],
  wildkeeper: [1, 2, 1, 2],
  necromancer: [2, 2, 2, 2],
  thorncaller: [1, 2, 1, 2],
};

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
  const [forest, hill, grove, wetlands] = expectedMovement[type];
  for (const [terrain, expected] of [
    ["plains", 1],
    ["forest", forest],
    ["hill", hill],
    ["grove", grove],
    ["ford", wetlands],
    ["marsh", wetlands],
    ["water", Infinity],
    ["bridge", 1],
    ["village", 1],
    ["keep", 1],
    ["ruins", 1],
    ["fort", 2],
  ] as const) {
    assert.equal(game.cost(unit, { terrain }), expected, `${type} entering ${terrain}`);
  }
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
  if (["wolf", "boar", "wildkeeper", "thorncaller"].includes(type)) {
    assert.equal(stat("forest").label, "1");
    assert.equal(stat("grove").label, "1");
    assert.equal(stat("hill").label, "2");
    assert.equal(stat("hill").className, "slow");
  }
  assert.deepEqual(view.game.snapshot(), before);
}

// Every woodland beast can traverse a full movement allowance of forest/grove hexes.
for (const type of ["wolf", "boar", "wildkeeper", "thorncaller"] as const) {
  const route = new Game();
  route.units = [];
  for (const tile of route.tiles) tile.terrain = "water";
  required(route.tile("0,0")).terrain = "plains";
  for (let col = 1; col <= 5; col++) {
    required(route.tile(`${col},0`)).terrain = col % 2 ? "forest" : "grove";
  }
  const beast = route.addUnit(type, "player", "0,0");
  assert.equal(route.cost(beast, { terrain: "forest" }), 1);
  assert.equal(route.cost(beast, { terrain: "grove" }), 1);
  const destination = `${TYPES[type].move},0`;
  assert.equal(route.reachable(beast.id).get(destination), TYPES[type].move);
  assert.equal(createGameView(route).reachable(beast.id).get(destination), TYPES[type].move);
  route.move(beast.id, destination);
  assert.equal(beast.tile, destination);
  assert.equal(route.reachable(beast.id).size, 0, "A spent move cannot move again");
  beast.tile = "0,0";
  beast.moved = false;
  beast.attacked = true;
  assert.equal(route.reachable(beast.id).size, 0, "Attacking also ends movement");
  beast.attacked = false;
  route.addUnit("wolf", "enemy", "1,0");
  assert.equal(route.reachable(beast.id).size, 0, "Occupied hexes still block the route");
  if (type === "boar") {
    assert.equal(route.cost(beast, { terrain: "marsh" }), 3);
    assert.equal(route.cost(beast, { terrain: "ford" }), 3);
    assert.equal(TYPES.boar.move, 3);
  }
}

// Entry costs affect both legal movement and long-range reverse routing.
for (const [type, distance] of [
  ["soldier", 4],
  ["archer", 3],
] as const) {
  const route = new Game();
  route.units = [];
  for (const tile of route.tiles) tile.terrain = "water";
  required(route.tile("0,0")).terrain = "plains";
  required(route.tile("1,0")).terrain = "ford";
  const goal = required(route.tile("2,0"));
  goal.terrain = "plains";
  const unit = route.addUnit(type, "player", "0,0");
  assert.equal(route.routeDistances(unit, [goal]).get("0,0"), distance);
  assert.equal(route.routeDistances(unit, [goal]).has("0,1"), false);
  assert.equal(route.reachable(unit.id).has(goal.id), type === "archer");
  if (type === "archer") {
    route.move(unit.id, goal.id);
    assert.equal(unit.tile, goal.id);
  } else {
    assert.throws(() => route.move(unit.id, goal.id), /outside/);
  }
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
