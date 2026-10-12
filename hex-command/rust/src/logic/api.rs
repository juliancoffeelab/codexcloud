//! Host-facing battle operations using stable identifiers and owned snapshots.

use super::{
    StateError,
    battle::{Board, Game},
};
use crate::data::*;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
/// Owned, serializable battle state. Operations borrow an immutable catalog.
///
/// Native simulation sessions use a compact internal roster; this host-facing
/// value keeps the snapshot that can be saved and restored across calls.
#[derive(Clone, Debug)]
pub struct Battle {
    state: State,
    board: Rc<Board>,
    catalog_fingerprint: String,
}
/// A legal movement destination for a unit that still has its action.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reachable {
    pub tile: TileId,
    pub cost: MovementCost,
}
/// Deterministic attack and retaliation damage for a specified position.
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(missing_as_null, hashmap_as_object))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatPreview {
    pub damage: i32,
    pub retaliation: i32,
    pub retaliation_weapon: Option<WeaponId>,
}
/// Failure to start, restore, query or modify a battle.
#[derive(Clone, Debug)]
pub enum BattleError {
    InvalidState(StateError),
    InvalidCommand(CommandError),
    UnknownMap(MapId),
    UnknownCampaign(CampaignId),
    UnknownMission(usize),
    CatalogMismatch,
}
impl std::fmt::Display for BattleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidState(error) => write!(f, "{error}"),
            Self::InvalidCommand(error) => write!(f, "{error}"),
            other => write!(f, "{other:?}"),
        }
    }
}
impl std::error::Error for BattleError {}
impl From<CommandError> for BattleError {
    fn from(error: CommandError) -> Self {
        Self::InvalidCommand(error)
    }
}
impl From<StateError> for BattleError {
    fn from(error: StateError) -> Self {
        Self::InvalidState(error)
    }
}
impl Battle {
    /// Start the requested map revision, or its latest authored revision.
    pub fn new(catalog: &Catalog, map: MapId, revision: Option<u32>) -> Result<Self, BattleError> {
        let state = catalog
            .maps
            .iter()
            .filter(|((id, rev), _)| id == &map.0 && revision.is_none_or(|r| *rev == r))
            .max_by_key(|((_, rev), _)| rev)
            .map(|(_, state)| state.clone())
            .ok_or(BattleError::UnknownMap(map))?;
        Self::restore(catalog, state)
    }
    /// Start an authored campaign mission without imported veterans.
    pub fn campaign(
        catalog: &Catalog,
        id: CampaignId,
        mission: usize,
    ) -> Result<Self, BattleError> {
        let definition = catalog
            .campaigns
            .get(&id.0)
            .ok_or_else(|| BattleError::UnknownCampaign(id.clone()))?;
        if mission >= definition.missions.len() {
            return Err(BattleError::UnknownMission(mission));
        }
        let game = Game::start_campaign(catalog, &id.0, mission, vec![], 0, vec![], vec![], vec![]);
        Ok(Self::from_game(game))
    }
    /// Validate a saved snapshot before creating any runtime indexes.
    pub fn restore(catalog: &Catalog, state: State) -> Result<Self, BattleError> {
        Ok(Self::from_game(Game::try_from_state(catalog, state)?))
    }
    fn from_game(game: Game<'_>) -> Self {
        Self {
            state: game.state(),
            board: game.board.clone(),
            catalog_fingerprint: game.rules.content_fingerprint.clone(),
        }
    }
    /// Borrow the coherent published state.
    pub fn snapshot(&self) -> &State {
        &self.state
    }
    /// Copy the owned snapshot for persistence or a transport boundary.
    pub fn save(&self) -> State {
        self.state.clone()
    }
    fn check_catalog(&self, catalog: &Catalog) -> Result<(), BattleError> {
        if self.catalog_fingerprint != catalog.content_fingerprint {
            return Err(BattleError::CatalogMismatch);
        }
        Ok(())
    }
    pub(crate) fn with_game<T>(
        &self,
        catalog: &Catalog,
        read: impl FnOnce(&Game<'_>) -> T,
    ) -> Result<T, BattleError> {
        self.check_catalog(catalog)?;
        let game = Game::from_state_with_board(catalog, self.state.clone(), self.board.clone());
        Ok(read(&game))
    }
    /// Apply one fully validated command, publishing its complete resulting state.
    pub fn apply(&mut self, catalog: &Catalog, command: Command) -> Result<Effects, BattleError> {
        self.check_catalog(catalog)?;
        let mut game = Game::from_state_with_board(catalog, self.state.clone(), self.board.clone());
        let mut effects = game.execute(command.clone())?;
        let next = game.state();
        super::effects::complete_effects(&self.state, &next, &command, &mut effects);
        self.state = next;
        self.board = game.board.clone();
        Ok(effects)
    }
    /// Return legal destinations in the same stable discovery order as the rules.
    pub fn reachable(
        &self,
        catalog: &Catalog,
        unit: UnitId,
    ) -> Result<Vec<Reachable>, BattleError> {
        self.with_game(catalog, |game| {
            let actor = game.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
            Ok::<_, CommandError>(
                game.reachable(actor)
                    .iter()
                    .copied()
                    .map(|(tile, cost)| Reachable {
                        tile: TileId(game.board.ids[tile].clone()),
                        cost: MovementCost(cost),
                    })
                    .collect(),
            )
        })?
        .map_err(Into::into)
    }
    /// Query the finite terrain entry cost or impassability for a unit type.
    pub fn movement_entry(
        &self,
        catalog: &Catalog,
        unit_type: UnitTypeId,
        tile: TileId,
    ) -> Result<Passability, BattleError> {
        self.with_game(catalog, |game| {
            let kind = *game
                .rules
                .types
                .get(&unit_type.0)
                .ok_or(CommandError::UnknownType(unit_type))?;
            let site = *game
                .board
                .indices
                .get(&tile.0)
                .ok_or(CommandError::UnknownTile(tile))?;
            Ok::<Passability, CommandError>(
                match game.rules.units[kind].costs[game.terrain[site]] {
                    Some(cost) => Passability::Passable(MovementCost(cost)),
                    None => Passability::Blocked,
                },
            )
        })?
        .map_err(Into::into)
    }
    /// Preview a legal attack from the current or specified hypothetical tile.
    pub fn preview(
        &self,
        catalog: &Catalog,
        unit: UnitId,
        target: UnitId,
        weapon: WeaponId,
        from: Option<TileId>,
    ) -> Result<CombatPreview, BattleError> {
        self.with_game(catalog, |game| {
            let actor = game.unit(unit.0).ok_or(CommandError::UnknownUnit(unit))?;
            let victim = game
                .unit(target.0)
                .ok_or(CommandError::UnknownUnit(target))?;
            let weapon_index = game.rules.units[actor.kind]
                .attacks
                .iter()
                .position(|a| a.id == weapon.0)
                .ok_or(CommandError::UnknownWeapon(weapon))?;
            let tile = if let Some(from) = from {
                *game
                    .board
                    .indices
                    .get(&from.0)
                    .ok_or(CommandError::UnknownTile(from))?
            } else {
                actor.tile
            };
            if !game.can_attack(actor, tile, victim, weapon_index) {
                return Err(CommandError::IllegalAttack);
            }
            let (damage, retaliation, response) = game.preview(actor, victim, weapon_index, tile);
            Ok(CombatPreview {
                damage,
                retaliation,
                retaliation_weapon: response
                    .map(|w| WeaponId(game.rules.units[victim.kind].attacks[w].id.clone())),
            })
        })?
        .map_err(Into::into)
    }
}
