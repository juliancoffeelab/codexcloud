const fs = require('../../../../scripts/pixel-fs.cjs');
const path = require('node:path');
const { execFileSync } = require('node:child_process');
const sourceDirectory = require('../../../../scripts/pixel-source-path.cjs');
const root = __dirname;
const project = path.resolve(root, '../../../..');
const entries = [];

function create(id, label, base, robe, edit) {
  const source = path.join(project, 'art/pixel/units', base);
  const original = fs.readFileSync(path.join(source, 'player.txt'), 'utf8').trimEnd().split('\n');
  const rows = original.map(row => [...row]);
  const palette = JSON.parse(fs.readFileSync(path.join(source, 'palette.json')));
  Object.assign(palette, {
    C: [...robe[0], 255], c: [...robe[1], 255], d: [...robe[2], 255],
    F: [0, 89, 103, 255], f: [0, 62, 73, 255], Z: [6, 38, 42, 255],
    R: [255, 66, 72, 255], r: [153, 20, 40, 255], Q: [60, 7, 19, 255],
    G: [250, 194, 57, 255], g: [174, 112, 28, 255],
    X: [219, 251, 255, 255], x: [84, 188, 224, 255],
    E: [155, 221, 141, 255], e: [66, 125, 81, 255],
  });
  for (let y = 0; y < 48; y++) for (let x = 0; x < 48; x++) {
    const symbol = original[y][x];
    // Hood and trailing cloak keep faction color. The scarf and central sash
    // belong to the body scheme, along with the leather belts and sleeve strips.
    const scarf = y >= 24 && y <= 27 && x >= 24 && x <= 33;
    const sash = y >= 32 && y <= 43 && x >= 24 && x <= 30;
    if ('Tto'.includes(symbol)) rows[y][x] = (scarf || sash)
      ? { T: 'C', t: 'c', o: 'd' }[symbol]
      : { T: 'F', t: 'f', o: 'Z' }[symbol];
    if (symbol === 'I' || symbol === 'i') rows[y][x] = symbol === 'I' ? 'C' : 'c';
    const boot = y >= 43 || (y >= 41 && x >= 28);
    if (!boot && (symbol === 'L' || symbol === 'l')) rows[y][x] = symbol === 'L' ? 'C' : 'c';
  }
  const set = (x, y, value) => { rows[y][x] = value; };
  const lines = strokes => strokes.forEach(([x, y, line]) => [...line].forEach((s, n) => set(x + n, y, s)));
  const clearHead = () => {
    for (let y = 14; y <= 23; y++) for (let x = 35; x <= 43; x++) set(x, y, '.');
  };
  edit({ rows, palette, set, lines, clearHead });
  const dir = sourceDirectory('human-magic/' + id);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, 'palette.json'), JSON.stringify(palette, null, 2) + '\n');
  const changes = [];
  for (let y = 0; y < 48; y++) for (let x = 0; x < 48; x++) {
    if (rows[y][x] !== original[y][x]) changes.push({ x, y, from: original[y][x], to: rows[y][x] });
  }
  fs.writeFileSync(path.join(dir, 'changes.json'), JSON.stringify({ base, changes }, null, 2) + '\n');
  for (const side of ['player', 'enemy']) {
    const matrix = rows.map(row => row.map(s => side === 'enemy' ? ({ F: 'R', f: 'r', Z: 'Q' }[s] || s) : s).join('')).join('\n') + '\n';
    const file = path.join(dir, side + '.txt');
    fs.writeFileSync(file, matrix);
    execFileSync('python3', ['scripts/render-pixel.py', file], { cwd: project, stdio: 'inherit' });
    entries.push({ label: `${label} — ${side === 'player' ? 'Teal' : 'Red'}`, image: path.relative(project, path.join(dir, side + '.png')) });
  }
}

const white = [[245, 244, 230], [193, 199, 183], [100, 119, 110]];
create('healer-l2', 'Healer L2', 'adept', white, ({ lines }) => {
  lines([[38, 20, 'EEe'], [38, 21, 'eee']]);
});
create('healer-l3', 'High healer L3', 'adept', white, ({ rows, set, lines, clearHead }) => {
  // Broad stole follows the existing shoulders and the outside of the sleeves.
  for (let y = 25; y <= 36; y++) for (let x = 17; x <= 35; x++) {
    const left = (y <= 28 && x >= 19 && x <= 23) || (y >= 28 && x >= 22 && x <= 24);
    const right = y >= 27 && y <= 31 && x >= 32 && x <= 34;
    if ((left || right) && 'CcLl'.includes(rows[y][x])) set(x, y, x === 22 || x === 34 ? 'g' : 'G');
  }
  clearHead();
  lines([
    [39, 15, 'G'], [38, 16, 'GGG'], [37, 17, 'gGIGg'],
    [36, 18, 'gGIIIGg'], [35, 19, 'gGGIIIGGg'],
    [36, 20, 'gGIIIGg'], [37, 21, 'gGIGg'], [38, 22, 'gGg'], [39, 23, 'G'],
  ]);
});
create('storm-mage-l2', 'Storm mage L2', 'adept', [[31, 66, 111], [20, 42, 76], [13, 26, 47]], ({ clearHead, lines }) => {
  clearHead();
  lines([
    [40, 14, 'X'], [39, 15, 'XX'], [38, 16, 'XXX'], [37, 17, 'XXXX'],
    [39, 18, 'Xx'], [38, 19, 'Xx'], [37, 20, 'Xx'],
    [37, 21, 'gGGGg'], [38, 22, 'gGg'], [39, 23, 'G'],
  ]);
});
create('archmage-l3', 'Archmage L3', 'mage', [[180, 28, 46], [117, 17, 34], [64, 15, 26]], ({ rows, set, clearHead, lines }) => {
  for (let y = 25; y <= 29; y++) for (let x = 18; x <= 35; x++) {
    if ('Cc'.includes(rows[y][x]) && ((x >= 19 && x <= 23) || x >= 32)) set(x, y, x === 19 || x === 35 ? 'g' : 'G');
  }
  clearHead();
  lines([
    [39, 14, 'X'], [38, 15, 'XXX'], [37, 16, 'xXXXx'],
    [37, 17, 'xXXXx'], [37, 18, 'xxXxx'], [38, 19, 'xxx'],
    [36, 20, 'gGxxxGg'], [37, 21, 'gGGGg'], [38, 22, 'gGg'], [39, 23, 'G'],
  ]);
});
fs.writeFileSync(path.join(root, 'lineup.json'), JSON.stringify(entries, null, 2) + '\n');
execFileSync('node', [path.join(project, 'art/pixel/reference/human-anatomies/preview.cjs'), path.join(root, 'lineup.json'), path.join(root, 'preview.png'), '2', '8'], { cwd: project, stdio: 'inherit' });
