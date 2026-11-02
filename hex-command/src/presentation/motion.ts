// Feedback is triggered after an action renders, never by rendering itself.
export type MotionKind = "select" | "move" | "attack" | "recruit" | "capture";
interface PulseRecord {
  className: string;
  timer: ReturnType<typeof setTimeout> | null;
}
const active = new WeakMap<Element, PulseRecord>();
const tileKinds = new Set(["select", "move", "attack", "recruit", "capture"]);

function reducedMotion() {
  return globalThis.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
}

function pulse(element: Element | null, kind: MotionKind | "turn") {
  if (!element || reducedMotion()) return;
  const className = `motion-${kind}`;
  const previous = active.get(element);
  const restarting = previous?.className === className;
  if (previous) {
    if (previous.timer !== null) clearTimeout(previous.timer);
    element.classList.remove(previous.className);
  }
  // Only an identical active animation needs a layout flush to restart.
  // A new animation starts when its class is added.
  if (restarting) element.getBoundingClientRect();
  element.classList.add(className);
  const record: PulseRecord = { className, timer: null };
  record.timer = setTimeout(() => {
    element.classList.remove(className);
    if (active.get(element) === record) active.delete(element);
  }, 260);
  active.set(element, record);
}

export function animateTile(tileId: string, kind: MotionKind) {
  if (!tileKinds.has(kind) || typeof document === "undefined") return;
  // Interactive ground nodes stay independent from the visual unit/HUD layers.
  const attribute =
    kind === "select" ? "data-tile" : kind === "capture" ? "data-indicator-tile" : "data-unit-tile";
  const tile = [...document.querySelectorAll<SVGElement>(`#map [${attribute}]`)].find(
    (node) => node.getAttribute(attribute) === String(tileId),
  );
  if (!tile) return;
  if (["move", "attack", "recruit"].includes(kind) && !tile.querySelector(".unit-art")) return;
  pulse(tile, kind);
}

export function animateTurn() {
  if (typeof document !== "undefined") pulse(document.querySelector("#end-turn"), "turn");
}
