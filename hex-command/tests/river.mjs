import assert from 'node:assert/strict';
import { Game } from '../dist/engine.mjs';
import { loadBattle, saveBattle } from '../dist/save.mjs';

let stored;
globalThis.localStorage = { setItem: (_, value) => stored = value, getItem: () => stored };
const river = new Game('river');
assert.equal(river.tiles.length, 135);
assert.equal(river.tiles.filter(tile => tile.terrain === 'bridge').length, 1);
assert.equal(river.snapshot().villages.length, 6);
river.move('u3', '5,3');
saveBattle(river, true);
assert.deepEqual(loadBattle().game.snapshot(), river.snapshot());
assert.equal(loadBattle().pendingEnemy, true);
assert.equal(loadBattle().game.tile('7,3').terrain, 'bridge');
const legacy = new Game();
legacy.move('u3', '3,1');
saveBattle(legacy);
const save = JSON.parse(stored);
delete save.state.mapId;
stored = JSON.stringify(save);
assert.deepEqual(loadBattle().game.snapshot(), legacy.snapshot());
save.state.mapId = 'unknown';
stored = JSON.stringify(save);
assert.equal(loadBattle().game.mapId, 'mirror');

// Published revision-1 river saves keep their original crossings and progress.
const oldRiver = new Game('river', 1);
oldRiver.move('u3', '4,4');
oldRiver.unit('u2').tile = '7,1';
saveBattle(oldRiver);
const oldSave = JSON.parse(stored);
delete oldSave.state.mapRevision;
stored = JSON.stringify(oldSave);
assert.deepEqual(loadBattle().game.snapshot(), oldRiver.snapshot());
assert.equal(loadBattle().game.tile('7,1').terrain, 'bridge');
const fordGame = new Game('river');
fordGame.unit('u2').tile = '4,7';
assert.equal(fordGame.reachable('u2').get('5,7'), 3);
assert.equal(fordGame.reachable('u2').has('6,7'), false);
assert.equal(fordGame.reachable('u2').has('9,7'), false);
assert.equal(fordGame.cost(fordGame.unit('u3'), fordGame.tile('5,7')), 2);
fordGame.move('u2', '5,7');
assert.equal(fordGame.unit('u2').tile, '5,7');
for (const type of ['commander','soldier','archer','scout']) {
  assert.equal(fordGame.cost({type},fordGame.tile('5,7')), ['soldier','commander'].includes(type) ? 3 : 2);
}

// Reaching the only crossing requires first moving away from the objective.
for (const type of ['soldier', 'scout']) {
  const game = new Game('river');
  for (const tile of game.tiles) {
    tile.terrain = tile.col === 7 ? 'water' : 'plains';
    tile.owner = null;
  }
  game.tile('7,0').terrain = 'bridge';
  game.units = [];
  const commander = game.addUnit('commander', 'player', '6,8');
  const enemy = game.addUnit(type, 'enemy', '8,8');
  if (type === 'scout') game.tile('5,8').terrain = 'village';
  let crossed = false, achieved = false;
  for (let turn = 0; turn < 14; turn++) {
    game.beginTurn('enemy');
    game.enemyAct(enemy.id);
    assert.notEqual(game.tile(enemy.tile).terrain, 'water');
    if (turn === 0) assert.notEqual(enemy.tile, '8,8', 'must detour toward the crossing');
    crossed ||= game.tile(enemy.tile).col < 7;
    achieved ||= type === 'scout' ? game.tile('5,8').owner === 'enemy' : commander.hp < 16;
    if (achieved) break;
  }
  assert.ok(crossed && achieved, `${type} must cross to its objective`);
}
// Bridge hexes permit ordinary movement while adjacent water stays forbidden.
river.unit('u2').tile = '6,3';
assert.ok(river.reachable('u2').has('7,3'));
assert.ok(!river.reachable('u2').has('8,2'));
river.move('u2', '7,3');
assert.equal(river.unit('u2').tile, '7,3');
console.log('PASS: river movement, save migration, interrupted checkpoint and enemy bridge detours');

// Exposure applies to both weapon types and retaliation, while Shield still works.
const exposed = new Game('river');
exposed.tile('6,7').terrain = 'plains';
for (const attackerType of ['soldier', 'archer']) {
  for (const defenderType of ['soldier', 'archer', 'scout', 'commander']) {
    exposed.units = [];
    const attacker = exposed.addUnit(attackerType, 'player', '4,7');
    const defender = exposed.addUnit(defenderType, 'enemy', '5,7');
    const damageInWater = exposed.damage(attacker,defender);
    defender.tile = '6,7';
    assert.equal(damageInWater,exposed.damage(attacker,defender)+1);
  }
}
exposed.units = [];
const wader = exposed.addUnit('soldier','player','5,7');
const defender = exposed.addUnit('soldier','enemy','6,7');
assert.deepEqual(exposed.preview(wader,defender),{damage:4,retaliation:5});
exposed.attack(wader.id,defender.id);
assert.equal(wader.hp,7);
assert.equal(defender.hp,8);
console.log('PASS: shallow water movement penalties and increased damage including retaliation');
