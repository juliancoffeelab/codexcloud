use super::PlanningSnapshot;
use crate::{data::*, logic::StateError};
use std::collections::HashSet;
impl PlanningSnapshot {
    /// Check persisted planning references before resuming them against a saved
    /// battle. Queued actions are validated for legality when the engine applies
    /// each action, after its preceding movement and intent changes.
    pub fn validate(&self, catalog: &Catalog, state: &State) -> Result<(), StateError> {
        let fail = |message: &str| StateError(message.to_owned());
        let tiles: HashSet<_> = state.tiles.iter().map(|tile| tile.id.as_str()).collect();
        if self.initialized && self.policy.is_none() {
            return Err(fail("initialized planner has no policy"));
        }
        if self.policy.is_some_and(|p| p.legacy().is_some()) {
            return Err(fail(
                "frozen reference policies run complete native matches, not live planners",
            ));
        }
        if state.campaign.is_some()
            && self
                .blocks
                .or_else(|| self.policy.and_then(|p| p.blocks()))
                .is_some_and(|b| !b.campaign.supports_campaign())
        {
            return Err(fail("battle-only policy cannot resume a campaign"));
        }
        if let Some(queue) = &self.priority {
            if !self
                .blocks
                .or_else(|| self.policy.and_then(|p| p.blocks()))
                .is_some_and(|b| b.planner.queued())
                || queue.cursor > queue.pending.len()
                || queue.known_next_id == 0
                || queue.known_next_id > state.next_id
                || queue.pending.len() > state.next_id as usize
                || queue.retry.len() > state.next_id as usize
                || queue.second_pass && !queue.retry.is_empty()
            {
                return Err(fail("invalid priority planning continuation"));
            }
            for ids in [&queue.pending, &queue.retry] {
                let mut unique = HashSet::new();
                if ids
                    .iter()
                    .any(|id| id.0 == 0 || id.0 >= queue.known_next_id || !unique.insert(*id))
                {
                    return Err(fail("invalid priority unit queue"));
                }
            }
            if queue
                .active
                .is_some_and(|id| id.0 == 0 || id.0 >= queue.known_next_id)
            {
                return Err(fail("invalid active priority unit"));
            }
            if queue.attempts.len() > state.next_id as usize * 3
                || queue
                    .attempts
                    .iter()
                    .any(|id| id.0 == 0 || id.0 >= queue.known_next_id)
            {
                return Err(fail("invalid priority search budget"));
            }
            let mut counts = std::collections::HashMap::new();
            for id in &queue.attempts {
                let count = counts.entry(id).or_insert(0);
                *count += 1;
                if *count > 3 {
                    return Err(fail("priority unit exceeds its search budget"));
                }
            }
        }
        let identity = |id: UnitId| id.0 > 0 && id.0 < state.next_id;
        let actor = |id: UnitId| state.units.iter().find(|u| u.id == format!("u{}", id.0));
        let friendly = |id: UnitId| actor(id).is_some_and(|u| u.side == state.turn);
        if self.cooperation.candidates > 128
            || self.cooperation.attempted.len() > state.next_id as usize
            || self.cooperation.fallback.len() > state.next_id as usize
            || self
                .cooperation
                .attempted
                .iter()
                .chain(&self.cooperation.fallback)
                .any(|id| !identity(*id))
        {
            return Err(fail("invalid cooperative search continuation"));
        }
        if let Some(orders) = &self.orders {
            let valid_tile = |tile: &TileId| tiles.contains(tile.0.as_str());
            let valid_actor =
                |id: UnitId| identity(id) && actor(id).is_none_or(|u| u.side == state.turn);
            if orders
                .recruitment
                .reserved
                .iter()
                .any(|tile| !valid_tile(tile))
                || orders
                    .tactics
                    .withdrawal_tiles
                    .iter()
                    .chain(&orders.tactics.defensive_tiles)
                    .chain(&orders.tactics.movement_tiles)
                    .chain(orders.tactics.leader_post.iter())
                    .any(|tile| !valid_tile(tile))
                || orders
                    .tactics
                    .terminal_arrivals
                    .iter()
                    .any(|(id, tile)| !valid_actor(*id) || !valid_tile(tile))
                || orders
                    .tactics
                    .terminal_targets
                    .iter()
                    .any(|id| !identity(*id))
                || orders.tactics.targets.iter().any(|goal| {
                    !goal.weight.is_finite()
                        || goal.weight <= 0.0
                        || goal.tiles.iter().any(|tile| !tiles.contains(tile.as_str()))
                })
                || orders
                    .recruitment
                    .requirements
                    .iter()
                    .any(|requirement| match requirement {
                        super::orders::RecruitRequirement::EnsureUnits { selector, count } => {
                            *count > 1_000_000
                                || match selector {
                                    super::orders::UnitSelector::Type(kind) => {
                                        !catalog.types.contains_key(&kind.0)
                                    }
                                    super::orders::UnitSelector::Family(_)
                                    | super::orders::UnitSelector::Role(_) => false,
                                }
                        }
                    })
                || orders
                    .tactics
                    .constraints
                    .iter()
                    .any(|constraint| match constraint {
                        super::orders::TacticalConstraint::Survive { actor } => {
                            !valid_actor(*actor)
                        }
                    })
                || orders.tactics.objectives.iter().any(|order| {
                    order.id.is_empty()
                        || match &order.objective {
                            super::orders::TacticalObjective::GoTo { actor, destination }
                            | super::orders::TacticalObjective::ClearPassage {
                                actor,
                                destination,
                            } => !valid_actor(*actor) || !valid_tile(destination),
                            super::orders::TacticalObjective::Occupy { actor, location } => {
                                !valid_actor(*actor) || !valid_tile(location)
                            }
                            super::orders::TacticalObjective::Escort {
                                actor,
                                guards,
                                selector: _,
                            } => !valid_actor(*actor) || *guards > 1_000_000,
                        }
                })
            {
                return Err(fail("invalid campaign planning orders"));
            }
        }
        if self.recruitment_remaining > 1_000_000
            || self.tactical_remaining.is_some_and(|n| n > 1_000_000)
        {
            return Err(fail("planning budget is out of range"));
        }
        if self.classic_ids.len() > state.next_id as usize
            || self.commands.len() > state.next_id as usize * 4 + 16
        {
            return Err(fail("planning continuation is out of range"));
        }
        if self.classic_ids.iter().any(|id| !identity(*id))
            || self.commitments.iter().any(|(id, _)| !identity(*id))
        {
            return Err(fail("unknown planning unit identity"));
        }
        let mut unique = HashSet::new();
        if self.classic_ids.iter().any(|id| !unique.insert(*id)) {
            return Err(fail("duplicate classic planning unit"));
        }
        unique.clear();
        if self.commitments.iter().any(|(id, _)| !unique.insert(*id)) {
            return Err(fail("duplicate planning commitment"));
        }
        for goal in self
            .goals
            .iter()
            .flatten()
            .chain(&self.classic_objectives)
            .chain(
                self.orders
                    .iter()
                    .flat_map(|orders| &orders.tactics.targets),
            )
        {
            if !goal.weight.is_finite()
                || goal.weight <= 0.0
                || goal.tiles.iter().any(|t| !tiles.contains(t.as_str()))
            {
                return Err(fail("invalid planning objective"));
            }
            if goal.actor.as_ref().is_some_and(|id| {
                id.strip_prefix('u')
                    .and_then(|id| id.parse::<u32>().ok())
                    .is_none_or(|id| !identity(UnitId(id)))
            }) {
                return Err(fail("invalid objective actor"));
            }
        }
        let validate_attack = |unit: UnitId, target: UnitId, weapon: &WeaponId| {
            friendly(unit)
                && actor(target).is_some()
                && actor(unit).is_some_and(|u| {
                    catalog.types.get(&u.r#type).is_some_and(|kind| {
                        catalog.units[*kind]
                            .attacks
                            .iter()
                            .any(|a| a.id == weapon.0)
                    })
                })
        };
        for command in &self.commands {
            let valid = match command {
                Command::Move { unit, tile } => friendly(*unit) && tiles.contains(tile.0.as_str()),
                Command::Attack {
                    unit,
                    target,
                    weapon,
                } => validate_attack(*unit, *target, weapon),
                Command::Recruit { unit, kind, tile } => {
                    friendly(*unit)
                        && catalog.types.contains_key(&kind.0)
                        && tiles.contains(tile.0.as_str())
                }
                Command::Advance { unit, kind } => {
                    friendly(*unit) && catalog.types.contains_key(&kind.0)
                }
                Command::SetUnitIntent { unit, .. } => friendly(*unit),
                Command::EndTurn
                | Command::DismissDialogue
                | Command::NextMission
                | Command::RecallMission { .. }
                | Command::RetryMission
                | Command::PreviousDialogue
                | Command::SkipDialogue => false,
            };
            if !valid {
                return Err(fail("invalid queued planning command"));
            }
        }
        if state.winner.is_none()
            && self
                .attack_continuation
                .as_ref()
                .is_some_and(|(unit, target, weapon)| !validate_attack(*unit, *target, weapon))
        {
            return Err(fail("invalid planned attack continuation"));
        }
        Ok(())
    }
}
