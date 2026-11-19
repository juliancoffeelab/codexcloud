//! Native ownership and typed OCaml transport over the battle engine.
use crate::data::Text;
use crate::sort::SharedSort;
use crate::{
    ai::{self, PlanningWorkspace},
    data::*,
    logic::{BattleSession, battle::Board},
    protocol::*,
};
use hex_command_battle::model::{DispatchUpdateProjection, TerrainUpdateProjection};
#[cfg(test)]
use ocamljs_bindgen::{Encode, EncodeAs};
use ocamljs_bindgen::{Encoded, ocamljs_bindgen};
use std::cell::RefCell;
use std::{
    collections::{BTreeMap, HashMap},
    rc::Rc,
};
// The SDK codec stays in the binding module; the shared BorrowFields derive is
// identical to upstream and remains independent of a host representation.
struct NativeFrame<'a>(&'a State);
impl<'a> From<&'a State> for NativeFrame<'a> {
    fn from(state: &'a State) -> Self {
        Self(state)
    }
}
impl ocamljs_bindgen::EncodeAs<StateFrame> for NativeFrame<'_> {
    fn encode_as(self, out: &mut ocamljs_bindgen::Encoder) {
        let State {
            map_modifiers,
            lighting,
            vision_groups,
            factions,
            ground_items,
            armies,
            condition_memory,
            map_id,
            map_revision,
            cols,
            rows,
            round,
            turn,
            gold,
            winner,
            next_id,
            combat_rng,
            log,
            units,
            campaign,
            last_combat,
            tiles: _,
        } = self.0;
        out.nested(|out| {
            ocamljs_bindgen::EncodeRef::encode_ref(map_modifiers, out);
            ocamljs_bindgen::EncodeRef::encode_ref(lighting, out);
            ocamljs_bindgen::EncodeRef::encode_ref(vision_groups, out);
            ocamljs_bindgen::EncodeRef::encode_ref(factions, out);
            ocamljs_bindgen::EncodeRef::encode_ref(ground_items, out);
            ocamljs_bindgen::EncodeRef::encode_ref(armies, out);
            ocamljs_bindgen::EncodeRef::encode_ref(condition_memory, out);
            ocamljs_bindgen::EncodeRef::encode_ref(map_id, out);
            ocamljs_bindgen::EncodeRef::encode_ref(map_revision, out);
            ocamljs_bindgen::EncodeRef::encode_ref(cols, out);
            ocamljs_bindgen::EncodeRef::encode_ref(rows, out);
            ocamljs_bindgen::EncodeRef::encode_ref(round, out);
            ocamljs_bindgen::EncodeRef::encode_ref(turn, out);
            ocamljs_bindgen::EncodeRef::encode_ref(gold, out);
            ocamljs_bindgen::EncodeRef::encode_ref(winner, out);
            ocamljs_bindgen::EncodeRef::encode_ref(next_id, out);
            ocamljs_bindgen::EncodeRef::encode_ref(combat_rng, out);
            ocamljs_bindgen::EncodeRef::encode_ref(log, out);
            ocamljs_bindgen::EncodeRef::encode_ref(units, out);
            ocamljs_bindgen::EncodeRef::encode_ref(campaign, out);
            ocamljs_bindgen::EncodeRef::encode_ref(last_combat, out);
        });
    }
}
enum TerrainDelta {
    Keep,
    Patch(Vec<usize>),
    Replace,
}
struct Applied {
    terrain: TerrainDelta,
    presentation: BattlefieldPresentation,
    combat: Option<CombatAnimation>,
    inspection: Option<BattleInspection>,
}
struct ChangedTiles<'a> {
    tiles: &'a [Tile],
    indices: std::vec::IntoIter<usize>,
}
impl<'a> Iterator for ChangedTiles<'a> {
    type Item = &'a Tile;
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|i| &self.tiles[i])
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}
impl ExactSizeIterator for ChangedTiles<'_> {}
type TerrainProjection<'a> =
    TerrainUpdateProjection<ocamljs_bindgen::SequenceProjection<ChangedTiles<'a>>, &'a Vec<Tile>>;
fn terrain_projection(tiles: &Vec<Tile>, delta: TerrainDelta) -> TerrainProjection<'_> {
    match delta {
        TerrainDelta::Keep => TerrainUpdateProjection::Keep,
        TerrainDelta::Patch(indices) => TerrainUpdateProjection::Patch {
            tiles: ocamljs_bindgen::SequenceProjection(ChangedTiles {
                tiles,
                indices: indices.into_iter(),
            }),
        },
        TerrainDelta::Replace => TerrainUpdateProjection::Replace { tiles },
    }
}
/// The live roster stays owned here between operations. A loan only binds rules;
/// dropping it returns the same runtime data, including caches and continuation scope.
struct RuntimeLoan<'a> {
    slot: std::cell::RefMut<'a, Option<hex_command_battle::BattleState>>,
    game: Option<BattleSession<'a>>,
}
impl<'a> RuntimeLoan<'a> {
    fn new(
        slot: &'a RefCell<Option<hex_command_battle::BattleState>>,
        rules: &'a Catalogue,
    ) -> Self {
        let mut slot = slot.borrow_mut();
        let runtime = slot.take().expect("non-reentrant battle operation");
        Self {
            slot,
            game: Some(BattleSession::from_runtime(rules, runtime)),
        }
    }
}
impl<'a> std::ops::Deref for RuntimeLoan<'a> {
    type Target = BattleSession<'a>;
    fn deref(&self) -> &Self::Target {
        self.game.as_ref().unwrap()
    }
}
impl std::ops::DerefMut for RuntimeLoan<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.game.as_mut().unwrap()
    }
}
impl Drop for RuntimeLoan<'_> {
    fn drop(&mut self) {
        *self.slot = Some(self.game.take().unwrap().into_runtime());
    }
}
/// Only terrain publication needs retained delta state; live units stay in runtime storage.
struct PublishedTerrain {
    map_id: Text,
    map_revision: u32,
    cols: i32,
    rows: i32,
    tiles: Vec<Tile>,
    board: Rc<Board>,
}
impl PublishedTerrain {
    fn new(game: &BattleSession<'_>) -> Self {
        Self {
            map_id: game.map_id().to_owned(),
            map_revision: game.map_revision(),
            cols: game.cols(),
            rows: game.rows(),
            tiles: game.tiles(),
            board: game.board_handle(),
        }
    }
}
struct Battle {
    runtime: RefCell<Option<hex_command_battle::BattleState>>,
    terrain: PublishedTerrain,
    seed: u32,
    revision: u32,
    workspace: Option<PlanningWorkspace>,
    options: Option<AiOptions>,
    pending: Option<Command>,
}
impl Battle {
    fn game<'a>(&'a self, rules: &'a Catalogue) -> RuntimeLoan<'a> {
        RuntimeLoan::new(&self.runtime, rules)
    }
    fn snapshot(&self, rules: &Catalogue) -> State {
        self.game(rules)
            .state_with_tiles(self.terrain.tiles.clone())
    }
    fn view(&self, rules: &Catalogue) -> EngineView {
        let game = self.game(rules);
        EngineView {
            presentation: game.presentation(),
            revision: self.revision,
            state: game.state_with_tiles(self.terrain.tiles.clone()),
        }
    }
    fn check(&self, revision: u32) -> Result<(), EngineError> {
        if revision != self.revision {
            Err(EngineError::StaleRevision {
                expected: revision,
                actual: self.revision,
            })
        } else {
            Ok(())
        }
    }
}

#[ocamljs_bindgen::ocamljs_bindgen(opaque)]
pub struct NativeEngineFactory {
    rules: Catalogue,
    battles: HashMap<u32, Battle>,
    next_id: u32,
}

#[ocamljs_bindgen(module=NativeEngineFactory,name="create")]
pub fn new_factory() -> Result<NativeEngineFactory, EngineError> {
    #[cfg(target_arch = "wasm32")]
    {
        factory_with_rules(Ok(Catalogue::embedded()))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        factory_with_rules(Catalogue::load())
    }
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn with_documents(documents: BTreeMap<Text, Text>) -> Result<NativeEngineFactory, EngineError> {
    #[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
    {
        factory_with_rules(Catalogue::from_json_bundle(&documents))
    }
    #[cfg(all(target_arch = "wasm32", not(feature = "runtime-catalog")))]
    {
        let _ = documents;
        Err(EngineError::InvalidRequest {
            message: "Custom catalogs require a runtime-catalog build".into(),
        })
    }
}
fn factory_with_rules(
    rules: Result<Catalogue, ContentError>,
) -> Result<NativeEngineFactory, EngineError> {
    let rules = rules.map_err(|error| EngineError::Content {
        path: error.path,
        message: error.message,
    })?;
    Ok(NativeEngineFactory {
        rules,
        battles: HashMap::new(),
        next_id: 1,
    })
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn catalog(factory: &NativeEngineFactory) -> CatalogView {
    let wiki = factory.rules.wiki();
    let map_view = |map: &MapDefinition| {
        let game = BattleSession::from_map(&factory.rules, map);
        BrowserMap {
            id: map.id().into(),
            revision: map.revision(),
            name: map.name().into(),
            description: map.description().into(),
            tags: map.tags().to_vec(),
            state: game.state(),
            presentation: game.presentation(),
        }
    };
    CatalogView {
        abilities: wiki.abilities().collect(),
        items: wiki.items().collect(),
        units: wiki.units().collect(),
        terrain: wiki.terrains().collect(),
        races: wiki.races().collect(),
        environment_effects: wiki
            .environment_effects()
            .map(|effect| BrowserEnvironment {
                id: effect.id().into(),
                name: effect.name().into(),
                description: effect.description().into(),
                effects: effect.effects().to_vec(),
            })
            .collect(),
        maps: wiki.maps().map(&map_view).collect(),
        campaigns: wiki
            .campaigns()
            .map(|campaign| BrowserCampaign {
                id: campaign.id().into(),
                name: campaign.name().into(),
                description: campaign.description().into(),
                tagline: campaign.tagline().into(),
                short_description: campaign.short_description().into(),
                race: campaign.race(),
                leader: campaign.leader_id().into(),
                tags: campaign.tags().to_vec(),
                characters: campaign.characters().to_vec(),
                ending: campaign.ending().to_vec(),
                carryover: campaign.carryover().clone(),
                missions: campaign
                    .chapters()
                    .iter()
                    .map(|chapter| {
                        let story = chapter.story();
                        let scripts = chapter.scripts();
                        let map = factory.rules.map(chapter.map_id(), chapter.map_revision());
                        BrowserChapter {
                            id: chapter.id().into(),
                            map: map_view(map),
                            subtitle: story.subtitle.clone(),
                            briefing: story.briefing.clone(),
                            debrief: story.debrief.clone(),
                            hints: story.hints.clone(),
                            intro: story.intro.clone(),
                            victory: story.victory.clone(),
                            scripted_events: scripts.events.clone(),
                            required_companions: scripts.required_companions.clone(),
                            arrivals: map.roster().arrivals.clone(),
                            enemy_groups: scripts.enemy_groups.clone(),
                        }
                    })
                    .collect(),
            })
            .collect(),
    }
}

#[ocamljs_bindgen(module=NativeEngineFactory,name="new_battle")]
pub fn create(
    factory: &mut NativeEngineFactory,
    start: Start,
    seed: u32,
) -> Result<BattleHandle, EngineError> {
    let mut game = factory.start(start)?;
    game.set_combat_rng(seed);
    let runtime = game.into_runtime();
    Ok(factory.insert_runtime(runtime, seed))
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn restore(
    factory: &mut NativeEngineFactory,
    saved: SavedGame,
) -> Result<BattleHandle, EngineError> {
    factory.restore_saved(saved)
}
/// Data-only JSON request, decoded at the untrusted persistence boundary.
#[ocamljs_bindgen(module = NativeEngineFactory)]
pub fn validate_undo(
    factory: &NativeEngineFactory,
    request: &TextSlice,
) -> Result<(), EngineError> {
    let request: UndoValidation =
        serde_json::from_str(request).map_err(|error| EngineError::InvalidSave {
            message: error.to_string(),
        })?;
    crate::undo::validate(&factory.rules, request)
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn view(
    factory: &NativeEngineFactory,
    id: u32,
    revision: u32,
) -> Result<EngineView, EngineError> {
    factory
        .battle(id, revision)
        .map(|battle| battle.view(&factory.rules))
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn inspect(
    factory: &NativeEngineFactory,
    id: u32,
    revision: u32,
) -> Result<BattleInspection, EngineError> {
    factory.battle(id, revision).map(|battle| {
        let game = battle.game(&factory.rules);
        crate::ai::policies::combat::inspect(game.view(), game.inspect(revision))
    })
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn preview_unit(
    factory: &NativeEngineFactory,
    id: u32,
    revision: u32,
    kind: Text,
    faction: Text,
    tile: Hex,
) -> Result<hex_command_battle::model::UnitDetails, EngineError> {
    let battle = factory.battle(id, revision)?;
    let game = battle.game(&factory.rules);
    let view = game.view();
    let kind = factory
        .rules
        .units()
        .position(|definition| definition.id() == kind)
        .ok_or_else(|| EngineError::InvalidRequest {
            message: format!("Unknown unit type: {kind}"),
        })?;
    let tile = *view
        .board()
        .indices
        .get(&tile)
        .ok_or_else(|| EngineError::InvalidRequest {
            message: "Unknown projection tile".into(),
        })?;
    if view.faction(&faction).is_none() {
        return Err(EngineError::InvalidRequest {
            message: format!("Unknown faction: {faction}"),
        });
    }
    Ok(game.preview_unit_details(kind, view.faction_identity(&faction), tile))
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn geometry(
    factory: &NativeEngineFactory,
    id: u32,
    revision: u32,
) -> Result<BoardInspection, EngineError> {
    factory
        .battle(id, revision)
        .map(|battle| battle.game(&factory.rules).inspect_board())
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn dispatch(
    factory: &mut NativeEngineFactory,
    id: u32,
    revision: u32,
    command: Command,
) -> Result<DispatchResult, EngineError> {
    let applied = factory.apply(id, revision, command, false)?;
    let battle = &factory.battles[&id];
    Ok(DispatchResult {
        view: EngineView {
            revision: battle.revision,
            state: battle.snapshot(&factory.rules),
            presentation: applied.presentation,
        },
        combat: applied.combat,
    })
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn dispatch_update(
    factory: &mut NativeEngineFactory,
    id: u32,
    revision: u32,
    command: Command,
    include_inspection: bool,
) -> Encoded<Result<DispatchUpdate, EngineError>> {
    let applied = factory.apply(id, revision, command, include_inspection);
    let state = applied.as_ref().ok().map(|_| {
        factory.battles[&id]
            .game(&factory.rules)
            .state_with_tiles(Vec::new())
    });
    let update = applied.map(|applied| {
        let battle = &factory.battles[&id];
        DispatchUpdateProjection {
            base_revision: revision,
            revision: battle.revision,
            state: NativeFrame::from(state.as_ref().unwrap()),
            terrain: terrain_projection(&battle.terrain.tiles, applied.terrain),
            presentation: applied.presentation,
            combat: applied.combat,
            inspection: applied.inspection,
        }
    });
    Encoded::new(ocamljs_bindgen::ResultProjection(update))
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn choose_ai_command(
    factory: &mut NativeEngineFactory,
    id: u32,
    revision: u32,
    options: AiOptions,
) -> Result<AiDecision, EngineError> {
    factory.decision(id, revision, options)
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn save(
    factory: &NativeEngineFactory,
    id: u32,
    revision: u32,
) -> Result<SavedGame, EngineError> {
    factory.checkpoint(id, revision)
}
#[ocamljs_bindgen(module=NativeEngineFactory,js_value("string"))]
pub fn save_json(
    factory: &NativeEngineFactory,
    id: u32,
    revision: u32,
) -> Result<Text, EngineError> {
    #[derive(serde::Serialize)]
    struct Checkpoint<'a> {
        seed: u32,
        state: &'a State,
    }
    let battle = factory.battle(id, revision)?;
    let state = battle.snapshot(&factory.rules);
    Ok(serde_json::to_string(&Checkpoint {
        seed: battle.seed,
        state: &state,
    })
    .expect("serializable checkpoint"))
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn dispose_battle(factory: &mut NativeEngineFactory, id: u32) -> Result<(), EngineError> {
    factory
        .battles
        .remove(&id)
        .map(|_| ())
        .ok_or(EngineError::UnknownBattle)
}
/// Decode persistence through the same Rust serde contract used by native restoration.
#[ocamljs_bindgen]
pub fn checkpoint_from_json(value: Text) -> Result<SavedGame, EngineError> {
    serde_json::from_str(&value).map_err(|error| EngineError::InvalidSave {
        message: error.to_string(),
    })
}

#[ocamljs_bindgen]
pub fn checkpoint_to_json(value: SavedGame) -> Text {
    serde_json::to_string(&value).expect("serializable checkpoint")
}

#[ocamljs_bindgen]
pub fn state_to_json(value: State) -> Text {
    serde_json::to_string(&value).expect("serializable state")
}
#[ocamljs_bindgen]
pub fn unit_actions_to_json(value: UnitActions) -> Text {
    serde_json::to_string(&value).expect("serializable actions")
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn preview_presentation(
    factory: &NativeEngineFactory,
    input: crate::logic::conditions::PresentationInput,
) -> BattlefieldPresentation {
    crate::logic::conditions::resolve(&factory.rules, &input)
}
impl NativeEngineFactory {
    fn apply(
        &mut self,
        id: u32,
        revision: u32,
        command: Command,
        include_inspection: bool,
    ) -> Result<Applied, EngineError> {
        let battle = self
            .battles
            .get_mut(&id)
            .ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        let mut game = RuntimeLoan::new(&battle.runtime, &self.rules);
        let ends_turn = matches!(command, Command::EndTurn);
        let planned = battle.pending.as_ref() == Some(&command);
        if planned {
            let faction = battle
                .workspace
                .as_ref()
                .and_then(PlanningWorkspace::proposal_faction)
                .map(str::to_owned);
            game.set_active_faction(faction);
        }
        let applied = game.execute(command);
        game.set_active_faction(None);
        applied?;
        let terrain = if terrain_replaced(&battle.terrain, &game) {
            battle.terrain = PublishedTerrain::new(&game);
            TerrainDelta::Replace
        } else {
            let changed = game.patch_tiles(&mut battle.terrain.tiles);
            battle.terrain.board = game.board_handle();
            if changed.is_empty() {
                TerrainDelta::Keep
            } else {
                TerrainDelta::Patch(changed)
            }
        };
        battle.revision = battle
            .revision
            .checked_add(1)
            .expect("battle revision exhausted");
        battle.pending = None;
        if ends_turn || !planned {
            battle.workspace = None;
            battle.options = None;
        }
        let presentation = game.presentation();
        let inspection = if include_inspection {
            game.refresh_mission();
            Some(crate::ai::policies::combat::inspect(
                game.view(),
                game.inspect(battle.revision),
            ))
        } else {
            None
        };
        let combat = game.take_combat_animation();
        Ok(Applied {
            terrain,
            presentation,
            combat,
            inspection,
        })
    }

    fn decision(
        &mut self,
        id: u32,
        revision: u32,
        options: AiOptions,
    ) -> Result<AiDecision, EngineError> {
        let battle = self
            .battles
            .get_mut(&id)
            .ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        if options
            .recruitment_limit
            .is_some_and(|limit| limit > 1_000_000)
        {
            return Err(EngineError::InvalidRequest {
                message: "AI recruitment limit is out of range".into(),
            });
        }
        if battle.options.as_ref() != Some(&options) {
            battle.workspace = Some(PlanningWorkspace::new(
                battle.seed,
                options
                    .recruitment_limit
                    .or_else(|| options.policy.default_recruitment_limit()),
                None,
                options.diplomacy,
            ));
            if let Some(blocks) = options.blocks {
                battle.workspace = battle.workspace.take().map(|w| w.with_blocks(blocks));
            }
            battle.options = Some(options.clone());
            battle.pending = None;
        }
        if let Some(command) = &battle.pending {
            return Ok(AiDecision {
                revision: battle.revision,
                command: command.clone(),
            });
        }
        let game = RuntimeLoan::new(&battle.runtime, &self.rules);
        let command = ai::choose_game_command(
            &game,
            options.policy,
            battle.workspace.as_mut().expect("workspace initialized"),
        )?;
        let command = game.command_snapshot(&command)?;
        battle.pending = Some(command.clone());
        Ok(AiDecision {
            revision: battle.revision,
            command,
        })
    }
    fn checkpoint(&self, id: u32, revision: u32) -> Result<SavedGame, EngineError> {
        let battle = self.battle(id, revision)?;
        Ok(SavedGame {
            seed: battle.seed,
            state: battle.snapshot(&self.rules),
        })
    }
    fn restore_saved(&mut self, saved: SavedGame) -> Result<BattleHandle, EngineError> {
        let game = BattleSession::try_from_state(&self.rules, saved.state).map_err(|error| {
            EngineError::InvalidSave {
                message: error.to_string(),
            }
        })?;
        let runtime = game.into_runtime();
        Ok(self.insert_runtime(runtime, saved.seed))
    }

    fn battle(&self, id: u32, revision: u32) -> Result<&Battle, EngineError> {
        let battle = self.battles.get(&id).ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        Ok(battle)
    }
    fn insert_runtime(
        &mut self,
        runtime: hex_command_battle::BattleState,
        seed: u32,
    ) -> BattleHandle {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("battle handle exhausted");
        let game = BattleSession::from_runtime(&self.rules, runtime);
        let terrain = PublishedTerrain::new(&game);
        let runtime = game.into_runtime();
        let battle = Battle {
            runtime: RefCell::new(Some(runtime)),
            terrain,
            seed,
            revision: 0,
            workspace: None,
            options: None,
            pending: None,
        };
        let view = battle.view(&self.rules);
        self.battles.insert(id, battle);
        BattleHandle { id, view }
    }
    fn start(&self, start: Start) -> Result<BattleSession<'_>, EngineError> {
        match start {
            Start::Scenario {
                map,
                revision,
                armies,
                first,
            } => {
                let definition = self
                    .rules
                    .maps()
                    .filter(|definition| {
                        definition.id() == map
                            && revision.is_none_or(|expected| definition.revision() == expected)
                    })
                    .max_by_key(|definition| definition.revision())
                    .ok_or_else(|| EngineError::UnknownMap { map: map.clone() })?;
                let mut state = BattleSession::from_map(&self.rules, definition).state();
                if armies.is_none() && state.units.is_empty() {
                    return Err(EngineError::InvalidRequest {
                        message: "Choose both races to start this scenario".into(),
                    });
                }
                if let Some(armies) = armies {
                    if map == "developer" {
                        return Err(EngineError::InvalidRequest {
                            message: "Developer armies are authored".into(),
                        });
                    }
                    let place_leaders = state.units.is_empty();
                    for (seat, faction) in ["player", "enemy"].into_iter().enumerate() {
                        let roster = self
                            .rules
                            .meta()
                            .simulation
                            .rosters
                            .iter()
                            .find(|r| r.id == armies[seat])
                            .ok_or_else(|| EngineError::InvalidRequest {
                                message: "Unknown faction".into(),
                            })?;
                        if place_leaders {
                            crate::logic::place_selected_leader(
                                &self.rules,
                                &mut state,
                                faction,
                                &roster.leader,
                            )
                            .map_err(|message| {
                                EngineError::InvalidRequest {
                                    message: message.into(),
                                }
                            })?;
                            continue;
                        }
                        for unit in state.units.iter_mut().filter(|u| u.faction == faction) {
                            let original = self.rules.unit(&unit.r#type);
                            let kind = if unit.role == Some(Role::Leader) {
                                roster.leader.as_str()
                            } else if roster.id == "human" || roster.pool.contains(&unit.r#type) {
                                unit.r#type.as_str()
                            } else {
                                roster
                                    .pool
                                    .iter()
                                    // Preserve the existing composition policy: prefer the same
                                    // role, otherwise use the first authored pool candidate.
                                    .min_by_key(|id| self.rules.unit(id).role() != original.role())
                                    .ok_or_else(|| EngineError::InvalidRequest {
                                        message: "Faction has no unit for the starting army".into(),
                                    })?
                                    .as_str()
                            };
                            let rule = self.rules.unit(kind);
                            unit.r#type = kind.to_owned();
                            unit.hp = rule.max_hp;
                            unit.experience = Some(0);
                        }
                    }
                    state.armies = Some(armies);
                }
                if let Some(first) = first {
                    state.turn = first;
                }
                Ok(BattleSession::from_state(&self.rules, state))
            }
            Start::Campaign { id } => {
                if !self.rules.campaigns().any(|campaign| campaign.id() == id) {
                    return Err(EngineError::UnknownCampaign { id });
                }
                Ok(BattleSession::start_campaign(
                    &self.rules,
                    &id,
                    0,
                    vec![],
                    0,
                    vec![],
                    vec![],
                    vec![],
                ))
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_dispatch_matches_owned_codec_and_includes_coherent_inspection() {
        let mut factory = new_factory().unwrap();
        let battle = mirror(&mut factory);
        let applied = factory.apply(battle.id, 0, Command::EndTurn, true).unwrap();
        let state = &factory.battles[&battle.id].snapshot(&factory.rules);
        let expected_inspection = inspect(&factory, battle.id, 1).unwrap();
        assert_eq!(
            serde_json::to_value(&applied.inspection).unwrap(),
            serde_json::to_value(Some(&expected_inspection)).unwrap()
        );
        // All three terrain forms borrow the authoritative graph, including
        // nested authored Arc rules, and retain the owned SDK wire contract.
        for terrain in [
            TerrainDelta::Keep,
            TerrainDelta::Patch(vec![0]),
            TerrainDelta::Replace,
        ] {
            let owned_terrain = match &terrain {
                TerrainDelta::Keep => TerrainUpdate::Keep,
                TerrainDelta::Patch(indices) => TerrainUpdate::Patch {
                    tiles: indices.iter().map(|&i| state.tiles[i].clone()).collect(),
                },
                TerrainDelta::Replace => TerrainUpdate::Replace {
                    tiles: state.tiles.clone(),
                },
            };
            let owned = DispatchUpdate {
                base_revision: 0,
                revision: 1,
                state: StateFrame::from(state),
                terrain: owned_terrain,
                presentation: applied.presentation.clone(),
                combat: applied.combat.clone(),
                inspection: applied.inspection.clone(),
            };
            let projection = DispatchUpdateProjection {
                base_revision: 0u32,
                revision: 1u32,
                state: NativeFrame::from(state),
                terrain: terrain_projection(&state.tiles, terrain),
                presentation: applied.presentation.clone(),
                combat: applied.combat.clone(),
                inspection: applied.inspection.clone(),
            };
            let mut expected = ocamljs_bindgen::Encoder::new();
            owned.encode(&mut expected);
            let mut actual = ocamljs_bindgen::Encoder::new();
            projection.encode_as(&mut actual);
            assert_eq!(actual.bytes, expected.bytes);
        }
        let checkpoint = save_json(&factory, battle.id, 1).unwrap();
        assert!(
            factory
                .apply(
                    battle.id,
                    1,
                    Command::Move {
                        unit: UnitId(u32::MAX),
                        tile: Hex::new(0, 0).into()
                    },
                    true
                )
                .is_err()
        );
        assert_eq!(checkpoint, save_json(&factory, battle.id, 1).unwrap());
        assert_eq!(
            serde_json::to_value(inspect(&factory, battle.id, 1).unwrap()).unwrap(),
            serde_json::to_value(expected_inspection).unwrap()
        );
    }

    #[test]
    fn checkpoint_discards_unexecuted_plans_and_restores_resolved_state() {
        let mut factory = NativeEngineFactory {
            rules: Catalogue::embedded(),
            battles: HashMap::new(),
            next_id: 1,
        };
        let state = BattleSession::new(&factory.rules, "mirror").state();
        let runtime = BattleSession::from_state(&factory.rules, state.clone()).into_runtime();
        let original = factory.insert_runtime(runtime, 17);
        let options = AiOptions {
            blocks: None,
            policy: Policy::Balanced,
            diplomacy: Diplomacy::Diplomatic,
            recruitment_limit: Some(2),
        };
        factory.decision(original.id, 0, options).unwrap();
        let saved = factory.checkpoint(original.id, 0).unwrap();
        let json = serde_json::to_value(&saved).unwrap();
        assert_eq!(json.as_object().unwrap().len(), 2);
        assert_eq!(json["state"], serde_json::to_value(&state).unwrap());
        let restored = factory.restore_saved(saved).unwrap();
        let battle = &factory.battles[&restored.id];
        assert!(battle.workspace.is_none() && battle.pending.is_none() && battle.options.is_none());
        assert_eq!(
            serde_json::to_value(battle.snapshot(&factory.rules)).unwrap(),
            json["state"]
        );
        let mut old = json;
        old["planner"] = serde_json::json!({});
        assert!(serde_json::from_value::<SavedGame>(old).is_err());
    }
    fn mirror(factory: &mut NativeEngineFactory) -> BattleHandle {
        create(
            factory,
            Start::Scenario {
                map: "mirror".into(),
                revision: None,
                armies: None,
                first: None,
            },
            17,
        )
        .unwrap()
    }

    #[test]
    fn static_queries_are_detached_and_excluded_from_dynamic_publications() {
        let mut factory = new_factory().unwrap();
        let original = mirror(&mut factory);
        let board = geometry(&factory, original.id, 0).unwrap();
        let dynamic = inspect(&factory, original.id, 0).unwrap();
        let json = serde_json::to_value(&dynamic).unwrap();
        assert!(json.get("objectives").is_some());
        assert!(json.get("costs").is_none() && json.get("neighbors").is_none());
        let battle = &factory.battles[&original.id];
        for (index, id) in battle.terrain.board.ids.iter().enumerate() {
            assert_eq!(
                board.neighbors[&id.to_string()],
                battle.terrain.board.adjacent[index]
                    .iter()
                    .map(|&next| battle.terrain.board.ids[next].clone())
                    .collect::<Vec<_>>()
            );
        }
        dispatch(&mut factory, original.id, 0, Command::EndTurn).unwrap();
        let next = geometry(&factory, original.id, 1).unwrap();
        assert_eq!(next.neighbors, board.neighbors);
        assert!(matches!(
            geometry(&factory, original.id, 0),
            Err(EngineError::StaleRevision { actual: 1, .. })
        ));
        dispose_battle(&mut factory, original.id).unwrap();
        assert!(matches!(
            geometry(&factory, original.id, 1),
            Err(EngineError::UnknownBattle)
        ));
        // Previously returned static data owns no live battle references.
        assert_eq!(next.neighbors, board.neighbors);
    }

    #[test]
    fn geometry_preserves_adjacency_after_reordered_or_repainted_restore() {
        let mut factory = new_factory().unwrap();
        let original = mirror(&mut factory);
        let expected = geometry(&factory, original.id, 0).unwrap();
        let mut saved = save(&factory, original.id, 0).unwrap();
        saved.state.tiles.reverse();
        saved.state.tiles[0].terrain = "plains".into();
        let restored = restore(&mut factory, saved).unwrap();
        let actual = geometry(&factory, restored.id, 0).unwrap();
        assert_eq!(
            actual.neighbors.keys().collect::<Vec<_>>(),
            expected.neighbors.keys().collect::<Vec<_>>()
        );
        for (tile, neighbors) in actual.neighbors {
            let mut actual = neighbors;
            let mut expected = expected.neighbors[&tile].clone();
            actual.sort();
            expected.sort();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn typed_custom_catalog_keeps_static_rules_and_topology_factory_local() {
        let mut default = new_factory().unwrap();
        let original = mirror(&mut default);
        let mut documents = crate::catalogue_fixture::bundle();
        let mut terrain: serde_json::Value =
            serde_json::from_str(&documents["game/terrain.json"]).unwrap();
        terrain
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["id"] == "forest")
            .unwrap()["cost"] = 3.into();
        documents.insert(
            "game/terrain.json".into(),
            serde_json::to_string(&terrain).unwrap(),
        );
        let path = "game/scenario/mirror-v1/map.json";
        let mut map: serde_json::Value = serde_json::from_str(&documents[path]).unwrap();
        let removed = Hex::new(0, 0).to_string();
        map["tiles"]
            .as_array_mut()
            .unwrap()
            .retain(|tile| tile["col"] != 0 || tile["row"] != 0);
        documents.insert(path.into(), serde_json::to_string(&map).unwrap());
        let mut custom = with_documents(documents).unwrap();
        let custom_battle = mirror(&mut custom);
        assert_ne!(
            inspect(&default, original.id, 0).unwrap().units[0]
                .details
                .terrain
                .iter()
                .map(|t| t.cost)
                .collect::<Vec<_>>(),
            inspect(&custom, custom_battle.id, 0).unwrap().units[0]
                .details
                .terrain
                .iter()
                .map(|t| t.cost)
                .collect::<Vec<_>>()
        );
        let old_topology = geometry(&default, original.id, 0).unwrap();
        let new_topology = geometry(&custom, custom_battle.id, 0).unwrap();
        assert!(old_topology.neighbors.contains_key(&removed));
        assert!(!new_topology.neighbors.contains_key(&removed));
        assert!(
            matches!(with_documents(BTreeMap::new()), Err(EngineError::Content { path, .. }) if path == "manifest.json")
        );
    }

    #[test]
    fn direct_checkpoint_json_matches_native_save_and_checks_live_revision() {
        let mut factory = new_factory().unwrap();
        let original = mirror(&mut factory);
        dispatch(&mut factory, original.id, 0, Command::EndTurn).unwrap();
        let saved = save(&factory, original.id, 1).unwrap();
        let json = save_json(&factory, original.id, 1).unwrap();
        assert_eq!(json, checkpoint_to_json(saved.clone()));
        let decoded: SavedGame = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            serde_json::to_value(&saved).unwrap()
        );
        assert_eq!(
            state_to_json(saved.state.clone()),
            serde_json::to_string(&saved.state).unwrap()
        );
        let action = inspect(&factory, original.id, 1).unwrap().units.remove(0);
        assert_eq!(
            unit_actions_to_json(action.clone()),
            serde_json::to_string(&action).unwrap()
        );
        assert!(matches!(
            save_json(&factory, original.id, 0),
            Err(EngineError::StaleRevision { actual: 1, .. })
        ));
        dispose_battle(&mut factory, original.id).unwrap();
        assert!(matches!(
            save_json(&factory, original.id, 1),
            Err(EngineError::UnknownBattle)
        ));
    }
}

fn terrain_replaced(old: &PublishedTerrain, game: &BattleSession<'_>) -> bool {
    old.map_id != game.map_id()
        || old.map_revision != game.map_revision()
        || old.cols != game.cols()
        || old.rows != game.rows()
        || old.tiles.len() != game.board().ids.len()
        || (!Rc::ptr_eq(&old.board, &game.board_handle())
            && old
                .tiles
                .iter()
                .zip(&game.board().ids)
                .any(|(a, b)| a.hex() != *b))
}
