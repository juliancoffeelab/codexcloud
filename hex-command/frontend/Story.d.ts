/** Campaign display policy over native catalog metadata and detached state. */
import type { Mission, Dialogue, CampaignState, PublicUnit, State, Goal } from "../rust/pkg/hex_command";
import type { t as Catalog } from "./Catalog";
import type { view as View } from "./Native";
import type { Immutable } from "./Presentation";
export interface chapter { readonly source: Immutable<Mission>; readonly briefing: string; readonly debrief: string | undefined; readonly hints: readonly string[]; readonly intro: readonly Readonly<Dialogue>[]; readonly victory: readonly Readonly<Dialogue>[]; readonly recruits: readonly string[] }
export interface progress { readonly label: string; readonly current: number; readonly total: number; readonly done_: boolean; readonly tiles: readonly string[] }
export function mission(catalog: Catalog, state: Immutable<CampaignState>): chapter;
export function goal_tiles(goal: Immutable<Goal>): readonly string[];
export function progress(catalog: Catalog, view: View, goal: Immutable<Goal>): progress;
export function active_tiles(catalog: Catalog, view: View, goal: Immutable<Goal>): readonly string[];
export function objective_labels(catalog: Catalog, view: View): Readonly<Record<string, readonly string[]>>;
export function dialogue_pending(state: Immutable<CampaignState> | undefined): boolean;
export function returns_with_story(catalog: Catalog, state: Immutable<CampaignState>, unit: Immutable<PublicUnit>): boolean;
export function ends_refuge_mediation(catalog: Catalog, state: Immutable<State>, unit: Immutable<PublicUnit>): boolean;
