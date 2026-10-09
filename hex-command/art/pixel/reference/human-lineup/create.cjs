const fs = require('../../../../scripts/pixel-fs.cjs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const sourceDirectory = require('../../../../scripts/pixel-source-path.cjs');
const root = __dirname, project = path.resolve(root, '../../../..');
const unit = (id) => `art/pixel/units/${id}/player.png`;
const ref = (group, id) => path.relative(project, path.join(sourceDirectory(`${group}/${id}`), 'player.png'));
const cell = (label, image) => ({ label, image });
const sword = cell('Swordsman — L1', ref('infantry-progression', 'leather-swordsman'));
const spear = cell('Spearman — L1', ref('infantry-progression', 'leather-spearman'));
const scout = cell('Scout — L1', unit('scout'));
const archer = cell('Archer — L1', unit('archer'));
const adept = cell('Adept — L1', unit('adept'));
const sections = {
  infantry: [
    [sword, cell('Sword infantry — L2', unit('swordsman')), cell('Champion — L3', ref('human-level3', 'champion'))],
    [sword, cell('Hammerman — L2', ref('human-anatomies', 'hammerman-l2')), cell('Warhammer — L3', ref('human-anatomies', 'warhammer-l3'))],
    [spear, cell('Spear infantry — L2', unit('spearman')), cell('Sentinel — L3', ref('human-level3', 'sentinel'))],
    [spear, cell('Halberdier — L2', ref('human-anatomies', 'halberdier-l2')), cell('Halberd master — L3', ref('human-anatomies', 'halberdmaster-l3'))],
  ],
  ranged: [
    [scout, cell('Ranger — L2', unit('ranger')), cell('Shadowblade — L3', ref('scout-level3', 'shadowblade'))],
    [scout, cell('Slinger — L2', 'art/pixel/units/slinger/player.png'), cell('Fireslinger — L3', ref('scout-level3', 'fireslinger'))],
    [archer, cell('Marksman — L2', unit('marksman')), cell('Sharpshooter — L3', ref('human-level3', 'sharpshooter'))],
    [archer, cell('Crossbowman — L2', ref('human-anatomies', 'crossbowman-l2')), cell('Arbalester — L3', ref('human-anatomies', 'arbalester-l3'))],
  ],
  magic: [
    [adept, cell('Storm mage — L2', ref('human-magic', 'storm-mage-l2')), cell('Archmage — L3', ref('human-magic', 'archmage-l3'))],
    [adept, cell('Healer — L2', ref('human-magic', 'healer-l2')), cell('High healer — L3', ref('human-magic', 'healer-l3'))],
  ],
  cavalry: [
    [cell('Rider — L1', ref('human-anatomies', 'lancer-l1')), cell('Lancer — L2', ref('human-anatomies', 'lancer-l2')), cell('Knight — L3', ref('human-anatomies', 'lancer-l3'))],
    [cell('Horse archer — L1', ref('human-anatomies', 'horse-archer-l1')), cell('Horse bowman — L2', ref('human-anatomies', 'horse-archer-l2')), cell('Horse marksman — L3', ref('human-anatomies', 'horse-archer-l3'))],
  ],
  existing: [
    [cell('Commander — L1', unit('commander')), cell('Marshal — L2', unit('marshal')), cell('Grand Marshal — L3', ref('human-level3', 'grand-marshal'))],
    [cell('Game: Heavy infantry', unit('soldier')), cell('Game: Warden', unit('warden')), cell('Game: Stone mage', unit('mage'))],
  ],
};
function render(name, entries, scale) {
  const file = path.join(root, name + '.json');
  fs.writeFileSync(file, JSON.stringify(entries, null, 2) + '\n');
  execFileSync('node', [path.join(project, 'art/pixel/reference/human-anatomies/preview.cjs'), file, path.join(root, name + '.png'), '3', String(scale)], { cwd: project, stdio: 'inherit' });
}
function redCell(entry) {
  if (!entry.image) return entry;
  const source = path.join(project,entry.image);
  const dir = path.dirname(source), enemy = path.join(dir,'enemy.png');
  if(!fs.existsSync(enemy))throw new Error(`Missing red faction image: ${enemy}`);
  const image=path.relative(project,enemy);
  return {...entry,image};
}
for (const [name, rows] of Object.entries(sections)) {
  render(name,rows.flat(),6);
  render(name+'-red',rows.flat().map(redCell),6);
}
const full=Object.values(sections).flat(2);
render('full',full,6);
render('full-red',full.map(redCell),6);
