import { CAMPAIGNS } from "./campaign-data";
import type { CampaignId, CampaignState, RecruitType } from "./types";
import { MAPS } from "./maps";
import { Game, TYPES } from "./engine";
import type { Side, Unit, UnitType, VillageState } from "./types";

export const SAVE_KEY = "hex-command-battle";
export const SAVE_VERSION = 1;
export interface LoadedBattle {
  game: Game;
  pendingEnemy: boolean;
}

export function saveBattle(game: Game, pendingEnemy = false): void {
  try {
    localStorage.setItem(
      SAVE_KEY,
      JSON.stringify({
        version: SAVE_VERSION,
        pendingEnemy,
        state: { ...game.snapshot(), nextId: game.nextId, log: game.log },
      }),
    );
  } catch {
    // Storage may be unavailable or full. The battle remains playable.
  }
}

const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;
const side = (value: unknown): value is Side => value === "player" || value === "enemy";
const integer = (value: unknown): value is number =>
  typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
const unitType = (value: unknown): value is UnitType =>
  typeof value === "string" && Object.hasOwn(TYPES, value);

function validUnit(
  value: unknown,
  game: Game,
  nextId: number,
  ids: Set<string>,
  occupied: Set<string>,
): value is Unit {
  if (
    !record(value) ||
    typeof value.id !== "string" ||
    !/^u[1-9]\d*$/.test(value.id) ||
    Number(value.id.slice(1)) >= nextId ||
    ids.has(value.id) ||
    !unitType(value.type) ||
    !side(value.side) ||
    typeof value.tile !== "string" ||
    !game.tile(value.tile) ||
    game.tile(value.tile)!.terrain === "water" ||
    occupied.has(value.tile) ||
    !integer(value.hp) ||
    value.hp < 1 ||
    value.hp > TYPES[value.type].hp ||
    (value.name !== undefined && (typeof value.name !== "string" || value.name.length > 80)) ||
    (value.role !== undefined && value.role !== "leader" && value.role !== "courier") ||
    (value.role === "leader" && !TYPES[value.type].leader) ||
    typeof value.moved !== "boolean" ||
    typeof value.attacked !== "boolean" ||
    (value.retreating !== undefined &&
      (typeof value.retreating !== "boolean" || value.side !== "enemy"))
  )
    return false;
  return true;
}
function validVillage(value: unknown, villages: VillageState[]): value is VillageState {
  return (
    record(value) &&
    typeof value.id === "string" &&
    villages.some((tile) => tile.id === value.id) &&
    (value.owner === null || side(value.owner))
  );
}

function campaignState(value: unknown, mapId: string, winner: Side | null): CampaignState | null {
  if (
    !record(value) ||
    typeof value.id !== "string" ||
    !Object.hasOwn(CAMPAIGNS, value.id) ||
    !integer(value.mission)
  )
    return null;
  const definition = CAMPAIGNS[value.id as CampaignId];
  const mission = definition.missions[value.mission];
  if (
    !mission ||
    mission.map.id !== mapId ||
    !Array.isArray(value.recall) ||
    value.recall.length > 4 ||
    !value.recall.every((type) => unitType(type) && !TYPES[type].leader) ||
    !integer(value.carryGold) ||
    value.carryGold > 8
  )
    return null;
  const allowedMarks = new Set([
    ...(mission.refuge ? ["held:5,4"] : []),
    ...mission.events.map((event) => `event:${event.id}`),
    ...mission.waves.map((_, index) => `wave:${index}`),
    ...mission.goals.flatMap((goal) =>
      goal.kind === "seal"
        ? goal.tiles.map((tile) => `sealed:${tile}`)
        : goal.kind === "hold"
          ? [`held:${goal.tile}`]
          : [],
    ),
  ]);
  if (
    !Array.isArray(value.marks) ||
    new Set(value.marks).size !== value.marks.length ||
    !value.marks.every((mark) => typeof mark === "string" && allowedMarks.has(mark))
  )
    return null;
  if (
    value.refuge !== undefined &&
    (!mission.refuge ||
      !record(value.refuge) ||
      !integer(value.refuge.heldTurns) ||
      value.refuge.heldTurns > 6 ||
      !integer(value.refuge.lastCountedRound) ||
      typeof value.refuge.escalated !== "boolean")
  )
    return null;
  if (
    value.choices !== undefined &&
    (!Array.isArray(value.choices) ||
      value.choices.length > 1 ||
      value.id !== "ashen-oath" ||
      value.mission < 1 ||
      !value.choices.every((choice) => choice === "refuge-peace" || choice === "refuge-war"))
  )
    return null;
  const failure = [
    "Your leader fell. The company cannot continue without them.",
    "The courier fell before reaching safety.",
  ];
  const knownLines = [
    ...mission.intro,
    ...mission.events.flatMap((event) => event.dialogue),
    ...mission.events.flatMap((event) => event.warDialogue ?? []),
    ...mission.victory,
    ...(mission.refuge?.escalation ?? []),
    ...(mission.refuge?.warVictory ?? []),
    ...(mission.withoutDain?.intro ?? []),
    ...(mission.legacyDialogue ?? []),
    ...definition.ending,
    ...failure.map((text) => ({ speaker: "The company", portrait: definition.leader, text })),
  ];
  if (
    !Array.isArray(value.dialogue) ||
    value.dialogue.length > knownLines.length ||
    !value.dialogue.every(
      (line) =>
        record(line) &&
        knownLines.some(
          (known) =>
            known.speaker === line.speaker &&
            known.portrait === line.portrait &&
            known.text === line.text,
        ),
    ) ||
    !integer(value.dialogueIndex) ||
    value.dialogueIndex > value.dialogue.length ||
    typeof value.completed !== "boolean" ||
    value.completed !== (winner === "player" && value.mission === definition.missions.length - 1)
  )
    return null;
  if (
    winner === null
      ? value.outcome !== null
      : winner === "player"
        ? value.outcome !== "Mission complete. The company can rest and regroup."
        : !failure.includes(String(value.outcome))
  )
    return null;
  return {
    id: value.id as CampaignId,
    mission: value.mission,
    recall: [...value.recall] as RecruitType[],
    carryGold: value.carryGold,
    marks: [...value.marks] as string[],
    dialogue: value.dialogue.map((line) => ({
      speaker: String(line.speaker),
      portrait: line.portrait as CampaignState["dialogue"][number]["portrait"],
      text: String(line.text),
    })),
    dialogueIndex: value.dialogueIndex,
    outcome: value.outcome as string | null,
    completed: value.completed,
    ...(value.refuge
      ? { refuge: { ...(value.refuge as NonNullable<CampaignState["refuge"]>) } }
      : {}),
    ...(value.choices
      ? { choices: [...(value.choices as NonNullable<CampaignState["choices"]>)] }
      : {}),
  };
}

export function loadBattle(): LoadedBattle {
  const fresh = (): LoadedBattle => ({ game: new Game(), pendingEnemy: false });
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(SAVE_KEY) ?? "null");
    if (!record(saved) || saved.version !== SAVE_VERSION) return fresh();
    const s = saved.state;
    if (
      !record(s) ||
      (s.mapId !== undefined && (typeof s.mapId !== "string" || !Object.hasOwn(MAPS, s.mapId)))
    )
      return fresh();
    // The constructor rejects unsupported revisions, including malformed persisted values.
    if (s.mapRevision !== undefined && s.mapRevision !== null && typeof s.mapRevision !== "number")
      return fresh();
    const game = new Game(s.mapId as string | undefined, s.mapRevision ?? 1);
    if (
      !integer(s.round) ||
      s.round < 1 ||
      !side(s.turn) ||
      !(s.winner === null || side(s.winner)) ||
      !(s.turn === "player" || s.winner) ||
      typeof saved.pendingEnemy !== "boolean" ||
      (saved.pendingEnemy && (s.turn !== "player" || s.winner)) ||
      !record(s.gold) ||
      !integer(s.gold.player) ||
      !integer(s.gold.enemy) ||
      !integer(s.nextId) ||
      s.nextId < 1 ||
      !Array.isArray(s.units) ||
      !Array.isArray(s.villages) ||
      !Array.isArray(s.log) ||
      s.log.length > 8 ||
      !s.log.every(
        (line: unknown): line is string => typeof line === "string" && line.length <= 1000,
      )
    )
      return fresh();
    const isCampaign = game.mapId !== "mirror" && game.mapId !== "river";
    const campaign = isCampaign ? campaignState(s.campaign, game.mapId, s.winner) : null;
    if ((isCampaign && !campaign) || (!isCampaign && s.campaign !== undefined)) return fresh();
    if (campaign) game.campaign = campaign;
    const ids = new Set<string>(),
      occupied = new Set<string>();
    const units: Unit[] = [];
    for (const unit of s.units as unknown[]) {
      if (!validUnit(unit, game, s.nextId, ids, occupied)) return fresh();
      ids.add(unit.id);
      occupied.add(unit.tile);
      units.push({ ...unit });
    }
    const villages = game.tiles.filter((tile) => tile.terrain === "village");
    if (
      s.villages.length !== villages.length ||
      new Set((s.villages as unknown[]).map((v) => (record(v) ? v.id : undefined))).size !==
        villages.length
    )
      return fresh();
    for (const village of s.villages as unknown[]) {
      if (!validVillage(village, villages)) return fresh();
      game.tile(village.id)!.owner = village.owner;
    }
    // Move the refuge's two villages without discarding a player's ongoing company.
    if (game.mapId === "ashen-refuge" && game.mapRevision === 1) {
      for (const row of [1, 7]) {
        const oldTile = game.tile(`5,${row}`)!;
        const newTile = game.tile(`3,${row}`)!;
        newTile.terrain = "village";
        newTile.owner = units.find((unit) => unit.tile === newTile.id)?.side ?? oldTile.owner;
        oldTile.terrain = "plains";
        oldTile.owner = null;
      }
      game.mapRevision = 2;
    }
    const sites = game.tiles.filter((tile) => ["grove", "ruins", "fort"].includes(tile.terrain));
    if (campaign) {
      if (
        !Array.isArray(s.sites) ||
        s.sites.length !== sites.length ||
        new Set(s.sites.map((v) => (record(v) ? v.id : undefined))).size !== sites.length
      )
        return fresh();
      for (const site of s.sites as unknown[]) {
        if (!validVillage(site, sites)) return fresh();
        game.tile(site.id)!.owner = site.owner;
      }
      for (const team of ["player", "enemy"] as const) {
        if (units.filter((unit) => unit.side === team && unit.role === "leader").length > 1)
          return fresh();
      }
      if (!s.winner && !units.some((unit) => unit.side === "player" && unit.role === "leader"))
        return fresh();
      if (
        !s.winner &&
        CAMPAIGNS[campaign.id].missions[campaign.mission]!.goals.some(
          (goal) => goal.kind === "escort",
        ) &&
        !units.some((unit) => unit.side === "player" && unit.role === "courier")
      )
        return fresh();
    }
    if (
      !campaign &&
      !s.winner &&
      ["player", "enemy"].some(
        (team) => !units.some((unit) => unit.side === team && unit.type === "commander"),
      )
    )
      return fresh();
    Object.assign(game, {
      round: s.round,
      turn: s.turn,
      winner: s.winner,
      gold: { ...s.gold },
      units,
      nextId: s.nextId,
      log: [...s.log],
    });
    return { game, pendingEnemy: saved.pendingEnemy };
  } catch {
    return fresh();
  }
}
