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
    fn view(&self) -> EngineView {
        EngineView {
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
            .map(|state| {
                let board = Board::new(&state.tiles);
                self.insert(state, seed, board)
            });
        encode(result)
    }
    /// Validate [`SavedGame`] and restore state and AI continuation under a new handle.
    /// The restored handle starts at revision zero; invalid saves leave the factory unchanged.
    pub fn restore(&mut self, saved: Ts<SavedGame>) -> Result<Ts<BattleResponse>, JsError> {
        encode(decode::<SavedGame>(saved).and_then(|saved| self.restore_saved(saved)))
    }
    /// Return a detached [`EngineView`] when the handle and expected revision are current.
    pub fn view(&self, id: u32, revision: u32) -> Result<Ts<ViewResponse>, JsError> {
        encode(self.battle(id, revision).map(Battle::view))
    }
    /// Publish legal actions and display forecasts without exposing mutable state.
    pub fn inspect(&self, id: u32, revision: u32) -> Result<Ts<InspectionResponse>, JsError> {
        encode(
            self.battle(id, revision)
                .map(|battle| battle.game(&self.rules).inspect(revision)),
        )
    }
    /// Return [`ReachableTile`] destinations and costs for a stable unit identity.
    /// The query leaves battle state and revision unchanged.
    pub fn reachable(
        &self,
        id: u32,
        revision: u32,
        unit: u32,
    ) -> Result<Ts<ReachableResponse>, JsError> {
        encode(self.battle(id, revision).and_then(|battle| {
            let game = battle.game(&self.rules);
            let actor = game
                .unit(unit)
                .ok_or(CommandError::UnknownUnit(UnitId(unit)))?;
            Ok(game
                .reachable(actor)
                .iter()
                .copied()
                .map(|(tile, cost)| ReachableTile {
                    tile: TileId(game.board.ids[tile].clone()),
                    cost,
                })
                .collect::<Vec<_>>())
        }))
    }
    /// Decode [`AttackQuery`] and return an [`AttackPreview`] for a legal attack.
    /// An optional hypothetical origin affects the query without moving the attacker.
    #[wasm_bindgen(js_name = previewAttack)]
    pub fn preview_attack(
        &self,
        id: u32,
        revision: u32,
        query: Ts<AttackQuery>,
    ) -> Result<Ts<AttackResponse>, JsError> {
        encode(decode::<AttackQuery>(query).and_then(|query| {
            let battle = self.battle(id, revision)?;
            let game = battle.game(&self.rules);
            let actor = game
                .unit(query.unit.0)
                .ok_or(CommandError::UnknownUnit(query.unit))?;
            let target = game
                .unit(query.target.0)
                .ok_or(CommandError::UnknownUnit(query.target))?;
            let weapon = self.rules.units[actor.kind]
                .attacks
                .iter()
                .position(|attack| attack.id == query.weapon.0)
                .ok_or_else(|| CommandError::UnknownWeapon(query.weapon.clone()))?;
            let from = match query.from {
                Some(tile) => *game
                    .board
                    .indices
                    .get(&tile.0)
                    .ok_or_else(|| CommandError::UnknownTile(tile))?,
                None => actor.tile,
            };
            if !game.can_attack(actor, from, target, weapon) {
                return Err(CommandError::IllegalAttack.into());
            }
            let (damage, retaliation, response) = game.preview(actor, target, weapon, from);
            Ok(AttackPreview {
                damage,
                retaliation,
                response: response
                    .map(|index| self.rules.units[target.kind].attacks[index].clone()),
            })
        }))
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
    /// Return a [`SavedGame`] checkpoint including any pending AI command and continuation.
    pub fn save(&self, id: u32, revision: u32) -> Result<Ts<SaveResponse>, JsError> {
        encode(self.checkpoint(id, revision))
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
        let mut effects = game.execute(command.clone())?;
        let state = game.state();
        crate::logic::complete_effects(&battle.state, &state, &command, &mut effects);
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
            view: battle.view(),
            effects,
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
        if options.recruitment_limit > 1_000_000 {
            return Err(EngineError::InvalidRequest {
                message: "AI recruitment limit is out of range".into(),
            });
        }
        if battle.options.as_ref() != Some(&options) {
            battle.workspace = Some(PlanningWorkspace::new(
                battle.seed,
                options.recruitment_limit,
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
            &self.rules,
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
        let planner =
            battle
                .workspace
                .as_ref()
                .zip(battle.options.as_ref())
                .map(|(workspace, options)| SavedPlanner {
                    options: options.clone(),
                    workspace: workspace.snapshot(),
                    pending: battle.pending.clone(),
                });
        Ok(SavedGame {
            seed: battle.seed,
            state: battle.state.clone(),
            planner,
        })
    }
    fn restore_saved(&mut self, mut saved: SavedGame) -> Result<BattleHandle, EngineError> {
        let game = Game::try_from_state(&self.rules, saved.state).map_err(|error| {
            EngineError::InvalidSave {
                message: error.to_string(),
            }
        })?;
        let state = game.state();
        if let Some(planner) = &mut saved.planner {
            if planner.workspace.blocks != planner.options.blocks
                || planner.workspace.seed != saved.seed
                || planner.workspace.diplomacy != planner.options.diplomacy
                || planner
                    .workspace
                    .policy
                    .is_some_and(|policy| policy != planner.options.policy)
                || planner.workspace.recruitment_remaining > planner.options.recruitment_limit
            {
                return Err(EngineError::InvalidSave {
                    message: "planner configuration does not match its checkpoint".into(),
                });
            }
            planner
                .workspace
                .validate(&self.rules, &state)
                .map_err(|error| EngineError::InvalidSave {
                    message: error.to_string(),
                })?;
            if let Some(pending) = &planner.pending {
                match game.fork().execute(pending.clone()) {
                    Ok(_) => {}
                    Err(CommandError::IllegalMove | CommandError::IllegalRecruit) => {
                        // Rebalanced movement or prices can invalidate a queued action.
                        // Replan from the saved turn without replenishing recruits.
                        planner.workspace = PlanningWorkspace::new(
                            planner.workspace.seed,
                            planner.workspace.recruitment_remaining,
                            planner.workspace.goals.clone(),
                            planner.workspace.diplomacy,
                        )
                        .with_optional_blocks(planner.workspace.blocks)
                        .snapshot();
                        planner.pending = None;
                    }
                    Err(error) => {
                        return Err(EngineError::InvalidSave {
                            message: error.to_string(),
                        });
                    }
                }
            }
        }
        let board = game.board.clone();
        let handle = self.insert(state, saved.seed, board);
        if let Some(planner) = saved.planner {
            let battle = self.battles.get_mut(&handle.id).expect("inserted battle");
            battle.workspace = Some(PlanningWorkspace::from_snapshot(planner.workspace));
            battle.options = Some(planner.options);
            battle.pending = planner.pending;
        }
        Ok(handle)
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
        let view = battle.view();
        self.battles.insert(id, battle);
        BattleHandle { id, view }
    }
    fn start(&self, start: Start) -> Result<State, EngineError> {
        match start {
            Start::Scenario { map, revision } => self
                .rules
                .maps
                .iter()
                .filter(|((id, rev), _)| {
                    id == &map && revision.is_none_or(|expected| expected == *rev)
                })
                .max_by_key(|((_, revision), _)| revision)
                .map(|(_, state)| state.clone())
                .ok_or(EngineError::UnknownMap { map }),
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
    fn rebalanced_catalog_restores_health_and_replans_a_stale_move() {
        let mut factory = NativeEngineFactory {
            rules: Rules::embedded().unwrap(),
            battles: HashMap::new(),
            next_id: 1,
        };
        let state = Game::new(&factory.rules, "mirror").state();
        let board = Board::new(&state.tiles);
        let original = factory.insert(state, 17, board);
        let options = AiOptions {
            blocks: None,
            policy: AiPolicy::Mixed,
            diplomacy: AiDiplomacy::Restrained,
            recruitment_limit: 1,
        };
        factory.decision(original.id, 0, options.clone()).unwrap();
        let mut saved = factory.checkpoint(original.id, 0).unwrap();
        let (kind, actor_id, command) = {
            let game = factory.battles[&original.id].game(&factory.rules);
            let actor = game.units.iter().find(|u| u.side == game.turn).unwrap();
            let (tile, cost) = game
                .reachable(actor)
                .iter()
                .copied()
                .max_by_key(|(_, cost)| *cost)
                .unwrap();
            assert!(cost > 1);
            let command = Command::Move {
                unit: UnitId(actor.id),
                tile: TileId(game.board.ids[tile].clone()),
            };
            assert!(game.fork().execute(command.clone()).is_ok());
            (actor.kind, format!("u{}", actor.id), command)
        };
        let finished = saved
            .state
            .units
            .iter_mut()
            .find(|u| u.side == saved.state.turn && u.id != actor_id)
            .unwrap();
        finished.hp = 1;
        finished.moved = true;
        finished.attacked = true;
        let finished_id = finished.id.clone();
        let planner = saved.planner.as_mut().unwrap();
        planner.pending = Some(command);
        planner.workspace.recruitment_remaining = 0;
        let mut bundle = crate::data::catalog::embedded_bundle();
        let path = format!("game/units/{}.json", factory.rules.units[kind].id);
        let source = bundle.get_mut(&path).unwrap();
        let mut definition: serde_json::Value = serde_json::from_str(source).unwrap();
        definition["hp"] = serde_json::json!(factory.rules.units[kind].hp - 1);
        definition["move"] = serde_json::json!(1);
        *source = serde_json::to_string(&definition).unwrap();
        factory.rules = Rules::from_json_bundle(&bundle).unwrap();
        let restored = factory.restore_saved(saved).unwrap();
        let actor = restored
            .view
            .state
            .units
            .iter()
            .find(|u| u.id == actor_id)
            .unwrap();
        assert_eq!(actor.hp, factory.rules.units[kind].hp);
        assert!(!actor.moved && !actor.attacked);
        let finished = restored
            .view
            .state
            .units
            .iter()
            .find(|u| u.id == finished_id)
            .unwrap();
        assert_eq!(finished.hp, 1);
        assert!(finished.moved && finished.attacked);
        assert_eq!(
            factory.battles[&restored.id]
                .workspace
                .as_ref()
                .unwrap()
                .snapshot()
                .recruitment_remaining,
            0
        );
        let next = factory.decision(restored.id, 0, options).unwrap();
        assert!(
            factory.battles[&restored.id]
                .game(&factory.rules)
                .execute(next.command)
                .is_ok()
        );
    }

    #[test]
    fn save_restores_pending_ai_decision_and_continuation() {
        for (policy, blocks) in [
            (AiPolicy::Classic, None),
            (AiPolicy::Mixed, None),
            (AiPolicy::Priority, None),
            (AiPolicy::Battler, None),
            (AiPolicy::Classic, AiPolicy::Priority.blocks()),
        ] {
            let mut factory = NativeEngineFactory {
                rules: Rules::embedded().unwrap(),
                battles: HashMap::new(),
                next_id: 1,
            };
            let state = factory
                .start(Start::Scenario {
                    map: "mirror".into(),
                    revision: None,
                })
                .unwrap();
            let board = Board::new(&state.tiles);
            let original = factory.insert(state, 17, board);
            let options = AiOptions {
                blocks,
                policy,
                diplomacy: AiDiplomacy::Restrained,
                recruitment_limit: 2,
            };
            let first = factory.decision(original.id, 0, options.clone()).unwrap();
            let saved = factory.checkpoint(original.id, 0).unwrap();
            assert!(saved.planner.as_ref().unwrap().pending.is_some());
            let saved: SavedGame =
                serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
            let restored = factory.restore_saved(saved).unwrap();
            let repeated = factory.decision(restored.id, 0, options.clone()).unwrap();
            assert_eq!(first.command, repeated.command);
            // A few individual decisions exercise the cursor and staged follow-up,
            // without running a turn, campaign, or tournament simulation.
            for revision in 0..3 {
                let first = factory
                    .decision(original.id, revision, options.clone())
                    .unwrap();
                let second = factory
                    .decision(restored.id, revision, options.clone())
                    .unwrap();
                assert_eq!(first.command, second.command);
                factory.apply(original.id, revision, first.command).unwrap();
                factory
                    .apply(restored.id, revision, second.command)
                    .unwrap();
                assert_eq!(
                    serde_json::to_value(&factory.battles[&original.id].state).unwrap(),
                    serde_json::to_value(&factory.battles[&restored.id].state).unwrap()
                );
            }
            // Save after recruitment, when queued policies have a real duty cursor.
            let saved = factory.checkpoint(original.id, 3).unwrap();
            let saved: SavedGame =
                serde_json::from_str(&serde_json::to_string(&saved).unwrap()).unwrap();
            let resumed = factory.restore_saved(saved).unwrap();
            assert_eq!(
                factory
                    .decision(original.id, 3, options.clone())
                    .unwrap()
                    .command,
                factory
                    .decision(resumed.id, 0, options.clone())
                    .unwrap()
                    .command
            );
        }
    }
}
