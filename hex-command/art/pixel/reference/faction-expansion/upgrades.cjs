// Local equipment masks for these exact saved poses, in 64×64 slot coordinates.
// Clothing recolors are supplied by process.cjs; this module adds real armor.
const heads = {
  "skeleton-flanker": [37, 20, 43, 27],
  "skeleton-impact-caster": [32, 17, 37, 23],
  "skeleton-scythe-caster": [32, 18, 37, 23],
  "skeleton-axeman": [28, 16, 34, 23],
  "skeleton-rider": [27, 10, 31, 16],
};
const plates = {
  "skeleton-flanker": [
    [18, 26, 23, 29],
    [29, 29, 36, 33],
    [15, 43, 19, 46],
    [41, 43, 45, 47],
  ],
  "skeleton-impact-caster": [
    [21, 25, 26, 28],
    [36, 25, 41, 28],
  ],
  "skeleton-scythe-caster": [
    [20, 27, 26, 30],
    [37, 27, 42, 30],
  ],
  "skeleton-axeman": [
    [19, 25, 23, 28],
    [29, 27, 35, 31],
    [17, 45, 20, 48],
    [41, 45, 44, 48],
  ],
  "skeleton-rider": [
    [26, 18, 31, 22],
    [28, 31, 34, 36],
  ],
  "dwarf-axeguard": [
    [35, 30, 41, 35],
    [28, 39, 34, 46],
    [34, 43, 40, 46],
  ],
  "dwarf-hammerer": [
    [16, 30, 22, 34],
    [28, 40, 34, 47],
  ],
  "dwarf-pickaxe": [
    [18, 30, 24, 34],
    [28, 39, 34, 46],
  ],
  "dwarf-poleaxe": [
    [17, 28, 23, 33],
    [28, 41, 34, 47],
  ],
  "dwarf-crossbow": [
    [14, 29, 20, 34],
    [28, 38, 34, 46],
  ],
  "dwarf-arquebus": [
    [14, 28, 21, 34],
    [27, 39, 34, 46],
  ],
  "dwarf-runesmith": [
    [22, 27, 26, 34],
    [29, 38, 37, 45],
  ],
  "dwarf-goat-rider": [
    [16, 20, 22, 23],
    [21, 28, 26, 32],
  ],
  "elf-spearman": [
    [24, 23, 28, 27],
    [34, 23, 37, 27],
  ],
  "elf-bladedancer": [
    [24, 23, 28, 27],
    [33, 23, 36, 27],
  ],
  "elf-archer": [[28, 26, 32, 28]],
  "elf-flanker": [
    [20, 26, 24, 29],
    [34, 29, 38, 31],
  ],
  "elf-caster": [[30, 23, 34, 25]],
  "elf-deer-archer": [
    [27, 20, 31, 23],
    [30, 32, 34, 36],
  ],
  "fox-spearguard": [
    [28, 39, 31, 46],
    [36, 39, 39, 46],
  ],
};
const belts = {
  "skeleton-flanker": [31, 36, 35, 36],
  "skeleton-impact-caster": [30, 34, 34, 35],
  "skeleton-scythe-caster": [30, 36, 34, 36],
  "skeleton-axeman": [31, 35, 35, 35],
  "skeleton-rider": [29, 26, 32, 26],
  "dwarf-axeguard": [28, 41, 33, 41],
  "dwarf-hammerer": [29, 44, 34, 44],
  "dwarf-pickaxe": [29, 43, 34, 43],
  "dwarf-poleaxe": [29, 45, 34, 45],
  "dwarf-crossbow": [28, 40, 34, 40],
  "dwarf-arquebus": [28, 43, 34, 43],
  "dwarf-runesmith": [30, 42, 36, 42],
  "dwarf-goat-rider": [22, 28, 26, 28],
  "elf-spearman": [27, 29, 34, 29],
  "elf-bladedancer": [27, 30, 32, 30],
  "elf-archer": [29, 35, 34, 35],
  "elf-flanker": [26, 35, 32, 35],
  "elf-caster": [30, 36, 35, 36],
  "elf-deer-archer": [29, 26, 33, 26],
  "fox-spearguard": [30, 39, 35, 39],
};
module.exports = function ({ job, level, rows, palette, origin }) {
  const renewed = require("./dwarf-renewed.cjs");
  if (renewed.owns(job.id)) return renewed.upgrade({ job, level, rows, palette, origin });
  const changes = [],
    [ox, oy] = origin;
  function set(gx, gy, to, reason) {
    const x = gx - ox,
      y = gy - oy;
    if (!rows[y] || rows[y][x] === undefined) return;
    const from = rows[y][x];
    if (from === to) return;
    rows[y][x] = to;
    changes.push({ x, y, from, to, reason });
  }
  const get = (x, y) => rows[y - oy]?.[x - ox];
  function region(a, mapping, reason) {
    const [x0, y0, x1, y1] = a;
    for (let y = y0; y <= y1; y++)
      for (let x = x0; x <= x1; x++) if (mapping[get(x, y)]) set(x, y, mapping[get(x, y)], reason);
  }
  if (level === 1) return changes;
  const armor = { I: "A", i: "a", V: "A", v: "a" };
  if (job.race === "dwarf") Object.assign(armor, { B: "A", b: "a", W: "A", w: "a" });
  for (const a of plates[job.id] || [])
    region(a, armor, "Tier shoulder/chest/leg armor on the saved pose");
  if (job.race === "dwarf" && job.id !== "dwarf-goat-rider") {
    const shafts = {
      "dwarf-hammerer": [82.3, 1.35],
      "dwarf-pickaxe": [75.5, 1.14],
      "dwarf-poleaxe": [57.2, 0.85],
    };
    const shaft = shafts[job.id];
    for (let y = 37; y <= 48; y++)
      for (let x = 22; x <= 43; x++) {
        if (shaft && Math.abs(x - (shaft[0] - shaft[1] * y)) <= 2.5) continue;
        const s = get(x, y);
        if (armor[s])
          set(
            x,
            y,
            armor[s],
            "Full fitted iron/silver apron armor; preserve diagonal wooden shaft and grips",
          );
      }
    if (level === 2) {
      palette.A = [174, 194, 203, 255];
      palette.a = [90, 116, 129, 255];
    }
    if (level === 3) {
      const brows = {
        "dwarf-axeguard": [25, 25, 35, 25],
        "dwarf-hammerer": [24, 25, 34, 25],
        "dwarf-pickaxe": [25, 25, 35, 25],
        "dwarf-poleaxe": [24, 24, 35, 24],
        "dwarf-crossbow": [25, 22, 35, 22],
        "dwarf-arquebus": [24, 23, 35, 23],
        "dwarf-runesmith": [26, 25, 37, 25],
      };
      region(brows[job.id], { A: "G", a: "G" }, "Gold brow rim on the veteran helmet");
    }
  }
  if (job.race === "merfolk") {
    // Coral chest straps and the guardian's shield become silver; gold remains trim.
    for (let y = 0; y < rows.length; y++)
      for (let x = 0; x < rows[y].length; x++) {
        if (rows[y][x] === "C" || rows[y][x] === "c")
          set(
            x + ox,
            y + oy,
            rows[y][x] === "C" ? "A" : "a",
            "Silver coral equipment on advanced merfolk",
          );
      }
  }
  if (job.race === "skeleton") {
    const h = heads[job.id];
    if (h) {
      const [x0, y0, x1, y1] = h;
      region([x0, y0, x1, y0 + 1], { I: "A", i: "a" }, "Connected skull helmet band");
      if (level === 3) {
        region(
          [x0, y0 + 2, x0 + 1, y1],
          { I: "A", i: "a" },
          "Near face-guard plate; eye sockets stay open",
        );
        region(
          [x1 - 1, y0 + 2, x1, y1],
          { I: "a", i: "a" },
          "Far face-guard plate; eye sockets stay open",
        );
        if (!job.id.includes("caster")) {
          // Three points grow only from connected helmet cells.
          for (const x of [x0 + 1, Math.floor((x0 + x1) / 2), x1 - 1]) {
            if (["A", "a", "O"].includes(get(x, y0))) {
              set(x, y0 - 1, "A", "Connected jagged crown point");
              set(x, y0 - 2, "O", "Crown contour");
            }
          }
        }
      }
    }
  }
  if (level === 3) {
    const belt = belts[job.id];
    if (belt)
      region(
        belt,
        { A: "G", a: "G", B: "G", b: "G", V: "G", v: "G" },
        "Connected gold belt clasp/trim",
      );
    // No global leather conversion: beards, boots, wood and reins remain their material.
    if (job.race === "dwarf") palette.G = [249, 197, 58, 255];
    if (job.race === "skeleton") palette.E = [151, 232, 174, 255];
  }
  return changes;
};
