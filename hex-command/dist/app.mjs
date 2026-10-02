import { MAPS } from "./maps.mjs?v=2b696541ff5e";
import { createMoveUndo } from "./undo.mjs?v=a9b7694248a6";
import { soundEnabled, setSoundEnabled, unlockSound, playSound, playInteraction } from "./sound.mjs?v=bddeabe42514";
import { animateTile, animateTurn } from "./motion.mjs?v=fd2467be9e52";
import { Game, TYPES, TERRAIN } from "./engine.mjs?v=9d86854f39f3";
import { abilityMarkup } from "./unit-ability.mjs?v=e353d99162f5";
import { loadBattle, saveBattle } from "./save.mjs?v=dd96a765fa08";
import {
  artSprite,
  loadArtStyle,
  savedArtStyle,
  saveArtStyle,
} from "./art.mjs?v=e288e02de731";
import { arsenalMarkup, attackChoicesMarkup } from "./combat.mjs?v=7cb6b0d2c016";
import { iconMarkup } from "./icons.mjs?v=a28758f5c35e";
const $ = (selector) => document.querySelector(selector),
  map = $("#map"),
  panel = $("#unit-panel");
const restoredBattle = loadBattle();
const moveHistory = createMoveUndo();
let game = restoredBattle.game,
  selected = null,
  target = null,
  epoch = 0,
  focusedTile = game.units.find(u => u.side === "player")?.tile || game.tiles[0].id,
  inspectedTile = null;
let previousFeedbackState = null;
const soundControl = $("#sound-effects");
soundControl.checked = soundEnabled();
// Audio starts only after a real interaction, never on initial load or replay.
document.addEventListener("pointerdown", unlockSound, { capture: true, passive: true });
// Touch completion supplies activation on browsers that defer it past pointerdown.
document.addEventListener("pointerup", unlockSound, { capture: true, passive: true });
document.addEventListener("touchend", unlockSound, { capture: true, passive: true });
document.addEventListener("keydown", unlockSound, { capture: true });
document.addEventListener("click", unlockSound, { capture: true });
soundControl.addEventListener("change", async () => {
  setSoundEnabled(soundControl.checked);
  if (soundControl.checked) {
    await unlockSound();
    playSound("select");
  }
});
function selectionFeedback(tileId, attackType = null, audible = true) {
  animateTile(tileId, "select");
  if (audible) playInteraction({ attackType, unitType: game.at(tileId)?.type, terrain: game.tile(tileId)?.terrain });
}
function actionFeedback() {
  const now = game.snapshot(), before = previousFeedbackState;
  previousFeedbackState = now;
  if (!before) return;
  const oldUnits = new Map(before.units.map(unit => [unit.id, unit]));
  const effects = new Map();
  let moved = false, recruited = false, attacked = false, captured = false, movingUnit = null;
  for (const unit of now.units) {
    const old = oldUnits.get(unit.id);
    if (!old) { effects.set(unit.tile, "recruit"); recruited = true; }
    else {
      if (old.tile !== unit.tile) { effects.set(unit.tile, "move"); moved = true; movingUnit = unit; }
      if (old.hp > unit.hp) { effects.set(unit.tile, "attack"); attacked = true; }
    }
    oldUnits.delete(unit.id);
  }
  for (const fallen of oldUnits.values()) { effects.set(fallen.tile, "attack"); attacked = true; }
  for (const [tile, kind] of effects) animateTile(tile, kind);
  for (const village of now.villages) {
    if (village.owner && village.owner !== before.villages.find(tile => tile.id === village.id)?.owner) {
      animateTile(village.id, "capture"); captured = true;
    }
  }
  const turnChanged = before.turn !== now.turn || before.round !== now.round;
  if (turnChanged) animateTurn();
  if (attacked) {
    const attacker = now.units.find(unit => unit.side === before.turn && unit.attacked &&
      before.units.some(old => old.id === unit.id && !old.attacked)) ||
      before.units.find(unit => unit.side === before.turn && !now.units.some(alive => alive.id === unit.id));
    playSound(game.lastCombat?.sound || (attacker?.type === "archer" ? "attack-bow" : "attack-melee"));
  } else if (now.winner && now.winner !== before.winner) playSound(now.winner === "player" ? "victory" : "defeat");
  else if (recruited) playSound("recruit");
  else if (moved) playInteraction({ unitType: movingUnit.type, terrain: game.tile(movingUnit.tile)?.terrain });
  else if (captured) playInteraction({ terrain: "village" });
}
let artStyle = savedArtStyle();
const artControl = $("#art-style");
artControl.value = artStyle;
document.documentElement.dataset.artStyle = artStyle;
const colors = {
    plains: "#39564f",
    forest: "#244c43",
    hill: "#5a6050",
    water: "#1b3c52",
    bridge: "#1b3c52",
    ford: "#1b3c52",
    village: "#69745b",
    keep: "#526d74",
  };
const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
function message(text) {
  $("#status").textContent = text;
}
function active() {
  return game.turn === "player" && !game.winner;
}
function hexPoints(x, y) {
  return Array.from({ length: 6 }, (_, i) => {
    const angle = ((60 * i - 30) * Math.PI) / 180;
    return `${x + 35 * Math.cos(angle)},${y + 35 * Math.sin(angle)}`;
  }).join(" ");
}
function tileCenter(tile) {
  return {
    x: 43 + 61 * (tile.col + (tile.row % 2) / 2),
    y: 42 + 53 * tile.row,
  };
}
function renderTerrain(tile, x, y) {
  return `<g clip-path="url(#hex-clip-${tile.col}-${tile.row})">${artSprite(artStyle, "terrain", tile.terrain, x - 35, y - 35, 70, 70, "terrain-art")}</g>`;
}
function renderUnit(unit, x, y) {
  const teamColor = unit.side === "player" ? "#6fdec0" : "#ff9590";
  const spriteSize = unit.type === "commander" ? 62 : 56;
  const body = `<ellipse class="team-base" cx="${x}" cy="${y + 19}" rx="19" ry="7" fill="${teamColor}" stroke="${teamColor}"/>${artSprite(artStyle, "units", unit.type, x - spriteSize / 2, y + 20 - spriteSize, spriteSize, spriteSize, `unit-art ${unit.side === "enemy" ? "enemy-unit" : ""} ${unit.attacked ? "spent-unit" : ""}`, unit.side)}`;
  return `${body}<rect class="hp-bg" x="${x - 19}" y="${y + 27}" width="38" height="4" rx="2"/><rect class="hp" x="${x - 19}" y="${y + 27}" width="${(38 * unit.hp) / TYPES[unit.type].hp}" height="4" rx="2"/>${unit.attacked ? `<text class="acted" x="${x + 21}" y="${y - 17}">✓</text>` : ""}`;
}
function render() {
  if (game.turn === "player") saveBattle(game);
  const unit = game.unit(selected),
    canAct = active() && unit?.side === "player",
    moves = canAct ? game.reachable(selected) : new Map(),
    targets = canAct ? game.targets(selected) : [];
  map.setAttribute("viewBox", `0 0 ${758 + 61 * (game.cols - 11)} ${452 + 53 * (game.rows - 7)}`);
  map.style.setProperty("--map-scale", game.cols / 11);
  map.setAttribute("aria-label", `${MAPS[game.mapId].name} battlefield`);
  $("#village-total").textContent = game.tiles.filter(tile => tile.terrain === "village").length;
  const focusWasMap = document.activeElement?.closest?.("[data-tile]");
  const clips = game.tiles
    .map((tile) => {
      const { x, y } = tileCenter(tile);
      return `<clipPath id="hex-clip-${tile.col}-${tile.row}"><polygon points="${hexPoints(x, y)}"/></clipPath>`;
    })
    .join("");
  map.innerHTML =
    `<defs>${clips}</defs>` +
    game.tiles
      .map((tile) => {
        const occupant = game.at(tile.id);
        const { x, y } = tileCenter(tile);
        const isTarget = targets.some((u) => u.tile === tile.id);
        const className = [
          "hex",
          `terrain-${tile.terrain}`,
          unit?.tile === tile.id ? "selected" : "",
          moves.has(tile.id) ? "reachable" : "",
          isTarget ? "target" : "",
        ]
          .filter(Boolean)
          .join(" ");
        const label = `${TERRAIN[tile.terrain].name}, column ${tile.col + 1}, row ${tile.row + 1}${tile.owner ? `, ${tile.owner === "player" ? "your" : "enemy"} territory` : ""}${occupant ? `, ${occupant.side === "player" ? "your" : "enemy"} ${TYPES[occupant.type].name}, ${occupant.hp} health${occupant.attacked ? ", turn finished" : ""}` : ""}${moves.has(tile.id) ? ", move here" : ""}${isTarget ? ", attack available" : ""}`;
        return `<g class="${className}" role="button" tabindex="${tile.id === focusedTile ? "0" : "-1"}" aria-label="${label}" data-tile="${tile.id}">
        <polygon class="tile-ground" points="${hexPoints(x, y)}" fill="${colors[tile.terrain]}"/>
        ${renderTerrain(tile, x, y)}
        <polygon class="tile-outline" points="${hexPoints(x, y)}" fill="none"/>
        ${occupant ? renderUnit(occupant, x, y) : ""}
        ${tile.owner ? `<circle class="territory-badge" cx="${x + 23}" cy="${y + 17}" r="5" fill="${tile.owner === "player" ? "#6fdec0" : "#ff9590"}"/>` : ""}
      </g>`;
      })
      .join("");
  if (focusWasMap)
    map
      .querySelector(`[data-tile="${focusedTile}"]`)
      ?.focus({ preventScroll: true });
  $("#gold").textContent = game.gold.player;
  $("#income").textContent = `+${game.income("player")}`;
  $("#villages").textContent = game.income("player") / 2;
  $("#round").textContent = `Turn ${game.round}`;
  $("#turn-label").textContent = game.winner
    ? game.winner === "player" ? "Victory" : "Defeat"
    : game.turn === "player" ? "End turn" : "Enemy turn";
  $("#round").title = `Round ${game.round}`;
  $("#end-turn").disabled = !active();
  $("#undo-move").disabled = !moveHistory.canUndo(game);
  $("#new-game").disabled = game.turn === "enemy" && !game.winner;
  $("#log").innerHTML = game.log.map((text) => `<li>${text}</li>`).join("");
  renderPanel();
  renderTerrainPanel();
  actionFeedback();
  if (game.winner)
    message(
      game.winner === "player"
        ? "Victory! The enemy commander has fallen. The field is yours."
        : "Your commander has fallen. Start a new battle and try a different approach.",
    );
}
function stat(icon, label, value) {
  return `<span class="unit-stat">${iconMarkup(icon)}<span>${label}: <b>${value}</b></span></span>`;
}
function renderPanel() {
  delete panel.dataset.previewAttack;
  panel.hidden = false;
  if (game.winner) {
    panel.innerHTML = `<div class="unit-title"><h2>${game.winner === "player" ? "The field is yours." : "An army needs a leader."}</h2></div><p>${game.winner === "player" ? "Your army defeated the enemy commander." : "Keep your commander protected and use villages to grow your army."}</p><button id="play-again" class="primary">Play again</button>`;
    $("#play-again").addEventListener("click", openMapPicker);
    return;
  }
  const unit = game.unit(selected);
  if (!unit) {
    panel.hidden = true;
    panel.innerHTML = "";
    return;
  }
  const type = TYPES[unit.type],
    tile = game.tile(unit.tile),
    victim = game.unit(target),
    attackAvailable =
      victim &&
      game.targets(unit.id).includes(victim) &&
      active() &&
      unit.side === "player";
  function healthNumbers(combatant, damage) {
    const after = damage === null ? combatant.hp : Math.max(0, combatant.hp - damage);
    return `<b>${combatant.hp}</b>${damage === null ? '' : `<span aria-label="health after attack"> → <b class="${after === 0 ? 'fatal' : ''}">${after}</b></span>`}<span> / ${TYPES[combatant.type].hp} HP</span>`;
  }
  function damageResult(combatant, damage) {
    return damage >= combatant.hp ? 'Falls in battle' : damage ? `−${damage} HP` : 'Unharmed';
  }
  function unitCard(combatant, damage = null) {
    const info = TYPES[combatant.type];
    const after = damage === null ? combatant.hp : Math.max(0, combatant.hp - damage);
    const portrait = artSprite(artStyle, "units", combatant.type, 0, 0, 72, 90,
        `portrait ${combatant.side === "enemy" ? "enemy-unit" : ""}`, combatant.side);
    return `<div class="combatant ${combatant.side}"><span class="unit-side">${combatant.side === "player" ? "Your unit" : "Enemy unit"}</span>${portrait}<h2>${info.name}</h2><div class="health-numbers">${healthNumbers(combatant, damage)}</div><div class="health-track" role="img" aria-label="${after} of ${info.hp} health${damage === null ? '' : ` after attack; ${combatant.hp} before`}">${damage === null ? '' : `<span class="health-lost" style="width:${combatant.hp/info.hp*100}%"></span>`}<span class="health-left" style="width:${after/info.hp*100}%"></span></div>${damage === null ? '' : `<p class="damage-result">${damageResult(combatant, damage)}</p>`}</div>`;
  }
  const initialAttack = attackAvailable ? game.bestAttack(unit, victim).id : null;
  const forecast = attackAvailable ? game.previewDetails(unit, victim, initialAttack) : null;
  const html = attackAvailable
    ? `<div class="duel">${unitCard(unit, forecast.retaliation)}<span class="duel-divider" aria-hidden="true">⚔</span>${unitCard(victim, forecast.damage)}</div>${attackChoicesMarkup(game, unit, victim)}`
    : `<div class="unit-detail">${unitCard(unit)}<div class="unit-info"><div class="unit-stats">${stat("move", "Move", type.move)}<p class="unit-state"><span>${TERRAIN[tile.terrain].name}</span><span>${unit.attacked ? 'Turn finished' : unit.moved ? 'Can still attack' : 'Ready'}</span></p></div>${abilityMarkup({ game, unit, canAct: active() && unit.side === "player" })}${arsenalMarkup(unit)}</div></div>`;
  panel.innerHTML = html;
  function showForecast(attackId) {
    const preview = game.previewDetails(unit, victim, attackId);
    panel.dataset.previewAttack = attackId;
    for (const [combatant, damage] of [[unit, preview.retaliation], [victim, preview.damage]]) {
      const card = panel.querySelector(`.duel .combatant.${combatant.side}`);
      const after = Math.max(0, combatant.hp - damage), maximum = TYPES[combatant.type].hp;
      card.querySelector('.health-numbers').innerHTML = healthNumbers(combatant, damage);
      card.querySelector('.health-left').style.width = `${after / maximum * 100}%`;
      card.querySelector('.health-track').setAttribute('aria-label', `${after} of ${maximum} health after attack; ${combatant.hp} before`);
      card.querySelector('.damage-result').textContent = damageResult(combatant, damage);
    }
    panel.querySelectorAll('[data-attack]').forEach(button => {
      button.classList.toggle('previewing', button.dataset.attack === attackId);
    });
  }
  if (attackAvailable) showForecast(initialAttack);
  panel.querySelectorAll("[data-inspect]").forEach(button => button.addEventListener("click", () => {
    showForecast(button.dataset.inspect);
    const detail = document.getElementById(button.getAttribute("aria-controls"));
    detail.hidden = !detail.hidden;
    button.setAttribute("aria-expanded", String(!detail.hidden));
  }));
  panel.querySelectorAll('[data-attack]:not(:disabled)').forEach(button => {
    button.addEventListener('pointerenter', event => { if (event.pointerType === 'mouse') showForecast(button.dataset.attack); });
    button.addEventListener('focus', () => showForecast(button.dataset.attack));
  });
  panel.querySelectorAll("[data-attack]").forEach(button => button.addEventListener("click", () => {
    try {
      game.attack(selected, target, button.dataset.attack);
      moveHistory.clear();
      target = null;
      render();
      message(
        game.winner
          ? $("#status").textContent
          : "Attack complete. Select another unit or end your turn.",
      );
    } catch (error) {
      message(error.message);
    }
  }));
}
const terrainDescriptions = {
  plains: "Open grassland with easy footing.",
  forest: "Trees shelter units and obstruct ranged shots.",
  hill: "Rocky high ground offers cover and obstructs ranged shots.",
  water: "Deep water cannot be crossed.",
  bridge: "A sturdy crossing over water with easy footing, but no protection.",
  ford: "Shallow water can be crossed on foot. Heavy armor slows movement, and the open water leaves units exposed.",
  village: "Capture this settlement by moving a unit onto it.",
  keep: "A fortified home for its army’s commander.",
};
function terrainRecruitment(tile) {
  if (!tile || tile.terrain === "water" || game.at(tile.id) || game.winner || game.unit(selected)) return "";
  const keep = game.neighbors(tile.id).find(neighbor => neighbor.terrain === "keep" && neighbor.owner === "player");
  if (!keep) return "";
  const commander = game.units.find(unit => unit.side === "player" && unit.type === "commander");
  const stationed = commander?.tile === keep.id;
  const hint = !stationed ? "Return your commander to this keep to recruit." :
    !active() ? "Recruit on your turn." : "New recruits act next turn.";
  const buttons = ["soldier", "archer", "scout"].map(name => {
    const type = TYPES[name];
    const enabled = active() && stationed && game.recruitTiles(commander.id, name).some(destination => destination.id === tile.id);
    return `<button class="recruit-button" data-recruit="${name}"${enabled ? "" : " disabled"}><span>${type.name}</span><span>${type.cost} gold</span></button>`;
  }).join("");
  return `<div class="terrain-recruitment"><h3>Recruit here</h3><div class="recruit-list">${buttons}</div><p class="recruit-hint">${hint}${active() && stationed && game.gold.player < TYPES.scout.cost ? " Not enough gold." : ""}</p></div>`;
}
function renderTerrainPanel() {
  const terrainPanel = $("#terrain-panel"), tile = game.tile(inspectedTile);
  terrainPanel.hidden = !tile;
  if (!tile) {
    terrainPanel.innerHTML = "";
    return;
  }
  const info = TERRAIN[tile.terrain];
  const facts = [];
  if (Number.isFinite(info.cost)) {
    facts.push(tile.terrain === "ford" ? "Movement: 3 for soldiers and commanders; 2 for archers and scouts" :
      `Movement: ${info.cost}${["forest", "hill"].includes(tile.terrain) ? " (scouts: 1)" : ""}`);
    facts.push(tile.terrain === "ford" ? "Exposure: +1 damage received from all attacks" : tile.terrain === "hill" ? "Protection: −1 damage from all attacks" :
      tile.terrain === "forest" ? "Protection: −1 damage from ranged attacks" : "Protection: none");
  } else facts.push("Movement: impassable");
  if (["forest", "hill"].includes(tile.terrain)) facts.push("Blocks ranged shots through this hex");
  if (tile.terrain === "village") {
    facts.push("+2 gold at the start of its owner’s turn", "Heals a friendly occupant by 2 HP at the start of its turn");
  }
  if (tile.terrain === "keep") {
    facts.push("Its owner’s commander can recruit into adjacent empty land hexes", "Select an empty adjacent hex to recruit", "Keeps cannot be captured");
  }
  const ownership = ["village", "keep"].includes(tile.terrain)
    ? `<p class="terrain-owner ${tile.owner || "neutral"}">${tile.owner === "player" ? "Your " : tile.owner === "enemy" ? "Enemy " : "Unclaimed "}${tile.terrain}</p>` : "";
  terrainPanel.innerHTML = `${terrainRecruitment(tile)}<div class="terrain-detail"><div class="terrain-portrait">${artSprite(artStyle, "terrain", tile.terrain, 0, 0, 80, 80, "terrain-card-art")}<span class="unit-side">Terrain</span></div><div class="terrain-info"><h2>${info.name}</h2>${ownership}<p>${terrainDescriptions[tile.terrain]}</p><ul class="terrain-facts" role="list">${facts.map(fact => `<li>${fact}</li>`).join("")}</ul></div></div>`;
  terrainPanel.querySelectorAll("[data-recruit]").forEach(button => button.addEventListener("click", () => {
    try {
      const commander = game.units.find(unit => unit.side === "player" && unit.type === "commander");
      game.recruit(commander?.id, button.dataset.recruit, tile.id);
      moveHistory.clear();
      render();
      message("Recruited. Your new unit will act next turn.");
    } catch (error) { message(error.message); }
  }));
}
function clearSelection(clearInspection = false) {
  if (!selected && !target && !(clearInspection && inspectedTile)) return;
  selected = null;
  target = null;
  if (clearInspection) inspectedTile = null;
  render();
  message("Selection cleared.");
}
function choose(tileId) {
  if (!game.tile(tileId)) return;
  inspectedTile = tileId;
  focusedTile = tileId;
  renderTerrainPanel();
  if (!active()) { selectionFeedback(tileId); return; }
  try {
    const occupant = game.at(tileId),
      unit = game.unit(selected);
    focusedTile = tileId;
    if (occupant && (occupant.id === selected || occupant.id === target)) {
      clearSelection();
      selectionFeedback(tileId);
      return;
    }
    if (
      occupant?.side === "enemy" &&
      unit?.side === "player" &&
      game.targets(selected).includes(occupant)
    ) {
      target = occupant.id;
      render();
      // Selecting a target is a quiet UI cue; the weapon sounds on resolution.
      animateTile(tileId, "select");
      playSound("select");
      message("Tap a weapon to attack. Use the magnifying glass for damage details.");
      if (matchMedia("(max-width:900px)").matches)
        panel.scrollIntoView({ block: "nearest" });
      return;
    }
    if (occupant) {
      selected = occupant.id;
      target = null;
      render();
      selectionFeedback(tileId);
      message(
        occupant.side === "player"
          ? occupant.attacked
            ? "This unit has finished its turn."
            : `Selected ${TYPES[occupant.type].name.toLowerCase()}. Choose a blue hex to move or a red enemy to attack.`
          : "Enemy unit selected. Select one of your units to act.",
      );
      return;
    }
    if (unit?.side === "player" && game.reachable(selected).has(tileId)) {
      moveHistory.move(game, selected, tileId);
      target = null;
      render();
      message(
        "Moved. You can attack an enemy in range or select another unit.",
      );
      return;
    }
    clearSelection();
    selectionFeedback(tileId);
    const tile = game.tile(tileId);
    message(
      `${TERRAIN[tile.terrain].name}: ${TERRAIN[tile.terrain].description}`,
    );
  } catch (error) {
    message(error.message);
  }
}
map.addEventListener("click", (event) => {
  const cell = event.target.closest("[data-tile]");
  if (cell) choose(cell.dataset.tile);
});
map.addEventListener("keydown", (event) => {
  const cell = event.target.closest("[data-tile]");
  if (!cell) return;
  if (["Enter", " "].includes(event.key)) {
    event.preventDefault();
    choose(cell.dataset.tile);
    return;
  }
  const tile = game.tile(cell.dataset.tile);
  let { col, row } = tile;
  if (event.key === "ArrowLeft") col--;
  else if (event.key === "ArrowRight") col++;
  else if (event.key === "ArrowUp") row--;
  else if (event.key === "ArrowDown") row++;
  else return;
  event.preventDefault();
  const next = game.tile(`${col},${row}`);
  if (next) {
    map.querySelector('[tabindex="0"]')?.setAttribute("tabindex", "-1");
    focusedTile = next.id;
    const element = map.querySelector(`[data-tile="${next.id}"]`);
    element.setAttribute("tabindex", "0");
    element.focus();
  }
});
async function endTurn() {
  if (!active()) throw new Error("Wait for your turn.");
  const currentEpoch = epoch;
  moveHistory.clear();
  selected = null;
  target = null;
  saveBattle(game, true);
  game.endTurn();
  render();
  message("The enemy is planning its moves…");
  await delay(350);
  if (currentEpoch !== epoch) return;
  game.enemyRecruit();
  render();
  const queue = game.units
    .filter((u) => u.side === "enemy")
    .sort(
      (a, b) =>
        ({ scout: 0, archer: 1, soldier: 2, commander: 3 })[a.type] -
        { scout: 0, archer: 1, soldier: 2, commander: 3 }[b.type],
    )
    .map((u) => u.id);
  for (const id of queue) {
    await delay(230);
    if (currentEpoch !== epoch) return;
    game.enemyAct(id);
    render();
    if (game.winner) break;
  }
  if (!game.winner) {
    game.endTurn();
    render();
    message("Your turn. Capture villages, recruit reinforcements, or attack.");
  }
  saveBattle(game);
  return game.snapshot();
}
function reset(mapId = "mirror") {
  epoch++;
  moveHistory.clear();
  game = new Game(mapId);
  previousFeedbackState = null;
  selected = null;
  target = null;
  focusedTile = game.units.find(unit => unit.side === "player")?.tile || game.tiles[0].id;
  inspectedTile = null;
  render();
  message("Select a unit. Capture villages. Defeat the red commander.");
  return game.snapshot();
}
function undoMove() {
  const restored = moveHistory.undo(game);
  if (!restored) return game.snapshot();
  selected = restored.unitId;
  target = null;
  focusedTile = restored.tileId;
  inspectedTile = restored.tileId;
  // A rewind should not replay capture or combat feedback from forward actions.
  previousFeedbackState = null;
  render();
  animateTile(restored.tileId, "move");
  message("Move undone. Choose a new destination.");
  return game.snapshot();
}
$("#undo-move").addEventListener("click", undoMove);
$("#end-turn").addEventListener("click", () =>
  endTurn().catch((error) => message(error.message)),
);
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !document.querySelector("dialog[open]")) clearSelection(true);
});
const resetDialog = $("#reset-dialog");
const settingsDialog = $("#settings-dialog");
$("#settings-button").addEventListener("click", () => { settingsDialog.showModal(); playSound("select"); });
$("#close-settings").addEventListener("click", () => settingsDialog.close());
settingsDialog.addEventListener("click", event => {
  if (event.target === settingsDialog) {
    const bounds = settingsDialog.getBoundingClientRect();
    if (event.clientX < bounds.left || event.clientX > bounds.right ||
        event.clientY < bounds.top || event.clientY > bounds.bottom) settingsDialog.close();
  }
});
artControl.addEventListener("change", async () => {
  const nextStyle = artControl.value;
  artControl.disabled = true;
  if (await loadArtStyle(nextStyle)) {
    artStyle = nextStyle;
    saveArtStyle(artStyle);
    document.documentElement.dataset.artStyle = artStyle;
    render();
  } else {
    artControl.value = artStyle;
    message("Could not load that artwork. Please try again.");
  }
  artControl.disabled = false;
});
function renderMapPicker() {
  const preview = layout => {
    const features = new Map(layout.terrain.map(tile => [`${tile.col},${tile.row}`, tile.terrain]));
    const palette = { plains: "#8d9f4f", forest: "#536e3d", hill: "#85866c", water: "#407899", bridge: "#b3ab8d", ford: "#78a4a6", village: "#d4b476", keep: "#b9b597" };
    let tiles = "";
    for (let row = 0; row < layout.rows; row++) {
      for (let col = 0; col < layout.cols; col++) {
        const { x, y } = tileCenter({ col, row });
        tiles += `<polygon points="${hexPoints(x,y)}" fill="${palette[features.get(`${col},${row}`) || "plains"]}" stroke="#263322" stroke-width="2"/>`;
      }
    }
    return `<svg class="map-preview" viewBox="0 0 ${85 + 61 * (layout.cols - .5)} ${84 + 53 * (layout.rows - 1)}" aria-hidden="true">${tiles}</svg>`;
  };
  $("#map-picker").innerHTML = `<legend>Battlefield</legend>` + Object.values(MAPS).map(layout =>
    `<label class="map-choice"><input type="radio" name="battle-map" value="${layout.id}" ${layout.id === game.mapId ? "checked" : ""}/><span>${preview(layout)}<strong>${layout.name}</strong><span class="map-description">${layout.description}</span></span></label>`
  ).join("");
}
function openMapPicker() {
  settingsDialog.close();
  renderMapPicker();
  resetDialog.showModal();
}
$("#new-game").addEventListener("click", openMapPicker);
$("#keep-playing").addEventListener("click", () => resetDialog.close());
$("#confirm-reset").addEventListener("click", () => {
  resetDialog.close();
  reset($("input[name=battle-map]:checked").value);
});
render();
if (restoredBattle.pendingEnemy) endTurn().catch(error => message(error.message));
const initialArtStyle = artStyle;
loadArtStyle(initialArtStyle).then(async (ready) => {
  if (!ready && artStyle === initialArtStyle && !artControl.disabled) {
    for (const fallback of ["normal", "low-poly", "pixel"].filter(style => style !== initialArtStyle)) {
      if (await loadArtStyle(fallback)) {
        if (artStyle === initialArtStyle && !artControl.disabled) {
          artStyle = fallback;
          artControl.value = artStyle;
          document.documentElement.dataset.artStyle = artStyle;
          render();
          message("Using alternate artwork. Retry your preferred style in Settings.");
        }
        return;
      }
    }
    message("Artwork could not load. Please reload to retry.");
  }
});
if (document.modelContext?.registerTool) {
  const toolset = [
    {
      name: "get_battle_state",
      description:
        "Read units, gold, village ownership, the current turn, and victory state.",
      inputSchema: {
        type: "object",
        properties: {},
        additionalProperties: false,
      },
      annotations: { readOnlyHint: true },
      execute: () => game.snapshot(),
    },
    {
      name: "get_unit_actions",
      description:
        "Read legal moves, attacks, and recruiting locations for a unit.",
      inputSchema: {
        type: "object",
        properties: { unitId: { type: "string" } },
        required: ["unitId"],
        additionalProperties: false,
      },
      annotations: { readOnlyHint: true },
      execute: (input) => {
        if (!game.unit(input.unitId)) throw new Error("Unknown unit.");
        return {
          moves: [...game.reachable(input.unitId)].map(([tile, cost]) => ({
            tile,
            cost,
          })),
          attacks: game.targets(input.unitId).map((u) => ({
            id: u.id,
            ...game.preview(game.unit(input.unitId), u),
            recommendedAttackId: game.bestAttack(game.unit(input.unitId), u).id,
            weapons: game.attackOptions(game.unit(input.unitId), u).map(attack => ({
              ...attack, ...game.previewDetails(game.unit(input.unitId), u, attack.id),
            })),
          })),
          recruitment: Object.fromEntries(
            ["soldier", "archer", "scout"].map((type) => [
              type,
              game.recruitTiles(input.unitId, type).map((t) => t.id),
            ]),
          ),
        };
      },
    },
    {
      name: "move_unit",
      description:
        "Move one of your units to a legal hex. Captures a village when entered.",
      inputSchema: {
        type: "object",
        properties: { unitId: { type: "string" }, tileId: { type: "string" } },
        required: ["unitId", "tileId"],
        additionalProperties: false,
      },
      execute: (input) => {
        moveHistory.move(game, input.unitId, input.tileId);
        selected = input.unitId;
        inspectedTile = game.unit(selected)?.tile || inspectedTile;
        target = null;
        render();
        return game.snapshot();
      },
    },
    {
      name: "attack_unit",
      description:
        "Attack an enemy in range and resolve damage and retaliation.",
      inputSchema: {
        type: "object",
        properties: {
          unitId: { type: "string" },
          targetId: { type: "string" },
          attackId: { type: "string", description: "Weapon id from get_unit_actions; omit to choose the safest effective attack." },
        },
        required: ["unitId", "targetId"],
        additionalProperties: false,
      },
      execute: (input) => {
        game.attack(input.unitId, input.targetId, input.attackId);
        moveHistory.clear();
        selected = input.unitId;
        inspectedTile = game.unit(selected)?.tile || inspectedTile;
        target = null;
        render();
        return game.snapshot();
      },
    },
    {
      name: "recruit_unit",
      description:
        "Spend gold to recruit a soldier, archer, or scout adjacent to your commander on the keep.",
      inputSchema: {
        type: "object",
        properties: {
          commanderId: { type: "string" },
          type: { type: "string", enum: ["soldier", "archer", "scout"] },
          tileId: { type: "string" },
        },
        required: ["commanderId", "type", "tileId"],
        additionalProperties: false,
      },
      execute: (input) => {
        game.recruit(input.commanderId, input.type, input.tileId);
        moveHistory.clear();
        inspectedTile = input.tileId;
        render();
        return game.snapshot();
      },
    },
    {
      name: "undo_move",
      description: "Undo the latest move in the current player turn, restoring its village ownership changes. Attacking, recruiting, or ending the turn clears earlier moves.",
      inputSchema: { type: "object", properties: {}, additionalProperties: false },
      execute: undoMove,
    },
    {
      name: "end_turn",
      description:
        "End your turn, let the enemy army act, and return the resulting battle state.",
      inputSchema: {
        type: "object",
        properties: {},
        additionalProperties: false,
      },
      execute: endTurn,
    },
    {
      name: "start_new_battle",
      description: "Replace the current battle with a fresh battle.",
      inputSchema: {
        type: "object",
        properties: { mapId: { type: "string", enum: Object.keys(MAPS) } },
        additionalProperties: false,
      },
      execute: (input = {}) => reset(input.mapId ?? "mirror"),
    },
  ];
  for (const tool of toolset)
    try {
      Promise.resolve(document.modelContext.registerTool(tool)).catch(() => {});
    } catch {}
}
