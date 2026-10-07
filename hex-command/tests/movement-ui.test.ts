import { test } from "vitest";
import assert from "node:assert/strict";
import { artworkForStyle } from "../src/presentation/art";
import { Game, TERRAIN, TYPES } from "./game-fixture";
import { createGameView } from "./fixture-view";
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
    artwork: artworkForStyle("normal"),
    artLoading: false,
    sound: false,
    canUndo: false,
    recruitment: null,
  };
}

// Fixed rule expectations: forest, hills, grove, then shallow water/marsh.
const expectedMovement: Partial<Record<UnitType, readonly [number, number, number, number]>> = {
  swordsman: [2, 2, 2, 2],
  spearman: [2, 2, 2, 2],
  tidekeeper: [3, 3, 3, 1],
  reefguard: [3, 3, 3, 1],
  harpooner: [3, 3, 3, 1],
  wavemender: [3, 3, 3, 1],
  commander: [2, 2, 2, 3],
  soldier: [2, 2, 2, 3],
  archer: [2, 2, 2, 2],
  scout: [1, 1, 1, 2],
  warden: [2, 2, 2, 3],
  adept: [2, 2, 2, 2],
  skeleton: [2, 2, 2, 2],
  bonearcher: [2, 2, 2, 2],
  wolf: [1, 2, 1, 2],
  direwolf: [1, 1, 1, 2],
  boar: [1, 2, 1, 3],
  wildkeeper: [1, 2, 1, 2],
  necromancer: [2, 2, 2, 2],
  thorncaller: [1, 2, 1, 2],
};

// Every movement stat follows the production cost query without changing published state.

for (const type of Object.keys(TYPES) as UnitType[]) {
  test(
    "Published movement stats match engine costs for every unit — " + JSON.stringify([type]),
    async () => {
      const game = new Game();
      game.units = [];
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
      const [forest, hill, grove, wetlands] = required(
        expectedMovement[type] ?? expectedMovement[TYPES[type].family ?? type],
      );
      const ground = TYPES[type].race === "merfolk" ? 2 : 1;
      for (const [terrain, expected] of [
        ["plains", ground],
        ["forest", forest],
        ["hill", hill],
        ["grove", grove],
        ["ford", wetlands],
        ["marsh", wetlands],
        ["water", TYPES[type].race === "merfolk" ? 1 : Infinity],
        ["bridge", 1],
        ["village", ground],
        ["keep", ground],
        ["ruins", ground],
        ["fort", ground + 1],
      ] as const) {
        assert.equal(game.cost(unit, { terrain }), expected, `${type} entering ${terrain}`);
      }
      assert.equal(stat("water").className, TYPES[type].race === "merfolk" ? "easy" : "blocked");
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
    },
  );
}

test.each([
  ["reefguard", "plains", 2],
  ["reefguard", "water", 4],
  ["reefguard", "bridge", 4],
  ["harpooner", "plains", 2],
  ["harpooner", "water", 5],
] as const)("%s routes across %s reach %i tiles", (type, terrain, steps) => {
  const game = new Game();
  game.units = [];
  for (const tile of game.tiles) tile.terrain = "forest";
  for (let col = 0; col <= 5; col++) required(game.tile(`${col},0`)).terrain = terrain;
  const unit = game.addUnit(type, "player", "0,0");
  const destination = `${steps},0`;
  assert.equal(game.reachable(unit.id).get(destination), steps * (terrain === "plains" ? 2 : 1));
  if (steps < 5) assert.equal(game.reachable(unit.id).has(`${steps + 1},0`), false);
  game.move(unit.id, destination);
  assert.equal(unit.tile, destination);
});

// Every woodland beast can traverse a full movement allowance of forest/grove hexes.
for (const type of ["wolf", "boar", "wildkeeper", "thorncaller"] as const) {
  test(
    "Woodland beasts spend their allowance across forests and groves — " + JSON.stringify([type]),
    async () => {
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
      }
    },
  );
}

// Terrain keeps its protection and healing facts; movement costs belong to units.

for (const terrain of Object.keys(TERRAIN) as TerrainType[]) {
  test(
    "Terrain panel shows protection and healing rather than generic movement — " +
      JSON.stringify([terrain]),
    async () => {
      const game = new Game();
      const unit = required(game.units[0]);
      const view = published(game, unit.id);
      required(game.tile(unit.tile)).terrain = terrain;
      const terrainPanel = required(terrainPanelView({ ...view, game: createGameView(game) }));
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
