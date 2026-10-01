import assert from 'node:assert/strict';
import { Game } from '../dist/engine.mjs';
import { loadBattle, saveBattle } from '../dist/save.mjs';

const game = new Game();
// Odd rows are offset by half a hex; reflecting across the keeps requires
// different column transforms for odd and even rows.
const mirror = tile => game.tile(`${(tile.row % 2 ? 10 : 11) - tile.col},${tile.row}`);
for (const id of ['3,1', '3,5']) {
  const village = game.tile(id);
  const enemyVillage = mirror(village);
  assert.equal(enemyVillage.terrain, 'village');
  const neighbors = game.neighbors(id);
  assert.equal(neighbors.some(tile => tile.terrain === 'forest'), false);
  assert.equal(neighbors.filter(tile => tile.terrain === 'hill').length, 1);
  for (const neighbor of neighbors) {
    assert.equal(mirror(neighbor).terrain, neighbor.terrain);
    assert.ok(game.neighbors(enemyVillage.id).includes(mirror(neighbor)));
  }
}
for (const tile of game.tiles.filter(tile => ['forest', 'hill'].includes(tile.terrain))) {
  assert.equal(mirror(tile).terrain, tile.terrain);
}
console.log('PASS: village approaches and central forests/hills mirror on staggered hexes');

// Existing v1 saves store battle progress rather than terrain. Updating the map
// must preserve an ongoing battle, including units on changed ground.
let stored;
globalThis.localStorage = {
  setItem: (_key, value) => { stored = value; },
  getItem: () => stored,
};
game.unit('u3').tile = '4,2'; // Former forest, now plains.
game.unit('u6').tile = '6,0'; // Former plains, now hills.
game.unit('u3').hp = 5;
game.unit('u3').moved = true;
game.tile('3,1').owner = 'player';
game.round = 4;
game.gold.player = 23;
game.record('Ongoing battle.');
saveBattle(game, true);
const loaded = loadBattle();
assert.deepEqual(loaded.game.snapshot(), game.snapshot());
assert.equal(loaded.pendingEnemy, true);
assert.equal(loaded.game.nextId, game.nextId);
assert.deepEqual(loaded.game.log, game.log);
assert.equal(loaded.game.tile('4,2').terrain, 'plains');
assert.equal(loaded.game.tile('6,0').terrain, 'hill');
console.log('PASS: saved battles retain progress and receive updated terrain');
