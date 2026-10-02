import assert from 'node:assert/strict';
import { Game, TYPES } from '../dist/engine.mjs';
import { abilityMarkup } from '../dist/unit-ability.mjs';

const game = new Game();
for (const unit of game.units) {
  const html = abilityMarkup({ game, unit, canAct: true });
  assert.doesNotMatch(html, /data-recruit=|recruit-list|Return to your keep/);
  if (unit.type === 'commander') {
    assert.match(html, /Command/);
    assert.match(html, /Losing this unit ends the battle/);
  } else assert.ok(html.includes(TYPES[unit.type].description));
}
console.log('PASS: commanders show command information; unit abilities contain no purchase controls');
