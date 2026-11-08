//! Native ownership and typed OCaml transport over the battle engine.
use crate::sort::SharedSort;
use crate::{
    ai::{self, PlanningWorkspace},
    data::*,
    logic::{
        Game,
        battle::{Board, RuntimeCache},
    },
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
fn terrain_projection(state: &State, delta: TerrainDelta) -> TerrainProjection<'_> {
    match delta {
        TerrainDelta::Keep => TerrainUpdateProjection::Keep,
        TerrainDelta::Patch(indices) => TerrainUpdateProjection::Patch {
            tiles: ocamljs_bindgen::SequenceProjection(ChangedTiles {
                tiles: &state.tiles,
                indices: indices.into_iter(),
            }),
        },
        TerrainDelta::Replace => TerrainUpdateProjection::Replace {
            tiles: &state.tiles,
        },
    }
}
struct Battle {
    runtime_cache: RefCell<RuntimeCache>,
    state: State,
    board: Rc<Board>,
    seed: u32,
    revision: u32,
    workspace: Option<PlanningWorkspace>,
    options: Option<AiOptions>,
    pending: Option<Command>,
}
impl Battle {
    fn game<'catalog>(&self, rules: &'catalog Rules) -> Game<'catalog> {
        let game = self.take_game(rules);
        *self.runtime_cache.borrow_mut() = game.runtime_cache();
        game
    }
    fn take_game<'catalog>(&self, rules: &'catalog Rules) -> Game<'catalog> {
        Game::from_borrowed_state(
            rules,
            &self.state,
            self.board.clone(),
            self.runtime_cache.take(),
        )
    }
    fn view(&self, rules: &Rules) -> EngineView {
        EngineView {
            presentation: self.game(rules).presentation(),
            revision: self.revision,
            state: self.state.clone(),
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
    rules: Rules,
    battles: HashMap<u32, Battle>,
    next_id: u32,
}

#[ocamljs_bindgen(module=NativeEngineFactory,name="create")]
pub fn new_factory() -> Result<NativeEngineFactory, EngineError> {
    #[cfg(target_arch = "wasm32")]
    {
        factory_with_rules(Ok(crate::data::baked_rules()))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        factory_with_rules(Rules::load())
    }
}
#[ocamljs_bindgen(module=NativeEngineFactory)]
pub fn with_documents(
    documents: BTreeMap<String, String>,
) -> Result<NativeEngineFactory, EngineError> {
    #[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
    {
        factory_with_rules(Rules::from_json_bundle(&documents))
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
    rules: Result<Rules, ContentError>,
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
    let mut campaigns: Vec<_> = factory.rules.campaigns.values().cloned().collect();
    campaigns.order_by(|a, b| a.id.cmp(&b.id));
    let mut items: Vec<_> = factory.rules.items.values().cloned().collect();
    items.order_by(|a, b| a.id.cmp(&b.id));
    CatalogView {
        items,
        units: factory.rules.units.clone(),
        terrain: factory.rules.terrain_definitions.clone(),
        races: factory.rules.race_definitions.clone(),
        maps: factory.rules.map_definitions.clone(),
        campaigns,
    }
}
#[ocamljs_bindgen(module=NativeEngineFactory,name="new_battle")]
pub fn create(
    factory: &mut NativeEngineFactory,
    start: Start,
    seed: u32,
) -> Result<BattleHandle, EngineError> {
    let mut state = factory.start(start)?;
    state.combat_rng = seed;
    let board = Board::new(&state.tiles);
    let game = Game::from_state_with_board(&factory.rules, state, board.clone());
    let cache = game.runtime_cache();
    let tiles = game.tiles();
    Ok(factory.insert(game.into_state(tiles), seed, board, cache))
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
pub fn validate_undo(factory: &NativeEngineFactory, request: &str) -> Result<(), EngineError> {
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
            state: battle.state.clone(),
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
    let update = applied.map(|applied| {
        let battle = &factory.battles[&id];
        DispatchUpdateProjection {
            base_revision: revision,
            revision: battle.revision,
            state: NativeFrame::from(&battle.state),
            terrain: terrain_projection(&battle.state, applied.terrain),
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
) -> Result<String, EngineError> {
    #[derive(serde::Serialize)]
    struct Checkpoint<'a> {
        seed: u32,
        state: &'a State,
    }
    let battle = factory.battle(id, revision)?;
    Ok(serde_json::to_string(&Checkpoint {
        seed: battle.seed,
        state: &battle.state,
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
pub fn checkpoint_from_json(value: String) -> Result<SavedGame, EngineError> {
    serde_json::from_str(&value).map_err(|error| EngineError::InvalidSave {
        message: error.to_string(),
    })
}

#[ocamljs_bindgen]
pub fn checkpoint_to_json(value: SavedGame) -> String {
    serde_json::to_string(&value).expect("serializable checkpoint")
}

#[ocamljs_bindgen]
pub fn state_to_json(value: State) -> String {
    serde_json::to_string(&value).expect("serializable state")
}
#[ocamljs_bindgen]
pub fn unit_actions_to_json(value: UnitActions) -> String {
    serde_json::to_string(&value).expect("serializable actions")
}
#[ocamljs_bindgen]
pub fn preview_presentation(
    input: crate::logic::conditions::PresentationInput,
) -> BattlefieldPresentation {
    crate::logic::conditions::resolve(&input).0
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
        let mut game = battle.take_game(&self.rules);
        let ends_turn = matches!(command, Command::EndTurn);
        let planned = battle.pending.as_ref() == Some(&command);
        game.execute(command)?;
        let replaced = terrain_replaced(&battle.state, &battle.board, &game);
        let (tiles, terrain) = if replaced {
            (game.tiles(), TerrainDelta::Replace)
        } else {
            let changed = game.patch_tiles(&mut battle.state.tiles);
            let terrain = if changed.is_empty() {
                TerrainDelta::Keep
            } else {
                TerrainDelta::Patch(changed)
            };
            (std::mem::take(&mut battle.state.tiles), terrain)
        };
        battle.board = game.board_handle();
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
        *battle.runtime_cache.borrow_mut() = game.runtime_cache();
        let combat = game.take_combat_animation();
        battle.state = game.into_state(tiles);
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
        let game = battle.game(&self.rules);
        let command = ai::choose_game_command(
            &game,
            options.policy,
            battle.workspace.as_mut().expect("workspace initialized"),
        )?;
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
            state: battle.state.clone(),
        })
    }
    fn restore_saved(&mut self, saved: SavedGame) -> Result<BattleHandle, EngineError> {
        let game = Game::try_from_state(&self.rules, saved.state).map_err(|error| {
            EngineError::InvalidSave {
                message: error.to_string(),
            }
        })?;
        let cache = game.runtime_cache();
        let board = game.board_handle();
        let tiles = game.tiles();
        Ok(self.insert(game.into_state(tiles), saved.seed, board, cache))
    }

    fn battle(&self, id: u32, revision: u32) -> Result<&Battle, EngineError> {
        let battle = self.battles.get(&id).ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        Ok(battle)
    }
    fn insert(
        &mut self,
        state: State,
        seed: u32,
        board: Rc<Board>,
        cache: RuntimeCache,
    ) -> BattleHandle {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("battle handle exhausted");
        let battle = Battle {
            runtime_cache: RefCell::new(cache),
            state,
            board,
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
    fn start(&self, start: Start) -> Result<State, EngineError> {
        match start {
            Start::Scenario {
                map,
                revision,
                armies,
                first,
            } => {
                let mut state = self
                    .rules
                    .maps
                    .iter()
                    .filter(|((id, rev), _)| {
                        id == &map && revision.is_none_or(|expected| expected == *rev)
                    })
                    .max_by_key(|((_, revision), _)| revision)
                    .map(|(_, state)| state.clone())
                    .ok_or(EngineError::UnknownMap { map: map.clone() })?;
                if let Some(armies) = armies {
                    if map == "developer" {
                        return Err(EngineError::InvalidRequest {
                            message: "Developer armies are authored".into(),
                        });
                    }
                    for side in [Side::Player, Side::Enemy] {
                        let roster = self
                            .rules
                            .rosters
                            .iter()
                            .find(|r| r.id == armies[side.index()])
                            .ok_or_else(|| EngineError::InvalidRequest {
                                message: "Unknown faction".into(),
                            })?;
                        for unit in state.units.iter_mut().filter(|u| u.side == side) {
                            let original = &self.rules.units[self.rules.types[&unit.r#type]];
                            let kind = if unit.role == Some(Role::Leader) {
                                roster.leader.as_str()
                            } else if roster.id == "human" || roster.pool.contains(&unit.r#type) {
                                unit.r#type.as_str()
                            } else {
                                roster
                                    .pool
                                    .iter()
                                    .find(|id| {
                                        self.rules.units[self.rules.types[*id]].role
                                            == original.role
                                    })
                                    .or_else(|| roster.pool.first())
                                    .ok_or_else(|| EngineError::InvalidRequest {
                                        message: "Faction has no unit for the starting army".into(),
                                    })?
                                    .as_str()
                            };
                            let rule = &self.rules.units[self.rules.types[kind]];
                            unit.r#type = kind.to_owned();
                            unit.hp = rule.hp;
                            unit.experience = Some(0);
                        }
                    }
                    state.armies = Some(armies);
                }
                state.turn = first.unwrap_or(Side::Player);
                Ok(state)
            }
            Start::Campaign { id } => {
                if !self.rules.campaigns.contains_key(&id) {
                    return Err(EngineError::UnknownCampaign { id });
                }
                Ok(
                    Game::start_campaign(&self.rules, &id, 0, vec![], 0, vec![], vec![], vec![])
                        .state(),
                )
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
        let state = &factory.battles[&battle.id].state;
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
                terrain: terrain_projection(state, terrain),
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
            rules: Rules::embedded().unwrap(),
            battles: HashMap::new(),
            next_id: 1,
        };
        let state = Game::new(&factory.rules, "mirror").state();
        let original = factory.insert(
            state.clone(),
            17,
            Board::new(&state.tiles),
            RuntimeCache::default(),
        );
        let options = AiOptions {
            blocks: None,
            policy: AiPolicy::Balanced,
            diplomacy: AiDiplomacy::Diplomatic,
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
        assert_eq!(serde_json::to_value(&battle.state).unwrap(), json["state"]);
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
        let costs = geometry(&factory, original.id, 0).unwrap().costs;
        for rule in &factory.rules.units {
            for (terrain, cost) in factory.rules.terrain_names.iter().zip(&rule.costs) {
                assert_eq!(costs[&rule.id][terrain], *cost);
            }
        }
        let board = geometry(&factory, original.id, 0).unwrap();
        let dynamic = inspect(&factory, original.id, 0).unwrap();
        let json = serde_json::to_value(&dynamic).unwrap();
        assert!(json.get("objectives").is_some());
        assert!(json.get("costs").is_none() && json.get("neighbors").is_none());
        let battle = &factory.battles[&original.id];
        for (index, id) in battle.board.ids.iter().enumerate() {
            assert_eq!(
                board.neighbors[&id.to_string()],
                battle.board.adjacent[index]
                    .iter()
                    .map(|&next| battle.board.ids[next].clone())
                    .collect::<Vec<_>>()
            );
        }
        dispatch(&mut factory, original.id, 0, Command::EndTurn).unwrap();
        assert_eq!(geometry(&factory, original.id, 1).unwrap().costs, costs);
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
        let mut documents = crate::data::catalog::embedded_bundle();
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
            geometry(&default, original.id, 0).unwrap().costs,
            geometry(&custom, custom_battle.id, 0).unwrap().costs
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

fn terrain_replaced(old: &State, board: &Rc<Board>, game: &Game<'_>) -> bool {
    old.map_id != game.map_id()
        || old.map_revision != game.map_revision()
        || old.cols != game.cols()
        || old.rows != game.rows()
        || old.tiles.len() != game.board().ids.len()
        || (!Rc::ptr_eq(board, &game.board_handle())
            && old
                .tiles
                .iter()
                .zip(&game.board().ids)
                .any(|(a, b)| a.hex() != *b))
}
