/** One activation policy for native buttons. Touches in scrolling content remain
 * taps; safe fixed controls activate on press. Consequential actions opt out. */
export function installPressActivation(root: Document) {
  type Contact = { id: number; button: HTMLButtonElement };
  let contact: Contact | undefined;
  let completed: number | undefined;
  let key: { button: HTMLButtonElement; value: string; release: boolean } | undefined;
  const buttonAt = (target: EventTarget | null) =>
    target instanceof Element ? target.closest<HTMLButtonElement>("button") : null;
  const blocked = (button: HTMLButtonElement) =>
    button.disabled || button.getAttribute("aria-disabled") === "true";
  const tapOnly = (button: HTMLButtonElement, pointerType: string) =>
    button.dataset.activation === "release" ||
    (pointerType === "touch" && !!button.closest("[data-touch-scroll]"));
  const clear = () => {
    contact?.button.removeAttribute("data-pressed");
    key?.button.removeAttribute("data-pressed");
    contact = key = undefined;
    completed = undefined;
  };
  const down = (event: PointerEvent) => {
    completed = undefined;
    if (!event.isPrimary || event.button !== 0) return;
    const button = buttonAt(event.target);
    if (!button || blocked(button) || tapOnly(button, event.pointerType)) return;
    event.preventDefault(); // Retain focus explicitly; suppress compatibility mouse events.
    event.stopImmediatePropagation();
    button.focus({ preventScroll: true });
    contact = { id: event.pointerId, button };
    button.setAttribute("data-pressed", "");
    button.click();
  };
  const up = (event: PointerEvent) => {
    if (contact?.id !== event.pointerId) return;
    contact.button.removeAttribute("data-pressed");
    completed = contact.id;
    contact = undefined;
    event.preventDefault();
    event.stopImmediatePropagation();
  };
  const cancel = (event: PointerEvent) => {
    if (contact?.id === event.pointerId) clear();
  };
  const click = (event: MouseEvent) => {
    // The originating gesture owns its trailing click even if it closed a dialog.
    // The next pointer-down clears this ownership; no time-based suppression.
    if (completed === undefined || !event.isTrusted || event.detail === 0) return;
    completed = undefined;
    event.preventDefault();
    event.stopImmediatePropagation();
  };
  const keydown = (event: KeyboardEvent) => {
    if (!["Enter", " "].includes(event.key) || event.altKey || event.ctrlKey || event.metaKey)
      return;
    const button = buttonAt(event.target);
    if (!button || blocked(button)) return;
    event.preventDefault();
    if (event.repeat) return;
    key = { button, value: event.key, release: button.dataset.activation === "release" };
    button.setAttribute("data-pressed", "");
    if (!key.release) button.click();
  };
  const keyup = (event: KeyboardEvent) => {
    if (!key || key.value !== event.key) return;
    event.preventDefault();
    const activation = key;
    key = undefined;
    activation.button.removeAttribute("data-pressed");
    if (
      activation.release &&
      root.activeElement === activation.button &&
      activation.button.isConnected &&
      !blocked(activation.button)
    )
      activation.button.click();
  };
  root.addEventListener("pointerdown", down, true);
  root.addEventListener("pointerup", up, true);
  root.addEventListener("pointercancel", cancel, true);
  root.addEventListener("click", click, true);
  root.addEventListener("keydown", keydown, true);
  root.addEventListener("keyup", keyup, true);
  root.defaultView?.addEventListener("blur", clear);
  return () => {
    clear();
    root.removeEventListener("pointerdown", down, true);
    root.removeEventListener("pointerup", up, true);
    root.removeEventListener("pointercancel", cancel, true);
    root.removeEventListener("click", click, true);
    root.removeEventListener("keydown", keydown, true);
    root.removeEventListener("keyup", keyup, true);
    root.defaultView?.removeEventListener("blur", clear);
  };
}
