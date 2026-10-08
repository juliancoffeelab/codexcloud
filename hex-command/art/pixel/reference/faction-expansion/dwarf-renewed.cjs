// Material ownership for the four regenerated dwarf anatomies, in full 64-slot pixels.
// Helmets and armor describe these exact poses; no inherited apron rectangles apply.
const poses = {
  "dwarf-crossbow": {
    helmet: [20, 19, 33, 23],
    beard: [[22, 27, 34, 30]],
    leather: [[21, 39, 34, 47]],
    wood: [
      [32, 32, 48, 33],
      [29, 30, 31, 31],
    ],
    metal: [
      [47, 26, 53, 28],
      [47, 34, 53, 38],
    ],
    shoulders: [[13, 30, 18, 33]],
    chest: { 39: [24, 33], 40: [23, 33], 41: [24, 34], 42: [22, 34], 43: [22, 34] },
    belt: [22, 44, 34, 44],
  },
  "dwarf-pickaxe": {
    helmet: [24, 20, 34, 24],
    beard: [[26, 29, 35, 33]],
    leather: [[25, 40, 35, 48]],
    wood: [
      [42, 24, 45, 29],
      [29, 36, 32, 38],
    ],
    metal: [
      [46, 17, 54, 21],
      [51, 23, 56, 33],
    ],
    shoulders: [[16, 31, 21, 34]],
    chest: { 39: [31, 34], 40: [30, 35], 41: [31, 34], 42: [30, 34], 43: [29, 34], 44: [26, 34] },
    belt: [26, 45, 34, 45],
  },
  "dwarf-pistol-goat-rider": {
    helmet: [19, 10, 31, 14],
    beard: [[23, 17, 29, 22]],
    leather: [
      [21, 27, 29, 33],
      [23, 35, 30, 40],
    ],
    wood: [[44, 13, 51, 14]],
    metal: [[42, 11, 53, 12]],
    fur: [
      [10, 33, 21, 53],
      [31, 33, 42, 52],
      [45, 28, 54, 51],
    ],
    shoulders: [[18, 19, 21, 21]],
    chest: { 27: [21, 25], 28: [21, 26], 29: [21, 26], 30: [21, 26] },
    belt: [22, 31, 26, 31],
  },
  "dwarf-axe-pick": {
    helmet: [26, 17, 38, 22],
    beard: [[27, 27, 36, 33]],
    leather: [[27, 37, 37, 45]],
    wood: [
      [12, 23, 16, 28],
      [47, 26, 50, 30],
    ],
    metal: [
      [7, 18, 15, 23],
      [46, 17, 53, 20],
      [54, 23, 57, 27],
    ],
    shoulders: [
      [21, 29, 24, 32],
      [39, 29, 42, 32],
    ],
    chest: { 37: [28, 35], 38: [28, 36], 39: [28, 36], 40: [27, 37], 41: [27, 37], 42: [27, 37] },
    belt: [27, 43, 37, 43],
  },
};
const materialReason =
  "Connected material plane on the regenerated dwarf pose; outline, grip and silhouette retained";
function editRows(rows) {
  const changes = [];
  const set = (x, y, to, reason = materialReason) => {
    const from = rows[y]?.[x];
    if (!from || from === to || from === "." || from === "O") return;
    rows[y][x] = to;
    changes.push({ x, y, from, to, reason });
  };
  const map = ([x0, y0, x1, y1], mapping, reason) => {
    for (let y = y0; y <= y1; y++)
      for (let x = x0; x <= x1; x++)
        if (mapping[rows[y]?.[x]]) set(x, y, mapping[rows[y][x]], reason);
  };
  return { changes, set, map };
}
function clean(id, rows, palette) {
  const pose = poses[id];
  const { changes, set, map } = editRows(rows);
  for (const a of pose.beard)
    map(
      a,
      { W: "B", w: "b", V: "B", v: "b", I: "B", i: "b" },
      "Warm contiguous beard hair below the face",
    );
  for (const a of pose.leather)
    map(
      a,
      { W: "B", w: "b", V: "B", v: "b" },
      "Leather apron and trousers stay warm, separate from tier steel",
    );
  for (const a of pose.wood)
    map(
      a,
      { B: "W", b: "w", V: "W", v: "w", I: "W", i: "w" },
      "Wooden stock or shaft has a coherent warm material ramp",
    );
  for (const a of pose.metal)
    map(
      a,
      { B: "a", b: "a", V: "a", v: "a", W: "a", w: "a", I: "A", i: "a" },
      "Steel weapon surface has no leather, cloth or ivory extraction flecks",
    );
  if (pose.fur) {
    palette.J = [209, 190, 150, 255];
    palette.j = [151, 126, 88, 255];
    for (const a of pose.fur)
      map(
        a,
        { H: "I", h: "i", V: "J", v: "j" },
        "Goat coat stays warm ivory and natural shadows, separate from rider skin and tier armor",
      );
  }
  const [x0, y0, x1, y1] = pose.helmet;
  const steel = [];
  for (let y = y0; y <= y1; y++)
    for (let x = x0; x <= x1; x++) if ("AaBbvVwWiIE".includes(rows[y][x])) steel.push([x, y]);
  const center = steel.reduce((sum, [x]) => sum + x, 0) / steel.length;
  for (const [x, y] of steel) {
    let color = x < center - 2 || y === y1 ? "a" : "M";
    if (x >= center + 0.5 && x <= center + 3 && y <= y0 + 2) color = "A";
    set(
      x,
      y,
      color,
      "Broad helmet midtone, connected upper-right highlight and left/bottom shadow",
    );
  }
  Object.assign(palette, {
    A: [199, 214, 221, 255],
    a: [88, 108, 121, 255],
    M: [139, 160, 173, 255],
    P: [151, 173, 188, 255],
    p: [84, 111, 128, 255],
    S: [210, 226, 236, 255],
  });
  return changes;
}
function upgrade({ job, level, rows, palette }) {
  const pose = poses[job.id];
  const { changes, set, map } = editRows(rows);
  // The generic process's tier recolors must not turn wood, mount coats or aprons into steel.
  Object.assign(palette, { V: [148, 106, 57, 255], v: [88, 61, 36, 255] });
  if (level === 1) return changes;
  const veteran = level === 3;
  Object.assign(palette, {
    A: veteran ? [219, 232, 239, 255] : [193, 211, 221, 255],
    a: veteran ? [109, 139, 154, 255] : [91, 119, 136, 255],
    M: veteran ? [160, 186, 200, 255] : [139, 160, 173, 255],
    P: veteran ? [177, 199, 211, 255] : [151, 173, 188, 255],
    p: veteran ? [99, 126, 143, 255] : [84, 111, 128, 255],
    S: veteran ? [229, 241, 246, 255] : [210, 226, 236, 255],
    G: veteran ? [249, 197, 58, 255] : [238, 184, 52, 255],
  });
  for (const a of pose.shoulders)
    map(a, { I: "P", i: "p" }, "Small shoulder plate above the intact cream work sleeve");
  const top = Math.min(...Object.keys(pose.chest).map(Number));
  for (const [yy, [left, right]] of Object.entries(pose.chest)) {
    const y = Number(yy);
    for (let x = left; x <= right; x++) {
      if (!"BbVvWwAaIi".includes(rows[y][x])) continue;
      let c = x >= right - 1 ? "p" : "P";
      if (x === left && y <= top + 2) c = "S";
      set(
        x,
        y,
        c,
        "Fitted continuous cuirass above waist, preserving the beard, hands and diagonal shaft",
      );
    }
  }
  const [left, waist, right] = pose.belt;
  for (let x = left; x <= right; x++)
    if ("BbVvWwAa".includes(rows[waist][x]))
      set(x, waist, "b", "Dark leather waist belt; apron and trousers below remain leather");
  const buckle = Math.floor((left + right) / 2);
  if ("Bb".includes(rows[waist][buckle]))
    set(
      buckle,
      waist,
      veteran ? "G" : "a",
      "Small buckle at actual waist instead of a floating stripe",
    );
  return changes;
}
module.exports = { owns: (id) => Object.hasOwn(poses, id), clean, upgrade, poses };
