import { buildActions } from "./matching.js?v=material-picks-1";
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
  const [catalog, workbench, sourceInfo] = await Promise.all([
    loadJSON("catalog.json?v=full-collection-1"),
    loadJSON("workbench.json?v=factions-1"),
    loadJSON("sources.json"),
  ]);
  const byId = new Map(catalog.map((item) => [item.id, item]));
  const kindNames = {
    "recorded-foley": "Recorded Foley",
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
  const buffers = new Map(),
    expanded = new Set();
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
    const rows = buildActions(unit, catalog, { extras: $("#extras").checked });
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
    return rows.map((row) => {
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
  function chosenFor(action) {
    const saved = assignments[keyFor(action)];
    return saved && byId.has(saved.id)
      ? saved
      : action.suggested[0]
        ? { id: action.suggested[0], take: 0 }
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
    cancelAnimationFrame(frame);
    request++;
    controller?.abort();
    controller = null;
    if (source) {
      const active = source;
      source = null;
      try {
        active.stop();
      } catch {}
    }
    if (reset) {
      selection = null;
      playingId = null;
      $("#playing").textContent = "Ready when you are.";
      $("#seek").value = 0;
      $("#seek").disabled = true;
      $("#time").textContent = "0:00 / 0:00";
      drawWave();
    }
  }
  async function unlock() {
    if (!context) {
      const C = window.AudioContext || window.webkitAudioContext;
      if (!C) throw Error("This browser does not support Web Audio");
      context = new C({ latencyHint: "interactive" });
      master = context.createGain();
      master.gain.value = Number($("#volume").value) / 100;
      master.connect(context.destination);
    }
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
  async function getBuffer(item, signal) {
    let pending = buffers.get(item.id);
    if (!pending) {
      pending = fetch(item.path, { signal })
        .then((r) => {
          if (!r.ok) throw Error(`Audio returned HTTP ${r.status}`);
          return r.arrayBuffer();
        })
        .then((b) => context.decodeAudioData(b));
      buffers.set(item.id, pending);
      pending.catch(() => {
        if (buffers.get(item.id) === pending) buffers.delete(item.id);
      });
      while (buffers.size > 6) buffers.delete(buffers.keys().next().value);
    }
    return pending;
  }
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
        gain = context.createGain();
      node.buffer = buffer;
      gain.gain.value = $("#normalize").checked
        ? 10 ** ((byId.get(playingId)?.gainDb || 0) / 20)
        : 1;
      node.connect(gain);
      gain.connect(master);
      source = node;
      beganAt = context.currentTime;
      resumedAt = position;
      node.onended = () => {
        node.onended = null;
        node.disconnect();
        gain.disconnect();
        if (source === node) source = null;
        resolve();
      };
      node.start(0, start + position, Math.max(0.001, duration - position));
      tick();
    });
  }
  async function play(list) {
    halt();
    const ticket = request;
    const abort = new AbortController();
    controller = abort;
    try {
      await unlock();
      for (const ref of list) {
        if (ticket !== request) return;
        const item = byId.get(ref.id);
        if (!item) throw Error("Sound not found");
        $("#playing").textContent = `Loading ${pretty(item.name)}…`;
        const buffer = await getBuffer(item, abort.signal);
        if (ticket !== request) return;
        const take = ref.take === null ? null : item.takes?.[ref.take || 0];
        const start = take?.start || 0,
          duration = Math.min(
            take?.duration || buffer.duration,
            buffer.duration - start,
          );
        selection = { buffer, start, duration, ref };
        playingId = item.id;
        $("#seek").max = duration;
        $("#seek").value = 0;
        $("#seek").disabled = false;
        $("#playing").textContent =
          `${pretty(item.name)}${take && item.takes.length > 1 ? ` · take ${(ref.take || 0) + 1}` : ""}`;
        drawWave(buffer);
        await startNode(buffer, start, duration, 0, ticket);
      }
      if (ticket === request) {
        cancelAnimationFrame(frame);
        $("#seek").value = selection?.duration || 0;
        $("#time").textContent =
          `${time(selection?.duration || 0)} / ${time(selection?.duration || 0)}`;
        $("#playing").textContent += " · finished";
      }
    } catch (error) {
      if (ticket === request)
        $("#playing").textContent = `${error.message}. Tap again to retry.`;
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
    if (document.hidden) halt();
  });
  $("#volume").oninput = (e) => {
    const value = Number(e.target.value);
    $("#volume-value").textContent = `${value}%`;
    if (master)
      master.gain.setTargetAtTime(value / 100, context.currentTime, 0.015);
  };
  $("#normalize").onchange = () => {
    if (selection) play([selection.ref]);
  };
  function pickUnit(key) {
    const list =
      entityMode === "terrain" ? workbench.terrains : activeFaction.units;
    const next = list.find((u) => u.key === key);
    if (!next) return;
    halt();
    unit = next;
    if (entityMode === "terrain") rememberedTerrain = key;
    else rememberedUnit = key;
    renderUnit();
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
    const saved = assignments[keyFor(action)];
    if (saved?.id === id)
      takes.value = saved.take === null ? "full" : String(saved.take || 0);
    const selectedRef = () => ({
      id,
      take: takes.value === "full" ? null : Number(takes.value),
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
    if (!matches.length) {
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
    for (const u of matches
      .filter((x) => entityMode === "terrain" || x.level === 1)
      .slice(0, 10)) {
      const button = el("button");
      button.dataset.unit = u.key;
      const image = el("img");
      image.src = u.artwork;
      image.alt = "";
      image.width = 64;
      image.height = 64;
      button.append(image, el("span", "", u.name));
      button.onclick = () => pickUnit(u.key);
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
  $("#show-all-recordings").onclick = () => {
    visibleCount = catalog.length;
    renderLibrary();
  };
  $("#search").oninput = () => renderLibrary(true);
  $("#pack").onchange = () => renderLibrary(true);
  $("#category").onchange = () => renderLibrary(true);
  $("#favorites-only").onchange = () => renderLibrary(true);
  function showView(name) {
    document.querySelectorAll("[data-view]").forEach((tab) => {
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
  for (const tab of document.querySelectorAll("[data-view]"))
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
      (x) => x.source === pack.source_url && x.pack !== "current",
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
  for (let i = 0; i < 49; i++) {
    const bar = el("b");
    bar.style.height = `${8 + Math.abs(Math.sin(i * 2.1) * Math.sin(i * 0.17)) * 65}px`;
    $(".signal").append(bar);
  }
  $("#total").textContent = catalog.length;
  $("#library-count").textContent = catalog.length;
  drawWave();
  setFaction(activeFaction.id);
}
