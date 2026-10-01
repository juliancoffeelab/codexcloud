// Feedback is triggered after an action renders, never by rendering itself.
const active = new WeakMap();
const tileKinds = new Set(["select", "move", "attack", "recruit", "capture"]);

function reducedMotion() {
  return globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
}

function pulse(element, kind) {
  if (!element || reducedMotion()) return;
  const previous = active.get(element);
  if (previous) {
    clearTimeout(previous.timer);
    element.classList.remove(previous.className);
  }
  const className = `motion-${kind}`;
  // A repeated action on the same node should restart its brief feedback.
  element.classList.remove(className);
  element.getBoundingClientRect();
  element.classList.add(className);
  const record = { className, timer: null };
  record.timer = setTimeout(() => {
    element.classList.remove(className);
    if (active.get(element) === record) active.delete(element);
  }, 260);
  active.set(element, record);
}

export function animateTile(tileId, kind) {
  if (!tileKinds.has(kind) || typeof document === "undefined") return;
  const tile = [...document.querySelectorAll("#map [data-tile]")]
    .find((node) => node.dataset.tile === String(tileId));
  if (!tile) return;
  if (["move", "attack", "recruit"].includes(kind) && !tile.querySelector(".unit-art")) return;
  pulse(tile, kind);
}

export function animateTurn() {
  if (typeof document !== "undefined") pulse(document.querySelector("#end-turn"), "turn");
}
