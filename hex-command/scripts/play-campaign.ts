/** A headless control surface for playtesting the live engine. No substitute rules or player AI. */
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname } from "node:path";
import { createHash } from "node:crypto";
import { Game, TYPES, TERRAIN, RACES } from "../src/game/engine";
import { CAMPAIGNS } from "../src/game/campaign-data";
import { campaignMission, goalProgress, endsRefugeMediation } from "../src/game/campaign-progress";
import { enemyActionIds } from "../src/game/enemy-turn";
import type { CampaignId, GameState, RecruitType, UnitType } from "../src/game/types";

interface Trial {
  rulesFingerprint?: string;
  state: GameState;
  moves: GameState[];
  transcript: unknown[];
  outcomes: { mission: string; winner: string; round: number; survivors: number }[];
}
function rulesFingerprint(id: CampaignId): string {
  const hash = createHash("sha256");
  hash.update(
    JSON.stringify({ campaign: CAMPAIGNS[id], units: TYPES, terrain: TERRAIN, races: RACES }),
  );
  for (const file of [
    "engine.ts",
    "campaign-progress.ts",
    "enemy-turn.ts",
    "movement.ts",
    "progression.ts",
    "races.ts",
  ])
    hash.update(readFileSync(new URL(`../src/game/${file}`, import.meta.url)));
  return hash.digest("hex");
}
const trialPath = process.argv[2];
if (!trialPath) throw new Error("Usage: play-campaign.ts TRIAL.json REQUEST_JSON");
const path: string = trialPath;
const request: unknown = JSON.parse(process.argv[3] ?? '{"op":"observe"}');
function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error("Expected an object.");
  return value as Record<string, unknown>;
}
function string(value: unknown): string {
  if (typeof value !== "string") throw new Error("Expected a string.");
  return value;
}
const input = record(request);
let trial: Trial;
let game: Game;
if (input.op === "start") {
  if (existsSync(path))
    throw new Error("Trial already exists. Use next, retry, or a new trial path.");
  const id = string(input.campaign);
  if (!Object.hasOwn(CAMPAIGNS, id)) throw new Error("Unknown campaign.");
  game = Game.startCampaign(id as CampaignId);
  if (input.mission !== undefined)
    throw new Error("Start at mission one; carryover must be earned.");
  game.advanceDialogue(true);
  trial = {
    rulesFingerprint: rulesFingerprint(id as CampaignId),
    state: game.captureState(),
    moves: [],
    transcript: [],
    outcomes: [],
  };
} else {
  // This file is produced only by this CLI, not imported as an external battle save.
  trial = JSON.parse(readFileSync(path, "utf8")) as Trial;
  if (
    trial.rulesFingerprint &&
    trial.rulesFingerprint !== rulesFingerprint(trial.state.campaign!.id)
  )
    throw new Error("Campaign rules changed during this trial. Start a fresh trial file.");
  game = new Game(trial.state.mapId, trial.state.mapRevision);
  game.restoreState(trial.state);
}
const symbols = {
  plains: ".",
  forest: "F",
  hill: "H",
  water: "~",
  bridge: "=",
  ford: "s",
  village: "V",
  keep: "K",
  marsh: "M",
  ruins: "R",
  fort: "T",
  grove: "G",
} as const;
function observation(includeRules = false, observedGame = game) {
  const game = observedGame;
  const mission = campaignMission(game.campaign!);
  const relevantTypes = new Set([
    ...game.units.map((unit) => unit.type),
    ...mission.recruits,
    ...mission.enemyRecruits,
    ...mission.waves.flatMap((wave) => wave.units.map((unit) => unit.type)),
  ]);
  for (const type of relevantTypes)
    for (const advanced of TYPES[type].advancesTo ?? []) relevantTypes.add(advanced);
  return {
    campaign: game.campaign!.id,
    mission: game.campaign!.mission + 1,
    name: mission.map.name,
    briefing: mission.briefing,
    hints: mission.hints,
    conversation: game.campaign!.dialogue.map((line) => ({ ...line })),
    requiredCompanions: mission.requiredCompanions ?? [],
    round: game.round,
    turn: game.turn,
    winner: game.winner,
    outcome: game.campaign!.outcome,
    debrief: game.winner === "player" ? (mission.debrief ?? null) : null,
    completed: game.campaign!.completed,
    choices: game.campaign!.choices ?? [],
    refuge: game.campaign!.refuge ?? null,
    vigil: game.campaign!.vigil ?? null,
    gold: game.gold.player,
    objectives: mission.goals.map((goal) => ({ ...goal, ...goalProgress(game, goal) })),
    units: game.units.map((unit) => ({
      ...unit,
      maxHp: TYPES[unit.type].hp,
      promotions: game.promotionOptions(unit.id),
    })),
    territory: game.tiles
      .filter((tile) => ["village", "grove", "ruins", "fort", "keep"].includes(tile.terrain))
      .map((tile) => ({ tile: tile.id, terrain: tile.terrain, owner: tile.owner })),
    map: {
      cols: game.cols,
      rows: game.rows,
      legend: symbols,
      rowsByIndex: Array.from({ length: game.rows }, (_, row) =>
        Array.from(
          { length: game.cols },
          (_, col) => symbols[game.tile(`${col},${row}`)!.terrain],
        ).join(""),
      ),
    },
    recruits: game.recruitmentOptions("player").map((type) => ({
      type,
      cost: TYPES[type].cost,
      hp: TYPES[type].hp,
      move: TYPES[type].move,
      description: TYPES[type].description,
    })),
    sidequests: mission.events.map((event) => ({
      label: event.label,
      trigger: event.trigger,
      gold: event.gold,
      done: game.campaign!.marks.includes(`event:${event.id}`),
    })),
    waves: mission.waves.map((wave) => ({
      id: wave.id,
      round: wave.round,
      label: wave.label,
      at: wave.units.map((unit) => `${unit.col},${unit.row}`),
      requiresUnsealed: wave.requiresUnsealed,
      resolved: game.campaign!.marks.includes(`wave:${wave.id}`),
    })),
    journal: game.log,
    ...(includeRules
      ? {
          rules: {
            actions: [
              "Each unit can move once and attack once per turn. Move BEFORE attacking: attacking also ends movement. Unspent movement is not reusable.",
              "Units cannot move through occupied hexes. They have no zone of control and do not intercept attacks: a screen protects by occupying an enemy approach, not merely standing behind its leader. Coordinates are col,row with zero-based indices and odd-row-offset hex geometry; use options for legal destinations.",
              "Only a leader on its owned keep can recruit into adjacent empty passable terrain. Merfolk can recruit into water. Recruits cannot act until the next player turn. Recruitment does not consume the leader's attack or movement.",
              "Dealing positive damage earns 1 XP; defeating an enemy earns 6 XP per enemy level. Retaliation can also earn XP. Ready units with enough XP can advance: {op:advance,unit:ID,type:TYPE}. Promotion preserves wounds and consumes the turn. Between chapters, returning survivors rest to full health and keep XP and promoted types. Retries restore the incoming chapter checkpoint.",
              "Merfolk move normally on plains, swim through water, and are protected in marsh. Forests, groves and hills cost 3 movement and expose them to +1 damage. Land races take +1 damage in marsh and shallow water. Adjacent wavemenders heal allies at turn start; multiple healers do not stack.",
              "Combat is deterministic. Incoming flat damage modifiers apply before terrain cover/exposure. A surviving defender replies with its strongest reachable weapon of the same melee/ranged category.",
              "End turn executes enemy recruitment and actions, then starts your next turn with village income and village/grove healing. Enemy units have the same stats and rules as yours.",
              "All mission goals must be completed. Losing your leader, required courier, or named required companion is defeat. Killing the enemy leader alone does not complete an escort or territory objective.",
              "Move undo is available until an attack, recruitment or end turn. An invalid action stops the batch; earlier successful actions remain applied, and the returned observation is authoritative.",
            ],
            units: Object.fromEntries([...relevantTypes].map((type) => [type, TYPES[type]])),
            races: RACES,
            terrain: Object.fromEntries(
              Object.entries(TERRAIN).map(([id, terrain]) => [
                id,
                {
                  ...terrain,
                  cost: Number.isFinite(terrain.cost) ? terrain.cost : "impassable",
                },
              ]),
            ),
          },
        }
      : {}),
  };
}
function save() {
  trial.state = game.captureState();
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify(trial));
}
function noteOutcome() {
  if (game.winner)
    trial.outcomes.push({
      mission: game.mapId,
      winner: game.winner,
      round: game.round,
      survivors: game.units.filter((u) => u.side === "player").length,
    });
}
function executeEnemyTurn(target: Game) {
  target.endTurn();
  target.enemyRecruit();
  for (const id of enemyActionIds(target.units)) {
    target.enemyAct(id);
    if (target.winner) break;
  }
  if (!target.winner) target.endTurn();
  target.advanceDialogue(true);
}
let result: unknown;
let error: string | undefined;
try {
  switch (input.op) {
    case "start":
      result = observation(true);
      break;
    case "observe":
      result = observation(input.rules === true);
      break;
    case "forecast": {
      if (game.winner || game.turn !== "player") throw new Error("No player turn is active.");
      const projection = new Game(game.mapId, game.mapRevision);
      projection.restoreState(game.captureState());
      executeEnemyTurn(projection);
      result = {
        note: "Exact live-engine enemy response if you end the turn now. Further player actions invalidate this forecast. The trial is unchanged.",
        observation: observation(false, projection),
      };
      break;
    }
    case "options": {
      const ids = Array.isArray(input.units) ? input.units.map(string) : [string(input.unit)];
      result = ids.map((id) => {
        const unit = game.unit(id);
        if (!unit || unit.side !== "player") throw new Error("Choose a living player unit.");
        const moves = game.reachable(id);
        const from = input.from === undefined ? unit.tile : string(input.from);
        if (from !== unit.tile && !moves.has(from))
          throw new Error("Preview only your current hex or a legal move.");
        return {
          id,
          from,
          promotions: game.promotionOptions(id),
          moves: Object.fromEntries(moves),
          attacks: unit.attacked
            ? []
            : game.units
                .filter((target) => target.side === "enemy")
                .flatMap((target) =>
                  game.attackOptions(unit, target, from).map((attack) => {
                    const detail = game.previewDetails(unit, target, attack.id, from);
                    return {
                      target: target.id,
                      tile: target.tile,
                      weapon: attack.id,
                      deal: detail.damage,
                      take: detail.retaliation,
                      yourHp: Math.max(0, unit.hp - detail.retaliation),
                      enemyHp: Math.max(0, target.hp - detail.damage),
                      endsMediation:
                        detail.damage >= target.hp && endsRefugeMediation(game, target),
                    };
                  }),
                ),
          recruitment: TYPES[unit.type].leader
            ? Object.fromEntries(
                game
                  .recruitmentOptions("player")
                  .map((type) => [type, game.recruitTiles(id, type).map((tile) => tile.id)]),
              )
            : undefined,
        };
      });
      break;
    }
    case "threats": {
      const id = string(input.unit);
      const unit = game.unit(id);
      if (!unit || unit.side !== "player") throw new Error("Choose a living player unit.");
      if (!Array.isArray(input.tiles) || input.tiles.length > 12)
        throw new Error("Provide up to twelve candidate tiles.");
      const reachable = game.reachable(id);
      result = {
        note: "Potential next-turn attacks by current enemies, queried from the real engine. Each enemy is considered independently with refreshed actions; summed damage is a conservative bound, not a prediction. Recruitment, reinforcements, other planned moves and future healing are excluded.",
        positions: input.tiles.map((value) => {
          const tile = string(value);
          if (tile !== unit.tile && !reachable.has(tile))
            throw new Error("Choose the current hex or a legal move.");
          const projection = new Game(game.mapId, game.mapRevision);
          projection.restoreState(game.captureState());
          const target = projection.unit(id)!;
          target.tile = tile;
          const hits = projection.units
            .filter((enemy) => enemy.side === "enemy")
            .flatMap((enemy) => {
              enemy.moved = false;
              enemy.attacked = false;
              const attacks = [enemy.tile, ...projection.reachable(enemy.id).keys()]
                .flatMap((from) =>
                  projection.attackOptions(enemy, target, from).map((weapon) => ({
                    enemy: enemy.id,
                    type: enemy.type,
                    from,
                    weapon: weapon.id,
                    damage: projection.previewDetails(enemy, target, weapon.id, from).damage,
                  })),
                )
                .sort((first, second) => second.damage - first.damage);
              return attacks.length ? [attacks[0]!] : [];
            });
          return {
            tile,
            hp: target.hp,
            maxCombinedDamage: hits.reduce((sum, hit) => sum + hit.damage, 0),
            hits,
          };
        }),
      };
      break;
    }
    case "act": {
      if (game.winner || game.turn !== "player") throw new Error("No player turn is active.");
      if (!Array.isArray(input.actions)) throw new Error("Provide an actions array.");
      for (const raw of input.actions) {
        const action = record(raw);
        const id = string(action.unit);
        if (game.unit(id)?.side !== "player") throw new Error("Only player units can act.");
        if (action.op === "move") {
          const before = game.captureState();
          game.move(id, string(action.tile));
          if (!game.winner) trial.moves.push(before);
        } else if (action.op === "attack") {
          game.attack(
            id,
            string(action.target),
            action.weapon === undefined ? undefined : string(action.weapon),
          );
          trial.moves = [];
        } else if (action.op === "advance") {
          game.advanceUnit(id, string(action.type) as UnitType);
          trial.moves = [];
        } else if (action.op === "recruit") {
          const type = string(action.type);
          if (!game.recruitmentOptions("player").includes(type as RecruitType))
            throw new Error("That troop is not in this mission’s pool.");
          game.recruit(id, type as RecruitType, string(action.tile));
          trial.moves = [];
        } else throw new Error("Unknown action; choose move, attack, advance or recruit.");
        trial.transcript.push({ round: game.round, mission: game.mapId, action });
        game.advanceDialogue(true);
        if (game.winner) {
          noteOutcome();
          break;
        }
      }
      if (input.endTurn === true && !game.winner) {
        const playerRound = game.round;
        trial.moves = [];
        executeEnemyTurn(game);
        trial.transcript.push({
          mission: game.mapId,
          enemyTurnAfter: playerRound,
          state: game.snapshot(),
        });
        noteOutcome();
      }
      result = observation();
      break;
    }
    case "undo": {
      if (game.winner || game.turn !== "player" || !trial.moves.length)
        throw new Error("No move can be undone.");
      game.restoreState(trial.moves.pop()!);
      game.advanceDialogue(true);
      trial.transcript.push({ undo: true });
      result = observation();
      break;
    }
    case "retry": {
      if (game.winner !== "enemy") throw new Error("Retry only a defeated mission.");
      game = game.retryCampaignMission();
      game.advanceDialogue(true);
      trial.moves = [];
      trial.transcript.push({ retry: game.mapId });
      result = observation(true);
      break;
    }
    case "next": {
      if (!Array.isArray(input.recall))
        throw new Error("Choose up to four survivor IDs in recall.");
      game = game.nextCampaignMission(input.recall.map(string));
      game.advanceDialogue(true);
      trial.moves = [];
      trial.transcript.push({ next: game.mapId, recall: input.recall });
      result = observation(true);
      break;
    }
    case "results":
      result = trial.outcomes;
      break;
    default:
      throw new Error("Unknown operation.");
  }
} catch (caught) {
  error = caught instanceof Error ? caught.message : String(caught);
  result = observation();
}
save();
console.log(JSON.stringify({ ...(error ? { error } : {}), result }));
