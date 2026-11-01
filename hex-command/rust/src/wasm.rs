//! Browser ownership boundary. The catalog is owned once by the factory; battles
//! hold snapshots and exclusive planner workspaces, borrowing rules only per call.
use crate::{
    ai::{self, PlanningWorkspace},
    data::*,
    logic::{Game, battle::Board},
    protocol::*,
};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::rc::Rc;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

fn encode<T, Response>(result: Result<T, EngineError>) -> Result<Ts<Response>, JsError>
where
    Response: Tsify + Serialize + From<Result<T, EngineError>>,
{
    Ok(Response::from(result).into_ts()?)
}
fn decode<T: Tsify + DeserializeOwned>(value: Ts<T>) -> Result<T, EngineError>
where
    T::JsType: Clone,
{
    value
        .to_rust()
        .map_err(|error| EngineError::InvalidRequest {
            message: error.to_string(),
        })
}
struct Battle {
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
        Game::from_state_with_board(rules, self.state.clone(), self.board.clone())
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

/// Owns one validated catalog and independent battles for a browser host.
#[wasm_bindgen]
pub struct NativeEngineFactory {
    rules: Rules,
    battles: HashMap<u32, Battle>,
    next_id: u32,
}

#[wasm_bindgen]
impl NativeEngineFactory {
    /// Omit the bundle for the host default: disk on native, embedded JSON on WASM.
    /// An explicit bundle supplies relative path -> JSON text.
    /// Construction errors carry an encoded [`EngineResult`] failure for the host adapter.
    #[wasm_bindgen(constructor)]
    pub fn new(bundle: Option<Ts<CatalogDocuments>>) -> Result<NativeEngineFactory, JsValue> {
        let loaded = if let Some(bundle) = bundle {
            decode(bundle).and_then(|bundle| {
                Rules::from_json_bundle(&bundle.0).map_err(|error| EngineError::Content {
                    path: error.path,
                    message: error.message,
                })
            })
        } else {
            Rules::load().map_err(|error| EngineError::Content {
                path: error.path,
                message: error.message,
            })
        };
        let rules = loaded.map_err(|error| match encode::<(), UnitResponse>(Err(error)) {
            Ok(response) => response.js_value(),
            Err(error) => error.into(),
        })?;
        Ok(Self {
            rules,
            battles: HashMap::new(),
            next_id: 1,
        })
    }

    /// Return owned catalog metadata as an encoded [`EngineResult<CatalogView>`].
    pub fn catalog(&self) -> Result<Ts<CatalogResponse>, JsError> {
        let mut campaigns: Vec<_> = self.rules.campaigns.values().cloned().collect();
        campaigns.sort_by(|a, b| a.id.cmp(&b.id));
        encode(Ok(CatalogView {
            units: self.rules.units.clone(),
            terrain: self.rules.terrain_definitions.clone(),
            races: self.rules.race_definitions.clone(),
            maps: self.rules.map_definitions.clone(),
            campaigns,
        }))
    }

    /// Decode [`Start`] and return a new [`BattleHandle`] at revision zero.
    /// The seed is retained for deterministic AI planning.
    pub fn create(&mut self, start: Ts<Start>, seed: u32) -> Result<Ts<BattleResponse>, JsError> {
        let result = decode::<Start>(start)
            .and_then(|start| self.start(start))
            .map(|mut state| {
                state.combat_rng = Some(seed);
                let board = Board::new(&state.tiles);
                let game = Game::from_state_with_board(&self.rules, state, board.clone());
                self.insert(game.state(), seed, board)
            });
        encode(result)
    }
    /// Validate [`SavedGame`] and restore resolved state under a new handle.
    /// The restored handle starts at revision zero; invalid saves leave the factory unchanged.
    pub fn restore(&mut self, saved: Ts<SavedGame>) -> Result<Ts<BattleResponse>, JsError> {
        encode(decode::<SavedGame>(saved).and_then(|saved| self.restore_saved(saved)))
    }
    /// Return a detached [`EngineView`] when the handle and expected revision are current.
    pub fn view(&self, id: u32, revision: u32) -> Result<Ts<ViewResponse>, JsError> {
        encode(
            self.battle(id, revision)
                .map(|battle| battle.view(&self.rules)),
        )
    }
    /// Publish legal actions and display forecasts without exposing mutable state.
    pub fn inspect(&self, id: u32, revision: u32) -> Result<Ts<InspectionResponse>, JsError> {
        encode(
            self.battle(id, revision)
                .map(|battle| battle.game(&self.rules).inspect(revision)),
        )
    }
    /// Board topology and rule costs do not belong to each unit-action refresh.
    pub fn geometry(&self, id: u32, revision: u32) -> Result<Ts<BoardResponse>, JsError> {
        encode(
            self.battle(id, revision)
                .map(|battle| battle.game(&self.rules).inspect_board()),
        )
    }
    /// Incremental browser publication. The complete snapshot API remains
    /// available for explicit inspection and persistence tools.
    #[wasm_bindgen(js_name = dispatchUpdate)]
    pub fn dispatch_update(
        &mut self,
        id: u32,
        revision: u32,
        command: Ts<Command>,
    ) -> Result<Ts<UpdateResponse>, JsError> {
        encode(
            decode::<Command>(command).and_then(|command| self.apply_update(id, revision, command)),
        )
    }
    /// Apply one decoded [`Command`] against the expected revision.
    /// Success returns [`DispatchResult`]; rejected commands leave state unchanged.
    pub fn dispatch(
        &mut self,
        id: u32,
        revision: u32,
        command: Ts<Command>,
    ) -> Result<Ts<DispatchResponse>, JsError> {
        encode(decode::<Command>(command).and_then(|command| self.apply(id, revision, command)))
    }
    /// Return an [`AiDecision`] without applying it or advancing the battle revision.
    /// Repeated requests reuse the pending command until dispatch; changed options start a new phase.
    #[wasm_bindgen(js_name = chooseAiCommand)]
    pub fn choose_ai_command(
        &mut self,
        id: u32,
        revision: u32,
        options: Ts<AiOptions>,
    ) -> Result<Ts<AiResponse>, JsError> {
        encode(
            decode::<AiOptions>(options).and_then(|options| self.decision(id, revision, options)),
        )
    }
    /// Return a [`SavedGame`] checkpoint of resolved game state.
    pub fn save(&self, id: u32, revision: u32) -> Result<Ts<SaveResponse>, JsError> {
        encode(self.checkpoint(id, revision))
    }
    /// Serialize the unchanged save format in Rust, avoiding a large temporary
    /// JS object graph solely to JSON.stringify it again for localStorage.
    #[wasm_bindgen(js_name = saveJson)]
    pub fn save_json(&self, id: u32, revision: u32) -> Result<Ts<SerializedSaveResponse>, JsError> {
        encode(
            self.checkpoint(id, revision)
                .map(|saved| serde_json::to_string(&saved).expect("serializable checkpoint")),
        )
    }
    /// Release a battle and its planner; later requests return [`EngineError::UnknownBattle`].
    /// All results use the encoded [`EngineResult`] envelope.
    #[wasm_bindgen(js_name = disposeBattle)]
    pub fn dispose_battle(&mut self, id: u32) -> Result<Ts<UnitResponse>, JsError> {
        encode(
            self.battles
                .remove(&id)
                .map(|_| ())
                .ok_or(EngineError::UnknownBattle),
        )
    }
}

impl NativeEngineFactory {
    fn apply_update(
        &mut self,
        id: u32,
        revision: u32,
        command: Command,
    ) -> Result<DispatchUpdate, EngineError> {
        let before = self.battle(id, revision)?;
        let old_tiles = before.state.tiles.clone();
        let old_map = (
            before.state.map_id.clone(),
            before.state.map_revision,
            before.state.cols,
            before.state.rows,
        );
        let result = self.apply(id, revision, command)?;
        let (state, tiles) = StateFrame::split(result.view.state);
        let replaced = old_map
            != (
                state.map_id.clone(),
                state.map_revision,
                state.cols,
                state.rows,
            )
            || old_tiles.len() != tiles.len()
            || old_tiles.iter().zip(&tiles).any(|(a, b)| a.id != b.id);
        let terrain = if replaced {
            TerrainUpdate::Replace { tiles }
        } else {
            let changed: Vec<_> = tiles
                .into_iter()
                .zip(&old_tiles)
                .filter_map(|(tile, old)| (tile != *old).then_some(tile))
                .collect();
            if changed.is_empty() {
                TerrainUpdate::Keep
            } else {
                TerrainUpdate::Patch { tiles: changed }
            }
        };
        Ok(DispatchUpdate {
            base_revision: revision,
            revision: result.view.revision,
            state,
            terrain,
            presentation: result.view.presentation,
            combat: result.combat,
        })
    }
    fn apply(
        &mut self,
        id: u32,
        revision: u32,
        command: Command,
    ) -> Result<DispatchResult, EngineError> {
        let battle = self
            .battles
            .get_mut(&id)
            .ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        let mut game = battle.game(&self.rules);
        let ends_turn = matches!(command, Command::EndTurn);
        let planned = battle.pending.as_ref() == Some(&command);
        game.execute(command)?;
        let state = game.state();
        battle.state = state;
        battle.board = game.board.clone();
        battle.revision = battle
            .revision
            .checked_add(1)
            .expect("battle revision exhausted");
        battle.pending = None;
        if ends_turn || !planned {
            battle.workspace = None;
            battle.options = None;
        }
        Ok(DispatchResult {
            view: EngineView {
                presentation: game.presentation(),
                revision: battle.revision,
                state: battle.state.clone(),
            },
            combat: game.combat_animation.take(),
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
        Ok(self.insert(game.state(), saved.seed, game.board.clone()))
    }
    fn battle(&self, id: u32, revision: u32) -> Result<&Battle, EngineError> {
        let battle = self.battles.get(&id).ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        Ok(battle)
    }
    fn insert(&mut self, state: State, seed: u32, board: Rc<Board>) -> BattleHandle {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("battle handle exhausted");
        let battle = Battle {
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
    fn checkpoint_discards_unexecuted_plans_and_restores_resolved_state() {
        let mut factory = NativeEngineFactory {
            rules: Rules::embedded().unwrap(),
            battles: HashMap::new(),
            next_id: 1,
        };
        let state = Game::new(&factory.rules, "mirror").state();
        let original = factory.insert(state.clone(), 17, Board::new(&state.tiles));
        let options = AiOptions {
            blocks: None,
            policy: AiPolicy::Balanced,
            diplomacy: AiDiplomacy::Restrained,
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
}

/// Offline authoring uses the live Rust condition evaluator, without a second JS implementation.
#[wasm_bindgen(js_name = previewPresentation)]
pub fn preview_presentation(
    input: Ts<crate::logic::conditions::PresentationInput>,
) -> Result<Ts<BattlefieldPresentation>, JsError> {
    let input = input.to_rust().map_err(|e| JsError::new(&e.to_string()))?;
    Ok(crate::logic::conditions::resolve(&input).0.into_ts()?)
}
