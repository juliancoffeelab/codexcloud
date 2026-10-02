import assert from "node:assert/strict";
import { Game, TERRAIN, TYPES } from "../src/game/engine";
import { MAPS } from "../src/game/maps";
import { createGameView } from "../src/game/view";
import {
  createBattleCommands,
  recruitmentContext,
  type BattleCommands,
  type BattleView,
} from "../src/controller/view";
import { required } from "./helpers";

const live = new Game();
const enemy = live.addUnit("soldier", "enemy", "2,3");
const view = createGameView(live);
const originalState = view.snapshot();
const originalTiles = structuredClone(view.tiles);

// Shared definitions cannot silently change query answers between publications.
assert.throws(() => Object.assign(TYPES.soldier.attacks[0], { damage: 99 }), TypeError);
assert.throws(() => Object.assign(TERRAIN.plains, { name: "Changed" }), TypeError);
assert.throws(
  () => Object.assign(required(MAPS.mirror.terrain[0]), { terrain: "forest" }),
  TypeError,
);

// Rendering receives plain, immutable data and pure queries, never engine commands.
assert.equal(Object.getPrototypeOf(view), Object.prototype);
for (const command of ["move", "attack", "recruit", "endTurn", "restoreState", "addUnit"]) {
  assert.equal(command in view, false, `${command} must stay outside the published view`);
}
for (const value of [
  view,
  view.gold,
  view.log,
  view.units,
  view.tiles,
  ...view.units,
  ...view.tiles,
]) {
  assert(Object.isFrozen(value));
}
assert.throws(() => Object.assign(required(view.unit("u2")), { hp: 0 }), TypeError);
assert.throws(() => Object.assign(required(view.tile("3,1")), { owner: "enemy" }), TypeError);
assert.equal(Reflect.set(view.gold, "player", 0), false);
assert.equal(Reflect.set(view.log, "0", "Changed from outside"), false);
assert.equal(Reflect.set(view.units, "length", 0), false);
assert.equal(Reflect.set(view.tiles, "length", 0), false);

// Query results keep snapshot identities where callers compare units with includes().
const soldier = required(view.unit("u2"));
const target = required(view.unit(enemy.id));
assert.equal(view.at(soldier.tile), soldier);
assert(view.targets(soldier.id).includes(target));
assert(Object.isFrozen(view.targets(soldier.id)));
assert(Object.isFrozen(view.neighbors(soldier.tile)));
assert(Object.isFrozen(view.recruitTiles("u1", "soldier")));
const details = view.previewDetails(soldier, target, "sword");
assert.notEqual(details.attack, TYPES.soldier.attacks[0]);
assert.notEqual(
  details.response,
  TYPES.soldier.attacks.find((weapon) => weapon.id === details.response?.id),
);
for (const value of [
  details,
  details.attack,
  details.damageDetail,
  details.response,
  details.retaliationDetail,
]) {
  if (value) assert(Object.isFrozen(value));
}
assert.throws(() => Object.assign(details.attack, { damage: 99 }), TypeError);
assert.equal(TYPES.soldier.attacks[0].damage, 4);

// Maps returned by queries may be changed at runtime without changing the revision.
const reachable = view.reachable("u3");
const expectedReachable = [...reachable];
Map.prototype.clear.call(reachable);
Map.prototype.set.call(reachable, "0,0", 99);
assert.deepEqual([...view.reachable("u3")], expectedReachable);
assert.deepEqual(view.snapshot(), originalState);

// Later engine actions and direct fixture changes cannot rewrite an old publication.
live.move("u3", "3,1");
live.gold.player = 0;
live.log[0] = "Later event";
required(live.unit("u2")).hp = 1;
required(live.tile("3,1")).terrain = "forest";
assert.deepEqual(view.snapshot(), originalState);
assert.deepEqual(view.tiles, originalTiles);
assert.equal(required(view.unit("u3")).tile, "1,2");
assert.equal(view.gold.player, 12);
assert.equal(required(view.unit("u2")).hp, 12);
assert.equal(required(view.tile("3,1")).terrain, "village");

const context = required(recruitmentContext(view, null, "0,2"));
assert(Object.isFrozen(context));
assert.equal(context.commander, view.unit("u1"));
assert.equal(context.stationed, true);
assert.equal(recruitmentContext(view, "u1", "0,2"), null);

// The command facade forwards calls while omitting controller reads and mutation access.
const chosen: string[] = [];
const controller = {
  game: live,
  canUndo: () => true,
  recruitmentContext: () => context,
  choose(tileId: string) {
    chosen.push(tileId);
  },
  focusTile: () => {},
  clearSelection: () => {},
  attackSelected: () => {},
  recruitHere: () => {},
  endTurn: async () => undefined,
  undoMove: () => live.snapshot(),
  reset: () => live.snapshot(),
  setArtStyle: async () => {},
  setSound: async () => {},
  error: () => {},
};
const commands = createBattleCommands(controller);
assert(Object.isFrozen(commands));
for (const read of ["game", "canUndo", "recruitmentContext"]) assert.equal(read in commands, false);
commands.choose("3,1");
assert.deepEqual(chosen, ["3,1"]);

// Compile-time contract checks are intentionally never executed.
function enforceViewContract(published: BattleView, operations: BattleCommands): void {
  // @ts-expect-error Published games cannot execute engine commands.
  published.game.move("u3", "3,1");
  // @ts-expect-error Unit health is read-only.
  required(published.game.unit("u2")).hp = 0;
  // @ts-expect-error Tile ownership is read-only.
  required(published.game.tile("3,1")).owner = "enemy";
  // @ts-expect-error Gold is read-only.
  published.game.gold.player = 0;
  // @ts-expect-error Published arrays cannot be modified.
  published.game.units.push(soldier);
  // @ts-expect-error Published logs cannot be modified.
  published.game.log[0] = "Changed";
  // @ts-expect-error Query maps expose only read operations to TypeScript callers.
  published.game.reachable("u3").set("0,0", 1);
  // @ts-expect-error Undo availability belongs to the published view.
  operations.canUndo();
  // @ts-expect-error Components cannot reach the mutable engine through commands.
  operations.game;
  // @ts-expect-error Recruitment belongs to the published view, not controller reads.
  operations.recruitmentContext(undefined);
  // @ts-expect-error Published undo availability cannot be overwritten.
  published.canUndo = true;
}
void enforceViewContract;

console.log(
  "PASS: immutable detached views, isolated queries, and command-only component contracts",
);
