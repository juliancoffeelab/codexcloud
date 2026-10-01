import assert from 'node:assert/strict';
import { Game, TYPES } from '../dist/engine.mjs';
import { abilityMarkup } from '../dist/unit-ability.mjs';

const game = new Game();
const commander = game.units.find(unit => unit.side === 'player' && unit.type === 'commander');
const markup = (unit = commander, canAct = true) => abilityMarkup({ game, unit, canAct });
const hintCount = html => (html.match(/New recruits act next turn\./g) || []).length;

let html = markup();
assert.equal((html.match(/class="ability">Recruit/g) || []).length, 1);
assert.equal(hintCount(html), 1);
assert.equal((html.match(/data-recruit=/g) || []).length, 3);
assert.doesNotMatch(html, /Recruit an ally|Return to your keep|Recruit while/);
assert.doesNotMatch(html, / disabled/);
for (const name of ['soldier', 'archer', 'scout']) {
  assert.ok(html.includes(`data-recruit="${name}"`));
  assert.ok(html.includes(`${TYPES[name].cost} gold`));
}

assert.equal((markup(commander, false).match(/ disabled/g) || []).length, 3);
game.gold.player = 0;
assert.equal((markup().match(/ disabled/g) || []).length, 3);
game.gold.player = 12;
for (const tile of game.neighbors(commander.tile).filter(tile => tile.terrain !== 'water' && !game.at(tile.id))) {
  game.addUnit('soldier', 'player', tile.id);
}
assert.equal((markup().match(/ disabled/g) || []).length, 3);

commander.tile = '2,3';
html = markup();
assert.match(html, /Return to your keep to recruit\./);
assert.equal(hintCount(html), 1);
assert.doesNotMatch(html, /data-recruit=/);
commander.tile = '10,3';
assert.match(markup(), /Return to your keep/);

const enemy = game.units.find(unit => unit.side === 'enemy' && unit.type === 'commander');
assert.match(markup(enemy), /Recruits allies from its keep/);
assert.doesNotMatch(markup(enemy), /data-recruit=/);
const soldier = game.units.find(unit => unit.type === 'soldier');
assert.ok(markup(soldier).includes(TYPES.soldier.description));
assert.doesNotMatch(markup(soldier), /data-recruit=/);
console.log('PASS: recruitment lives in commander ability, with ownership and availability respected');
