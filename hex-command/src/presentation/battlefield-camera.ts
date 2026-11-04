import { traceInstant } from "./performance-trace";
/** DOM camera transforms the single GPU canvas and its transparent input surface
 * together. Camera changes retain native terrain/light/fog caches; resizing the
 * surface can refresh the display-resolution framebuffer. */
export function battlefieldCamera(viewport: HTMLDivElement) {
  let zoom = 1;
  let drag: { id: number; x: number; y: number; left: number; top: number } | undefined;
  let dragged = false;
  let pendingZoom = 0;
  const board = () => viewport.querySelector<SVGSVGElement>("svg");
  const setZoom = (next: number, x: number, y: number) => {
    const svg = board();
    if (!svg) return;
    next = Math.max(0.5, Math.min(3, next));
    if (next === zoom) return;
    const ratio = next / zoom;
    const left = (viewport.scrollLeft + x) * ratio - x;
    const top = (viewport.scrollTop + y) * ratio - y;
    const minimum = parseFloat(getComputedStyle(svg).minWidth) / zoom;
    zoom = next;
    traceInstant("Camera zoom", { zoom });
    svg.style.width = `calc(100% * ${zoom})`;
    svg.style.minWidth = `${minimum * zoom}px`;
    cancelAnimationFrame(pendingZoom);
    pendingZoom = requestAnimationFrame(() => {
      viewport.scrollLeft = left;
      viewport.scrollTop = top;
    });
  };
  const wheel = (event: WheelEvent) => {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    const bounds = viewport.getBoundingClientRect();
    setZoom(
      zoom * Math.exp(-event.deltaY * 0.002),
      event.clientX - bounds.left,
      event.clientY - bounds.top,
    );
  };
  const key = (event: KeyboardEvent) => {
    const factor =
      event.key === "+" || event.key === "="
        ? 1.2
        : event.key === "-"
          ? 1 / 1.2
          : event.key === "0"
            ? 1 / zoom
            : null;
    if (factor === null || event.ctrlKey || event.metaKey || event.altKey) return;
    event.preventDefault();
    setZoom(zoom * factor, viewport.clientWidth / 2, viewport.clientHeight / 2);
  };
  const down = (event: PointerEvent) => {
    if (event.pointerType !== "mouse" || ![0, 1].includes(event.button)) return;
    drag = {
      id: event.pointerId,
      x: event.clientX,
      y: event.clientY,
      left: viewport.scrollLeft,
      top: viewport.scrollTop,
    };
    dragged = false;
    if (event.button === 1) event.preventDefault();
  };
  const move = (event: PointerEvent) => {
    if (!drag || event.pointerId !== drag.id) return;
    const dx = event.clientX - drag.x,
      dy = event.clientY - drag.y;
    if (!dragged && Math.hypot(dx, dy) < 6) return;
    dragged = true;
    viewport.setPointerCapture(event.pointerId);
    viewport.style.cursor = "grabbing";
    event.preventDefault();
    viewport.scrollLeft = drag.left - dx;
    viewport.scrollTop = drag.top - dy;
  };
  const up = (event: PointerEvent) => {
    if (drag?.id !== event.pointerId) return;
    if (viewport.hasPointerCapture(event.pointerId))
      viewport.releasePointerCapture(event.pointerId);
    if (dragged) traceInstant("Camera pan", { left: viewport.scrollLeft, top: viewport.scrollTop });
    drag = undefined;
    viewport.style.cursor = "";
  };
  const click = (event: MouseEvent) => {
    if (!dragged || event.detail === 0) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    dragged = false;
  };
  const leave = () => {
    if (!dragged) drag = undefined;
  };
  viewport.addEventListener("wheel", wheel, { passive: false });
  viewport.addEventListener("keydown", key);
  viewport.addEventListener("pointerdown", down);
  viewport.addEventListener("pointermove", move);
  viewport.addEventListener("pointerup", up);
  viewport.addEventListener("pointercancel", up);
  viewport.addEventListener("lostpointercapture", up);
  viewport.addEventListener("pointerleave", leave);
  viewport.addEventListener("click", click, true);
  return {
    destroy() {
      cancelAnimationFrame(pendingZoom);
      viewport.removeEventListener("wheel", wheel);
      viewport.removeEventListener("keydown", key);
      viewport.removeEventListener("pointerdown", down);
      viewport.removeEventListener("pointermove", move);
      viewport.removeEventListener("pointerup", up);
      viewport.removeEventListener("pointercancel", up);
      viewport.removeEventListener("lostpointercapture", up);
      viewport.removeEventListener("pointerleave", leave);
      viewport.removeEventListener("click", click, true);
    },
  };
}
