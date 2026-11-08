// Genuine frozen browser-era metadata. Historical tests must not stamp an old
// version onto today's authored rosters, dialogue or geometry.
import frozen from "../packages/save-migrations/src/legacy/data/catalog-v6.json";
import stages from "../packages/save-migrations/src/legacy/data/stages.json";
export { frozen };
export function historicalState(id, chapter) {
  const campaign = frozen.campaigns.find((campaign) => campaign.id === id);
  const mission = campaign.missions[chapter],
    map = frozen.maps[mission.map.id];
  const cells = new Map(map.terrain.map((cell) => [`${cell[0]},${cell[1]}`, cell]));
  const units = mission.map.units.map((incoming, index) => {
    const { col, row, ...unit } = incoming;
    return {
      ...unit,
      id: `u${index + 1}`,
      tile: `${col},${row}`,
      hp: frozen.units[unit.type].hp,
      experience: 0,
      moved: false,
      attacked: false,
    };
  });
  return {
    combatRng: 17,
    mapId: map.id,
    mapRevision: map.revision,
    cols: map.cols,
    rows: map.rows,
    round: 1,
    turn: "player",
    winner: null,
    gold: { player: mission.gold, enemy: mission.enemyGold },
    nextId: units.length + 1,
    log: [],
    units,
    tiles: Array.from({ length: map.cols * map.rows }, (_, index) => ({
      id: `${index % map.cols},${Math.floor(index / map.cols)}`,
      col: index % map.cols,
      row: Math.floor(index / map.cols),
      terrain: "plains",
      owner: null,
    })).map((tile) => {
      const cell = cells.get(tile.id);
      return cell
        ? {
            ...tile,
            terrain: cell[2],
            owner: cell[3],
            ...(cell[4] === null ? {} : { floor: cell[4] }),
            ...(cell[5] === null ? {} : { connections: cell[5] }),
            ...(cell[6] === null ? {} : { deadEnds: cell[6] }),
          }
        : tile;
    }),
    campaign: {
      id,
      mission: chapter,
      recall: [],
      carryGold: 0,
      completed: false,
      outcome: null,
      marks: [],
      choices: [],
      dialogue: structuredClone(stages[map.id].dialogue),
      dialogueIndex: 0,
    },
  };
}
export const historicalBattle = (state) => ({ captureState: () => structuredClone(state) });
