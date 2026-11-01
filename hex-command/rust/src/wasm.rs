//! Browser ownership boundary. The catalog is owned once by the factory; battles
//! retain live battle runtimes and exclusive planner workspaces between calls.
use crate::{
    ai::{self, PlanningWorkspace},
    data::*,
    logic::{Action, Game, action::Operation, battle::RuleSource},
    protocol::*,
};
use serde::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use serde::de::DeserializeOwned;
use std::collections::HashMap;
#[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
use std::rc::Rc;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

// Without the importer, the linker can discard the catalog module's Tsify
// registration objects. Anchor its outbound declarations at the facade, using
// the derived declarations so the TypeScript contract stays owned by Rust.
#[cfg(not(feature = "runtime-catalog"))]
mod catalog_declarations {
    use super::*;
    #[wasm_bindgen(typescript_custom_section)]
    const TERRAIN: &str = <TerrainDefinition as Tsify>::DECL;
    #[wasm_bindgen(typescript_custom_section)]
    const RACE: &str = <RaceDefinition as Tsify>::DECL;
    #[wasm_bindgen(typescript_custom_section)]
    const TERRAIN_MODIFIER: &str = <TerrainModifier as Tsify>::DECL;
    #[wasm_bindgen(typescript_custom_section)]
    const ENVIRONMENT: &str = <EnvironmentalModifiers as Tsify>::DECL;
}

mod input;
#[cfg(target_arch = "wasm32")]
mod read;
mod views;
#[cfg(target_arch = "wasm32")]
mod wire;
fn project<T: Serialize, Response: Tsify>(
    result: Result<T, EngineError>,
) -> Result<Ts<Response>, JsError> {
    #[cfg(target_arch = "wasm32")]
    let value = wire::value(&EngineResult::from(result))?;
    #[cfg(not(target_arch = "wasm32"))]
    let value = {
        let serializer = serde_wasm_bindgen::Serializer::new()
            .serialize_missing_as_null(true)
            .serialize_maps_as_objects(true);
        let value = EngineResult::from(result)
            .serialize(&serializer)
            .map_err(|e| JsError::new(&e.to_string()))?;
        value
    };
    Ok(Ts::new_unchecked(value))
}
fn encode<T, Response>(result: Result<T, EngineError>) -> Result<Ts<Response>, JsError>
where
    Response: Tsify + Serialize + From<Result<T, EngineError>>,
{
    #[cfg(target_arch = "wasm32")]
    {
        Ok(Ts::new_unchecked(wire::value(&Response::from(result))?))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Ok(Response::from(result).into_ts()?)
    }
}
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(target_arch = "wasm32")]
fn decode<T: read::Decode>(value: Ts<T>) -> Result<T, EngineError> {
    T::decode(value)
}

struct Battle {
    #[cfg(target_arch = "wasm32")]
    tile_inputs: js_sys::Map,
    game: Game<'static>,
    seed: u32,
    revision: u32,
    workspace: Option<PlanningWorkspace>,
    options: Option<AiOptions>,
    pending: Option<Action>,
}
impl Battle {
    #[cfg(test)]
    fn view(&self) -> EngineView {
        EngineView {
            revision: self.revision,
            state: self.game.state(),
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
    #[cfg(target_arch = "wasm32")]
    catalog_inputs: input::Catalog,
    rules: RuleSource<'static>,
    battles: HashMap<u32, Battle>,
    next_id: u32,
}

#[wasm_bindgen]
impl NativeEngineFactory {
    /// Omit the bundle for the host default: disk on native, baked statics on WASM.
    /// An explicit bundle supplies relative path -> JSON text.
    /// Construction errors carry an encoded [`EngineResult`] failure for the host adapter.
    #[wasm_bindgen(constructor)]
    pub fn new(bundle: Option<Ts<CatalogDocuments>>) -> Result<NativeEngineFactory, JsValue> {
        let loaded = if let Some(bundle) = bundle {
            #[cfg(any(not(target_arch = "wasm32"), feature = "runtime-catalog"))]
            {
                decode(bundle).and_then(|bundle| {
                    Rules::from_json_bundle(&bundle.0)
                        .map(|rules| RuleSource::Shared(Rc::new(rules)))
                        .map_err(|error| EngineError::Content {
                            path: error.path,
                            message: error.message,
                        })
                })
            }
            #[cfg(all(target_arch = "wasm32", not(feature = "runtime-catalog")))]
            {
                let _ = bundle;
                Err(EngineError::InvalidRequest {
                    message: "Custom catalogs require a runtime-catalog build".into(),
                })
            }
        } else {
            #[cfg(target_arch = "wasm32")]
            {
                Ok(RuleSource::Borrowed(&BAKED_RULES))
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                Rules::load()
                    .map(|rules| RuleSource::Shared(Rc::new(rules)))
                    .map_err(|error| EngineError::Content {
                        path: error.path,
                        message: error.message,
                    })
            }
        };
        let rules = loaded.map_err(|error| match encode::<(), UnitResponse>(Err(error)) {
            Ok(response) => response.js_value(),
            Err(error) => error.into(),
        })?;
        #[cfg(target_arch = "wasm32")]
        let catalog_inputs = {
            wire::prepare(0);
            input::Catalog::new(&rules)
        };
        Ok(Self {
            #[cfg(target_arch = "wasm32")]
            catalog_inputs,
            rules,
            battles: HashMap::new(),
            next_id: 1,
        })
    }

    /// Return owned catalog metadata as an encoded [`EngineResult<CatalogView>`].
    pub fn catalog(&self) -> Result<Ts<CatalogResponse>, JsError> {
        project(Ok(views::Catalog(&self.rules)))
    }

    /// Decode [`Start`] and return a new [`BattleHandle`] at revision zero.
    /// The seed is retained for deterministic AI planning.
    pub fn create(&mut self, start: Ts<Start>, seed: u32) -> Result<Ts<BattleResponse>, JsError> {
        let result = decode::<Start>(start)
            .and_then(|start| self.start(start))
            .map(|mut game| {
                game.combat_rng = seed;
                self.insert_game(game, seed)
            });
        self.created(result)
    }
    /// Validate [`SavedGame`] and restore resolved state under a new handle.
    /// The restored handle starts at revision zero; invalid saves leave the factory unchanged.
    pub fn restore(&mut self, saved: Ts<SavedGame>) -> Result<Ts<BattleResponse>, JsError> {
        let result = decode::<SavedGame>(saved).and_then(|saved| self.restore_saved(saved));
        self.created(result)
    }
    /// Return a detached [`EngineView`] when the handle and expected revision are current.
    pub fn view(&self, id: u32, revision: u32) -> Result<Ts<ViewResponse>, JsError> {
        project(self.battle(id, revision).map(|b| views::View {
            game: &b.game,
            revision: b.revision,
        }))
    }
    /// Publish legal actions and display forecasts without exposing mutable state.
    pub fn inspect(&self, id: u32, revision: u32) -> Result<Ts<InspectionResponse>, JsError> {
        encode(
            self.battle(id, revision)
                .map(|battle| battle.game.inspect(revision)),
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
        #[cfg(target_arch = "wasm32")]
        let parsed = self.battles.get(&id).and_then(|b| {
            input::operation(
                &command.js_value(),
                &self.catalog_inputs,
                &b.tile_inputs,
                &b.game,
            )
        });
        #[cfg(not(target_arch = "wasm32"))]
        let parsed: Option<Operation> = None;
        let result = parsed
            .map(Ok)
            .unwrap_or_else(|| decode::<Command>(command).map(Operation::Wire))
            .and_then(|command| self.apply_known(id, revision, command));
        project(result.map(|()| {
            let b = &self.battles[&id];
            views::Dispatch {
                view: views::View {
                    game: &b.game,
                    revision: b.revision,
                },
            }
        }))
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
        let result = input::options(&options.js_value())
            .and_then(|options| self.decision(id, revision, options));
        project(result.map(|()| {
            let b = &self.battles[&id];
            views::Decision {
                game: &b.game,
                revision: b.revision,
                command: b.pending.as_ref().expect("chosen command"),
            }
        }))
    }
    /// Return a [`SavedGame`] checkpoint of resolved game state.
    pub fn save(&self, id: u32, revision: u32) -> Result<Ts<SaveResponse>, JsError> {
        project(self.battle(id, revision).map(|b| views::Saved {
            game: &b.game,
            seed: b.seed,
        }))
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
    fn apply_known(
        &mut self,
        id: u32,
        revision: u32,
        command: Operation,
    ) -> Result<(), EngineError> {
        let battle = self
            .battles
            .get_mut(&id)
            .ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        let ends_turn = matches!(
            command,
            Operation::Wire(Command::EndTurn) | Operation::Planned(Action::EndTurn)
        );
        let planned =
            matches!(&command,Operation::Planned(action) if battle.pending == Some(*action));
        battle.game.combat_animation.clear();
        #[cfg(target_arch = "wasm32")]
        let board = battle.game.board.clone();
        battle.game.execute(command)?;
        #[cfg(target_arch = "wasm32")]
        if !std::rc::Rc::ptr_eq(&board, &battle.game.board) {
            battle.tile_inputs = input::tiles(&battle.game);
        }
        battle.revision = battle
            .revision
            .checked_add(1)
            .expect("battle revision exhausted");
        battle.pending = None;
        if ends_turn || !planned {
            battle.workspace = None;
            battle.options = None;
        }
        Ok(())
    }
    fn decision(&mut self, id: u32, revision: u32, options: AiOptions) -> Result<(), EngineError> {
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
        if battle.pending.is_some() {
            return Ok(());
        }
        let command = ai::choose_game_command(
            &battle.game,
            options.policy,
            battle.workspace.as_mut().expect("workspace initialized"),
        )?;
        battle.pending = Some(command);
        Ok(())
    }
    #[cfg(test)]
    fn checkpoint(&self, id: u32, revision: u32) -> Result<SavedGame, EngineError> {
        let battle = self.battle(id, revision)?;
        Ok(SavedGame {
            seed: battle.seed,
            state: battle.game.state(),
        })
    }
    fn restore_saved(&mut self, saved: SavedGame) -> Result<u32, EngineError> {
        let game = Game::try_from_source(self.rules.clone(), saved.state).map_err(|error| {
            EngineError::InvalidSave {
                message: error.to_string(),
            }
        })?;
        Ok(self.insert_game(game, saved.seed))
    }
    fn battle(&self, id: u32, revision: u32) -> Result<&Battle, EngineError> {
        let battle = self.battles.get(&id).ok_or(EngineError::UnknownBattle)?;
        battle.check(revision)?;
        Ok(battle)
    }
    #[cfg(test)]
    fn insert(&mut self, state: State, seed: u32) -> BattleHandle {
        let id = self.insert_game(Game::from_source(self.rules.clone(), state), seed);
        BattleHandle {
            id,
            view: self.battles[&id].view(),
        }
    }
    fn created(&self, result: Result<u32, EngineError>) -> Result<Ts<BattleResponse>, JsError> {
        project(result.map(|id| {
            let battle = &self.battles[&id];
            views::Handle {
                id,
                view: views::View {
                    game: &battle.game,
                    revision: battle.revision,
                },
            }
        }))
    }
    fn insert_game(&mut self, game: Game<'static>, seed: u32) -> u32 {
        #[cfg(target_arch = "wasm32")]
        wire::prepare(game.terrain.len());
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("battle handle exhausted");
        #[cfg(target_arch = "wasm32")]
        let tile_inputs = input::tiles(&game);
        let battle = Battle {
            #[cfg(target_arch = "wasm32")]
            tile_inputs,
            game,
            seed,
            revision: 0,
            workspace: None,
            options: None,
            pending: None,
        };
        self.battles.insert(id, battle);
        id
    }
    fn start(&self, start: Start) -> Result<Game<'static>, EngineError> {
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
                    .map(|(_, state)| state.instantiate())
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
                            let kind = if original.leader {
                                roster.leader.as_ref()
                            } else if roster.id == "human"
                                || roster.pool.iter().any(|id| id == &unit.r#type)
                            {
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
                                    .as_ref()
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
                Ok(Game::from_source(self.rules.clone(), state))
            }
            Start::Campaign { id } => {
                if !self.rules.campaigns.contains_key(&id) {
                    return Err(EngineError::UnknownCampaign { id });
                }
                Ok(Game::start_campaign_source(
                    self.rules.clone(),
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
    fn checkpoint_discards_unexecuted_plans_and_restores_resolved_state() {
        let mut factory = NativeEngineFactory {
            rules: RuleSource::Borrowed(&BAKED_RULES),
            battles: HashMap::new(),
            next_id: 1,
        };
        let state = Game::new(&factory.rules, "mirror").state();
        let original = factory.insert(state.clone(), 17);
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
        let battle = &factory.battles[&restored];
        assert!(battle.workspace.is_none() && battle.pending.is_none() && battle.options.is_none());
        assert_eq!(
            serde_json::to_value(battle.game.state()).unwrap(),
            json["state"]
        );
        let mut old = json;
        old["planner"] = serde_json::json!({});
        assert!(serde_json::from_value::<SavedGame>(old).is_err());
    }
}
