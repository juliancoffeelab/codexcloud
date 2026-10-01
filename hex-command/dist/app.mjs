import { Game, TYPES, TERRAIN } from "./engine.mjs";
import { loadBattle, saveBattle } from "./save.mjs";
import {
  artSprite,
  loadArtStyle,
  savedArtStyle,
  saveArtStyle,
} from "./art.mjs?v=de1133bee4a0";
const $ = (selector) => document.querySelector(selector),
  map = $("#map"),
  panel = $("#unit-panel");
const restoredBattle = loadBattle();
let game = restoredBattle.game,
  selected = game.units.find(u => u.side === "player" && u.type !== "commander")?.id || game.units.find(u => u.side === "player")?.id,
  target = null,
  recruitType = null,
  epoch = 0,
  focusedTile = game.unit(selected)?.tile || "1,3";
let artStyle = savedArtStyle();
const artControl = $("#art-style");
artControl.value = artStyle;
document.documentElement.dataset.artStyle = artStyle;
const colors = {
    plains: "#39564f",
    forest: "#244c43",
    hill: "#5a6050",
    water: "#1b3c52",
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
    targets = canAct ? game.targets(selected) : [],
    recruits =
      recruitType && canAct ? game.recruitTiles(selected, recruitType) : [];
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
          moves.has(tile.id) && !recruitType ? "reachable" : "",
          isTarget && !recruitType ? "target" : "",
          recruits.some((t) => t.id === tile.id) ? "recruitable" : "",
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
  $("#new-game").disabled = game.turn === "enemy" && !game.winner;
  $("#cancel").hidden = !recruitType;
  $("#log").innerHTML = game.log.map((text) => `<li>${text}</li>`).join("");
  renderPanel();
  if (game.winner)
    message(
      game.winner === "player"
        ? "Victory! The enemy commander has fallen. The field is yours."
        : "Your commander has fallen. Start a new battle and try a different approach.",
    );
}
const statIcons = {
  move: '<path d="m5 17 4-4 3 3 7-9M14 7h5v5"/>',
  damage: '<path d="m5 19 3-3m-2-3 5 5m-3-4L17 4l3-1-1 3-10 9"/>',
  range: '<circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/><path d="M12 2v3m0 14v3M2 12h3m14 0h3"/>',
};
function stat(icon, label, value) {
  return `<span class="unit-stat"><svg viewBox="0 0 24 24" aria-hidden="true">${statIcons[icon]}</svg><span>${label}: <b>${value}</b></span></span>`;
}
function renderPanel() {
  panel.hidden = false;
  if (game.winner) {
    panel.innerHTML = `<div class="unit-title"><h2>${game.winner === "player" ? "The field is yours." : "An army needs a leader."}</h2></div><p>${game.winner === "player" ? "Your army defeated the enemy commander." : "Keep your commander protected and use villages to grow your army."}</p><button id="play-again" class="primary">Play again</button>`;
    $("#play-again").addEventListener("click", reset);
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
  function unitCard(combatant, damage = null) {
    const info = TYPES[combatant.type];
    const after = damage === null ? combatant.hp : Math.max(0, combatant.hp - damage);
    const portrait = artSprite(artStyle, "units", combatant.type, 0, 0, 72, 90,
        `portrait ${combatant.side === "enemy" ? "enemy-unit" : ""}`, combatant.side);
    return `<div class="combatant ${combatant.side}"><span class="unit-side">${combatant.side === "player" ? "Your unit" : "Enemy unit"}</span>${portrait}<h2>${info.name}</h2><div class="health-numbers"><b>${combatant.hp}</b>${damage !== null ? `<span aria-label="health after attack"> → <b class="${after === 0 ? 'fatal' : ''}">${after}</b>` : ''}<span> / ${info.hp} HP</span></div><div class="health-track" role="img" aria-label="${after} of ${info.hp} health${damage !== null ? ` after attack; ${combatant.hp} before` : ''}"><span class="health-lost" style="width:${combatant.hp/info.hp*100}%"></span><span class="health-left" style="width:${after/info.hp*100}%"></span></div>${damage !== null ? `<p class="damage-result">${after === 0 ? 'Falls in battle' : damage ? `−${damage} HP` : 'Unharmed'}</p>` : ''}</div>`;
  }
  const preview = attackAvailable ? game.preview(unit, victim) : null;
  let html = attackAvailable
    ? `<div class="duel">${unitCard(unit, preview.retaliation)}<span class="duel-divider" aria-hidden="true">⚔</span>${unitCard(victim, preview.damage)}</div><p class="combat-note">${preview.retaliation ? `Enemy retaliates for ${preview.retaliation} damage.` : 'No retaliation.'}</p>`
    : `<div class="unit-detail">${unitCard(unit)}<div class="unit-info"><div class="unit-stats">${stat("move", "Move", type.move)}${stat("damage", "Damage", type.damage)}${stat("range", "Range", type.range)}</div><p class="ability">${type.ability}</p><p>${type.description}</p><p class="unit-state">${TERRAIN[tile.terrain].name} · ${unit.attacked ? 'Turn finished' : unit.moved ? 'Can still attack' : 'Ready'}</p></div></div>`;
  if (unit.side === "player" && unit.type === "commander" && !attackAvailable) {
    if (tile.terrain === "keep" && tile.owner === "player") {
      html +=
        '<h3 class="recruit-title">Recruit an ally</h3><div class="recruit-list">';
      for (const name of ["soldier", "archer", "scout"])
        html += `<button class="recruit-button" data-recruit="${name}" ${!active() || !game.recruitTiles(unit.id, name).length ? "disabled" : ""}><span>${TYPES[name].name}</span><span>${TYPES[name].cost} gold</span></button>`;
      html += "</div><p>New recruits act next turn.</p>";
    } else html += "<p>Return to your keep to recruit.</p>";
  }
  if (attackAvailable) html += '<button id="attack" class="primary attack-button">Attack</button>';
  panel.innerHTML = html;
  panel.querySelectorAll("[data-recruit]").forEach((button) =>
    button.addEventListener("click", () => {
      recruitType = button.dataset.recruit;
      target = null;
      render();
      message(
        `Choose a mint-bordered hex to recruit a ${recruitType} for ${TYPES[recruitType].cost} gold.`,
      );
      if (matchMedia("(max-width:900px)").matches)
        map.scrollIntoView({ block: "center" });
    }),
  );
  $("#attack")?.addEventListener("click", () => {
    try {
      game.attack(selected, target);
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
  });
}
function clearSelection() {
  if (!active() || (!selected && !target && !recruitType)) return;
  selected = null;
  target = null;
  recruitType = null;
  render();
  message("Selection cleared.");
}
function choose(tileId) {
  if (!active()) return;
  try {
    const occupant = game.at(tileId),
      unit = game.unit(selected);
    focusedTile = tileId;
    if (occupant && (occupant.id === selected || occupant.id === target)) {
      clearSelection();
      return;
    }
    if (recruitType) {
      if (
        game.recruitTiles(selected, recruitType).some((t) => t.id === tileId)
      ) {
        game.recruit(selected, recruitType, tileId);
        recruitType = null;
        render();
        message("Recruited. Your new unit will act next turn.");
        return;
      }
      if (occupant?.side === "player") recruitType = null;
      else {
        clearSelection();
        return;
      }
    }
    if (
      occupant?.side === "enemy" &&
      unit?.side === "player" &&
      game.targets(selected).includes(occupant)
    ) {
      target = occupant.id;
      render();
      message("Review the damage, then press Attack.");
      if (matchMedia("(max-width:900px)").matches)
        panel.scrollIntoView({ block: "nearest" });
      return;
    }
    if (occupant) {
      selected = occupant.id;
      target = null;
      render();
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
      game.move(selected, tileId);
      target = null;
      render();
      message(
        "Moved. You can attack an enemy in range or select another unit.",
      );
      return;
    }
    clearSelection();
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
  else clearSelection();
});
document.addEventListener("click", event => {
  const controlClick = event.composedPath().some(node =>
    node instanceof Element && node.matches("#map, button, select, input, dialog, a, summary"));
  if (!controlClick) clearSelection();
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
  selected = null;
  target = null;
  recruitType = null;
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
    selected =
      game.units.find((u) => u.side === "player" && u.type !== "commander")
        ?.id || game.units.find((u) => u.side === "player")?.id;
    render();
    message("Your turn. Capture villages, recruit reinforcements, or attack.");
  }
  saveBattle(game);
  return game.snapshot();
}
function reset() {
  epoch++;
  game = new Game();
  selected = "u2";
  target = null;
  recruitType = null;
  focusedTile = "1,3";
  render();
  message("Select a unit. Capture villages. Defeat the red commander.");
  return game.snapshot();
}
$("#end-turn").addEventListener("click", () =>
  endTurn().catch((error) => message(error.message)),
);
$("#cancel").addEventListener("click", () => {
  recruitType = null;
  render();
  message("Recruitment cancelled. Select a unit to act.");
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !document.querySelector("dialog[open]")) clearSelection();
});
const resetDialog = $("#reset-dialog");
const settingsDialog = $("#settings-dialog");
$("#settings-button").addEventListener("click", () => settingsDialog.showModal());
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
$("#new-game").addEventListener("click", () => {
  settingsDialog.close();
  if (game.winner) reset();
  else resetDialog.showModal();
});
$("#keep-playing").addEventListener("click", () => resetDialog.close());
$("#confirm-reset").addEventListener("click", () => {
  resetDialog.close();
  reset();
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
        game.move(input.unitId, input.tileId);
        selected = input.unitId;
        target = null;
        recruitType = null;
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
        },
        required: ["unitId", "targetId"],
        additionalProperties: false,
      },
      execute: (input) => {
        game.attack(input.unitId, input.targetId);
        selected = input.unitId;
        target = null;
        recruitType = null;
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
        render();
        return game.snapshot();
      },
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
        properties: {},
        additionalProperties: false,
      },
      execute: reset,
    },
  ];
  for (const tool of toolset)
    try {
      Promise.resolve(document.modelContext.registerTool(tool)).catch(() => {});
    } catch {}
}
