/** Small editable checkpoints backed by the actual OCaml native owner. */
import { afterEach, afterAll } from "vitest";
import * as Native from "@frontend/Native";
import * as Catalog from "@frontend/Catalog";
import type { State, Start, PublicUnit, SavedGame } from "../rust/pkg/hex_command";
import { required } from "./helpers";
export * as Native from "@frontend/Native";
export * as Catalog from "@frontend/Catalog";
export * as Presentation from "@frontend/Presentation";
export * as Story from "@frontend/Story";
export const factory = await Native.create_factory();
export const catalog = Native.catalog(factory);
const handles = new Set<Native.battle>();
export function board(start: Start = { kind: "scenario", map: "mirror" }) {
  const battle = Native.create(factory, start, 17);
  handles.add(battle);
  const fixture = {
    battle,
    get view() {
      return Native.view(battle);
    },
    get state(): Native.Immutable<State> {
      return Native.state(Native.view(battle));
    },
    edit(change: (state: State) => void) {
      const checkpoint = structuredClone(Native.save(battle)) as SavedGame;
      change(checkpoint.state);
      Native.replace(battle, checkpoint);
      return fixture.view;
    },
    add(type: string, side: "player" | "enemy", tile: string): Native.Immutable<PublicUnit> {
      let unit!: PublicUnit;
      fixture.edit((state) => {
        unit = { id: `u${state.nextId++}`, type, side, tile, hp: 1, moved: false, attacked: false };
        unit.hp = Catalog.unit(catalog, type).hp;
        state.units.push(unit);
      });
      return required(Native.unit(fixture.view, unit.id));
    },
  };
  return fixture;
}
export function campaign(id: string, chapter = 0) {
  const fixture = board({ kind: "campaign", id });
  for (let index = 0; index < chapter; index++) {
    fixture.edit((state) => {
      state.winner = "player";
      state.gold.player = 0;
      if (state.campaign) state.campaign.dialogueIndex = state.campaign.dialogue.length;
    });
    Native.dispatch(fixture.battle, { kind: "nextMission" });
  }
  return fixture;
}
afterEach(() => {
  for (const battle of handles) Native.dispose(battle);
  handles.clear();
});
afterAll(() => Native.dispose_factory(factory));
