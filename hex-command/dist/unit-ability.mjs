import { TYPES } from "./engine.mjs?v=ee5d72abe121";

export function abilityMarkup({ game, unit, canAct }) {
  const type = TYPES[unit.type];
  const heading = `<p class="ability">${type.ability}</p>`;
  if (unit.type === "commander" && unit.side === "enemy") {
    return `${heading}<p>Recruits allies from its keep. New recruits act next turn.</p>`;
  }
  if (unit.type !== "commander") {
    return `${heading}<p>${type.description}</p>`;
  }

  const tile = game.tile(unit.tile);
  const hint = '<p class="recruit-hint">New recruits act next turn.</p>';
  if (tile.terrain !== "keep" || tile.owner !== unit.side) {
    return `${heading}<p>Return to your keep to recruit.</p>${hint}`;
  }

  const buttons = ["soldier", "archer", "scout"].map((name) => {
    const recruit = TYPES[name];
    const disabled = !canAct || !game.recruitTiles(unit.id, name).length;
    return `<button class="recruit-button" data-recruit="${name}"${disabled ? " disabled" : ""}><span>${recruit.name}</span><span>${recruit.cost} gold</span></button>`;
  }).join("");
  return `${heading}<div class="recruit-list">${buttons}</div>${hint}`;
}
