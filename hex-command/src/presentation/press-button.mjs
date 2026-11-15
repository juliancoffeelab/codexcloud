import { Children, cloneElement } from "react";

const guards = new WeakMap();

// The application owns this listener lifetime. A gesture remains owned when its
// button disappears; pointer-up still reaches Radix and other input owners.
export function installPressGuard(onPress) {
  const doc = document;
  let gesture;
  const clear = () => {
    gesture = undefined;
  };
  const down = (event) => {
    if (event.isPrimary) clear();
    else if (gesture) gesture.fallback = false;
  };
  const up = (event) => {
    if (gesture?.id === event.pointerId) {
      gesture.released = true;
      gesture.x = event.clientX;
      gesture.y = event.clientY;
    }
  };
  const cancel = (event) => {
    if (gesture?.id === event.pointerId) clear();
  };
  const click = (event) => {
    if (!gesture || !event.isTrusted || event.detail === 0) return;
    // Older browsers expose click as MouseEvent. With no intervening primary
    // down, its matching release coordinates/button identify this sequence.
    const matches =
      typeof event.pointerId === "number" && event.pointerId >= 0
        ? event.pointerId === gesture.id
        : gesture.fallback &&
          gesture.released &&
          event.button === 0 &&
          Math.abs(event.clientX - gesture.x) <= 1 &&
          Math.abs(event.clientY - gesture.y) <= 1;
    if (!matches) return;
    clear();
    event.preventDefault();
    event.stopImmediatePropagation();
  };
  const guard = {
    claim(event) {
      gesture = { id: event.pointerId, released: false, fallback: true };
      onPress(event);
    },
  };
  guards.set(doc, guard);
  doc.addEventListener("pointerdown", down, true);
  doc.addEventListener("pointerup", up, true);
  doc.addEventListener("pointercancel", cancel, true);
  doc.addEventListener("click", click, true);
  doc.defaultView.addEventListener("blur", clear);
  return {
    dispose() {
      clear();
      if (guards.get(doc) === guard) guards.delete(doc);
      doc.removeEventListener("pointerdown", down, true);
      doc.removeEventListener("pointerup", up, true);
      doc.removeEventListener("pointercancel", cancel, true);
      doc.removeEventListener("click", click, true);
      doc.defaultView.removeEventListener("blur", clear);
    },
  };
}

// An explicit opt-in around a native button. Cloning preserves its native
// attributes and ref; the action is called directly, never through .click().
export function PressButton({ onActivate, children }) {
  const button = Children.only(children);
  return cloneElement(button, {
    onPointerDown(event) {
      const node = event.currentTarget;
      if (
        !event.isPrimary ||
        event.button !== 0 ||
        node.disabled ||
        node.getAttribute("aria-disabled") === "true"
      )
        return;
      const guard = guards.get(node.ownerDocument);
      if (!guard) return; // A standalone/startup button retains native activation.
      guard.claim(event.nativeEvent);
      event.preventDefault();
      node.focus({ preventScroll: true });
      onActivate();
    },
    onClick(event) {
      if (event.currentTarget.getAttribute("aria-disabled") !== "true") onActivate();
    },
  });
}
