import { hex, cellKey } from "./hex-fixture.mjs";
import { Native, Catalog, catalog, createBattle, restoreBattle } from "./native-fixture.mjs";
const TYPES = Object.fromEntries(Catalog.units(catalog).map((rule) => [rule.id, rule]));
const TERRAIN = Object.fromEntries(Catalog.terrain(catalog).map((rule) => [rule.id, rule]));
const RACES = Object.fromEntries(Catalog.races(catalog).map((rule) => [rule.id, rule]));
const DAMAGE_TYPES = { blade: "Blade", pierce: "Pierce", impact: "Impact" };
function developerFixture(types = []) {
  const battle = createBattle({ kind: "scenario", map: "developer" });
  try {
    const checkpoint = structuredClone(Native.save(battle));
    const leaders = new Set(
      ["player", "enemy"].map(
        (side) =>
          checkpoint.state.units.find(
            (unit) => unit.faction === side && TYPES[unit.type]?.family === "commander",
          ).id,
      ),
    );
    checkpoint.state.units = checkpoint.state.units.filter(
      (unit) => leaders.has(unit.id) || types.includes(unit.type),
    );
    return Game.fromCheckpoint(checkpoint);
  } finally {
    Native.dispose(battle);
  }
}
class Game {
  #battle;
  #signature = "";
  constructor(map = "mirror", revision) {
    this.#battle =
      typeof map === "string"
        ? createBattle({
            kind: "scenario",
            map,
            ...(revision === void 0 ? {} : { revision }),
          })
        : restoreBattle(map);
    this.restoreData(structuredClone(Native.state(Native.view(this.#battle))));
  }
  static startCampaign(id, mission = 0) {
    const fixture = new Game();
    Native.dispose(fixture.#battle);
    fixture.#battle = createBattle({ kind: "campaign", id });
    fixture.restoreData(structuredClone(Native.state(Native.view(fixture.#battle))));
    for (let chapter = 0; chapter < mission; chapter++) {
      fixture.gold.player = 0;
      fixture.winner = "playerVictory";
      fixture.nextCampaignMission([]);
    }
    return fixture;
  }
  /** Revalidate fixture edits as a native checkpoint before publishing them. */
  native() {
    const state = this.captureState(),
      signature = JSON.stringify(state);
    if (signature !== this.#signature) {
      const checkpoint = structuredClone(Native.save(this.#battle));
      Native.replace(this.#battle, { ...checkpoint, state });
      this.#signature = signature;
    }
    return this.#battle;
  }
  restoreData(state, retainUnits = false) {
    if (retainUnits && state.mapId === this.mapId)
      state.units = state.units.map((unit) => {
        const previous = this.unit(unit.id);
        if (!previous) return unit;
        for (const key of Object.keys(previous)) Reflect.deleteProperty(previous, key);
        return Object.assign(previous, unit);
      });
    delete this.campaign;
    delete this.lastCombat;
    Object.assign(this, state);
    this.#signature = JSON.stringify(state);
  }
  apply(command) {
    Native.dispatch(this.native(), command);
    this.restoreData(structuredClone(Native.state(Native.view(this.#battle))), true);
  }
  captureState() {
    return structuredClone({ ...this });
  }
  restoreState(state) {
    const checkpoint = structuredClone(Native.save(this.#battle));
    Native.replace(this.#battle, { ...checkpoint, state });
    this.restoreData(structuredClone(state));
  }
  tile(id) {
    return this.tiles.find((tile) => tile.id === cellKey(id) || tile.location === id);
  }
  unit(id) {
    return this.units.find((unit) => unit.id === id);
  }
  at(id) {
    return this.units.find((unit) => cellKey(unit.tile) === cellKey(id));
  }
  addUnit(type, faction, tile) {
    const unit = {
      items: [],
      id: `u${this.nextId++}`,
      type,
      faction,
      revealedTo: [],
      perceptionEffects: [],
      tile: hex(this.tile(tile) ?? tile),
      hp: Native.preview_unit(this.#battle, type, faction, hex(this.tile(tile) ?? tile)).hp,
      experience: 0,
      moved: false,
      attackSpent: false,
    };
    this.units.push(unit);
    return unit;
  }
  profile(unit) {
    return Native.unit_details(this.publishedView(), unit.id);
  }
  record(message) {
    this.log = [message, ...this.log].slice(0, 8);
  }
  publishedView() {
    return Native.view(this.native());
  }
  nativeBattle() {
    return this.native();
  }
  checkpoint() {
    return Native.save(this.native());
  }
  snapshot() {
    const { mapId, mapRevision, round, turn, gold, winner, units, campaign, tiles } =
      this.captureState();
    const sites = tiles
      .filter((tile) => ["grove", "fort", "ruins"].includes(tile.terrain))
      .map(({ id, owner }) => ({ id, owner }));
    return {
      mapId,
      mapRevision,
      round,
      turn,
      gold,
      winner,
      units,
      ...(campaign ? { campaign } : {}),
      villages: tiles
        .filter((tile) => tile.terrain === "village")
        .map(({ id, owner }) => ({ id, owner })),
      ...(campaign || sites.length ? { sites } : {}),
    };
  }
  neighbors(id) {
    return Native.neighbors(this.publishedView(), id).map((tile) => this.tile(tile.id));
  }
  cost(unit, tile) {
    return (
      Native.unit_details(Native.view(this.#battle), unit.id).terrain.find(
        (row) => cellKey(row.tile) === cellKey(tile),
      ).cost ?? Infinity
    );
  }
  reachable(id) {
    return new Map(Object.entries(Native.reachable(this.publishedView(), id ?? "")));
  }
  targets(id) {
    return Native.targets(this.publishedView(), id ?? "").map((unit) => this.unit(unit.id));
  }
  recruitTiles(id, type) {
    return Native.recruit_tiles(this.publishedView(), id, type).map((tile) => this.tile(tile.id));
  }
  recruitmentOptions(faction) {
    return Native.recruitment_options(this.publishedView(), faction);
  }
  promotionOptions(id) {
    return Native.promotion_options(this.publishedView(), id);
  }
  attackOptions(unit, target, from = unit.tile) {
    if (from !== cellKey(unit.tile)) return [];
    return Native.attack_options(this.publishedView(), unit.id, target.id).map(
      (attack) => attack.attack,
    );
  }
  bestAttack(unit, target, from = unit.tile) {
    if (from !== cellKey(unit.tile)) return null;
    return Native.best_attack(this.publishedView(), unit.id, target.id)?.attack ?? null;
  }
  previewDetails(unit, target, attackId, from = unit.tile) {
    const weapon = attackId ?? this.bestAttack(unit, target, from)?.id;
    const details =
      from === cellKey(unit.tile) &&
      Native.attack_details(this.publishedView(), unit.id, target.id, weapon ?? "");
    if (!details) throw new Error("That attack cannot reach this enemy.");
    return details;
  }
  preview(unit, target, attackId, from = unit.tile) {
    const { damage, retaliation } = this.previewDetails(unit, target, attackId, from);
    return { damage, retaliation };
  }
  canAttackFrom(unit, from, target, attackId = null) {
    return (
      !!unit &&
      !!target &&
      from === cellKey(unit.tile) &&
      this.attackOptions(unit, target, from).some(
        (attack) => attackId === null || attack.id === attackId,
      )
    );
  }
  income(faction) {
    return Native.income(this.publishedView(), faction);
  }
  move(id, tile) {
    this.apply({ kind: "move", unit: Native.unit_id(id), tile });
    return this.unit(id);
  }
  attack(id, target, weapon) {
    const selected = weapon ?? this.bestAttack(this.unit(id), this.unit(target))?.id;
    if (!selected) throw new Error("That attack cannot reach this enemy.");
    this.apply({
      kind: "attack",
      unit: Native.unit_id(id),
      target: Native.unit_id(target),
      weapon: selected,
    });
  }
  recruit(id, type, tile) {
    const unit = `u${this.nextId}`;
    this.apply({ kind: "recruit", unit: Native.unit_id(id), type, tile });
    return this.unit(unit);
  }
  advanceUnit(id, type) {
    this.apply({ kind: "advance", unit: Native.unit_id(id), type });
  }
  endTurn() {
    this.apply({ kind: "endTurn" });
  }
  enemyStep() {
    Native.enemy_step(this.native(), {
      policy: "classic",
      diplomacy: "diplomatic",
      recruitmentLimit: 2,
    });
    this.restoreData(structuredClone(Native.state(Native.view(this.#battle))), true);
  }
  advanceDialogue(skip = false) {
    this.apply({ kind: skip ? "skipDialogue" : "dismissDialogue" });
  }
  previousDialogue() {
    this.apply({ kind: "previousDialogue" });
  }
  nextCampaignMission(ids) {
    this.apply({ kind: "recallMission", units: ids.map(Native.unit_id) });
    return this;
  }
  retryCampaignMission() {
    this.apply({ kind: "retryMission" });
    return this;
  }
  dispose() {
    Native.dispose(this.#battle);
  }
  static fromCheckpoint(checkpoint) {
    return new Game(checkpoint);
  }
}
export { DAMAGE_TYPES, Game, RACES, TERRAIN, TYPES, catalog, developerFixture };
