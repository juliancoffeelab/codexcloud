import { cellKey } from "../game/hex";
import Phaser from "phaser";
import type { PhaserBattlefieldFrame } from "./phaser-battlefield";
import type { BattleCommands } from "../controller/view";
import { traceConfiguration, traceInstant } from "./performance-trace";
import { battlefieldSpriteSize, mapDimensions } from "../ui/geometry";

import { BattlefieldPicking } from "./phaser-picking";

/** A fixed viewport: gestures move the camera, never document layout or scrolling. */
export function installBattlefieldInput(
  scene: Phaser.Scene,
  current: () => PhaserBattlefieldFrame | undefined,
  actions: () => BattleCommands,
  invalidate: () => void = () => {},
) {
  const picking = new BattlefieldPicking(scene.cameras.main);
  const camera = scene.cameras.main,
    canvas = scene.game.canvas,
    host = canvas.parentElement!;
  canvas.tabIndex = 0;
  canvas.setAttribute("role", "application");
  canvas.setAttribute("aria-label", "Battlefield");
  canvas.setAttribute("aria-describedby", "battlefield-controls battlefield-description");
  const workspace = host.closest<HTMLElement>(".workspace");
  const gl = (scene.game.renderer as Phaser.Renderer.WebGL.WebGLRenderer).gl;
  const viewportLimit = gl.getParameter(gl.MAX_VIEWPORT_DIMS) as Int32Array;
  const surfaceLimit = Math.min(
    4096,
    gl.getParameter(gl.MAX_TEXTURE_SIZE) as number,
    gl.getParameter(gl.MAX_RENDERBUFFER_SIZE) as number,
    viewportLimit[0]!,
    viewportLimit[1]!,
  );
  const refreshBounds = () => {
    scene.scale.updateBounds();
    if (layoutPending || density !== devicePixelRatio) {
      layoutPending = false;
      layout();
    }
    if (layoutReady) applyScroll();
  };
  const position = (pointer: Phaser.Input.Pointer) => {
    if (pointer.wasTouch) {
      const touch = Array.from((pointer.event as TouchEvent).changedTouches ?? []).find(
        (t) => t.identifier === pointer.identifier,
      );
      if (touch) return { x: touch.clientX, y: touch.clientY };
    }
    const event = pointer.event as MouseEvent;
    return { x: event.clientX, y: event.clientY };
  };
  let zoom = 1,
    displayScale = 1,
    left = 0,
    top = 0;
  let pendingTop: number | undefined;
  let viewportWidth = 1,
    viewportHeight = 1,
    contentWidth = 1,
    contentHeight = 1;
  let pixelX = 1,
    pixelY = 1;
  let layoutPending = true;
  let layoutReady = false;
  let previousCapacity = 0;
  let previousZoom = 1;
  let pendingReveal: { ids: string[]; center: boolean } | undefined;
  let inspection = "";
  let inspectionView: PhaserBattlefieldFrame["view"] | undefined;
  const inspectionTiles = (view: PhaserBattlefieldFrame["view"]) =>
    [
      cellKey(view.game.unit(view.selected)?.tile),
      cellKey(view.game.unit(view.target)?.tile),
      view.inspectedTile,
    ].flatMap((id) => (id ? [id] : []));
  let surfaceWidth = 0,
    surfaceHeight = 0,
    density = 0;
  let commanderGame: PhaserBattlefieldFrame["view"]["game"] | undefined;
  let commanderTile: PhaserBattlefieldFrame["tiles"][number] | undefined;
  let commanderHint: number | null | undefined;
  const viewOrigin = new Phaser.Math.Vector2();
  let frameDelta = 0;
  let gestureId = 0;
  let sampleTrace = 0;
  let lastSample = "";
  const record = (event: string, details?: Record<string, unknown>) => {
    if (!traceConfiguration()) return;
    traceInstant(`Battlefield ${event}`, {
      ...details,
      mapId: current()?.view.game.mapId,
      sessionVersion: current()?.view.sessionVersion,
      gestureId,
      observedAt: performance.now(),
      activePointerIds: pointers().map((pointer) => pointer.id),
      cancelledContacts,
      left,
      top,
      zoom,
      viewportWidth,
      viewportHeight,
      contentWidth,
      contentHeight,
    });
  };
  let gesture = false;
  let cancelledContacts = false;
  let inertia: Phaser.Tweens.Tween | undefined;
  const stopInertia = (reason = "interrupted") => {
    if (inertia) record("inertia stop", { reason });
    inertia?.stop();
    inertia = undefined;
  };
  let drag:
    | {
        pointer: Phaser.Input.Pointer;
        x: number;
        y: number;
        left: number;
        top: number;
        lastX: number;
        lastY: number;
        time: number;
        vx: number;
        vy: number;
      }
    | undefined;
  let pinch: { distance: number; zoom: number; x: number; y: number } | undefined;
  const pointers = () => scene.input.manager.pointers.filter((p) => p.isDown);
  let visibleDirty = true;
  let visible = { left: 0, top: 0, right: 1, bottom: 1 };
  const markVisible = () => {
    visibleDirty = true;
  };
  const visibleBounds = () => {
    if (!visibleDirty) return visible;
    visibleDirty = false;
    const canvasBounds = canvas.getBoundingClientRect();
    visible = { left: 0, top: 0, right: viewportWidth, bottom: viewportHeight };
    // Native details scrolling can clip the top of the canvas without changing
    // its allocation or camera viewport. Navigation must reach that hidden edge.
    for (let node = host.parentElement; node; node = node.parentElement) {
      const style = getComputedStyle(node);
      const clips = (overflow: string) => /^(auto|scroll|hidden|clip)$/.test(overflow);
      if (!clips(style.overflowX) && !clips(style.overflowY)) continue;
      const bounds = node.getBoundingClientRect();
      if (clips(style.overflowX)) {
        visible.left = Math.max(visible.left, bounds.left + node.clientLeft - canvasBounds.left);
        visible.right = Math.min(
          visible.right,
          bounds.left + node.clientLeft + node.clientWidth - canvasBounds.left,
        );
      }
      if (clips(style.overflowY)) {
        visible.top = Math.max(visible.top, bounds.top + node.clientTop - canvasBounds.top);
        visible.bottom = Math.min(
          visible.bottom,
          bounds.top + node.clientTop + node.clientHeight - canvasBounds.top,
        );
      }
    }
    return visible;
  };
  const verticalLimit = () => {
    const bounds = visibleBounds();
    return Math.max(-bounds.top, contentHeight - bounds.bottom);
  };
  let projection = "";
  const applyScroll = (clamp = false) => {
    // Clipping changes visibility only. Clamp on navigation, not on panel layout.
    if (clamp) {
      const bounds = visibleBounds();
      left = Phaser.Math.Clamp(
        left,
        -bounds.left,
        Math.max(-bounds.left, contentWidth - bounds.right),
      );
      top = Phaser.Math.Clamp(top, -bounds.top, verticalLimit());
    }
    const x = (left * pixelX) / camera.zoomX,
      y = (top * pixelY) / camera.zoomY;
    const next = `${x}:${y}:${camera.zoomX}:${camera.zoomY}:${camera.width}:${camera.height}`;
    if (next === projection) return;
    projection = next;
    camera.setOrigin(0, 0).setScroll(x, y);
    camera.preRender();
    invalidate();
  };
  const moveVertical = (next: number) => {
    top = next;
    applyScroll(true);
  };
  const layout = () => {
    visibleDirty = true;
    const frame = current();
    if (!frame) return;
    const size = mapDimensions(frame.view.game),
      nativeScale = 64 / battlefieldSpriteSize;
    const oldWidth = viewportWidth,
      oldHeight = viewportHeight;
    const worldCenterX = (left + previousCapacity / 2) / displayScale;
    viewportWidth = host.clientWidth;
    const minimum = ((window.innerWidth <= 540 ? 650 : 590) * frame.view.game.cols) / 11;
    const capacityWidth = Math.max(1, (workspace?.clientWidth ?? host.clientWidth + 2) - 2);
    const capacityHeight = Math.max(1, (workspace?.clientHeight ?? host.clientHeight + 2) - 2);
    const baseWidth = Math.max(capacityWidth, minimum);
    // A details column clips the existing world; it does not zoom the map.
    contentWidth = baseWidth * zoom;
    displayScale = contentWidth / size.width;
    contentHeight = size.height * displayScale;
    viewportHeight = Math.min(host.clientHeight, capacityHeight);
    // A new viewport invalidates drag anchors; a pinch changing map height does not.
    if (
      layoutReady &&
      (previousCapacity !== capacityWidth ||
        oldWidth !== viewportWidth ||
        density !== devicePixelRatio ||
        (oldHeight !== viewportHeight && previousZoom === zoom))
    )
      cancel();
    if (layoutReady && previousCapacity !== capacityWidth) {
      left = worldCenterX * displayScale - capacityWidth / 2;
    }
    previousCapacity = capacityWidth;
    previousZoom = zoom;
    layoutReady = true;
    const ratio = Math.min(
      devicePixelRatio,
      Math.sqrt(4_194_304 / (capacityWidth * capacityHeight)),
      surfaceLimit / capacityWidth,
      surfaceLimit / capacityHeight,
    );
    const pixelsWide = Math.ceil(capacityWidth * ratio);
    const pixelsHigh = Math.ceil(capacityHeight * ratio);
    // Integer CSS dimensions avoid layout rounding at fractional device densities.
    // Each axis compensates for its independently rounded framebuffer extent.
    pixelX = pixelsWide / capacityWidth;
    pixelY = pixelsHigh / capacityHeight;
    if (surfaceWidth !== pixelsWide || surfaceHeight !== pixelsHigh) {
      scene.scale.resize(pixelsWide, pixelsHigh);
      surfaceWidth = pixelsWide;
      surfaceHeight = pixelsHigh;
      traceInstant("Battlefield surface allocation", {
        width: pixelsWide,
        height: pixelsHigh,
        density: devicePixelRatio,
      });
    }
    density = devicePixelRatio;
    canvas.style.width = `${capacityWidth}px`;
    canvas.style.height = `${capacityHeight}px`;
    scene.scale.displaySize.setSize(capacityWidth, capacityHeight);
    scene.scale.updateBounds();
    scene.scale.displayScale.set(pixelX, pixelY);
    camera
      .setViewport(
        0,
        0,
        Math.ceil(viewportWidth * pixelX),
        Math.min(pixelsHigh, Math.ceil(viewportHeight * pixelY)),
      )
      .removeBounds();
    traceInstant("Battlefield viewport", {
      width: viewportWidth,
      height: viewportHeight,
      worldScale: displayScale,
    });
    // Upstream composes at native resolution, including its rounded final row/column.
    camera.setZoom(
      (contentWidth * pixelX) / (Math.ceil(size.width * nativeScale) / nativeScale),
      (contentHeight * pixelY) / (Math.ceil(size.height * nativeScale) / nativeScale),
    );
    applyScroll();
    if (pendingTop !== undefined) {
      const next = pendingTop;
      pendingTop = undefined;
      moveVertical(next);
    }
  };
  const pan = (dx: number, dy: number) => {
    left += dx;
    top += dy;
    applyScroll(true);
  };
  const zoomAt = (next: number, x: number, y: number) => {
    pendingReveal = undefined;
    next = Phaser.Math.Clamp(next, 0.5, 3);
    if (next === zoom) return;
    const ratio = next / zoom;
    left = (left + x) * ratio - x;
    pendingTop = (top + y) * ratio - y;
    zoom = next;
    layoutPending = true;
  };
  const down = (pointer: Phaser.Input.Pointer) => {
    if (cancelledContacts || document.querySelector("dialog[open]")) {
      record("pointer rejected", {
        reason: cancelledContacts ? "cancelled contacts" : "modal",
        pointerId: pointer.id,
      });
      return;
    }
    if (!pointer.wasTouch && !pointer.leftButtonDown() && !pointer.middleButtonDown()) return;
    gestureId++;
    stopInertia("pointer down");
    pendingReveal = undefined;
    if (traceConfiguration())
      record("pointer down", {
        pointerId: pointer.id,
        touch: pointer.wasTouch,
        ...position(pointer),
        eventTime: pointer.event.timeStamp,
      });
    canvas.focus({ preventScroll: true });
    const active = pointers();
    if (active.length >= 2) {
      const [a, b] = active.map(position) as [{ x: number; y: number }, { x: number; y: number }];
      const bounds = canvas.getBoundingClientRect();
      pinch = {
        distance: Phaser.Math.Distance.Between(a.x, a.y, b.x, b.y),
        zoom,
        x: (a.x + b.x) / 2 - bounds.left,
        y: (a.y + b.y) / 2 - bounds.top,
      };
      record("pinch start", { distance: pinch.distance });
      gesture = true;
      drag = undefined;
    } else {
      gesture = false;
      const point = position(pointer);
      drag = {
        pointer,
        x: point.x,
        y: point.y,
        left,
        top,
        lastX: point.x,
        lastY: point.y,
        time: pointer.event.timeStamp,
        vx: 0,
        vy: 0,
      };
    }
  };
  const move = (pointer: Phaser.Input.Pointer, _objects?: unknown, resampled = false) => {
    if (document.querySelector("dialog[open]")) {
      cancel("modal");
      return;
    }
    const active = pointers();
    if (!resampled && traceConfiguration())
      record("pointer move", {
        pointerId: pointer.id,
        touch: pointer.wasTouch,
        ...position(pointer),
        eventTime: pointer.event.timeStamp,
      });
    if (pinch && active.length >= 2) {
      const [a, b] = active.map(position) as [{ x: number; y: number }, { x: number; y: number }];
      zoomAt(
        (pinch.zoom * Phaser.Math.Distance.Between(a.x, a.y, b.x, b.y)) /
          Math.max(1, pinch.distance),
        pinch.x,
        pinch.y,
      );
      return;
    }
    if (!drag || pointer !== drag.pointer || !pointer.isDown) return;
    const { x, y } = position(pointer);
    let dx = x - drag.x,
      dy = y - drag.y;
    if (!gesture && Math.hypot(dx, dy) < 6) return;
    if (!gesture && pointer.wasTouch) {
      const distance = Math.hypot(dx, dy);
      drag.x += (dx / distance) * 5;
      drag.y += (dy / distance) * 5;
      dx = x - drag.x;
      dy = y - drag.y;
    }
    if (x !== drag.lastX || y !== drag.lastY) {
      const now = pointer.event.timeStamp,
        dt = Math.max(8, now - drag.time);
      drag.vx = (x - drag.lastX) / dt;
      drag.vy = (y - drag.lastY) / dt;
      drag.lastX = x;
      drag.lastY = y;
      drag.time = now;
    }
    gesture = true;
    canvas.style.cursor = "grabbing";
    left = drag.left - dx;
    top = drag.top - dy;
    applyScroll(true);
  };
  const up = (pointer: Phaser.Input.Pointer) => {
    if (traceConfiguration())
      record("pointer up", {
        pointerId: pointer.id,
        touch: pointer.wasTouch,
        ...position(pointer),
        eventTime: pointer.event.timeStamp,
        cancelled: pointer.wasCanceled,
        dragged: gesture,
        contacts: pointers().length,
      });
    if (cancelledContacts) {
      if (!pointers().length) cancelledContacts = false;
      return;
    }
    const frame = current();
    if (
      frame &&
      !document.querySelector("dialog[open]") &&
      !gesture &&
      !pointer.wasCanceled &&
      pointer.getDistance() / Math.max(pixelX, pixelY) < 6 &&
      (pointer.wasTouch || pointer.button === 0)
    ) {
      // Use the exact projection that draws the hexes, including native rounding.
      if (layoutPending || density !== devicePixelRatio) {
        layoutPending = false;
        layout();
      }
      applyScroll();
      const { x, y } = camera.getWorldPoint(pointer.x, pointer.y);
      const tile = picking.pick(frame, x, y);
      if (tile) {
        actions().choose(tile);
      } else {
        actions().clearSelection(true);
      }
    }
    if (!pointers().length) {
      if (
        pointer.wasTouch &&
        !pointer.wasCanceled &&
        gesture &&
        !pinch &&
        drag &&
        pointer.event.timeStamp - drag.time < 160
      ) {
        const vx = -drag.vx,
          vy = -drag.vy,
          speed = Math.hypot(vx, vy);
        if (speed > 0.1) {
          const duration = Math.min(900, speed / 0.004),
            fromLeft = left,
            fromTop = top;
          record("inertia start", { vx, vy, duration });
          inertia = scene.tweens.addCounter({
            from: 0,
            to: 1,
            duration,
            ease: "Quad.easeOut",
            onUpdate: (t) => {
              const progress = t.getValue() ?? 0;
              left = fromLeft + ((vx * duration) / 2) * progress;
              top = fromTop + ((vy * duration) / 2) * progress;
              applyScroll(true);
            },
            onComplete: () => {
              record("inertia complete");
              inertia = undefined;
            },
          });
        }
      }
      drag = undefined;
      pinch = undefined;
      gesture = false;
      canvas.style.cursor = "";
    } else drag = undefined;
  };
  const wheel = (
    pointer: Phaser.Input.Pointer,
    _objects: unknown,
    dx: number,
    dy: number,
    _dz: number,
  ) => {
    const event = pointer.event as WheelEvent;
    record("wheel", {
      dx,
      dy,
      deltaMode: event.deltaMode,
      zoomGesture: event.ctrlKey || event.metaKey,
    });
    cancel("wheel");
    pendingReveal = undefined;
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      zoomAt(zoom * Math.exp(-dy * 0.002), pointer.x / pixelX, pointer.y / pixelY);
      return;
    }
    event.preventDefault();
    const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? viewportHeight : 1;
    if (event.shiftKey && !dx) pan(dy * unit, 0);
    else pan(dx * unit, dy * unit);
  };
  const key = (event: KeyboardEvent) => {
    if (document.activeElement !== canvas || document.querySelector("dialog[open]")) return;
    if (!current() || event.ctrlKey || event.metaKey || event.altKey) return;
    if (!["Escape", "+", "=", "-", "0"].includes(event.key)) return;
    cancel("keyboard shortcut");
    pendingReveal = undefined;
    event.preventDefault();
    switch (event.key) {
      case "Escape":
        actions().clearSelection(true);
        return;
      case "+":
      case "=":
        zoomAt(zoom * 1.2, viewportWidth / 2, viewportHeight / 2);
        return;
      case "-":
        zoomAt(zoom / 1.2, viewportWidth / 2, viewportHeight / 2);
        return;
      case "0":
        zoomAt(1, viewportWidth / 2, viewportHeight / 2);
    }
  };
  // Browser pointer capture keeps Phaser's compatibility mouse events delivered
  // to its canvas while dragging beyond the map, including macOS trackpad drags.
  const capture = (event: PointerEvent) => {
    if (event.pointerType === "mouse" && [0, 1].includes(event.button)) {
      canvas.setPointerCapture(event.pointerId);
    }
  };
  const expectedRelease = new Set<number>();
  const release = (event: PointerEvent) => {
    if (canvas.hasPointerCapture(event.pointerId)) {
      expectedRelease.add(event.pointerId);
      canvas.releasePointerCapture(event.pointerId);
    }
  };
  const lostCapture = (event: PointerEvent) => {
    if (!expectedRelease.delete(event.pointerId)) cancel("lost pointer capture");
  };
  const cancel = (cause?: Event | string) => {
    const reason = typeof cause === "string" ? cause : (cause?.type ?? "viewport layout");
    record("gesture cancel", {
      reason,
      contacts: pointers().length,
      dragging: !!drag,
      pinching: !!pinch,
    });
    stopInertia(reason);
    cancelledContacts = pointers().length > 0;
    drag = undefined;
    pinch = undefined;
    gesture = true;
    canvas.style.cursor = "";
  };
  const continueDrag = () => {
    if (drag) move(drag.pointer, undefined, true);
  };
  const resize = () => {
    layoutPending = true;
  };
  const applyReveal = ({ ids, center }: { ids: string[]; center: boolean }) => {
    const frame = current();
    if (!frame) return;
    const clip = visibleBounds();
    const tiles = ids.flatMap((id) => frame.tiles.find((tile) => tile.id === id) ?? []);
    if (!tiles.length) {
      if (center) {
        left = (contentWidth - clip.left - clip.right) / 2;
        moveVertical((contentHeight - clip.top - clip.bottom) / 2);
      }
      return;
    }
    // Camera projection includes the rounded native terrain extent.
    const scaleX = camera.zoomX / pixelX,
      scaleY = camera.zoomY / pixelY;
    const bounds = tiles.map((tile) => {
      const occupant = tile.occupant;
      const visibleUnit =
        occupant &&
        !(
          frame.view.game.presentation.fog.map((hex) => cellKey(hex)).includes(tile.id) &&
          occupant.faction.relation === "hostile"
        );
      const radius = visibleUnit ? battlefieldSpriteSize / 2 : 30.625;
      return {
        left: (tile.x - radius) * scaleX,
        right: (tile.x + radius) * scaleX,
        top: (tile.y - (visibleUnit ? battlefieldSpriteSize - 20 : 35)) * scaleY,
        bottom: (tile.y + 35) * scaleY,
      };
    });
    const last = bounds.at(-1)!;
    let x1 = Math.min(...bounds.map((b) => b.left)),
      x2 = Math.max(...bounds.map((b) => b.right));
    let y1 = Math.min(...bounds.map((b) => b.top)),
      y2 = Math.max(...bounds.map((b) => b.bottom));
    if (x2 - x1 + 24 > clip.right - clip.left) {
      x1 = last.left;
      x2 = last.right;
    }
    if (y2 - y1 + 24 > clip.bottom - clip.top) {
      y1 = last.top;
      y2 = last.bottom;
    }
    if (center) {
      left = (x1 + x2 - clip.left - clip.right) / 2;
      moveVertical((y1 + y2 - clip.top - clip.bottom) / 2);
    } else {
      const visibleTop = top;
      left += Math.min(0, x1 - left - clip.left - 12) || Math.max(0, x2 - left - clip.right + 12);
      moveVertical(
        visibleTop +
          (Math.min(0, y1 - visibleTop - clip.top - 12) ||
            Math.max(0, y2 - visibleTop - clip.bottom + 12)),
      );
    }
    traceInstant("Battlefield camera reveal", { ids, center, left, top });
  };
  const updateCommander = () => {
    const frame = current();
    if (!frame) return;
    if (frame.view.game !== commanderGame) {
      commanderGame = frame.view.game;
      const commander = commanderGame.units.find(
        (unit) => unit.role === "leader" && commanderGame!.faction(unit.id).controllable,
      );
      commanderTile = frame.tiles.find((tile) => tile.id === cellKey(commander?.tile));
    }
    let hint: number | null = null;
    if (commanderTile && viewportWidth > 0 && viewportHeight > 0) {
      camera.getWorldPoint(0, 0, viewOrigin);
      const x = ((commanderTile.x - viewOrigin.x) * camera.zoomX) / pixelX,
        y = ((commanderTile.y - viewOrigin.y) * camera.zoomY) / pixelY;
      const sx = camera.zoomX / pixelX,
        sy = camera.zoomY / pixelY;
      if (
        x - (battlefieldSpriteSize / 2) * sx < 0 ||
        x + (battlefieldSpriteSize / 2) * sx > viewportWidth ||
        y + (20 - battlefieldSpriteSize) * sy < 0 ||
        y + 20 * sy > viewportHeight
      )
        hint = Math.round(
          (Math.atan2(y - viewportHeight / 2, x - viewportWidth / 2) * 180) / Math.PI + 90,
        );
    }
    if (hint !== commanderHint) {
      commanderHint = hint;
      host.dispatchEvent(new CustomEvent("battlefieldcommander", { detail: hint }));
    }
  };
  const prepareLayout = (_time: number, delta: number) => {
    frameDelta = delta;
    const frame = current();
    const view = frame?.view;
    if (frame && view && view !== inspectionView) {
      picking.prepare(frame);
      inspectionView = view;
      const ids = inspectionTiles(view);
      const nextInspection = ids.join("|");
      if (nextInspection !== inspection) {
        inspection = nextInspection;
        if (ids.length) {
          stopInertia("selection reveal");
          pendingReveal = { ids: [...new Set(ids)], center: false };
          // Read the details clip after Svelte publishes its layout. Selection
          // requests attention once; later publications and manual pans do not.
          layoutPending = true;
        }
      }
    }
    if (layoutPending || density !== devicePixelRatio) {
      layoutPending = false;
      layout();
    }
    if (pendingReveal) {
      const request = pendingReveal;
      pendingReveal = undefined;
      applyReveal(request);
    }
  };
  const sampleCamera = () => {
    updateCommander();
    const tracing = traceConfiguration();
    if (tracing) {
      const sample = `${left}:${top}:${zoom}:${viewportWidth}:${viewportHeight}`;
      if (sampleTrace !== tracing.id || sample !== lastSample || drag || pinch || inertia) {
        sampleTrace = tracing.id;
        lastSample = sample;
        const requestedLeft = drag ? drag.left - (drag.lastX - drag.x) : left;
        const requestedTop = drag ? drag.top - (drag.lastY - drag.y) : top;
        record("camera sample", {
          visibleBounds: visibleBounds(),
          deltaMs: frameDelta,
          dragging: !!drag,
          pinching: !!pinch,
          inertia: !!inertia,
          x: drag?.lastX,
          y: drag?.lastY,
          vx: drag?.vx,
          vy: drag?.vy,
          eventTime: drag?.time,
          sampledAt: performance.now(),
          requestedLeft,
          requestedTop,
          overshootX: requestedLeft - left,
          overshootY: requestedTop - top,
          canvasWidth: canvas.width,
          canvasHeight: canvas.height,
          pixelX,
          pixelY,
        });
      }
    }
  };
  // HUD expansion can translate the canvas without resizing it. Refresh Phaser's
  // page bounds before its native mouse/touch/wheel handlers transform an event.
  const boundEvents = [
    "mousedown",
    "mousemove",
    "mouseup",
    "touchstart",
    "touchmove",
    "touchend",
    "wheel",
  ] as const;
  const outsideCancel = () => cancel("pointer up outside");
  let observer: ResizeObserver | undefined;
  let dialogObserver: MutationObserver | undefined;
  let cleaned = false;
  const cleanup = () => {
    if (cleaned) return;
    cleaned = true;
    picking.clear();
    observer?.disconnect();
    dialogObserver?.disconnect();
    for (const type of boundEvents) canvas.removeEventListener(type, refreshBounds, true);
    scene.events.off(Phaser.Scenes.Events.PRE_UPDATE, prepareLayout);
    scene.events.off(Phaser.Scenes.Events.PRE_RENDER, sampleCamera);
    scene.events.off(Phaser.Scenes.Events.PRE_UPDATE, continueDrag);
    window.removeEventListener("resize", resize);
    window.removeEventListener("blur", cancel);
    document.removeEventListener("scroll", markVisible, true);
    stopInertia();
    canvas.removeEventListener("pointerdown", capture);
    canvas.removeEventListener("pointerup", release);
    canvas.removeEventListener("pointercancel", cancel);
    canvas.removeEventListener("lostpointercapture", lostCapture);
    canvas.removeEventListener("blur", cancel);
    scene.input.off("pointerdown", down);
    scene.input.off("pointermove", move);
    scene.input.off("pointerup", up);
    scene.input.off("pointerupoutside", outsideCancel);
    scene.input.off("wheel", wheel);
    scene.input.keyboard?.off("keydown", key);
  };
  // Register teardown before any input/listener/observer setup can fail.
  scene.events.once(Phaser.Scenes.Events.SHUTDOWN, cleanup);
  scene.events.once(Phaser.Scenes.Events.DESTROY, cleanup);
  for (const type of boundEvents)
    canvas.addEventListener(type, refreshBounds, { capture: true, passive: true });
  scene.input.addPointer(1);
  canvas.addEventListener("pointerdown", capture);
  canvas.addEventListener("pointerup", release);
  canvas.addEventListener("pointercancel", cancel);
  canvas.addEventListener("lostpointercapture", lostCapture);
  canvas.addEventListener("blur", cancel);
  scene.input.on("pointerdown", down);
  scene.input.on("pointermove", move);
  scene.input.on("pointerup", up);
  scene.input.on("pointerupoutside", outsideCancel);
  scene.input.on("wheel", wheel);
  scene.input.keyboard?.on("keydown", key);
  scene.events.on(Phaser.Scenes.Events.PRE_UPDATE, continueDrag);
  scene.events.on(Phaser.Scenes.Events.PRE_UPDATE, prepareLayout);
  scene.events.on(Phaser.Scenes.Events.PRE_RENDER, sampleCamera);
  observer = new ResizeObserver(resize);
  observer.observe(host);
  if (workspace) observer.observe(workspace);
  dialogObserver = new MutationObserver(() => {
    if (document.querySelector("dialog[open]")) cancel("modal opened");
  });
  dialogObserver.observe(document.body, {
    attributes: true,
    attributeFilter: ["open"],
    subtree: true,
  });
  window.addEventListener("resize", resize);
  window.addEventListener("blur", cancel);
  document.addEventListener("scroll", markVisible, { capture: true, passive: true });
  return {
    layout: resize,
    returnToCommander() {
      if (!commanderTile) return;
      record("return commander", { tile: commanderTile.id });
      cancel("return commander");
      pendingReveal = { ids: [commanderTile.id], center: true };
      layoutPending = true;
    },
    snapshot() {
      return { zoom, left, top };
    },
    reset(ids: string[]) {
      const frame = current();
      if (frame) picking.prepare(frame);
      cancel("scene reset");
      inspection = "";
      inspectionView = undefined;
      zoom = 1;
      left = top = 0;
      layoutReady = false;
      pendingReveal = { ids, center: true };
      layoutPending = true;
    },
  };
}
