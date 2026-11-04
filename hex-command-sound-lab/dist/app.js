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
  const [catalog, roster, sourceInfo] = await Promise.all([
    loadJSON("catalog.json?v=full-collection-1"),
    loadJSON("roster.json"),
    loadJSON("sources.json"),
  ]);
  const byId = new Map(catalog.map((item) => [item.id, item]));
  const kindNames = {
    "recorded-foley": "Recorded Foley",
    "recorded-weapons": "Recorded weapons",
    "natural-animal-recording": "Natural animal recording",
    "designed-effects": "Designed creature / RPG effect",
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
  const refs = (pack, pattern, limit = 5) =>
    catalog
      .filter((x) => x.pack === pack && pattern.test(x.name))
      .flatMap((x) =>
        (x.takes?.length ? x.takes : [{}]).map((_, take) => ({
          id: x.id,
          take,
        })),
      )
      .slice(0, limit);
  const weapons = (pattern, limit = 5) =>
    refs("still-north-media/weapon-textures", pattern, limit);
  const impacts = (pattern) =>
    refs("still-north-media/weapon-impacts", pattern);
  const kenney = (pattern) => refs("impact-sounds", pattern);
  const rpg = (pattern) => refs("rpg-audio", pattern);
  const spells = (pattern) => refs("rubberduck/80-cc0-rpg-sfx", pattern);
  const beasts = (pattern) => refs("rubberduck/80-cc0-creature-sfx", pattern);
  const baseline = (family) =>
    [1, 2, 3]
      .map((n) => ({ id: `current/${family}-${n}`, take: null }))
      .filter((x) => byId.has(x.id));
  const units = roster.units;
  let unit = units.find((x) => x.id === "soldier") || units[0];
  const mounted = (u) =>
    /horse|rider|knight|lancer/.test(u.id) || u.id === "rider";
  const ghost = (u) => /ghost|spectre|wraith|lich|necromancer/.test(u.id);
  const canine = (u) => /wolf|hound/.test(u.id);
  const undead = (u) =>
    /skeleton|skeletal|bone|revenant|zombie/.test(u.id) || ghost(u);
  const animal = (u) =>
    /wolf|hound|boar|bear|giant-bat|giant-spider/.test(u.id);
  function attackPicks(attack) {
    const name = attack.name.toLowerCase();
    if (/crossbow|arbalest/.test(name))
      return {
        refs: weapons(/^Crossbow Shoot$/),
        note: "Actual crossbow release recordings",
      };
    if (/bow/.test(name))
      return {
        refs: weapons(/^English Longbow Shoot$|^Scythian Recurve Shoot$/),
        note: "Actual longbow and recurve releases",
      };
    if (/fire/.test(name))
      return {
        refs: spells(/^spell_fire_/),
        note: "Designed fire spell effects",
      };
    if (/grave|soul|mire/.test(name))
      return {
        refs: refs("qubodup/3-dark-magic-spells", /fout/),
        note: "Designed dark magic · upstream synthesis",
      };
    if (
      /spark|light|surge|tide|spectral|bone ?burst|crushing (wave|root)/.test(
        name,
      )
    )
      return {
        refs: refs("jaggedstone/magic-spell-sfx", /magical/),
        note: "Designed spellcasting candidates",
      };
    if (/knife/.test(name))
      return {
        refs: weapons(/^Dagger Swing$/),
        note: "Actual dagger swing takes",
      };
    if (/sword|rustblade/.test(name))
      return {
        refs: weapons(/^Sabre Swing$|^Katana Swing$/),
        note: "Actual blade swing takes",
      };
    if (/spear|lance|trident|harpoon|halberd/.test(name))
      return {
        refs: weapons(/^Spear Swing$/),
        note: "Actual spear swing · polearm candidate",
      };
    if (/axe|scythe/.test(name))
      return {
        refs: weapons(/^Axe Swing$/),
        note: "Actual axe swing · scythe material candidate",
      };
    if (/mace|hammer|club/.test(name))
      return {
        refs: impacts(/^Mace /),
        note: "Actual mace-on-weapon contacts",
      };
    if (/staff|bough/.test(name))
      return {
        refs: kenney(/^impactWood_medium/),
        note: "Wood contact candidate for a staff or bough",
      };
    if (/fang|bite|tusk|claw|grasp/.test(name))
      return {
        refs: beasts(/^eat_|^grunt_/),
        note: "Designed mouth performances · not recorded animal attacks",
      };
    if (/trample/.test(name))
      return {
        refs: kenney(/^impactSoft_heavy/),
        note: "Heavy soft impact candidate",
      };
    if (/shield/.test(name))
      return {
        refs: kenney(/^impactPlate_medium/),
        note: "Plate impact candidate for a shield",
      };
    if (/sling|stone|shard|spike|thorn|net/.test(name))
      return {
        refs: weapons(/^Scythian Recurve Arrow Pass$|^Crossbow Bolt Pass$/),
        note: "Recorded projectile pass · texture candidate for this attack",
      };
    return {
      refs: spells(/^spell_0/),
      note: "Designed effect candidate; audition for fit",
    };
  }
  function actionRows() {
    if (mode === "current") {
      const old = ["soldier", "warden"].includes(unit.id)
        ? "unit-archer"
        : ghost(unit) || undead(unit)
          ? "unit-commander"
          : "terrain-plains";
      return [
        {
          key: "move",
          title: "Movement",
          icon: "↗",
          note: "Archived original-game movement family",
          refs: baseline(old),
        },
        ...unit.attacks.map((a) => ({
          key: `attack-${a.id}`,
          title: `Attack · ${a.name}`,
          icon: "⚔",
          note: "Archived original-game attack family",
          refs: baseline(
            a.sound === "attack-bow" ? "attack-bow" : "attack-melee",
          ),
        })),
      ];
    }
    const move = mounted(unit)
      ? refs("ezduzziteh/horse-trotting", /Trot/)
      : kenney(
          ghost(unit)
            ? /^footstep_carpet/
            : undead(unit)
              ? /^footstep_wood/
              : /^footstep_grass/,
        );
    const rows = [
      {
        key: "move",
        title: "Movement",
        icon: "↗",
        note: mounted(unit)
          ? "Horse trot Foley · jacket / bottle / hands"
          : animal(unit)
            ? "Grass footstep Foley · paw movement candidate"
            : "Recorded surface footsteps",
        refs: move,
      },
    ];
    for (const attack of unit.attacks) {
      const picks = attackPicks(attack);
      rows.push({
        key: `attack-${attack.id}`,
        title: `Attack · ${attack.name}`,
        icon: "⚔",
        note: picks.note,
        refs: picks.refs,
      });
    }
    rows.push({
      key: "hit",
      title: "Impact",
      icon: "◇",
      note: animal(unit)
        ? "Soft-body impact candidate"
        : ghost(unit)
          ? "Designed magical contact"
          : "Recorded weapon / material contact",
      refs: animal(unit)
        ? kenney(/^impactSoft_medium/)
        : ghost(unit)
          ? spells(/^spell_0/)
          : impacts(/Spear.*Blade|Sabre.*Blade/),
    });
    if (canine(unit))
      rows.push({
        key: "voice",
        title: "Voice · canine reference",
        icon: "♫",
        note: "Natural domestic dog growls · not wolf recordings",
        refs: refs("qubodup/dog-snarl-grunt-grumble", /dog-/),
      });
    else if (mounted(unit))
      rows.push({
        key: "voice",
        title: "Voice · horse",
        icon: "♫",
        note: "Natural outdoor horse neigh / snort",
        refs: refs("denischardonnet/horse-vocals", /028/),
      });
    else if (ghost(unit))
      rows.push({
        key: "voice",
        title: "Voice · ghost / lich",
        icon: "♫",
        note: "Performed human ghost voice · speech and reactions",
        refs: refs("sorth/ghost-voice-pack", /output/),
      });
    else if (animal(unit))
      rows.push({
        key: "voice",
        title: "Voice · creature design",
        icon: "♫",
        note: "Processed mouth performances · not authentic species calls",
        refs: beasts(/^(roar|grunt|howl)/),
      });
    else if (undead(unit))
      rows.push({
        key: "voice",
        title: "Voice · undead",
        icon: "♫",
        note: "Designed creature reactions",
        refs: beasts(/^hurt_|^monster_/),
      });
    const bow = unit.attacks.some((a) => /bow|arbalest/i.test(a.name));
    const gear = bow
      ? weapons(/Longbow Nock Arrow$|Crossbow Place Bolt$/)
      : unit.attacks.some((a) => /sword|knife|rustblade/i.test(a.name))
        ? weapons(/^Dagger Draw Fast$|^Sabre Draw Fast$/)
        : rpg(/^(cloth|bookFlip)/);
    rows.push({
      key: "gear",
      title: "Equipment & detail",
      icon: "⋮",
      note: bow
        ? "Recorded bow / crossbow loading"
        : "Cloth, pages or weapon handling",
      refs: gear,
    });
    return rows;
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
  function pickUnit(id) {
    halt();
    unit = units.find((u) => u.id === id) || unit;
    renderUnit();
  }
  function renderUnit() {
    $("#unit-role").textContent = `${unit.race} · ${unit.family}`;
    $("#unit-name").textContent = unit.name;
    $("#unit-description").textContent =
      `Audition ${unit.attacks.map((a) => a.name.toLowerCase()).join(", ") || "movement and equipment"} with movement, material and voice candidates. Suggested assignments, ready for your shortlist.`;
    $("#unit-image").src = `units/${unit.id}.png`;
    $("#unit-image").alt = unit.name;
    $("#unit-select").value = unit.id;
    document.querySelectorAll("[data-unit]").forEach((b) => {
      const active = b.dataset.unit === unit.id;
      b.classList.toggle("selected", active);
      b.setAttribute("aria-pressed", String(active));
    });
    const root = $("#action-cards");
    root.replaceChildren();
    for (const action of actionRows()) {
      const card = el("article", "sound-card");
      card.dataset.action = action.key;
      const info = el("div");
      info.append(
        el("h3", "sound-title", action.title),
        el("p", "sound-caption", action.note),
        el(
          "span",
          mode === "current" ? "tag neutral" : "tag",
          `${mode === "current" ? "ORIGINAL GAME" : "CC0"} · ${action.refs.length} PICKS`,
        ),
      );
      card.append(el("span", "action-icon", action.icon), info);
      const controls = el("div", "sound-controls");
      if (action.refs.length) {
        const button = el("button", "play", "▶");
        button.setAttribute(
          "aria-label",
          `Play random ${unit.name} ${action.title}`,
        );
        button.onclick = () =>
          play([action.refs[Math.floor(Math.random() * action.refs.length)]]);
        controls.append(button);
        action.refs.forEach((ref, i) => {
          const b = el("button", "variant", String(i + 1));
          b.title = `${byId.get(ref.id)?.name} · take ${(ref.take || 0) + 1}`;
          b.setAttribute(
            "aria-label",
            `${unit.name} ${action.title}, variation ${i + 1}`,
          );
          b.onclick = () => play([ref]);
          controls.append(b);
        });
        const all = el("button", "all", "All →");
        all.setAttribute("aria-label", `Play all ${action.title} picks`);
        all.onclick = () => play(action.refs);
        controls.append(all);
      }
      card.append(controls);
      root.append(card);
    }
  }
  const quick = [
    "soldier",
    "archer",
    "scout",
    "commander",
    "spearman",
    "adept",
    "necromancer",
    "skeleton",
    "wolf",
    "knight",
  ];
  for (const id of quick) {
    const u = units.find((x) => x.id === id);
    if (!u) continue;
    const b = el("button");
    b.dataset.unit = id;
    const img = el("img");
    img.src = `units/${id}.png`;
    img.alt = "";
    img.width = 64;
    img.height = 64;
    b.append(img, el("span", "", u.name));
    b.onclick = () => pickUnit(id);
    $("#roster").append(b);
  }
  for (const u of [...units].sort((a, b) => a.name.localeCompare(b.name))) {
    const option = el("option", "", `${u.name} · ${u.id}`);
    option.value = u.id;
    $("#unit-select").append(option);
  }
  $("#unit-select").onchange = (e) => pickUnit(e.target.value);
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
      actionRows()
        .filter(
          (a) =>
            a.key === "move" || a.key.startsWith("attack-") || a.key === "hit",
        )
        .map((a) => a.refs[Math.floor(Math.random() * a.refs.length)])
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
      `${items.length} RECORDINGS · ${items.reduce((sum, x) => sum + (x.takes?.length || 1), 0)} TAKES${only ? " · FAVORITES" : ""}`;
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
      $(`#${view}-view`).hidden = view !== name;
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
      unit: unit.id,
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
  renderUnit();
}
