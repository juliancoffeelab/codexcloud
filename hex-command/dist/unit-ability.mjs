import { TYPES } from "./engine.mjs?v=9d86854f39f3";

export function abilityMarkup({ unit }) {
  if (unit.type === "commander") {
    return '<p class="ability">Command</p><p>Losing this unit ends the battle.</p>';
  }
  const type = TYPES[unit.type];
  return `<p class="ability">${type.ability}</p><p>${type.description}</p>`;
}
