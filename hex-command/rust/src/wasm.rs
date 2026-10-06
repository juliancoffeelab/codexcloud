//! Browser ownership boundary. The catalog is owned once by the factory; battles
//! hold snapshots and exclusive planner workspaces, borrowing rules only per call.
use crate::{
    ai::{self, PlanningWorkspace},
    data::*,
    logic::Game,
    protocol::*,
};
use serde::{Serialize, de::DeserializeOwned};
use std::collections::HashMap;
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
    seed: u32,
    revision: u32,
    workspace: Option<PlanningWorkspace>,
    options: Option<AiOptions>,
    pending: Option<Command>,
}
impl Battle {
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
    /// Omit the bundle for embedded canonical JSON, or pass relative path -> JSON text.
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
            Rules::embedded().map_err(|error| EngineError::Content {
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
        let mut maps: Vec<_> = self
            .rules
            .maps
            .values()
            .map(|state| MapSummary {
                id: state.map_id.clone(),
                revision: state.map_revision,
                cols: state.cols,
                rows: state.rows,
            })
            .collect();
        maps.sort_by(|a, b| (&a.id, a.revision).cmp(&(&b.id, b.revision)));
        let mut campaigns: Vec<_> = self
            .rules
            .campaigns
            .values()
            .map(|campaign| CampaignSummary {
                id: campaign.id.clone(),
                name: campaign.name.clone(),
                missions: campaign.missions.len(),
            })
            .collect();
        campaigns.sort_by(|a, b| a.id.cmp(&b.id));
        let units = self
            .rules
            .units
            .iter()
            .map(|unit| UnitSummary {
                id: unit.id.clone(),
                name: unit.name.clone(),
                hp: unit.hp,
                movement: unit.r#move,
                cost: unit.cost,
                leader: unit.leader,
                attacks: unit.attacks.clone(),
                advances_to: unit.advances_to.clone(),
            })
            .collect();
        encode(Ok(CatalogView {
            maps,
            campaigns,
            units,
            terrains: self.rules.terrain_names.clone(),
        }))
    }

    /// Decode [`Start`] and return a new [`BattleHandle`] at revision zero.
    /// The seed is retained for deterministic AI planning.
    pub fn create(&mut self, start: Ts<Start>, seed: u32) -> Result<Ts<BattleResponse>, JsError> {
        let result = decode::<Start>(start)
            .and_then(|start| self.start(start))
            .map(|state| self.insert(state, seed));
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
            self.battle(id, revision).map(|battle| {
                Game::from_state(&self.rules, battle.state.clone()).inspect(revision)
            }),
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
            let game = Game::from_state(&self.rules, battle.state.clone());
            let actor = game
                .unit(unit)
                .ok_or(CommandError::UnknownUnit(UnitId(unit)))?;
            Ok(game
                .reachable(actor)
                .into_iter()
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
            let game = Game::from_state(&self.rules, battle.state.clone());
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
        let mut game = Game::from_state(&self.rules, battle.state.clone());
        let ends_turn = matches!(command, Command::EndTurn);
        let planned = battle.pending.as_ref() == Some(&command);
        let mut effects = game.execute(command.clone())?;
        let state = game.state();
        crate::logic::complete_effects(&battle.state, &state, &command, &mut effects);
        battle.state = state;
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
            battle.options = Some(options.clone());
            battle.pending = None;
        }
        if let Some(command) = &battle.pending {
            return Ok(AiDecision {
                revision: battle.revision,
                command: command.clone(),
            });
        }
        let game = Game::from_state(&self.rules, battle.state.clone());
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
            version: 1,
            seed: battle.seed,
            state: battle.state.clone(),
            planner,
        })
    }
    fn restore_saved(&mut self, saved: SavedGame) -> Result<BattleHandle, EngineError> {
        if saved.version != 1 {
            return Err(EngineError::InvalidSave {
                message: format!("unsupported save version {}", saved.version),
            });
        }
        let game = Game::try_from_state(&self.rules, saved.state).map_err(|error| {
            EngineError::InvalidSave {
                message: error.to_string(),
            }
        })?;
        let state = game.state();
        if let Some(planner) = &saved.planner {
            if planner.workspace.seed != saved.seed
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
                Game::from_state(&self.rules, state.clone())
                    .execute(pending.clone())
                    .map_err(|error| EngineError::InvalidSave {
                        message: error.to_string(),
                    })?;
            }
        }
        let handle = self.insert(state, saved.seed);
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
    fn insert(&mut self, state: State, seed: u32) -> BattleHandle {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("battle handle exhausted");
        let battle = Battle {
            state,
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
    fn save_restores_pending_ai_decision_and_continuation() {
        for policy in [AiPolicy::Classic, AiPolicy::Mixed] {
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
            let original = factory.insert(state, 17);
            let options = AiOptions {
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
        }
    }
}
