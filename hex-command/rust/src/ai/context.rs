//! Capability boundary between authored campaigns and mechanical battle queries.
use super::{PlanningWorkspace, orders::*};
use crate::logic::{battle::Board, geometry::TileSet};
use crate::{data::*, logic::Game};
use std::rc::Rc;

pub(super) struct BattleRules<'a> {
    pub units: &'a [UnitRule],
    pub recruitment: &'a RecruitmentProfiles,
}

enum EngineAccess<'a, 'catalog> {
    Borrowed(&'a Game<'catalog>),
    Draft(Game<'catalog>),
}

/// Read-only mechanical view. It exposes neither authored goals nor campaign
/// counters, and has no Deref or raw-engine accessor outside this module.
pub(crate) struct BattleView<'a, 'catalog> {
    engine: EngineAccess<'a, 'catalog>,
}
impl<'a, 'catalog> BattleView<'a, 'catalog> {
    pub(super) fn new(game: &'a Game<'catalog>) -> Self {
        Self {
            engine: EngineAccess::Borrowed(game),
        }
    }
    fn engine(&self) -> &Game<'catalog> {
        match &self.engine {
            EngineAccess::Borrowed(game) => game,
            EngineAccess::Draft(game) => game,
        }
    }
    fn engine_mut(&mut self) -> &mut Game<'catalog> {
        match &mut self.engine {
            EngineAccess::Draft(game) => game,
            EngineAccess::Borrowed(_) => panic!("read-only battle view"),
        }
    }
    pub(super) fn fork(&self) -> BattleDraft<'a, 'catalog> {
        BattleDraft(BattleView {
            engine: EngineAccess::Draft(self.engine().planning_fork()),
        })
    }
    pub(super) fn rules(&self) -> BattleRules<'_> {
        BattleRules {
            units: &self.engine().rules.units,
            recruitment: &self.engine().rules.recruitment,
        }
    }
    pub(super) fn board(&self) -> &Board {
        &self.engine().board
    }
    pub(super) fn units(&self) -> &Vec<Unit> {
        &self.engine().units
    }

    pub(super) fn terrain(&self) -> &Vec<usize> {
        &self.engine().terrain
    }
    pub(super) fn owners(&self) -> &Vec<Option<Side>> {
        &self.engine().owners
    }
    pub(super) fn map_id(&self) -> &str {
        &self.engine().map_id
    }
    pub(super) fn turn(&self) -> Side {
        self.engine().turn
    }
    pub(super) fn round(&self) -> u32 {
        self.engine().round
    }
    pub(super) fn gold(&self) -> [i32; 2] {
        self.engine().gold
    }
    pub(super) fn winner(&self) -> Option<Side> {
        self.engine().winner
    }
    pub(super) fn next_id(&self) -> u32 {
        self.engine().next_id
    }
    pub(super) fn occupied_tiles(&self) -> &TileSet {
        &self.engine().occupied_tiles
    }

    pub(super) fn attack_origins(&self, category: Category, range: u8) -> Rc<Vec<TileSet>> {
        self.engine().attack_origins(category, range)
    }
    pub(super) fn can_attack(
        &self,
        unit: &Unit,
        from: usize,
        target: &Unit,
        weapon: usize,
    ) -> bool {
        self.engine().can_attack(unit, from, target, weapon)
    }
    pub(super) fn attack_options(&self, unit: &Unit, target: &Unit, from: usize) -> Vec<usize> {
        self.engine().attack_options(unit, target, from)
    }
    pub(super) fn damage(&self, attacker: &Unit, target: &Unit, weapon: usize) -> i32 {
        self.engine().damage(attacker, target, weapon)
    }
    pub(super) fn exchange(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> Rc<crate::logic::exchange::ExchangeForecast> {
        self.engine().exchange(attacker, target, weapon, from)
    }
    pub(super) fn exchange_with_thorns(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
        reflection: i32,
    ) -> Rc<crate::logic::exchange::ExchangeForecast> {
        self.engine()
            .exchange_with_thorns(attacker, target, weapon, from, reflection)
    }
    /// Optimistic full-weapon values bound searches before an attack origin is chosen.
    pub(super) fn potential_damage(&self, attacker: &Unit, target: &Unit, weapon: usize) -> f64 {
        let distribution = self
            .engine()
            .damage_distribution(attacker, target, weapon, target.tile);
        // Drain can replace HP between strikes, so damage may exceed starting HP.
        if self.rules().units[target.kind]
            .attacks
            .iter()
            .any(|attack| attack.drain > 0)
        {
            distribution.mean
        } else {
            distribution.effective(target.hp)
        }
    }
    pub(super) fn potential_kill_chance(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
    ) -> f64 {
        self.engine()
            .damage_distribution(attacker, target, weapon, target.tile)
            .chance(target.hp)
    }
    pub(super) fn maximum_retaliation(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> i32 {
        self.engine()
            .maximum_retaliation(attacker, target, weapon, from)
    }
    pub(super) fn death_chance(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> f64 {
        self.engine().death_chance(attacker, target, weapon, from)
    }
    pub(super) fn preview(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> (i32, i32, Option<usize>) {
        self.engine().preview(unit, target, weapon, from)
    }
    pub(super) fn best_attack(&self, unit: &Unit, target: &Unit, from: usize) -> Option<usize> {
        self.engine().best_attack(unit, target, from)
    }
    pub(super) fn unit(&self, id: u32) -> Option<&Unit> {
        self.engine().unit(id)
    }
    pub(super) fn at(&self, tile: usize) -> Option<&Unit> {
        self.engine().at(tile)
    }
    pub(super) fn index(&self, id: u32) -> Option<usize> {
        self.engine().index(id)
    }
    pub(super) fn site(&self, id: &str) -> usize {
        self.engine().site(id)
    }
    pub(super) fn kind(&self, name: &str) -> usize {
        self.engine().kind(name)
    }
    pub(super) fn terrain_is(&self, tile: usize, terrain: Terrain) -> bool {
        self.engine().terrain_is(tile, terrain)
    }
    pub(super) fn promotion_options(&self, unit: &Unit) -> Vec<usize> {
        self.engine().promotion_options(unit)
    }
    pub(super) fn recruitment_options(&self, side: Side) -> Vec<usize> {
        self.engine().recruitment_options(side)
    }
    pub(super) fn placement_tiles(&self, leader: &Unit, kind: usize, price: bool) -> Vec<usize> {
        self.engine().placement_tiles(leader, kind, price)
    }
    pub(super) fn threat_reach_blocked(&self, unit: &Unit, occupied: TileSet) -> TileSet {
        self.engine().threat_reach_blocked(unit, occupied)
    }
    pub(super) fn cost(&self, kind: usize, tile: usize) -> u16 {
        self.engine().cost(kind, tile)
    }
    pub(super) fn reachable(&self, unit: &Unit) -> Rc<Vec<(usize, u16)>> {
        self.engine().reachable(unit)
    }
    pub(super) fn routes(&self, kind: usize, goals: &[usize]) -> Rc<Vec<u16>> {
        self.engine().routes(kind, goals)
    }
    pub(super) fn occupied_routes(&self, unit: &Unit, goals: &[usize], yielding: bool) -> Vec<u16> {
        self.engine().occupied_routes(unit, goals, yielding)
    }
    pub(super) fn occupied_routes_with_penalty(
        &self,
        unit: &Unit,
        goals: &[usize],
        friendly_penalty: Option<u16>,
    ) -> Vec<u16> {
        self.engine()
            .occupied_routes_with_penalty(unit, goals, friendly_penalty)
    }
    pub(super) fn income(&self, side: Side) -> i32 {
        self.engine().income(side)
    }
}
/// Owned temporary planning state. Only drafts can apply mechanical commands.
pub(super) struct BattleDraft<'a, 'catalog>(BattleView<'a, 'catalog>);
impl<'a, 'catalog> std::ops::Deref for BattleDraft<'a, 'catalog> {
    type Target = BattleView<'a, 'catalog>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, 'catalog> BattleDraft<'a, 'catalog> {
    pub(super) fn units_mut(&mut self) -> &mut Vec<Unit> {
        &mut self.0.engine_mut().units
    }
    /// Mechanical phase forecast includes engine-resolved healing and arrivals.
    pub(super) fn end_turn(&mut self) {
        self.0.engine_mut().arena_first = None;
        self.0.engine_mut().end_turn();
    }
    pub(super) fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        self.0.engine_mut().execute(command).map(|_| ())
    }
    pub(super) fn record(&mut self, message: String) {
        self.0.engine_mut().record(message);
    }
}
#[path = "context/forecast.rs"]
pub(super) mod forecast;
pub(crate) struct CampaignView<'a, 'catalog> {
    game: &'a Game<'catalog>,
    workspace: &'a PlanningWorkspace,
}
impl<'a, 'catalog> CampaignView<'a, 'catalog> {
    pub(super) fn new(game: &'a Game<'catalog>, workspace: &'a PlanningWorkspace) -> Self {
        Self { game, workspace }
    }
    pub(super) fn orders(&self, kind: super::campaigner::CampaignerKind) -> CampaignOrders {
        kind.campaign_orders(self.game, self.workspace)
    }
}

/// Exactly one isolated engine state for recruitment and tactical execution.
pub(super) struct PlanningContext<'catalog> {
    draft: BattleDraft<'catalog, 'catalog>,
    commands: Vec<Command>,
}
impl<'catalog> PlanningContext<'catalog> {
    pub(super) fn fork(game: &Game<'catalog>) -> Self {
        Self {
            draft: BattleDraft(BattleView {
                engine: EngineAccess::Draft(game.planning_fork()),
            }),
            commands: Vec::new(),
        }
    }
    pub(super) fn recruitment_context<'a>(
        &'a mut self,
        seed: u32,
        budget: Option<usize>,
    ) -> RecruitmentContext<'a, 'catalog> {
        RecruitmentContext {
            draft: &mut self.draft,
            commands: &mut self.commands,
            seed,
            budget,
        }
    }
    pub(super) fn tactical_context<'a>(
        &'a mut self,
        workspace: &'a mut PlanningWorkspace,
    ) -> TacticalContext<'a, 'catalog> {
        TacticalContext {
            draft: &mut self.draft,
            commands: &mut self.commands,
            workspace,
            combat_boundary: false,
        }
    }
    pub(super) fn finish(
        self,
        recruitment: RecruitmentReport,
        planning: PlanningReport,
    ) -> TurnPlan {
        TurnPlan {
            commands: self.commands,
            recruitment,
            planning,
        }
    }
}
pub(crate) struct RecruitmentContext<'a, 'catalog> {
    pub(super) draft: &'a mut BattleDraft<'catalog, 'catalog>,
    commands: &'a mut Vec<Command>,
    pub(super) seed: u32,
    pub(super) budget: Option<usize>,
}
impl RecruitmentContext<'_, '_> {
    pub(super) fn battle(&self) -> &BattleView<'_, '_> {
        self.draft
    }
    pub(super) fn emit(&mut self, command: Command) -> Result<(), CommandError> {
        self.draft.execute(command.clone())?;
        self.commands.push(command);
        Ok(())
    }
}
pub(crate) struct TacticalContext<'a, 'catalog> {
    pub(super) combat_boundary: bool,
    pub(super) draft: &'a mut BattleDraft<'catalog, 'catalog>,
    commands: &'a mut Vec<Command>,
    pub(super) workspace: &'a mut PlanningWorkspace,
}
impl TacticalContext<'_, '_> {
    pub(super) fn battle(&self) -> &BattleView<'_, '_> {
        self.draft
    }
    pub(super) fn emit(&mut self, command: Command) -> Result<(), CommandError> {
        self.draft.execute(command.clone())?;
        self.combat_boundary = command.is_attack();
        self.commands.push(command);
        Ok(())
    }
    pub(super) fn flush(&mut self) -> Result<(), CommandError> {
        while let Some(command) = self.workspace.commands.pop_front() {
            if self.draft.winner().is_some() {
                self.workspace.commands.clear();
                self.workspace.attack_continuation = None;
                self.workspace.finished = true;
                return Ok(());
            }
            self.emit(command)?;
            if self.combat_boundary {
                self.workspace.commands.clear();
                return Ok(());
            }
        }
        if self.draft.winner().is_some() {
            self.workspace.attack_continuation = None;
            self.workspace.finished = true;
            return Ok(());
        }
        if let Some((unit, target, weapon)) = self.workspace.attack_continuation.take() {
            let actor = self
                .draft
                .unit(unit.0)
                .ok_or(CommandError::UnknownUnit(unit))?
                .clone();
            if actor.retreating.is_some() {
                self.emit(super::intent_command(
                    &actor,
                    None,
                    actor.rebuilding,
                    actor.attacked,
                ))?;
            }
            self.emit(Command::Attack {
                unit,
                target,
                weapon,
            })?;
            return Ok(());
        }
        Ok(())
    }
}
