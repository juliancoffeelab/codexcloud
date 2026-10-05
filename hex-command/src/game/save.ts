import { CAMPAIGNS, CAMPAIGN_MAPS } from "./campaign-data";
import type { CampaignId, CampaignState, CampaignVeteran, RecruitType } from "./types";
import { MAPS } from "./maps";
import { distance, Game, TYPES } from "./engine";
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

function veteran(value: unknown): value is CampaignVeteran {
  return (
    record(value) &&
    unitType(value.type) &&
    integer(value.experience) &&
    value.experience <= (TYPES[value.type].experienceNeeded ?? 0) &&
    (value.name === undefined || (typeof value.name === "string" && value.name.length <= 80)) &&
    (value.role === undefined ||
      value.role === "courier" ||
      (value.role === "leader" && TYPES[value.type].leader === true))
  );
}

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
    !Number.isFinite(game.cost({ type: value.type }, game.tile(value.tile)!)) ||
    occupied.has(value.tile) ||
    !integer(value.hp) ||
    value.hp < 1 ||
    value.hp > TYPES[value.type].hp ||
    (value.experience !== undefined &&
      (!integer(value.experience) ||
        value.experience > (TYPES[value.type].experienceNeeded ?? 0))) ||
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

function campaignState(
  value: unknown,
  mapId: string,
  winner: Side | null,
  round: number,
): CampaignState | null {
  if (
    !record(value) ||
    typeof value.id !== "string" ||
    !Object.hasOwn(CAMPAIGNS, value.id) ||
    !integer(value.mission)
  )
    return null;
  const definition = CAMPAIGNS[value.id as CampaignId];
  const missionIndex = value.mission;
  const mission = definition.missions[missionIndex];
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
  const previousCompanions = new Set(
    definition.missions
      .slice(0, missionIndex)
      .flatMap((chapter) =>
        chapter.map.units
          .filter((unit) => unit.side === "player" && unit.name && unit.role !== "leader")
          .map((unit) => unit.name!),
      ),
  );
  if (
    value.fallenCompanions !== undefined &&
    (!Array.isArray(value.fallenCompanions) ||
      new Set(value.fallenCompanions).size !== value.fallenCompanions.length ||
      !value.fallenCompanions.every(
        (name) => typeof name === "string" && previousCompanions.has(name),
      ))
  )
    return null;
  if (
    value.veterans !== undefined &&
    (!Array.isArray(value.veterans) ||
      value.veterans.length !== value.recall.length ||
      !value.veterans.every(
        (entry, index) =>
          veteran(entry) && entry.type === (value.recall as unknown[])[index] && !entry.role,
      ))
  )
    return null;
  if (
    value.storyVeterans !== undefined &&
    (!Array.isArray(value.storyVeterans) ||
      value.storyVeterans.length > 8 ||
      !value.storyVeterans.every(
        (entry) =>
          veteran(entry) &&
          mission.map.units.some(
            (incoming) =>
              incoming.side === "player" &&
              (entry.role === "leader"
                ? incoming.role === "leader"
                : entry.name && entry.name === incoming.name) &&
              TYPES[incoming.type].family === TYPES[entry.type].family,
          ),
      ))
  )
    return null;
  const allowedMarks = new Set([
    ...mission.map.units
      .filter((unit) => unit.side === "player" && unit.name)
      .map((unit) => `fallen:${unit.name}`),
    ...(value.id === "green-accord" ? ["dain-fell"] : []),
    ...(mission.refuge ? ["held:5,4"] : []),
    ...mission.events.map((event) => `event:${event.id}`),
    ...mission.waves.map((_, index) => `wave:${index}`),
    ...mission.goals.flatMap((goal) =>
      goal.kind === "seal"
        ? goal.tiles.map((tile) => `sealed:${tile}`)
        : goal.kind === "hold" || goal.kind === "vigil"
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
  const vigilGoal = mission.goals.find((goal) => goal.kind === "vigil");
  if (
    value.vigil !== undefined &&
    (vigilGoal?.kind !== "vigil" ||
      !record(value.vigil) ||
      !integer(value.vigil.heldTurns) ||
      value.vigil.heldTurns > vigilGoal.turns ||
      !integer(value.vigil.lastCountedRound) ||
      value.vigil.lastCountedRound > round)
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
      !value.choices.every((choice) =>
        value.id === "ashen-oath"
          ? missionIndex >= 1 && (choice === "refuge-peace" || choice === "refuge-war")
          : value.id === "green-accord" && choice === "green-dain-dead",
      ))
  )
    return null;
  const failure = [
    "Your leader fell. The company cannot continue without them.",
    "The courier fell before reaching safety.",
    ...(mission.requiredCompanions ?? []).map(
      (name) => `${name} fell. The company cannot continue without them.`,
    ),
  ];
  const knownLines = [
    ...mission.intro,
    ...mission.events.flatMap((event) => event.dialogue),
    ...mission.events.flatMap((event) => event.warDialogue ?? []),
    ...mission.victory,
    ...(mission.refuge?.escalation ?? []),
    ...(mission.refuge?.warVictory ?? []),
    ...(mission.withoutDain?.intro ?? []),
    ...(mission.withoutDain?.victory ?? []),
    ...Object.values(mission.companionLosses ?? {}).flatMap((changes) => [
      ...(changes.intro ?? []),
      ...(changes.victory ?? []),
    ]),
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
            (known.portrait === line.portrait ||
              (known.portrait === "worker" && line.portrait === "scout")) &&
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
    ...(value.veterans ? { veterans: structuredClone(value.veterans) as CampaignVeteran[] } : {}),
    ...(value.storyVeterans
      ? { storyVeterans: structuredClone(value.storyVeterans) as CampaignVeteran[] }
      : {}),
    ...(value.fallenCompanions
      ? { fallenCompanions: [...value.fallenCompanions] as string[] }
      : {}),
    carryGold: value.carryGold,
    marks: [...value.marks] as string[],
    dialogue: value.dialogue.map((line) => ({
      speaker: String(line.speaker),
      portrait: knownLines.find(
        (known) => known.speaker === line.speaker && known.text === line.text,
      )!.portrait,
      text: String(line.text),
    })),
    dialogueIndex: value.dialogueIndex,
    outcome: value.outcome as string | null,
    completed: value.completed,
    ...(value.refuge
      ? { refuge: { ...(value.refuge as NonNullable<CampaignState["refuge"]>) } }
      : {}),
    ...(vigilGoal
      ? {
          vigil: value.vigil
            ? { ...(value.vigil as NonNullable<CampaignState["vigil"]>) }
            : { heldTurns: 0, lastCountedRound: round },
        }
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
    const isCampaign = Object.hasOwn(CAMPAIGN_MAPS, game.mapId);
    const campaign = isCampaign ? campaignState(s.campaign, game.mapId, s.winner, s.round) : null;
    if ((isCampaign && !campaign) || (!isCampaign && s.campaign !== undefined)) return fresh();
    if (campaign) game.campaign = campaign;
    const ids = new Set<string>(),
      occupied = new Set<string>();
    const units: Unit[] = [];
    for (const unit of s.units as unknown[]) {
      if (!validUnit(unit, game, s.nextId, ids, occupied)) return fresh();
      ids.add(unit.id);
      occupied.add(unit.tile);
      units.push({ ...unit, experience: unit.experience ?? 0 });
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
    if (campaign || s.sites !== undefined) {
      // The reed gatherers moved off the starting adept's hex. Upgrade the old
      // site's ownership without moving units or awarding the visit again.
      const savedSites =
        game.mapId === "lantern-bell" && game.mapRevision === 1 && Array.isArray(s.sites)
          ? s.sites.map((site) =>
              record(site) && site.id === "4,3"
                ? {
                    ...site,
                    id: "5,1",
                    owner: units.find((unit) => unit.tile === "5,1")?.side ?? site.owner,
                  }
                : site,
            )
          : s.sites;
      if (
        !Array.isArray(savedSites) ||
        savedSites.length !== sites.length ||
        new Set(savedSites.map((v) => (record(v) ? v.id : undefined))).size !== sites.length
      )
        return fresh();
      for (const site of savedSites as unknown[]) {
        if (!validVillage(site, sites)) return fresh();
        game.tile(site.id)!.owner = site.owner;
      }
      if (game.mapId === "lantern-bell" && game.mapRevision === 1) game.mapRevision = 2;
    } else {
      // Older noncampaign saves did not persist ownership of these sites.
      for (const site of sites) site.owner = null;
    }
    if (campaign) {
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
        (team) =>
          !units.some((unit) => unit.side === team && TYPES[unit.type].family === "commander"),
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
    // Older green saves did not remember named deaths. Infer only what the saved army proves.
    if (campaign?.id === "green-accord") {
      if (
        (campaign.mission !== 1 || game.mapRevision >= 2) &&
        !campaign.choices?.includes("green-dain-dead") &&
        !units.some((unit) => unit.name === "Captain Dain") &&
        !campaign.marks.includes("dain-fell")
      ) {
        campaign.marks.push("dain-fell");
      }
      // Earlier vigil maps never placed Dain on the board. Bring the living
      // truce participant into those saves, without inferring an unrecorded death.
      if (campaign.mission === 1 && game.mapRevision === 1) {
        if (
          !campaign.choices?.includes("green-dain-dead") &&
          !campaign.marks.includes("dain-fell") &&
          !units.some((unit) => unit.name === "Captain Dain")
        ) {
          const origin = game.tile("5,5")!;
          const destination = [...game.tiles]
            .filter(
              (tile) => Number.isFinite(game.cost({ type: "warden" }, tile)) && !game.at(tile.id),
            )
            .sort((a, b) => distance(a, origin) - distance(b, origin))[0];
          if (destination) game.addUnit("warden", "player", destination.id).name = "Captain Dain";
        }
        game.mapRevision = 2;
      }
      const missingCompanion = CAMPAIGNS[campaign.id].missions[
        campaign.mission
      ]!.requiredCompanions?.find(
        (name) => !units.some((unit) => unit.side === "player" && unit.name === name),
      );
      if (game.winner === "player" && missingCompanion) {
        game.winner = "enemy";
        campaign.completed = false;
        campaign.outcome = `${missingCompanion} fell. The company cannot continue without them.`;
        campaign.dialogueIndex = campaign.dialogue.length;
        campaign.dialogue.push({
          speaker: "The company",
          portrait: CAMPAIGNS[campaign.id].leader,
          text: campaign.outcome,
        });
        game.record(campaign.outcome);
      }
    }
    return { game, pendingEnemy: saved.pendingEnemy };
  } catch {
    return fresh();
  }
}
