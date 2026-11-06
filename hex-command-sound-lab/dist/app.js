import { DEATH_SOUNDS } from "./death-picks.js?v=shieldguard-pot-1";
import { eventLevelClass, playbackGainForRef } from "./event-levels.js?v=event-ranges-1";
import { iconMarkup, speakerMarkup } from "./icons.js?v=direct-tap-1";
import { AudioCache } from "./audio-cache.js?v=map-dock-1";
import { buildActions } from "./matching.js?v=shieldguard-pot-1";
const $ = (selector) => document.querySelector(selector);
const loadJSON = async (url) => {
  const response = await fetch(url);
  if (!response.ok) throw Error(`${url}: HTTP ${response.status}`);
  return response.json();
};
try {
  await start();
} catch (error) {
  $("#playing").textContent =
    `Could not load the soundboard: ${error.message}. Reload to retry.`;
}
async function start() {
  const [catalog, workbench, sourceInfo, playbackLevels] = await Promise.all([
    loadJSON("catalog.json?v=death-materials-2"),
    loadJSON("workbench.json?v=terrain-portraits-1"),
    loadJSON("sources.json?v=fish-slap-1"),
    loadJSON("playback-levels.json?v=death-materials-2"),
  ]);
  const byId = new Map(catalog.map((item) => [item.id, item]));
  const kindNames = {
    "recorded-foley": "Recorded Foley",
    "processed-foley": "Processed Foley",
    "processed-natural-recording": "Edited natural recording",
    "processed-effect": "Processed effect",
    "recorded-weapons": "Recorded weapons",
    "natural-animal-recording": "Natural animal recording",
    "designed-effects": "Designed effect",
    "performed-designed-voice": "Performed ghost voice",
    "designed-tonal-effects": "Designed magic",
    "fireplace-effect": "Fireplace effect",
    "synthesized-upstream-effects": "Designed dark magic",
    "current-game-synthesis": "Original game synthesis",
  };
  const packNames = Object.fromEntries(
    catalog.map((x) => [x.pack, x.packLabel || x.pack]),
  );
  let favorites;
  try {
    favorites = new Set(
      JSON.parse(localStorage.getItem("hex-sound-favorites") || "[]"),
    );
  } catch {
    favorites = new Set();
  }
  let context,
    master,
    source,
    request = 0,
    controller,
    selection,
    beganAt = 0,
    resumedAt = 0,
    frame,
    playingId,
    mode = "candidate",
    visibleCount = 60;
  let libraryPlayback = null,
    libraryMatches = [],
    sourceGain,
    sourceEnvelope;
  const cache = new AudioCache(() => context);
  const expanded = new Set();
  let dockEntity;
  let unitTapMode = "selection";
  try {
    const savedMode = localStorage.getItem("hex-sound-unit-tap-mode");
    unitTapMode = ["selection", "attack", "attack2", "death"].includes(savedMode)
      ? savedMode
      : "selection";
  } catch {}
  const secondaryOpen = new Set();
  let audioEnabled = false,
    tapStarted = 0;
  document.body.classList.add("field-mode");
  document.body.dataset.view = "units";
  const pretty = (name) =>
    name
      .replace(/_/g, " ")
      .replace(/([a-z])([A-Z])/g, "$1 $2")
      .replace(/-/g, " ");
  const time = (seconds) =>
    `${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
  function el(tag, cls, text) {
    const result = document.createElement(tag);
    if (cls) result.className = cls;
    if (text !== undefined) result.textContent = text;
    return result;
  }
  const factions = workbench.factions;
  const entityByKey = new Map(
    [...factions.flatMap((f) => f.units), ...workbench.terrains].map(
      (entity) => [entity.key, entity],
    ),
  );
  let activeFaction = factions.find((f) => f.id === "humans") || factions[0];
  let entityMode = "unit",
    unit = activeFaction.units[0];
  let rememberedUnit = unit.key,
    rememberedTerrain = workbench.terrains[0].key;
  let assignments;
  try {
    assignments = JSON.parse(
      localStorage.getItem("hex-sound-assignments-v1") || "{}",
    );
  } catch {
    assignments = {};
  }
  const openMatches = new Set();
  function saveAssignments() {
    try {
      localStorage.setItem(
        "hex-sound-assignments-v1",
        JSON.stringify(assignments),
      );
    } catch {
      $("#playing").textContent =
        "Assignments are tab-local; export to keep them.";
    }
  }
  function currentRows() {
    const rows = buildActions(unit, catalog, { extras: $("#extras").checked }).filter(row => row.key !== "hit");
    if (mode !== "current") return rows;
    if (unit.kind === "unit" && !unit.defined)
      return [
        {
          key: "baseline",
          title: "Original game",
          description:
            "This unreleased artwork has no historical game assignment. Switch to Suggestions & matches to audition its proposed actions.",
          suggested: [],
          alternatives: [],
        },
      ];
    const family =
      unit.kind === "terrain"
        ? /water|ford|marsh/.test(unit.id)
          ? "terrain-water"
          : /forest|grove/.test(unit.id)
            ? "terrain-forest"
            : /hill|mountain/.test(unit.id)
              ? "terrain-hill"
              : /village|house/.test(unit.id)
                ? "terrain-village"
                : /keep|fort|castle/.test(unit.id)
                  ? "terrain-keep"
                  : "terrain-plains"
        : "select";
    return rows.filter(row => row.key !== "death").map((row) => {
      const name = row.key.startsWith("attack-")
        ? /bow|crossbow|arbalest/i.test(row.title)
          ? "attack-bow"
          : "attack-melee"
        : row.key === "hit"
          ? "attack-melee"
          : family;
      const ids = catalog
        .filter(
          (x) =>
            x.pack === "current" && new RegExp(`^${name}-[123]$`).test(x.name),
        )
        .map((x) => x.id);
      return {
        ...row,
        suggested: ids,
        alternatives: [],
        description:
          "Archived game family for comparison; unreleased units have no historical assignment.",
      };
    });
  }
  function keyFor(action) {
    return `${unit.key}/${action.key}`;
  }
  function unitTapAction(entity, rows) {
    if (entity.kind === "terrain" || unitTapMode === "selection")
      return rows.find((row) => row.key === "tap");
    if (unitTapMode === "death") return rows.find((row) => row.key === "death");
    return rows.filter((row) => row.key.startsWith("attack-"))[unitTapMode === "attack2" ? 1 : 0];
  }
  function levelClassFor(action) {
    return eventLevelClass(unit, action, playbackLevels.reference);
  }
  function chosenFor(action) {
    const saved = assignments[keyFor(action)];
    return saved && byId.has(saved.id)
      ? { ...saved, levelClass: levelClassFor(action) }
      : action.suggested[0]
        ? { id: action.suggested[0], take: action.preferredTakes?.[action.suggested[0]] ?? 0, levelClass: levelClassFor(action) }
        : null;
  }
  function drawWave(buffer) {
    const c = $("#wave"),
      g = c.getContext("2d");
    g.clearRect(0, 0, c.width, c.height);
    g.fillStyle = "#b7d58e";
    const data = buffer?.getChannelData(0);
    for (let i = 0; i < 84; i++) {
      let peak = 0;
      if (data)
        for (
          let j = Math.floor((i * data.length) / 84);
          j < Math.floor(((i + 1) * data.length) / 84);
          j++
        )
          peak = Math.max(peak, Math.abs(data[j]));
      const height = Math.max(2, peak * 42);
      g.globalAlpha = data?.length ? 0.8 : 0.25;
      g.fillRect(i * 5, 22 - height / 2, 2, height);
    }
  }
  function halt(reset = true) {
    libraryPlayback = null;
    sourceGain = null;
    const stoppingEnvelope = sourceEnvelope;
    sourceEnvelope = null;
    updateLibraryControls();
    cancelAnimationFrame(frame);
    document
      .querySelectorAll(".sound-choice.is-playing")
      .forEach((card) => card.classList.remove("is-playing"));
    request++;
    controller?.abort();
    controller = null;
    if (source) {
      const active = source;
      source = null;
      try {
        if (stoppingEnvelope) {
          const now = context.currentTime;
          stoppingEnvelope.gain.cancelAndHoldAtTime(now);
          stoppingEnvelope.gain.linearRampToValueAtTime(0, now + 0.005);
          active.stop(now + 0.005);
        } else active.stop();
      } catch {}
    }
    if (reset) {
      selection = null;
      playingId = null;
      $("#playing").textContent = "Tap a unit or a sound.";
      $("#seek").value = 0;
      $("#seek").disabled = true;
      $("#time").textContent = "0:00 / 0:00";
      drawWave();
    }
  }
  function ensureContext() {
    if (!context || context.state === "closed") {
      const C = window.AudioContext || window.webkitAudioContext;
      if (!C) throw Error("This browser does not support Web Audio");
      context = new C({ latencyHint: "interactive" });
      master = context.createGain();
      master.gain.value = Number($("#volume").value) / 100;
      master.connect(context.destination);
    }
  }
  async function unlock() {
    ensureContext();
    const resume =
      context.state === "running" ? Promise.resolve() : context.resume();
    const primer = context.createBufferSource();
    primer.buffer = context.createBuffer(1, 1, context.sampleRate);
    primer.connect(master);
    primer.onended = () => primer.disconnect();
    primer.start();
    await resume;
    if (context.state !== "running")
      throw Error("Audio is paused by the browser. Tap play again");
  }
  function getBuffer(item) {
    return cache.load(item);
  }
  function updateReady() {
    document.querySelectorAll("[data-ready]").forEach((label) => {
      const ready = cache.ready.has(label.dataset.ready);
      label.textContent = ready ? "● Ready" : "○ Not prepared";
      label.classList.toggle("is-ready", ready);
    });
    const rows = currentRows();
    const defaults = rows.map(chosenFor).filter(Boolean);
    const count = defaults.filter((ref) => cache.ready.has(ref.id)).length;
    $("#cache-status").textContent = audioEnabled
      ? `${count}/${defaults.length} actions ready`
      : "Tap to prepare audio";
  }
  function warmSelection() {
    updateReady();
    if (
      !audioEnabled ||
      !context ||
      document.hidden ||
      !["units", "terrain"].includes(document.body.dataset.view)
    )
      return;
    const defaults = currentRows()
      .map(chosenFor)
      .filter(Boolean)
      .map((ref) => byId.get(ref.id));
    const visible = [...document.querySelectorAll(".sound-choice")]
      .filter((card) => {
        const a = card.getBoundingClientRect(),
          strip = card.parentElement.getBoundingClientRect(),
          panel = $("#sound-rows").getBoundingClientRect();
        return (
          strip.width > 0 &&
          a.right >= strip.left - 180 &&
          a.left <= strip.right + 180 &&
          a.bottom >= panel.top &&
          a.top <= panel.bottom
        );
      })
      .map((card) => byId.get(card.dataset.sound));
    const mapBounds = $("#roster").getBoundingClientRect();
    const nearby = [...$("#roster").children]
      .filter((tile) => {
        const r = tile.getBoundingClientRect();
        return (
          r.right >= mapBounds.left - 104 &&
          r.left <= mapBounds.right + 104 &&
          r.bottom >= mapBounds.top &&
          r.top <= mapBounds.bottom
        );
      })
      .map((tile) => {
        const entity = entityByKey.get(tile.dataset.unit);
        const action = unitTapAction(entity, buildActions(entity, catalog));
        return action
          ? byId.get(
              assignments[`${entity.key}/${action.key}`]?.id ||
                action.suggested[0],
            )
          : undefined;
      });
    void cache.warm(
      [...defaults, ...visible, ...nearby].filter(Boolean),
      updateReady,
      defaults.length + Math.min(visible.length, 3),
    );
  }
  async function resumeAndWarm() {
    try {
      await unlock();
      audioEnabled = true;
      warmSelection();
    } catch (error) {
      $("#cache-status").textContent = error.message;
    }
  }
  // Resume in the same gesture that plays a sound; no enable/arming step.
  const resumeOnGesture = () => {
    if (context?.state !== "running") void resumeAndWarm();
  };
  document.addEventListener("pointerdown", resumeOnGesture, { capture: true });
  document.addEventListener(
    "keydown",
    (event) => {
      if (event.key === "Enter" || event.key === " ") resumeOnGesture();
    },
    { capture: true },
  );
  let mapWarmTimer;
  $("#roster").addEventListener("scroll", () => {
    clearTimeout(mapWarmTimer);
    mapWarmTimer = setTimeout(warmSelection, 100);
  });
  function tick() {
    if (!selection || !source) return;
    const elapsed = Math.min(
      selection.duration,
      resumedAt + context.currentTime - beganAt,
    );
    $("#seek").value = elapsed;
    $("#time").textContent = `${time(elapsed)} / ${time(selection.duration)}`;
    frame = requestAnimationFrame(tick);
  }
  function startNode(buffer, start, duration, position, ticket) {
    return new Promise((resolve) => {
      const node = context.createBufferSource(),
        envelope = context.createGain(),
        gain = context.createGain();
      node.buffer = buffer;
      const item = byId.get(playingId);
      const profile = playbackLevels.entries[playingId];
      const gainDb = playbackGainForRef(profile, selection.ref);
      sourceGain = gain;
      gain.gain.value = $("#normalize").checked
        ? 10 ** ((gainDb ?? item.gainDb ?? 0) / 20)
        : 1;
      node.connect(gain);
      gain.connect(envelope);
      envelope.connect(master);
      sourceEnvelope = envelope;
      const now = context.currentTime;
      const remaining = Math.max(0.001, duration - position);
      const fadeIn = Math.min(0.005, remaining / 4);
      const fadeOut = Math.min(0.008, remaining / 4);
      envelope.gain.setValueAtTime(0, now);
      envelope.gain.linearRampToValueAtTime(1, now + fadeIn);
      envelope.gain.setValueAtTime(1, now + remaining - fadeOut);
      envelope.gain.linearRampToValueAtTime(0, now + remaining);
      source = node;
      beganAt = context.currentTime;
      resumedAt = position;
      node.onended = () => {
        node.onended = null;
        node.disconnect();
        gain.disconnect();
        envelope.disconnect();
        if (source === node) {
          source = null;
          sourceGain = null;
          sourceEnvelope = null;
          document
            .querySelectorAll(".sound-choice.is-playing")
            .forEach((card) => card.classList.remove("is-playing"));
        }
        resolve();
      };
      document
        .querySelectorAll(".sound-choice")
        .forEach((card) =>
          card.classList.toggle("is-playing", card.dataset.sound === playingId),
        );
      node.start(0, start + position, Math.max(0.001, duration - position));
      const scheduled = Math.max(0, performance.now() - tapStarted);
      $("#cache-status").textContent = `Ready · ${scheduled.toFixed(0)} ms`;
      $("#cache-status").dataset.scheduleMs = scheduled.toFixed(2);
      $("#cache-status").title =
        "Tap to Web Audio scheduling; speaker and Bluetooth latency is additional.";
      tick();
    });
  }
  function updateLibraryControls() {
    const active = Boolean(libraryPlayback);
    $("#library-play-all").textContent = active
      ? "■ Stop all"
      : `▶ Play all ${libraryMatches.length}`;
    $("#library-play-all").disabled = !active && !libraryMatches.length;
    $("#library-play-all").setAttribute("aria-pressed", String(active));
    $("#library-queue").textContent = active
      ? `${libraryPlayback.index + 1} / ${libraryPlayback.total} full recordings${libraryPlayback.failed ? ` · ${libraryPlayback.failed} skipped` : ""}`
      : `${libraryMatches.length} full recordings · ${time(libraryMatches.reduce((sum, item) => sum + item.duration, 0))}`;
  }
  async function play(list, { libraryBatch = false } = {}) {
    halt();
    const ticket = request;
    const abort = new AbortController();
    controller = abort;
    const batch = libraryBatch
      ? { index: 0, total: list.length, failed: 0 }
      : null;
    libraryPlayback = batch;
    updateLibraryControls();
    try {
      if (context?.state !== "running") await unlock();
      audioEnabled = true;
      for (let index = 0; index < list.length; index++) {
        if (ticket !== request) return;
        const ref = list[index],
          item = byId.get(ref.id);
        if (!item) throw Error("Sound not found");
        const prefix = batch ? `${index + 1}/${list.length} · ` : "";
        if (batch) {
          batch.index = index;
          updateLibraryControls();
        }
        try {
          tapStarted = performance.now();
          $("#playing").textContent = `${prefix}Loading ${pretty(item.name)}…`;
          const buffer = cache.peek(item.id) || (await getBuffer(item));
          if (ticket !== request) return;
          updateReady();
          const take = ref.take === null ? null : item.takes?.[ref.take || 0];
          const start = take?.start || 0;
          const duration = Math.min(
            take?.duration || buffer.duration,
            buffer.duration - start,
          );
          selection = { buffer, start, duration, ref };
          playingId = item.id;
          $("#seek").max = duration;
          $("#seek").value = 0;
          $("#seek").disabled = false;
          $("#playing").textContent =
            `${prefix}${pretty(item.name)}${take && item.takes.length > 1 ? ` · take ${(ref.take || 0) + 1}` : ""}`;
          drawWave(buffer);
          // Decode one recording ahead, rather than the whole collection.
          if (batch && index + 1 < list.length)
            void getBuffer(byId.get(list[index + 1].id)).catch(() => {});
          await startNode(buffer, start, duration, 0, ticket);
        } catch (error) {
          if (!batch) throw error;
          if (ticket !== request) return;
          batch.failed++;
          updateLibraryControls();
        }
      }
      if (ticket === request) {
        cancelAnimationFrame(frame);
        $("#seek").value = selection?.duration || 0;
        $("#time").textContent =
          `${time(selection?.duration || 0)} / ${time(selection?.duration || 0)}`;
        $("#playing").textContent = batch
          ? `Finished ${list.length - batch.failed}/${list.length} recordings${batch.failed ? ` · ${batch.failed} skipped` : ""}`
          : `${$("#playing").textContent} · finished`;
      }
    } catch (error) {
      if (ticket === request)
        $("#playing").textContent = `${error.message}. Tap again to retry.`;
    } finally {
      if (ticket === request) {
        libraryPlayback = null;
        updateLibraryControls();
      }
    }
  }
  $("#seek").onchange = async (e) => {
    if (!selection) return;
    const saved = selection,
      id = playingId;
    halt(false);
    const ticket = request;
    selection = saved;
    playingId = id;
    try {
      await unlock();
      await startNode(
        saved.buffer,
        saved.start,
        saved.duration,
        Math.min(Number(e.target.value), saved.duration - 0.001),
        ticket,
      );
    } catch (error) {
      $("#playing").textContent = error.message;
    }
  };
  $("#stop").onclick = () => halt();
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") halt();
  });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      halt();
      cache.cancelWarm();
    }
  });
  $("#volume").oninput = (e) => {
    const value = Number(e.target.value);
    $("#volume-value").textContent = `${value}%`;
    if (master)
      master.gain.setTargetAtTime(value / 100, context.currentTime, 0.015);
  };
  $("#normalize").onchange = () => {
    if (libraryPlayback) {
      if (sourceGain && selection) {
        const profile = playbackLevels.entries[playingId];
        const gainDb = playbackGainForRef(profile, selection.ref);
        const value = $("#normalize").checked
          ? 10 ** ((gainDb ?? byId.get(playingId).gainDb ?? 0) / 20)
          : 1;
        sourceGain.gain.setTargetAtTime(value, context.currentTime, 0.01);
      }
    } else if (selection) play([selection.ref]);
  };
  function pickUnit(key, audition = false) {
    const list =
      entityMode === "terrain" ? workbench.terrains : activeFaction.units;
    const next = list.find((u) => u.key === key);
    if (!next) return;
    halt();
    unit = next;
    if (entityMode === "terrain") rememberedTerrain = key;
    else rememberedUnit = key;
    renderUnit();
    if (audition) {
      const action = unitTapAction(unit, currentRows());
      const ref = action && chosenFor(action);
      if (ref) void play([ref]);
      else
        $("#playing").textContent =
          unitTapMode.startsWith("attack")
            ? `No ${unitTapMode === "attack2" ? "secondary" : "main"} attack sound assigned to this unit.`
            : unitTapMode === "death" ? "No death sound assigned to this unit." : "No sound assigned.";
    }
  }
  function soundCard(id, action, isSuggested) {
    const item = byId.get(id),
      card = el("div", "candidate-recording");
    card.dataset.recording = id;
    const button = el("button", "record-play", "▶");
    button.setAttribute(
      "aria-label",
      `Play ${unit.name} ${action.title}: ${pretty(item.name)}`,
    );
    const info = el("div", "record-info"),
      title = el("span", "record-title", pretty(item.name));
    info.append(
      title,
      el(
        "small",
        "record-meta",
        `${item.packLabel} · ${kindNames[item.kind] || item.kind}`,
      ),
    );
    const takes = el("select", "take-selector");
    takes.setAttribute(
      "aria-label",
      `${action.title}: ${pretty(item.name)} take`,
    );
    (item.takes || [{ start: 0, duration: item.duration }]).forEach(
      (take, i) => {
        const option = el(
          "option",
          "",
          `Take ${i + 1} · ${take.duration.toFixed(2)}s`,
        );
        option.value = i;
        takes.append(option);
      },
    );
    const full = el("option", "", "Full recording");
    full.value = "full";
    takes.append(full);
    takes.value = String(action.preferredTakes?.[id] ?? 0);
    const saved = assignments[keyFor(action)];
    if (saved?.id === id)
      takes.value = saved.take === null ? "full" : String(saved.take || 0);
    const selectedRef = () => ({
      id,
      take: takes.value === "full" ? null : Number(takes.value),
      levelClass: levelClassFor(action),
    });
    button.onclick = () => play([selectedRef()]);
    const use = el(
      "button",
      "use-sound",
      saved?.id === id ? "Selected" : "Use",
    );
    use.setAttribute(
      "aria-label",
      `Use ${pretty(item.name)} for ${unit.name} ${action.title}`,
    );
    use.setAttribute("aria-pressed", String(saved?.id === id));
    use.onclick = () => {
      assignments[keyFor(action)] = {
        ...selectedRef(),
        entity: unit.key,
        action: action.key,
      };
      saveAssignments();
      renderUnit();
    };
    takes.onchange = () => {
      if (assignments[keyFor(action)]?.id === id) {
        assignments[keyFor(action)] = {
          ...assignments[keyFor(action)],
          ...selectedRef(),
        };
        saveAssignments();
        renderUnit();
      }
    };
    const star = el(
      "button",
      `favorite${favorites.has(id) ? " saved" : ""}`,
      favorites.has(id) ? "★" : "☆",
    );
    star.setAttribute("aria-label", `Favorite ${item.name}`);
    star.setAttribute("aria-pressed", String(favorites.has(id)));
    star.onclick = () => {
      if (favorites.has(id)) favorites.delete(id);
      else favorites.add(id);
      persist();
      star.textContent = favorites.has(id) ? "★" : "☆";
      star.classList.toggle("saved", favorites.has(id));
      star.setAttribute("aria-pressed", String(favorites.has(id)));
    };
    card.append(button, info, takes, use, star);
    return card;
  }
  function renderUnit() {
    $("#unit-role").textContent =
      unit.kind === "terrain"
        ? "Terrain"
        : `${activeFaction.name} · Level ${unit.level}`;
    $("#entity-status").textContent =
      unit.kind === "terrain"
        ? "TILE"
        : unit.defined
          ? "IN GAME"
          : "ARTWORK / PROPOSED";
    $("#unit-name").textContent = unit.name;
    $("#unit-description").textContent =
      unit.kind === "terrain"
        ? "Select a tile cue, surface contact and optional interaction textures."
        : unit.defined
          ? "Game-defined attacks, with editable audition suggestions."
          : "Unreleased artwork roster. Weapon actions below are proposed from its equipment and role.";
    $("#unit-image").src = unit.artwork;
    $("#unit-image").alt = unit.name;
    $("#unit-select").value = unit.key;
    document.querySelectorAll("[data-unit]").forEach((b) => {
      const selected = b.dataset.unit === unit.key;
      b.classList.toggle("selected", selected);
      b.setAttribute("aria-pressed", String(selected));
      b.setAttribute(
        "aria-label",
        `Select and hear ${unit.kind === "unit" ? `${unitTapMode} on ` : ""}${b.dataset.unit}`,
      );
    });
    const root = $("#action-cards");
    root.replaceChildren();
    for (const action of currentRows()) {
      const article = el("article", "action-card");
      article.dataset.action = action.key;
      const heading = el("div", "action-heading"),
        titles = el("div");
      titles.append(
        el("h3", "", action.title),
        el("p", "", action.description),
      );
      heading.append(titles);
      if (action.hypothetical)
        heading.append(el("span", "hypothetical", "OPTIONAL / HYPOTHETICAL"));
      article.append(heading);
      const savedChoice = assignments[keyFor(action)];
      if (savedChoice && byId.has(savedChoice.id)) {
        const chosen = el(
          "div",
          "saved-choice",
          `Your choice: ${pretty(byId.get(savedChoice.id).name)} · ${savedChoice.take === null ? "full recording" : `take ${(savedChoice.take || 0) + 1}`}`,
        );
        const clear = el("button", "", "Clear");
        clear.onclick = () => {
          delete assignments[keyFor(action)];
          saveAssignments();
          renderUnit();
        };
        chosen.append(clear);
        article.append(chosen);
      }
      if (action.caveat) article.append(el("p", "match-caveat", action.caveat));
      const suggestions = el("section", "suggestion-group");
      suggestions.setAttribute("aria-label", `${action.title} suggestions`);
      suggestions.append(
        el("h4", "", `Suggested · ${action.suggested.length} recordings`),
      );
      if (!action.suggested.length)
        suggestions.append(
          el(
            "p",
            "empty-match",
            "No close recommendation in the current collection. Check plausible alternatives below.",
          ),
        );
      for (const id of action.suggested)
        suggestions.append(soundCard(id, action, true));
      article.append(suggestions);
      const more = el("details", "alternative-group"),
        summary = el(
          "summary",
          "",
          `Other plausible matches · ${action.alternatives.length} recordings`,
        );
      more.append(summary);
      const matchKey = keyFor(action);
      more.open = openMatches.has(matchKey);
      more.ontoggle = () => {
        if (more.open) openMatches.add(matchKey);
        else openMatches.delete(matchKey);
      };
      const alternatives = el("div", "alternative-recordings");
      const fill = () => {
        if (alternatives.childElementCount) return;
        if (!action.alternatives.length)
          alternatives.append(
            el(
              "p",
              "empty-match",
              "No additional recordings match this action.",
            ),
          );
        else
          for (const id of action.alternatives)
            alternatives.append(soundCard(id, action, false));
      };
      if (more.open) fill();
      more.addEventListener("toggle", () => {
        if (more.open) fill();
      });
      more.append(alternatives);
      article.append(more);
      root.append(article);
    }
    renderDock();
  }
  const dockGroups = new Map();
  const dockScroll = new Map();
  function renderDock() {
    const rows = currentRows();
    $("#dock-image").src = unit.artwork;
    $("#dock-name").textContent =
      unit.kind === "terrain" ? "Tile sounds" : "Attacks";
    $("#dock-meta").textContent =
      unit.kind === "terrain"
        ? "Tap a tile to hear it"
        : `Unit taps play ${unitTapMode === "attack" ? "the main attack" : unitTapMode === "attack2" ? "the secondary attack" : unitTapMode} sounds`;
    $("#tap-mode").hidden = unit.kind === "terrain";
    for (const button of document.querySelectorAll("[data-tap-mode]")) {
      const active = button.dataset.tapMode === unitTapMode;
      button.classList.toggle("selected", active);
      button.setAttribute("aria-pressed", String(active));
    }
    const root = $("#sound-rows");
    // Save horizontal positions before updating choices, without moving the panel.
    for (const strip of root.querySelectorAll(".action-sounds"))
      dockScroll.set(strip.dataset.context, strip.scrollLeft);
    const panelScroll = root.scrollTop;
    root.replaceChildren();
    const ordered =
      unit.kind === "unit"
        ? [
            ...rows.filter((row) => row.key.startsWith("attack-")),
            ...rows.filter(
              (row) => row.key === (unitTapMode === "hurt" ? "hit" : "tap"),
            ),
            ...rows.filter((row) => row.key === "death"),
            ...rows.filter((row) => row.key === "move"),
          ]
        : rows;
    if (
      unit.kind === "unit" &&
      !ordered.some((row) => row.key.startsWith("attack-"))
    )
      root.append(
        el(
          "p",
          "dock-empty",
          "No weapon attacks. Tap the unit to hear its cue.",
        ),
      );
    for (const action of ordered) {
      const key = keyFor(action),
        saved = assignments[key];
      const group = dockGroups.get(key) || "suggested";
      const row = el("section", "audition-row");
      row.dataset.action = action.key;
      const heading = el("div", "audition-row-heading");
      const label =
        action.key === "tap"
          ? unit.kind === "terrain"
            ? "Tile"
            : "Select"
          : action.key === "move"
            ? "Move"
            : action.key === "hit"
              ? "Hit"
              : action.title.replace(/^Attack · /, "");
      const attack = unit.attacks?.find(
        (item) => `attack-${item.id}` === action.key,
      );
      const symbol = attack
        ? action.damageType || attack.damageType
        : action.key === "move"
          ? "move"
          : action.key === "hit"
            ? "shield"
            : unit.kind === "terrain"
              ? "keep"
              : "recruit";
      const trigger = el("button", "action-trigger");
      trigger.innerHTML =
        action.key === "death" ? speakerMarkup() : iconMarkup(symbol);
      if (action.key === "death")
        trigger.append(el("span", "weapon-label", "Death"));
      if (attack) trigger.append(el("span", "weapon-label", action.attackName || attack.name));
      trigger.title = label;

      trigger.setAttribute("aria-label", `Hear ${unit.name} ${action.title}`);
      trigger.disabled = !chosenFor(action);
      trigger.onclick = () => {
        const ref = chosenFor(action);
        if (ref) void play([ref]);
      };
      heading.append(trigger);
      const groups = el("div", "row-groups");
      for (const [kind, text] of [
        ["suggested", "Suggested"],
        ["alternatives", "Other"],
        ...(saved ? [["saved", "Saved"]] : []),
      ]) {
        const b = el("button", kind === group ? "selected" : "", text);
        b.setAttribute("aria-pressed", String(kind === group));
        b.onclick = () => {
          dockGroups.set(key, kind);
          renderDock();
        };
        groups.append(b);
      }
      heading.append(groups);
      row.append(heading);
      const strip = el("div", "action-sounds");
      strip.dataset.context = `${key}/${group}`;
      strip.setAttribute("aria-label", `${action.title}: swipe recordings`);
      const ids = group === "saved" && saved ? [saved.id] : action[group] || [];
      for (const id of ids) {
        const item = byId.get(id),
          card = el("div", "sound-choice");
        card.dataset.sound = id;
        const takes = el("select", "choice-takes");
        takes.setAttribute(
          "aria-label",
          `${action.title}: ${pretty(item.name)} take`,
        );
        (item.takes || [{ duration: item.duration }]).forEach((take, i) => {
          const o = el("option", "", `${i + 1} · ${take.duration.toFixed(2)}s`);
          o.value = String(i);
          takes.append(o);
        });
        const full = el("option", "", "Full source");
        full.value = "full";
        takes.append(full);
        takes.value = String(action.preferredTakes?.[id] ?? 0);
        if (saved?.id === id)
          takes.value = saved.take === null ? "full" : String(saved.take || 0);
        const ref = () => ({
          id,
          take: takes.value === "full" ? null : Number(takes.value),
          levelClass: levelClassFor(action),
        });
        const sound = el("button", "sound-button");
        sound.innerHTML = speakerMarkup();
        sound.append(el("span", "clip-name", pretty(item.name)));
        sound.setAttribute(
          "aria-label",
          `Hear ${action.title}: ${pretty(item.name)}`,
        );
        sound.onclick = () => void play([ref()]);
        const ready = el("span", "sound-ready");
        ready.dataset.ready = id;
        const use = el(
          "button",
          `save-choice${saved?.id === id ? " chosen" : ""}`,
          saved?.id === id ? "✓" : "Use",
        );
        use.setAttribute(
          "aria-label",
          `Save ${pretty(item.name)} for ${unit.name} ${action.title}`,
        );
        use.title =
          saved?.id === id ? "Update saved take" : "Save for this action";
        use.onclick = () => {
          assignments[key] = { ...ref(), entity: unit.key, action: action.key };
          saveAssignments();
          renderDock();
        };
        card.append(sound, ready, takes, use);
        strip.append(card);
      }
      if (!ids.length)
        strip.append(
          el(
            "p",
            "dock-empty",
            group === "suggested"
              ? "No suggested sound. Try Other."
              : "No recordings in this group.",
          ),
        );
      if (group === "saved" && saved) {
        const reset = el("button", "clear-row-choice", "Reset");
        reset.onclick = () => {
          delete assignments[key];
          dockGroups.set(key, "suggested");
          saveAssignments();
          renderDock();
        };
        strip.append(reset);
      }
      row.append(strip);
      if (unit.kind === "unit" && !action.key.startsWith("attack-")) {
        const details = el("details", "secondary-cue");
        details.dataset.cue = action.key;
        const summary = el(
          "summary",
          "",
          action.key === "move"
            ? "Move · shared board click"
            : action.key === "death"
              ? "Death sound"
              : unitTapMode === "hurt"
                ? "Hurt alternatives"
                : "Selection alternatives",
        );
        details.append(summary, row);
        details.open = secondaryOpen.has(key);
        details.ontoggle = () => {
          if (details.open) secondaryOpen.add(key);
          else secondaryOpen.delete(key);
          warmSelection();
        };
        root.append(details);
      } else root.append(row);
      strip.scrollLeft = dockScroll.get(strip.dataset.context) || 0;
      let timer;
      strip.onscroll = () => {
        dockScroll.set(strip.dataset.context, strip.scrollLeft);
        clearTimeout(timer);
        timer = setTimeout(warmSelection, 100);
      };
    }
    root.scrollTop = dockEntity === unit.key ? panelScroll : 0;
    dockEntity = unit.key;
    requestAnimationFrame(warmSelection);
  }
  function populateUnits() {
    const q = $("#entity-search").value.trim().toLowerCase();
    const list =
      entityMode === "terrain" ? workbench.terrains : activeFaction.units;
    const matches = list.filter((x) =>
      `${x.name} ${x.id} ${x.role || ""} ${x.attacks?.map((a) => a.name).join(" ") || ""}`
        .toLowerCase()
        .includes(q),
    );
    const select = $("#unit-select"),
      quick = $("#roster");
    select.replaceChildren();
    quick.replaceChildren();
    $("#roster-count").textContent =
      `${matches.length} ${entityMode === "terrain" ? "terrain types" : "units"} · every level included`;
    $("#workbench-body").hidden = !matches.length;
    select.disabled = !matches.length;
    $("#sound-dock").hidden = !matches.length;
    if (!matches.length) {
      $("#sound-rows").replaceChildren();
      $("#dock-name").textContent = "No matching tiles";
      cache.cancelWarm();
      select.append(el("option", "", "No matching entries"));
      return;
    }
    for (const u of matches) {
      const option = el(
        "option",
        "",
        `${u.name}${u.level ? ` · L${u.level}` : ""} · ${u.id}`,
      );
      option.value = u.key;
      select.append(option);
    }
    if (!matches.some((x) => x.key === unit.key)) {
      const key = entityMode === "terrain" ? rememberedTerrain : rememberedUnit;
      unit = matches.find((x) => x.key === key) || matches[0];
    }
    for (const u of matches) {
      const button = el("button");
      button.dataset.unit = u.key;
      const image = el("img");
      image.src = u.artwork;
      image.alt = "";
      image.width = 64;
      image.height = 64;
      button.append(image, el("span", "", u.name));
      button.setAttribute("aria-label", `Select and hear ${u.name}`);
      button.onclick = () => pickUnit(u.key, true);
      quick.append(button);
    }
    renderUnit();
  }
  function setFaction(id) {
    halt();
    activeFaction = factions.find((f) => f.id === id) || activeFaction;
    unit = activeFaction.units[0];
    rememberedUnit = unit.key;
    $("#entity-search").value = "";
    document.querySelectorAll("[data-faction]").forEach((b) => {
      const selected = b.dataset.faction === id;
      b.classList.toggle("selected", selected);
      b.setAttribute("aria-pressed", String(selected));
    });
    populateUnits();
  }
  for (const faction of factions) {
    const button = el("button", "faction-button");
    button.dataset.faction = faction.id;
    button.append(
      el("strong", "", faction.name),
      el(
        "small",
        "",
        `${faction.units.length} units · ${faction.definedCount ? `${faction.definedCount} game-defined` : "unreleased"}`,
      ),
    );
    button.onclick = () => setFaction(faction.id);
    $("#factions").append(button);
  }
  $("#unit-select").onchange = (e) => pickUnit(e.target.value);
  $("#entity-search").oninput = populateUnits;
  $("#extras").onchange = () => renderUnit();
  for (const button of document.querySelectorAll("[data-tap-mode]")) {
    button.onclick = () => {
      unitTapMode = button.dataset.tapMode;
      try {
        localStorage.setItem("hex-sound-unit-tap-mode", unitTapMode);
      } catch {}
      renderUnit();
    };
  }
  $("#sound-rows").addEventListener("scroll", () => {
    clearTimeout(mapWarmTimer);
    mapWarmTimer = setTimeout(warmSelection, 100);
  });
  $("#previous-unit").onclick = () => {
    const options = [...$("#unit-select").options],
      i = options.findIndex((o) => o.value === unit.key);
    pickUnit(options[(i - 1 + options.length) % options.length]?.value);
  };
  $("#next-unit").onclick = () => {
    const options = [...$("#unit-select").options],
      i = options.findIndex((o) => o.value === unit.key);
    pickUnit(options[(i + 1) % options.length]?.value);
  };
  function setMode(next) {
    halt();
    mode = next;
    for (const [id, m] of [
      ["candidate-mode", "candidate"],
      ["current-mode", "current"],
    ]) {
      $(`#${id}`).classList.toggle("selected", m === mode);
      $(`#${id}`).setAttribute("aria-pressed", String(m === mode));
    }
    renderUnit();
  }
  $("#candidate-mode").onclick = () => setMode("candidate");
  $("#current-mode").onclick = () => setMode("current");
  $("#current-mode").textContent = "Original game";
  $("#sequence").onclick = () =>
    play(
      currentRows()
        .filter(
          (a) =>
            ["tap", "move", "hit"].includes(a.key) ||
            a.key.startsWith("attack-"),
        )
        .map(chosenFor)
        .filter(Boolean),
    );
  function persist() {
    try {
      localStorage.setItem(
        "hex-sound-favorites",
        JSON.stringify([...favorites]),
      );
    } catch {
      $("#playing").textContent =
        "Favorites only saved in this tab; export to keep them.";
    }
  }
  const packSelect = $("#pack");
  for (const [id, label] of Object.entries(packNames).sort((a, b) =>
    a[1].localeCompare(b[1]),
  )) {
    const o = el("option", "", label);
    o.value = id;
    packSelect.append(o);
  }
  function renderLibrary(reset = false) {
    if (reset) visibleCount = 60;
    const q = $("#search").value.trim().toLowerCase(),
      p = $("#pack").value,
      c = $("#category").value,
      only = $("#favorites-only").checked;
    const items = catalog.filter(
      (x) =>
        (p === "all" || x.pack === p) &&
        (c === "all" || x.categories?.includes(c)) &&
        (!only || favorites.has(x.id)) &&
        `${x.name} ${x.packLabel} ${x.creator} ${x.categories?.join(" ")} ${kindNames[x.kind]}`
          .toLowerCase()
          .includes(q),
    );
    libraryMatches = items;
    updateLibraryControls();
    $("#results").textContent =
      `Showing ${Math.min(visibleCount, items.length)} of ${items.length} RECORDINGS · ${items.reduce((sum, x) => sum + (x.takes?.length || 1), 0)} TAKES${only ? " · FAVORITES" : ""}`;
    const root = $("#library-list");
    root.replaceChildren();
    if (!items.length)
      root.append(
        el("p", "empty", "No sounds match. Change the search or filters."),
      );
    for (const item of items.slice(0, visibleCount)) {
      const row = el("div", "library-row");
      row.dataset.sound = item.id;
      const p = el("button", "play", "▶");
      p.setAttribute("aria-label", `Play ${pretty(item.name)}`);
      p.onclick = () => play([{ id: item.id, take: 0 }]);
      const info = el("div"),
        title = el("button", "library-title", pretty(item.name));
      title.setAttribute("aria-expanded", String(expanded.has(item.id)));
      title.title = "Show all takes and full recording";
      title.onclick = () => {
        if (expanded.has(item.id)) expanded.delete(item.id);
        else expanded.add(item.id);
        renderLibrary();
      };
      info.append(
        title,
        el(
          "small",
          "library-meta",
          `${item.packLabel} · ${time(item.duration || 0)} · ${item.takes?.length || 1} take${item.takes?.length === 1 ? "" : "s"}`,
        ),
        el(
          "small",
          "library-meta kind-label",
          kindNames[item.kind] || item.kind,
        ),
      );
      const dl = el("a", "download-file", "↓");
      dl.href = item.path;
      dl.download = item.path.split("/").at(-1);
      dl.setAttribute("aria-label", `Download ${item.name}`);
      const star = el(
        "button",
        `favorite${favorites.has(item.id) ? " saved" : ""}`,
        favorites.has(item.id) ? "★" : "☆",
      );
      star.setAttribute("aria-label", `Favorite ${item.name}`);
      star.setAttribute("aria-pressed", String(favorites.has(item.id)));
      star.onclick = () => {
        if (favorites.has(item.id)) favorites.delete(item.id);
        else favorites.add(item.id);
        persist();
        renderLibrary();
      };
      row.append(p, info, dl, star);
      if (expanded.has(item.id)) {
        const panel = el("div", "takes-panel");
        panel.append(
          el(
            "p",
            "",
            `${item.license} · ${item.creator || "Hexcommand"}${item.sourcePath ? ` · Original: ${item.sourcePath}` : ""}`,
          ),
        );
        const full = el("button", "", "▶ Full recording");
        full.onclick = () => play([{ id: item.id, take: null }]);
        panel.append(full);
        (item.takes || []).forEach((take, i) => {
          const b = el(
            "button",
            "",
            `▶ Take ${i + 1} · ${take.duration.toFixed(2)}s`,
          );
          b.onclick = () => play([{ id: item.id, take: i }]);
          panel.append(b);
        });
        if (item.source?.startsWith("https://")) {
          const link = el("a", "", "Source & license ↗");
          link.href = item.source;
          link.target = "_blank";
          link.rel = "noopener";
          panel.append(link);
        }
        row.append(panel);
      }
      root.append(row);
    }
    if (items.length > visibleCount) {
      const more = el(
        "button",
        "page-more",
        `Show more (${items.length - visibleCount} remaining)`,
      );
      more.onclick = () => {
        visibleCount += 60;
        renderLibrary();
      };
      root.append(more);
    }
  }
  $("#library-play-all").onclick = () => {
    if (libraryPlayback) halt();
    else if (libraryMatches.length)
      void play(
        libraryMatches.map((item) => ({ id: item.id, take: null })),
        { libraryBatch: true },
      );
  };
  $("#show-all-recordings").onclick = () => {
    visibleCount = catalog.length;
    renderLibrary();
  };
  $("#search").oninput = () => renderLibrary(true);
  $("#pack").onchange = () => renderLibrary(true);
  $("#category").onchange = () => renderLibrary(true);
  $("#favorites-only").onchange = () => renderLibrary(true);
  function showView(name) {
    document.body.dataset.view = name;
    cache.cancelWarm();
    document.querySelectorAll(".tabs [data-view]").forEach((tab) => {
      const active = tab.dataset.view === name;
      tab.classList.toggle("active", active);
      tab.setAttribute("aria-selected", String(active));
    });
    for (const view of ["units", "library", "sources"])
      $(`#${view}-view`).hidden =
        view === "units" ? !["units", "terrain"].includes(name) : view !== name;
    if (["units", "terrain"].includes(name)) {
      halt();
      entityMode = name === "terrain" ? "terrain" : "unit";
      $("#context-label").textContent =
        entityMode === "terrain" ? "CHOOSE TERRAIN" : "CHOOSE A FACTION";
      $("#factions").hidden = entityMode === "terrain";
      $("#entity-search").value = "";
      const list =
        entityMode === "terrain" ? workbench.terrains : activeFaction.units;
      const key = entityMode === "terrain" ? rememberedTerrain : rememberedUnit;
      unit = list.find((x) => x.key === key) || list[0];
      populateUnits();
    }
    if (name === "library") renderLibrary();
  }
  for (const tab of document.querySelectorAll(".tabs [data-view]"))
    tab.onclick = () => showView(tab.dataset.view);
  $("#browse-new").onclick = (e) => {
    e.preventDefault();
    showView("library");
  };
  $("#export").onclick = () => {
    const data = {
      entity: unit.key,
      assignments: Object.entries(assignments).map(([context, choice]) => ({
        context,
        ...choice,
        recording: byId.get(choice.id),
      })),
      exportedAt: new Date().toISOString(),
      favorites: catalog.filter((x) => favorites.has(x.id)),
    };
    const url = URL.createObjectURL(
      new Blob([JSON.stringify(data, null, 2)], { type: "application/json" }),
    );
    const link = el("a");
    link.href = url;
    link.download = "hexcommand-sound-favorites.json";
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  for (const pack of sourceInfo.packs) {
    const entries = catalog.filter(
      (x) =>
        x.source === pack.source_url && x.pack !== "current" && !x.derivedFrom,
    );
    const card = el("article");
    card.append(
      el("span", "tag", `READY TO PLAY · CC0 · ${entries.length} RECORDINGS`),
      el("h3", "", pack.id.split("/")[1].replace(/-/g, " ")),
      el("p", "", pack.creator),
      el("p", "", `${kindNames[pack.kind] || pack.kind}. ${pack.notes || ""}`),
    );
    const link = el("a", "", "Original source & license ↗");
    link.href = pack.source_url;
    link.target = "_blank";
    link.rel = "noopener";
    card.append(link);
    const browse = el("button", "primary", "Listen to this pack");
    browse.onclick = () => {
      packSelect.value = entries[0]?.pack || "all";
      $("#category").value = "all";
      $("#search").value = "";
      $("#favorites-only").checked = false;
      visibleCount = 60;
      showView("library");
    };
    card.append(browse);
    $("#source-cards").append(card);
  }
  const edits = catalog.filter((x) => x.derivedFrom);
  if (edits.length) {
    const card = el("article");
    card.append(
      el("span", "tag", `PROCESSED AUDITIONS · ${edits.length}`),
      el("h3", "", "Listening edits"),
      el(
        "p",
        "",
        "Non-destructive audition copies. Each retains its original source/hash and a reproducible recipe. Intentionally quieter edits are not boosted by Match levels.",
      ),
    );
    const link = el("a", "", "Processing recipes ↓");
    link.href = "audio-edits.json";
    link.download = "audio-edits.json";
    card.append(link);
    const browse = el("button", "primary", "Compare audition edits");
    browse.onclick = () => {
      packSelect.value = "audition-edits";
      $("#category").value = "all";
      $("#search").value = "";
      $("#favorites-only").checked = false;
      showView("library");
    };
    card.append(browse);
    $("#source-cards").append(card);
  }
  for (let i = 0; i < 49; i++) {
    const bar = el("b");
    bar.style.height = `${8 + Math.abs(Math.sin(i * 2.1) * Math.sin(i * 0.17)) * 65}px`;
    $(".signal").append(bar);
  }
  $("#total").textContent = catalog.length;
  $("#library-count").textContent = catalog.length;
  drawWave();
  // Rebind the explicitly trimmed horse cue and retire rejected saved swings once.
  try {
    if (!localStorage.getItem("hex-sound-curation-migration-v1")) {
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        const entity = entityByKey.get(key.slice(0, key.lastIndexOf("/")));
        const sound = byId.get(choice.id);
        if (!entity || !sound) continue;
        if (entity.kind === "terrain") {
          const oldMagic = [
            "edits/ruins-magic-soft-v1",
            "edits/grove-magic-soft-v1",
          ];
          if (key === "terrain/ruins/tap" && oldMagic.includes(choice.id)) {
            backup[key] = choice;
            delete assignments[key];
          } else if (
            key === "terrain/grove/tap" &&
            choice.id === "edits/grove-magic-soft-v1"
          ) {
            backup[key] = choice;
            assignments[key] = {
              ...choice,
              id: "edits/ruins-magic-soft-v1",
              take: 0,
            };
          }
          continue;
        }
        if (entity.kind !== "unit") continue;
        if (
          choice.id === "denischardonnet/horse-vocals/0286.mp3" &&
          /\/(tap|voice)$/.test(key)
        ) {
          backup[key] = choice;
          assignments[key] = {
            ...choice,
            id: "edits/horse-0286-trim-v1",
            take: 0,
          };
        } else if (
          /^(ghost|spectre|wraith)$/.test(entity.id) &&
          /\/(tap|voice)$/.test(key) &&
          sound.pack === "rpg-audio" &&
          /^(cloth[234]|handleSmallLeather[2]?)$/.test(sound.name)
        ) {
          backup[key] = choice;
          assignments[key] = {
            ...choice,
            id: "edits/ghost-grunt-02-trim-v1",
            take: 0,
          };
        } else if (
          sound.pack === "still-north-media/weapon-textures" &&
          / Swing$/.test(sound.name)
        ) {
          backup[key] = choice;
          delete assignments[key];
        }
      }
      if (Object.keys(backup).length) {
        localStorage.setItem(
          "hex-sound-before-curation-v1",
          JSON.stringify(backup),
        );
        saveAssignments();
      }
      localStorage.setItem("hex-sound-curation-migration-v1", "true");
    }
  } catch {}
  // Remap saved indices by source-time overlap after splitting merged bow takes.
  try {
    if (!localStorage.getItem("hex-sound-bow-review-v1")) {
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        const item = byId.get(choice.id);
        const old = item?.previousTakes?.[choice.take];
        if (choice.take == null || !old) continue;
        backup[key] = choice;
        let best = 0, score = -Infinity;
        item.takes.forEach((take, index) => {
          const overlap = Math.max(0, Math.min(old.start + old.duration,
            take.start + take.duration) - Math.max(old.start, take.start));
          const value = overlap / take.duration - Math.abs(take.start - old.start) * 0.0001;
          if (value > score) { score = value; best = index; }
        });
        assignments[key] = { ...choice, take: best };
      }
      localStorage.setItem("hex-sound-before-bow-review-v1", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-bow-review-v1", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-short-fout-v1")) {
      const replacements = {
        "qubodup/3-dark-magic-spells/dark_magic/fout-01.flac": "edits/fout-01-short-v1",
        "edits/ritual-fout-03-soft-v1": "edits/ritual-fout-03-short-v1",
      };
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        if (!replacements[choice.id] || choice.take === null) continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: replacements[choice.id], take: 0 };
      }
      localStorage.setItem("hex-sound-before-short-fout-v1", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-short-fout-v1", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-terrain-recut-v2")) {
      const replacements = {
        "edits/forest-book-03-soft-v1": "edits/forest-book-03-short-v1",
        "edits/burned-fire-soft-v1": "edits/burned-fire-recut-v2",
      };
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        if (!replacements[choice.id] || !key.startsWith("terrain/")) continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: replacements[choice.id], take: 0 };
      }
      localStorage.setItem("hex-sound-before-terrain-recut-v2", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-terrain-recut-v2", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-tree-selection-v1")) {
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        if (!/^woodland\/(?:treant(?:-l[23])?|wildkeeper|grovewarden)\/(?:tap|voice)$/.test(key)
          || choice.id !== "rubberduck/80-cc0-creature-sfx/grunt_04.ogg") continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: "rubberduck/80-cc0-creature-sfx/roar_03.ogg", take: 0 };
      }
      localStorage.setItem("hex-sound-before-tree-selection-v1", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-tree-selection-v1", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-wildkeeper-cloth-v1")) {
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        if (!/^woodland\/(?:wildkeeper|grovewarden)(?:-l[23])?\/(?:tap|voice)$/.test(key)
          || !/\/(grunt_04|roar_03)\.ogg$/.test(choice.id)) continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: "rpg-audio/cloth3", take: 0 };
      }
      localStorage.setItem("hex-sound-before-wildkeeper-cloth-v1", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-wildkeeper-cloth-v1", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-deer-rider-v1")) {
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        if (!/^beasts\/stag(?:-l[23])?\//.test(key)
          || choice.id !== "edits/deer-trot-short-v1") continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: "edits/horse-0286-trim-v1", take: 0 };
      }
      localStorage.setItem("hex-sound-before-deer-rider-v1", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-deer-rider-v1", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-soft-onsets-v1")) {
      const replacements = {
        "rpg-audio/knifeSlice2": "edits/knife-slice-2-no-hit-v1",
        "rubberduck/80-cc0-rpg-sfx/metal_01.ogg": "edits/metal-01-soft-onset-v1",
        "edits/burned-fire-recut-v2": "edits/burned-fire-soft-onset-v3",
      };
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        const rider = /^skeletons\/skeleton-rider(?:-l[23])?\/attack-spear$/.test(key)
          && choice.id === "edits/spear-sabre-one-shot-v1";
        const replacement = rider ? "rubberduck/80-cc0-rpg-sfx/blade_01.ogg"
          : replacements[choice.id];
        if (!replacement) continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: replacement, take: 0 };
      }
      localStorage.setItem("hex-sound-before-soft-onsets-v1", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-soft-onsets-v1", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-grove-short-v2")) {
      const key = "terrain/grove/tap", choice = assignments[key];
      if (choice && ["edits/ruins-magic-soft-v1", "edits/grove-magic-soft-v1"].includes(choice.id)) {
        localStorage.setItem("hex-sound-before-grove-short-v2", JSON.stringify({ [key]: choice }));
        assignments[key] = { ...choice, id: "edits/grove-short-magic-v2", take: 0 };
        saveAssignments();
      }
      localStorage.setItem("hex-sound-grove-short-v2", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-snake-texture-v2")) {
      const backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        if (!/^beasts\/(?:giant-snake|sea-serpent)(?:-l[23])?\//.test(key)
          || choice.id !== "edits/snake-belt-soft-v1") continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: "edits/snake-belt-texture-v2", take: 0 };
      }
      localStorage.setItem("hex-sound-before-snake-texture-v2", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-snake-texture-v2", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-four-deaths-v1")) {
      const backup = {}, allowed = new Set(Object.values(DEATH_SOUNDS));
      for (const [key, choice] of Object.entries(assignments)) {
        if (!key.endsWith("/death") || allowed.has(choice.id)) continue;
        const entity = entityByKey.get(key.slice(0, key.lastIndexOf("/")));
        if (entity?.kind !== "unit") continue;
        const death = buildActions(entity, catalog).find(row => row.key === "death");
        backup[key] = choice;
        assignments[key] = { ...choice, id: death.suggested[0], take: 0 };
      }
      for (const [key, choice] of Object.entries(assignments)) {
        if (!key.includes("/attack-") || choice.id !== "edits/metal-01-soft-onset-v1") continue;
        const entity = entityByKey.get(key.slice(0, key.lastIndexOf("/")));
        if (!entity) continue;
        const action = buildActions(entity, catalog).find(row => row.key === key.slice(key.lastIndexOf("/") + 1));
        if (action?.suggested[0] !== "edits/knife-slice-2-no-hit-v1") continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: action.suggested[0], take: 0 };
      }
      localStorage.setItem("hex-sound-before-four-deaths-v1", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-four-deaths-v1", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-death-materials-v2")) {
      const previous = new Set([
        "rubberduck/80-cc0-creature-sfx/grunt_01.ogg",
        "rubberduck/80-cc0-creature-sfx/eat_01.ogg",
        "edits/ghost-grunt-02-trim-v1",
        "impact-sounds/impactPlank_medium_004",
        "rpg-audio/metalPot3",
      ]), backup = {};
      for (const [key, choice] of Object.entries(assignments)) {
        if (!key.endsWith("/death") || !previous.has(choice.id)) continue;
        const entity = entityByKey.get(key.slice(0, key.lastIndexOf("/")));
        if (entity?.kind !== "unit") continue;
        const death = buildActions(entity, catalog).find(row => row.key === "death");
        if (choice.id === death.suggested[0]) continue;
        backup[key] = choice;
        assignments[key] = { ...choice, id: death.suggested[0], take: 0 };
      }
      localStorage.setItem("hex-sound-before-death-materials-v2", JSON.stringify(backup));
      saveAssignments();
      localStorage.setItem("hex-sound-death-materials-v2", "true");
    }
  } catch {}
  try {
    if (!localStorage.getItem("hex-sound-shieldguard-pot-v1")) {
      const key = "dwarves/dwarf-axeguard-l2/death", choice = assignments[key];
      if (choice?.id === "rubberduck/80-cc0-creature-sfx/cough_02.ogg") {
        localStorage.setItem("hex-sound-before-shieldguard-pot-v1", JSON.stringify({ [key]: choice }));
        assignments[key] = { ...choice, id: "rpg-audio/metalPot3", take: 0 };
        saveAssignments();
      }
      localStorage.setItem("hex-sound-shieldguard-pot-v1", "true");
    }
  } catch {}
  setFaction(activeFaction.id);
  // Decode ahead while suspended. The first ordinary tap resumes audio and plays.
  try {
    ensureContext();
    audioEnabled = true;
    warmSelection();
  } catch (error) {
    $("#playing").textContent = error.message;
  }
}
