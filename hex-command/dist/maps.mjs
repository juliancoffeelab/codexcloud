// Only overrides are listed; all other hexes are plains.
const terrain = (kind, positions) => positions.map(([col, row]) => ({ col, row, terrain: kind }));
const starters = (playerKeep, enemyKeep) => [
  { type: "commander", side: "player", col: playerKeep, row: 4 },
  { type: "soldier", side: "player", col: playerKeep + 1, row: 4 },
  { type: "scout", side: "player", col: playerKeep + 1, row: 3 },
  { type: "commander", side: "enemy", col: enemyKeep, row: 4 },
  { type: "soldier", side: "enemy", col: enemyKeep - 1, row: 4 },
  { type: "scout", side: "enemy", col: enemyKeep - 1, row: 3 },
];

export const MAPS = {
  mirror: {
    id: "mirror",
    name: "Classic mirror",
    description: "The original compact battlefield. Four villages and a wooded center.",
    cols: 11,
    rows: 7,
    terrain: [
      ...terrain("forest", [[5, 2], [6, 2], [4, 3], [5, 3], [6, 3], [5, 4], [6, 4]]),
      ...terrain("hill", [[3, 2], [3, 4], [8, 2], [8, 4], [5, 0], [6, 0], [5, 6], [6, 6]]),
      ...terrain("water", [[0, 0], [1, 0], [0, 6], [1, 6], [9, 0], [10, 0], [9, 6], [10, 6]]),
      ...terrain("village", [[3, 1], [3, 5], [7, 1], [7, 5]]),
      { col: 0, row: 3, terrain: "keep", owner: "player" },
      { col: 10, row: 3, terrain: "keep", owner: "enemy" },
    ],
    units: [
      { type: "commander", side: "player", col: 0, row: 3 },
      { type: "soldier", side: "player", col: 1, row: 3 },
      { type: "scout", side: "player", col: 1, row: 2 },
      { type: "commander", side: "enemy", col: 10, row: 3 },
      { type: "soldier", side: "enemy", col: 9, row: 3 },
      { type: "scout", side: "enemy", col: 9, row: 2 },
    ],
  },
  river: {
    id: "river",
    name: "River crossings",
    description: "A larger battlefield split by a river. Fight for three bridges and six villages.",
    cols: 15,
    rows: 9,
    terrain: [
      ...terrain("water", [[7, 0], [7, 2], [7, 3], [7, 5], [7, 6], [7, 8]]),
      ...terrain("bridge", [[7, 1], [7, 4], [7, 7]]),
      ...terrain("village", [[4, 1], [4, 4], [4, 7], [10, 1], [10, 4], [10, 7]]),
      ...terrain("forest", [[2, 1], [3, 2], [5, 3], [2, 6], [3, 6], [5, 8], [12, 1], [11, 2], [9, 3], [12, 6], [11, 6], [9, 8]]),
      ...terrain("hill", [[5, 0], [5, 2], [5, 6], [5, 7], [9, 0], [9, 2], [9, 6], [9, 7]]),
      { col: 1, row: 4, terrain: "keep", owner: "player" },
      { col: 13, row: 4, terrain: "keep", owner: "enemy" },
    ],
    units: starters(1, 13),
  },
};
