import { hex } from "./hex-fixture.mjs";
import { afterEach, afterAll } from "vitest";
import * as Native from "./native-adapter.mjs";
import * as Catalog from "./catalog-adapter.mjs";
import { required } from "./helpers.mjs";
import * as Native2 from "./native-adapter.mjs";
import * as Catalog2 from "./catalog-adapter.mjs";
import * as Presentation from "./presentation-adapter.mjs";
import * as Story from "./story-adapter.mjs";
const factory = await Native.create_factory();
const catalog = Native.catalog(factory);
const handles = /* @__PURE__ */ new Set();
function board(start = { kind: "scenario", map: "mirror" }) {
  const battle = Native.create(factory, start, 17);
  handles.add(battle);
  const fixture = {
    battle,
    get view() {
      return Native.view(battle);
    },
    get state() {
      return Native.state(Native.view(battle));
    },
    edit(change) {
      const checkpoint = structuredClone(Native.save(battle));
      change(checkpoint.state);
      Native.replace(battle, checkpoint);
      return fixture.view;
    },
    add(type, side, tile) {
      let unit;
      fixture.edit((state) => {
        unit = {
          experience: 0,
          id: `u${state.nextId++}`,
          type,
          side,
          tile: hex(tile),
          hp: 1,
          moved: false,
          attacked: false,
        };
        unit.hp = Catalog.unit(catalog, type).hp;
        state.units.push(unit);
      });
      return required(Native.unit(fixture.view, unit.id));
    },
  };
  return fixture;
}
function campaign(id, chapter = 0) {
  const fixture = board({ kind: "campaign", id });
  for (let index = 0; index < chapter; index++) {
    fixture.edit((state) => {
      state.winner = "player";
      state.gold.player = 0;
      if (state.campaign) state.campaign.dialogueIndex = state.campaign.dialogue.length;
    });
    Native.dispatch(fixture.battle, { kind: "recallMission", units: [] });
  }
  return fixture;
}
afterEach(() => {
  for (const battle of handles) Native.dispose(battle);
  handles.clear();
});
afterAll(() => Native.dispose_factory(factory));
export {
  Catalog2 as Catalog,
  Native2 as Native,
  Presentation,
  Story,
  board,
  campaign,
  catalog,
  factory,
};
