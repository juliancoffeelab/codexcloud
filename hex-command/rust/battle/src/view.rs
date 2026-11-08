//! Borrowed rule queries and explicitly granted mechanical projections.
//! Query delegates are inlineable across the crate boundary, including native
//! simulation builds without LTO. Large engine algorithms retain their own bodies.
#[cfg(feature = "wasm")]
use crate::logic::battle::RuntimeCache;
use crate::{data::*, logic::Game, logic::battle::Board, logic::geometry::TileSet, model::*};
use std::rc::Rc;

pub struct BattleRules<'a> {
    pub units: &'a [UnitRule],
    pub recruitment: &'a RecruitmentProfiles,
}

enum EngineAccess<'a, 'catalog> {
    Borrowed(&'a Game<'catalog>),
    Draft(Game<'catalog>),
}

/// Read-only mechanical view. It exposes neither authored goals nor campaign
/// counters, and has no Deref or raw-engine accessor outside this module.
pub struct BattleView<'a, 'catalog> {
    engine: EngineAccess<'a, 'catalog>,
    inactive: Rc<rustc_hash::FxHashSet<u32>>,
    scoped: Rc<Vec<crate::model::ScopedOrder>>,
}
impl<'a, 'catalog> BattleView<'a, 'catalog> {
    /// Borrow from an existing owner; this cannot manufacture an owner from a view.
    #[inline]
    pub fn new(session: &'a BattleSession<'catalog>) -> Self {
        Self {
            engine: EngineAccess::Borrowed(session.0.engine()),
            inactive: session.0.inactive.clone(),
            scoped: session.0.scoped.clone(),
        }
    }
    #[inline]
    pub(crate) fn engine(&self) -> &Game<'catalog> {
        match &self.engine {
            EngineAccess::Borrowed(game) => game,
            EngineAccess::Draft(game) => game,
        }
    }
    #[inline]
    fn engine_mut(&mut self) -> &mut Game<'catalog> {
        match &mut self.engine {
            EngineAccess::Draft(game) => game,
            EngineAccess::Borrowed(_) => panic!("read-only battle view"),
        }
    }
    #[inline]
    pub fn rules(&self) -> BattleRules<'_> {
        BattleRules {
            units: &self.engine().rules.units,
            recruitment: &self.engine().rules.recruitment,
        }
    }
    #[inline]
    pub fn weapons<'b>(&'b self, unit: &'b Unit) -> impl Iterator<Item = &'b Weapon> {
        self.engine().weapons(unit)
    }
    #[inline]
    pub fn weapon<'b>(&'b self, unit: &'b Unit, index: usize) -> &'b Weapon {
        self.engine().weapon(unit, index)
    }
    #[inline]
    pub fn board(&self) -> &Board {
        &self.engine().board
    }
    #[inline]
    pub fn can_act(&self, id: u32) -> bool {
        !self.inactive.contains(&id)
            && self
                .engine()
                .unit(id)
                .is_some_and(|u| self.engine().may_act(u))
    }
    #[inline]
    pub fn permits(&self, command: &Command) -> bool {
        match command {
            Command::Move { unit, tile } => {
                self.can_act(unit.0)
                    && self.unit(unit.0).is_some_and(|actor| {
                        self.scoped
                            .iter()
                            .filter(|order| order.actor == *unit)
                            .all(|order| order.allows_move(self, actor, self.site(&tile.0)))
                    })
            }
            Command::Attack { unit, target, .. } => {
                self.can_act(unit.0)
                    && self
                        .unit(unit.0)
                        .zip(self.unit(target.0))
                        .is_some_and(|(actor, target)| {
                            self.scoped
                                .iter()
                                .filter(|order| order.actor == *unit)
                                .all(|order| order.allows_attack(self, actor.tile, target.tile))
                        })
            }
            Command::Recruit { unit, .. }
            | Command::Advance { unit, .. }
            | Command::SetUnitIntent { unit, .. } => self.can_act(unit.0),
            _ => true,
        }
    }
    #[inline]
    pub fn available_combatant(&self, unit: &Unit) -> bool {
        self.engine().available_combatant(unit)
    }
    #[inline]
    pub fn hostile(&self, first: &Unit, second: &Unit) -> bool {
        self.engine().hostile(first, second)
    }
    #[inline]
    pub fn same_faction(&self, first: &Unit, second: &Unit) -> bool {
        self.engine().unit_faction(first) == self.engine().unit_faction(second)
    }
    #[inline]
    pub fn allied(&self, first: &Unit, second: &Unit) -> bool {
        self.engine().allied(first, second)
    }
    #[inline]
    pub fn hostile_to_phase(&self, unit: &Unit, side: Side) -> bool {
        self.engine().hostile_to_phase(unit, side)
    }
    #[inline]
    pub fn belongs_to_phase(&self, unit: &Unit, side: Side) -> bool {
        self.engine().belongs_to_phase(unit, side)
    }
    #[inline]
    pub fn control_roster(&self) -> Vec<(u32, Side, Option<String>, Option<Role>)> {
        self.engine().control_roster()
    }
    #[inline]
    pub fn units(&self) -> &Vec<Unit> {
        &self.engine().units
    }

    #[inline]
    pub fn terrain(&self) -> &Vec<usize> {
        &self.engine().terrain
    }
    #[inline]
    pub fn owners(&self) -> &Vec<Option<String>> {
        &self.engine().owners
    }
    #[inline]
    pub fn map_id(&self) -> &str {
        &self.engine().map_id
    }
    #[inline]
    pub fn turn(&self) -> Side {
        self.engine().turn
    }
    #[inline]
    pub fn round(&self) -> u32 {
        self.engine().round
    }
    #[inline]
    pub fn gold_for_phase(&self, side: Side) -> i32 {
        self.engine().gold_for_phase(side)
    }
    #[inline]
    pub fn gold_for_unit(&self, unit: &Unit) -> i32 {
        self.engine().gold_for_unit(unit)
    }
    #[inline]
    pub fn owns(&self, tile: usize, side: Side) -> bool {
        self.engine().owns(tile, side)
    }
    #[inline]
    pub fn captures(&self, tile: usize, side: Side) -> bool {
        self.engine().captures(tile, side)
    }
    #[inline]
    pub fn recruitment_pool_for(&self, unit: &Unit) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.engine().recruitment_pool_for(unit)
    }
    #[inline]
    pub fn recruitment_options_for(&self, unit: &Unit) -> Vec<usize> {
        self.engine().recruitment_options_for(unit)
    }
    #[inline]
    pub fn recruit_cost(&self, unit: &Unit, kind: usize) -> i32 {
        self.engine().recruit_cost(unit, kind)
    }
    #[inline]
    pub fn winner(&self) -> Option<Side> {
        self.engine().winner
    }
    #[inline]
    pub fn next_id(&self) -> u32 {
        self.engine().next_id
    }
    #[inline]
    pub fn occupied_tiles(&self) -> &TileSet {
        &self.engine().occupied_tiles
    }

    #[inline]
    pub fn attack_origins(&self, category: Category, range: u8) -> Rc<Vec<TileSet>> {
        self.engine().attack_origins(category, range)
    }
    #[inline]
    pub fn can_attack(&self, unit: &Unit, from: usize, target: &Unit, weapon: usize) -> bool {
        self.can_act(unit.id)
            && self
                .scoped
                .iter()
                .filter(|order| order.actor.0 == unit.id)
                .all(|order| order.allows_attack(self, from, target.tile))
            && self.engine().can_attack(unit, from, target, weapon)
    }
    #[inline]
    pub fn attack_options(&self, unit: &Unit, target: &Unit, from: usize) -> Vec<usize> {
        if !self.can_act(unit.id)
            || !self
                .scoped
                .iter()
                .filter(|order| order.actor.0 == unit.id)
                .all(|order| order.allows_attack(self, from, target.tile))
        {
            return vec![];
        }
        self.engine().attack_options(unit, target, from)
    }
    /// Hypothetical capability, independent of whose turn it is or which actors
    /// the planner controls. Command authorization stays in permits/can_act.
    #[inline]
    pub fn forecast_attack_options(&self, unit: &Unit, target: &Unit, from: usize) -> Vec<usize> {
        self.engine().attack_options(unit, target, from)
    }
    #[inline]
    pub fn damage(&self, attacker: &Unit, target: &Unit, weapon: usize) -> i32 {
        self.engine().damage(attacker, target, weapon)
    }
    #[inline]
    pub fn exchange(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> Rc<crate::logic::exchange::ExchangeForecast> {
        self.engine().exchange(attacker, target, weapon, from)
    }
    #[inline]
    pub fn exchange_with_thorns(
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
    #[inline]
    pub fn potential_damage(&self, attacker: &Unit, target: &Unit, weapon: usize) -> f64 {
        let distribution = self
            .engine()
            .damage_distribution(attacker, target, weapon, target.tile);
        // Drain can replace HP between strikes, so damage may exceed starting HP.
        if self.weapons(target).any(|attack| attack.drain > 0) {
            distribution.mean
        } else {
            distribution.effective(target.hp)
        }
    }
    #[inline]
    pub fn potential_kill_chance(&self, attacker: &Unit, target: &Unit, weapon: usize) -> f64 {
        self.engine()
            .damage_distribution(attacker, target, weapon, target.tile)
            .chance(target.hp)
    }
    #[inline]
    pub fn maximum_retaliation(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> i32 {
        self.engine()
            .maximum_retaliation(attacker, target, weapon, from)
    }
    #[inline]
    pub fn death_chance(&self, attacker: &Unit, target: &Unit, weapon: usize, from: usize) -> f64 {
        self.engine().death_chance(attacker, target, weapon, from)
    }
    #[inline]
    pub fn preview(
        &self,
        unit: &Unit,
        target: &Unit,
        weapon: usize,
        from: usize,
    ) -> (i32, i32, Option<usize>) {
        self.engine().preview(unit, target, weapon, from)
    }
    #[inline]
    pub fn unit(&self, id: u32) -> Option<&Unit> {
        self.engine().unit(id)
    }
    #[inline]
    pub fn at(&self, tile: usize) -> Option<&Unit> {
        self.engine().at(tile)
    }
    #[inline]
    pub fn index(&self, id: u32) -> Option<usize> {
        self.engine().index(id)
    }
    #[inline]
    pub fn site(&self, id: &Hex) -> usize {
        self.engine().site(id)
    }
    #[inline]
    pub fn unit_name(&self, unit: &Unit) -> String {
        self.engine().unit_name(unit).to_owned()
    }
    #[inline]
    pub fn kind(&self, name: &str) -> usize {
        self.engine().kind(name)
    }
    /// Base mechanical terrain properties before a unit's racial/type modifiers.
    /// Ordering: melee blade/pierce/impact, then ranged blade/pierce/impact.
    pub fn base_terrain_properties(&self, tile: usize) -> (u16, bool, [i32; 6]) {
        let t = &self.engine().rules.terrain_definitions[self.terrain()[tile]];
        let m = &t.damage_modifiers;
        (
            t.cost,
            t.passable,
            [
                m.melee + m.blade,
                m.melee + m.pierce,
                m.melee + m.impact,
                m.ranged + m.blade,
                m.ranged + m.pierce,
                m.ranged + m.impact,
            ],
        )
    }
    /// Authored terrain healing amount; ownership is observed separately.
    #[inline]
    pub fn terrain_healing(&self, tile: usize) -> i32 {
        self.engine().terrain_role(tile).healing()
    }
    #[inline]
    pub fn role_is(&self, tile: usize, role: TerrainRole) -> bool {
        self.engine().role_is(tile, role)
    }
    #[inline]
    pub fn is_keep(&self, tile: usize) -> bool {
        self.engine().is_keep(tile)
    }
    #[inline]
    pub fn promotion_options(&self, unit: &Unit) -> Vec<usize> {
        if !self.can_act(unit.id) {
            return vec![];
        }
        self.engine().promotion_options(unit)
    }
    #[inline]
    pub fn recruitment_options(&self, side: Side) -> Vec<usize> {
        self.engine().recruitment_options(side)
    }
    #[inline]
    pub fn placement_tiles(&self, leader: &Unit, kind: usize, price: bool) -> Vec<usize> {
        if !self.can_act(leader.id) {
            return vec![];
        }
        self.engine().placement_tiles(leader, kind, price)
    }
    #[inline]
    pub fn threat_reach_blocked(&self, unit: &Unit, occupied: TileSet) -> TileSet {
        self.engine().threat_reach_blocked(unit, occupied)
    }
    #[inline]
    pub fn cost(&self, kind: usize, tile: usize) -> u16 {
        self.engine().cost(kind, tile)
    }
    #[inline]
    pub fn reachable(&self, unit: &Unit) -> Rc<Vec<(usize, u16)>> {
        if !self.can_act(unit.id) {
            return Rc::new(vec![]);
        }
        let reachable = self.engine().reachable(unit);
        if self.scoped.is_empty() {
            return reachable;
        }
        Rc::new(
            reachable
                .iter()
                .copied()
                .filter(|(to, _)| {
                    self.scoped
                        .iter()
                        .filter(|order| order.actor.0 == unit.id)
                        .all(|order| order.allows_move(self, unit, *to))
                })
                .collect(),
        )
    }
    #[inline]
    pub fn routes(&self, kind: usize, goals: &[usize]) -> Rc<Vec<u16>> {
        self.engine().routes(kind, goals)
    }
    #[inline]
    pub fn routes_from(&self, kind: usize, start: usize) -> Rc<Vec<u16>> {
        self.engine().routes_from(kind, start)
    }
    #[inline]
    pub fn occupied_routes(&self, unit: &Unit, goals: &[usize], yielding: bool) -> Vec<u16> {
        self.engine().occupied_routes(unit, goals, yielding)
    }
    #[inline]
    pub fn occupied_routes_with_penalty(
        &self,
        unit: &Unit,
        goals: &[usize],
        friendly_penalty: Option<u16>,
    ) -> Vec<u16> {
        self.engine()
            .occupied_routes_with_penalty(unit, goals, friendly_penalty)
    }
    #[inline]
    pub fn income(&self, side: Side) -> i32 {
        self.engine().income(side)
    }
}

/// An opaque live battle. Only its owner can mint mechanical forecast sources.
pub struct BattleSession<'catalog>(BattleView<'catalog, 'catalog>);
impl<'catalog> std::ops::Deref for BattleSession<'catalog> {
    type Target = BattleView<'catalog, 'catalog>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'catalog> BattleSession<'catalog> {
    fn owned(game: Game<'catalog>) -> Self {
        Self(BattleView {
            engine: EngineAccess::Draft(game),
            inactive: Rc::default(),
            scoped: Rc::default(),
        })
    }
    pub fn from_state(rules: &'catalog Rules, state: State) -> Self {
        Self::owned(Game::from_state(rules, state))
    }
    pub fn from_state_with_campaign(
        rules: &'catalog Rules,
        state: State,
        definition: Option<&'catalog Campaign>,
    ) -> Self {
        Self::owned(Game::from_state_with_campaign(rules, state, definition))
    }
    pub fn new(rules: &'catalog Rules, map: &str) -> Self {
        Self::owned(Game::new_with_campaign(rules, map, None))
    }
    pub fn new_with_campaign(
        rules: &'catalog Rules,
        map: &str,
        definition: Option<&'catalog Campaign>,
    ) -> Self {
        Self::owned(Game::new_with_campaign(rules, map, definition))
    }
    pub fn start_campaign(
        rules: &'catalog Rules,
        id: &str,
        index: usize,
        veterans: Vec<Veteran>,
        carry: i32,
        choices: Vec<String>,
        story: Vec<Veteran>,
        fallen: Vec<String>,
    ) -> Self {
        Self::owned(Game::start_campaign(
            rules, id, index, veterans, carry, choices, story, fallen,
        ))
    }
    pub fn start_campaign_from_definition(
        rules: &'catalog Rules,
        definition: &'catalog Campaign,
        index: usize,
        veterans: Vec<Veteran>,
        carry: i32,
        flags: FlagHistory,
        story: Vec<Veteran>,
        fallen: Vec<String>,
    ) -> Self {
        Self::owned(Game::start_campaign_from_definition(
            rules, definition, index, veterans, carry, flags, story, fallen,
        ))
    }
    #[inline]
    pub fn view(&self) -> &BattleView<'catalog, 'catalog> {
        &self.0
    }
    #[inline]
    pub fn campaign_view(&self) -> CampaignView<'_, 'catalog> {
        CampaignView { view: &self.0 }
    }
    pub fn state(&self) -> State {
        self.0.engine().state()
    }
    pub fn campaign_snapshot(&self) -> Option<CampaignState> {
        self.0.engine().campaign_snapshot()
    }
    #[inline]
    pub fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        self.0.engine_mut().execute(command)
    }
    pub fn begin_turn(&mut self, side: Side) {
        self.0.engine_mut().begin_turn(side);
    }
    pub fn end_turn(&mut self) {
        self.0.engine_mut().end_turn();
    }
    pub fn record(&mut self, message: String) {
        self.0.engine_mut().record(message);
    }
    pub fn fork(&self) -> Self {
        Self::owned(self.0.engine().fork())
    }
    pub fn planning_fork(&self) -> Self {
        Self::owned(self.0.engine().planning_fork())
    }
    #[inline]
    pub fn action_forecasts(&self) -> ActionForecastSource<'_, 'catalog> {
        ActionForecastSource { view: &self.0 }
    }
    #[inline]
    pub fn phase_forecasts(&self) -> PhaseForecasts<'_, 'catalog> {
        PhaseForecasts { view: &self.0 }
    }
    pub fn restrict_positions(&mut self, scoped: &[ScopedOrder]) {
        self.0.scoped = Rc::new(scoped.to_vec());
    }
    pub fn restrict_actors(&mut self, inactive: &[UnitId]) {
        self.0.inactive = Rc::new(inactive.iter().map(|id| id.0).collect());
    }
    pub fn set_active_faction(&mut self, faction: Option<String>) {
        self.0.engine_mut().active_faction = faction;
    }
    pub fn set_combat_rng(&mut self, seed: u32) {
        self.0.engine_mut().combat_rng = seed;
    }
    pub fn configure_arena(&mut self, pools: Option<[Vec<usize>; 2]>, first: Option<Side>) {
        self.0.engine_mut().pools = pools;
        self.0.engine_mut().arena_first = first;
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_recording(&mut self) {
        self.0.engine_mut().recordings = Some(Records::default());
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn take_recordings(&mut self) -> Option<Records> {
        self.0.engine_mut().recordings.take()
    }
    pub fn clear_log(&mut self) {
        self.0.engine_mut().log.clear();
    }
    pub fn dismiss_all_dialogue(&mut self) {
        if let Some(campaign) = self.0.engine_mut().campaign.as_mut() {
            campaign.dialogue_index = campaign.dialogue.len();
        }
    }
    #[cfg(feature = "wasm")]
    pub fn take_combat_animation(&mut self) -> Option<CombatAnimation> {
        self.0.engine_mut().combat_animation.take()
    }
    #[cfg(feature = "wasm")]
    pub fn runtime_cache(&self) -> RuntimeCache {
        self.0.engine().runtime_cache()
    }
    #[cfg(feature = "wasm")]
    pub fn into_state(self, tiles: Vec<Tile>) -> State {
        match self.0.engine {
            EngineAccess::Draft(game) => game.into_state(tiles),
            EngineAccess::Borrowed(_) => unreachable!(),
        }
    }
    #[cfg(feature = "wasm")]
    pub fn from_state_with_board(rules: &'catalog Rules, state: State, board: Rc<Board>) -> Self {
        Self::owned(Game::from_state_with_board(rules, state, board))
    }
    #[cfg(feature = "wasm")]
    pub fn from_borrowed_state(
        rules: &'catalog Rules,
        state: &State,
        board: Rc<Board>,
        cache: RuntimeCache,
    ) -> Self {
        Self::owned(Game::from_borrowed_state(rules, state, board, cache))
    }
    pub fn try_from_state(rules: &'catalog Rules, state: State) -> Result<Self, crate::StateError> {
        Game::try_from_state(rules, state).map(Self::owned)
    }
}

/// Runtime-permitted planning intent. No authored predicates, event actions,
/// hidden objective definitions or unfiltered catalog access cross this boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignIntent {
    Hold,
    Escort,
    GoTo,
    Escape,
    Approach,
    Own,
    Capture,
    Defeat,
}
/// Completed achievements are omitted. Fulfilled ongoing requirements remain
/// available because guarding them or recovering them can still be necessary.
pub struct CampaignObjective<'a> {
    pub id: &'a str,
    pub optional: bool,
    pub satisfied: bool,
    pub intent: CampaignIntent,
    pub selector: Option<&'a UnitMatch>,
    pub tiles: Vec<Hex>,
    pub progress_current: u32,
    pub progress_total: u32,
    pub hold_turns_remaining: Option<u32>,
}
pub struct CampaignGroup<'a> {
    pub id: &'a str,
    pub active: bool,
    pub hold: Option<(Vec<Hex>, u16)>,
}

/// Authored battle/campaign observations, without AI decisions or mutable state.
#[derive(Clone, Copy)]
pub struct CampaignView<'a, 'catalog> {
    view: &'a BattleView<'a, 'catalog>,
}
impl<'a, 'catalog> std::ops::Deref for CampaignView<'a, 'catalog> {
    type Target = BattleView<'a, 'catalog>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.view
    }
}
impl<'a, 'catalog> CampaignView<'a, 'catalog> {
    #[inline]
    pub fn in_campaign(&self) -> bool {
        self.view.engine().campaign.is_some()
    }
    #[inline]
    pub fn planning_objectives(self) -> Vec<CampaignObjective<'a>> {
        self.view.engine().planning_objectives()
    }
    #[inline]
    pub fn requirements_disclosed(&self) -> bool {
        self.view.engine().requirements_disclosed()
    }
    #[inline]
    pub fn pending_arrivals(&self) -> bool {
        self.view.engine().script_pending_arrivals()
    }
    #[inline]
    pub fn required_companions(&self) -> &[String] {
        self.view
            .engine()
            .mission
            .as_ref()
            .map_or(&[], |m| m.required_companions.as_slice())
    }
    #[inline]
    pub fn enemy_leader_post(&self) -> Option<Hex> {
        if self.view.turn() != Side::Enemy {
            return None;
        }
        self.view
            .engine()
            .mission
            .as_ref()
            .and_then(|m| m.enemy_leader_post.as_ref())
            .map(|target| self.view.engine().board.ids[self.view.engine().reference_site(target)])
    }
    #[inline]
    pub fn defend_objectives(&self) -> bool {
        if self.view.turn() != Side::Enemy {
            return false;
        }
        self.view
            .engine()
            .mission
            .as_ref()
            .is_some_and(|m| m.defend_objectives)
    }
    pub fn groups(self) -> Vec<CampaignGroup<'a>> {
        let game = self.view.engine();
        if game.turn != Side::Enemy {
            return vec![];
        }
        game.mission
            .as_ref()
            .into_iter()
            .flat_map(|m| &m.enemy_groups)
            .map(|group| {
                let hold = match &group.order {
                    Some(GroupOrder::Hold {
                        area,
                        attack_radius,
                        when,
                    }) if when
                        .as_ref()
                        .is_none_or(|condition| game.event_condition_met(condition)) =>
                    {
                        Some((
                            area.iter()
                                .map(|name| game.board.ids[game.reference_site(name)].clone())
                                .collect(),
                            *attack_radius,
                        ))
                    }
                    _ => None,
                };
                CampaignGroup {
                    id: &group.id,
                    active: game.group_active(group),
                    hold,
                }
            })
            .collect()
    }
    #[inline]
    pub fn script_unit_matches(&self, select: &UnitMatch, unit: &Unit) -> bool {
        self.view.engine().script_unit_matches(select, unit)
    }
}

/// Capability granted by a live owner, not recoverable from a read-only view.
#[derive(Clone, Copy)]
pub struct ActionForecastSource<'a, 'catalog> {
    view: &'a BattleView<'a, 'catalog>,
}
impl<'catalog> ActionForecastSource<'_, 'catalog> {
    #[inline]
    pub fn fork(&self) -> ActionForecast<'catalog> {
        ActionForecast::from_view(self.view, true)
    }
}
/// A mechanical projection. It cannot invoke any AI selector or return a live owner.
pub struct ActionForecast<'catalog> {
    view: BattleView<'catalog, 'catalog>,
}
impl<'catalog> std::ops::Deref for ActionForecast<'catalog> {
    type Target = BattleView<'catalog, 'catalog>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}
impl<'catalog> ActionForecast<'catalog> {
    fn from_view(view: &BattleView<'_, 'catalog>, planning: bool) -> Self {
        let game = if planning {
            view.engine().planning_fork()
        } else {
            view.engine().fork()
        };
        Self {
            view: BattleView {
                engine: EngineAccess::Draft(game),
                inactive: view.inactive.clone(),
                scoped: view.scoped.clone(),
            },
        }
    }
    /// Match the planner projection contract: each nested fork resets combat RNG.
    pub fn fork(&self) -> Self {
        Self::from_view(&self.view, true)
    }
    #[inline]
    pub fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        self.view.engine_mut().execute(command)
    }
    pub fn project_intent(
        &mut self,
        id: u32,
        retreating: Option<bool>,
        rebuilding: Option<bool>,
        attacked: bool,
    ) {
        if let Some(index) = self.view.engine().index(id) {
            let unit = &mut self.view.engine_mut().units[index];
            unit.retreating = retreating;
            unit.rebuilding = rebuilding;
            unit.attacked = attacked;
        }
    }
    pub fn restrict_positions(&mut self, scoped: &[ScopedOrder]) {
        self.view.scoped = Rc::new(scoped.to_vec());
    }
    pub fn restrict_actors(&mut self, inactive: &[UnitId]) {
        self.view.inactive = Rc::new(inactive.iter().map(|id| id.0).collect());
    }
    #[inline]
    pub fn record(&mut self, message: String) {
        self.view.engine_mut().record(message);
    }
    #[inline]
    pub fn phase_forecasts(&self) -> PhaseForecasts<'_, 'catalog> {
        PhaseForecasts { view: &self.view }
    }
}

/// Mechanical next-phase requests; decision policies choose which requests to make.
#[derive(Clone, Copy)]
pub struct PhaseForecasts<'a, 'catalog> {
    view: &'a BattleView<'a, 'catalog>,
}
impl<'a, 'catalog> PhaseForecasts<'a, 'catalog> {
    /// The immutable source used by every projection from this capability.
    #[inline]
    pub fn view(&self) -> &'a BattleView<'a, 'catalog> {
        self.view
    }
    /// Escort projection uses the planning fork RNG reset before transitioning.
    pub fn planning_next_phase(&self) -> PhaseSnapshot<'catalog> {
        let mut game = self.view.engine().planning_fork();
        game.arena_first = None;
        game.end_turn();
        PhaseSnapshot {
            view: BattleView {
                engine: EngineAccess::Draft(game),
                inactive: self.view.inactive.clone(),
                scoped: self.view.scoped.clone(),
            },
        }
    }
    #[inline]
    pub fn next_phase(&self) -> PhaseSnapshot<'catalog> {
        self.project(None, None, false)
    }
    #[inline]
    pub fn next_phase_without(&self, eliminated: Option<u32>) -> PhaseSnapshot<'catalog> {
        self.project(None, eliminated, true)
    }
    #[inline]
    pub fn next_phase_with_actor(
        &self,
        actor: u32,
        tile: usize,
        eliminated: Option<u32>,
    ) -> PhaseSnapshot<'catalog> {
        self.project(Some((actor, tile)), eliminated, true)
    }

    /// Ordinary engine fork: preserve source RNG and apply one phase transition.
    fn project(
        &self,
        relocated: Option<(u32, usize)>,
        eliminated: Option<u32>,
        refresh_roster: bool,
    ) -> PhaseSnapshot<'catalog> {
        let mut game = self.view.engine().fork();
        game.arena_first = None;
        if let Some((actor, tile)) = relocated {
            if let Some(index) = game.index(actor) {
                game.units[index].tile = tile;
            }
        }
        if eliminated.is_some() {
            game.units.retain(|unit| Some(unit.id) != eliminated);
        }
        if refresh_roster {
            game.roster_changed();
        }
        game.end_turn();
        PhaseSnapshot {
            view: BattleView {
                engine: EngineAccess::Draft(game),
                inactive: self.view.inactive.clone(),
                scoped: self.view.scoped.clone(),
            },
        }
    }
}
/// Immutable result of a mechanical phase request. It has no projection factory.
pub struct PhaseSnapshot<'catalog> {
    view: BattleView<'catalog, 'catalog>,
}
impl<'catalog> std::ops::Deref for PhaseSnapshot<'catalog> {
    type Target = BattleView<'catalog, 'catalog>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.view
    }
}

impl<'a, 'catalog> BattleView<'a, 'catalog> {
    #[inline]
    pub fn factions(&self) -> &[Faction] {
        &self.engine().factions
    }
    #[inline]
    pub fn faction(&self, id: &str) -> Option<&Faction> {
        self.engine().faction(id)
    }
    #[inline]
    pub fn faction_relation(&self, first: &str, second: &str) -> Relation {
        self.engine().faction_relation(first, second)
    }
    #[inline]
    pub fn faction_for_phase(&self, side: Side) -> &str {
        self.engine().faction_for_phase(side)
    }
    #[inline]
    pub fn unit_faction<'u>(&self, unit: &'u Unit) -> &'u str {
        self.engine().unit_faction(unit)
    }
    #[inline]
    pub fn may_act(&self, unit: &Unit) -> bool {
        self.engine().may_act(unit)
    }
    #[inline]
    pub fn active_faction(&self) -> Option<&str> {
        self.engine().active_faction.as_deref()
    }
    #[inline]
    pub fn automatic_factions(&self) -> Vec<(String, Option<FactionPolicy>)> {
        self.engine().automatic_factions()
    }
    #[inline]
    pub fn gold(&self) -> &Gold {
        &self.engine().gold
    }
    #[inline]
    pub fn ground_items(&self) -> &[GroundItem] {
        &self.engine().ground_items
    }
    #[inline]
    pub fn map_revision(&self) -> u32 {
        self.engine().map_revision
    }
    #[inline]
    pub fn cols(&self) -> i32 {
        self.engine().cols
    }
    #[inline]
    pub fn rows(&self) -> i32 {
        self.engine().rows
    }
    #[inline]
    pub fn is_campaign(&self) -> bool {
        self.engine().campaign.is_some()
    }
    #[inline]
    pub fn has_pools(&self) -> bool {
        self.engine().pools.is_some()
    }
    #[inline]
    pub fn site_at(&self, col: i32, row: i32) -> usize {
        self.engine()
            .board
            .positions
            .iter()
            .position(|position| *position == (col, row))
            .unwrap()
    }
    #[inline]
    pub fn terrain_role(&self, tile: usize) -> TerrainRole {
        self.engine().terrain_role(tile)
    }
    #[inline]
    pub fn faction_income(&self, id: &str) -> i32 {
        self.engine().faction_income(id)
    }
    #[inline]
    pub fn damage_distribution(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        tile: usize,
    ) -> Rc<crate::logic::damage::DamageDistribution> {
        self.engine()
            .damage_distribution(attacker, target, weapon, tile)
    }
    #[inline]
    pub fn threat_reach(&self, unit: &Unit, occupancy: Option<&[Option<Side>]>) -> TileSet {
        self.engine().threat_reach(unit, occupancy)
    }
}
impl<'a, 'catalog> CampaignView<'a, 'catalog> {
    #[inline]
    pub fn battle(&self) -> &'a BattleView<'a, 'catalog> {
        self.view
    }
    #[inline]
    pub fn mediation_at_risk(&self, target: &Unit) -> bool {
        self.view.engine().mediation_at_risk(target)
    }
}

impl BattleView<'_, '_> {
    #[inline]
    pub fn attack_masks(&self, category: Category, range: u8) -> Rc<Vec<u128>> {
        self.engine().attack_masks(category, range)
    }
}
impl BattleSession<'_> {
    pub fn campaign_state(&self) -> Option<&CampaignState> {
        self.0.engine().campaign.as_deref()
    }
    pub fn event_condition_met(&self, condition: &Condition) -> bool {
        self.0.engine().event_condition_met(condition)
    }
}
impl<'catalog> BattleSession<'catalog> {
    #[inline]
    pub fn catalogue(&self) -> &'catalog Rules {
        self.0.engine().rules
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn next_campaign(&self) -> Self {
        Self::owned(self.0.engine().next_campaign())
    }
    #[cfg(feature = "wasm")]
    pub fn take_last_combat(&mut self) -> Option<LastCombat> {
        self.0.engine_mut().last_combat.take()
    }
}

impl BattleView<'_, '_> {
    #[inline]
    pub fn last_combat(&self) -> Option<&LastCombat> {
        self.engine().last_combat.as_ref()
    }
    #[inline]
    pub fn combat_present(&self) -> bool {
        self.engine().combat_present
    }
    #[inline]
    pub fn armies(&self) -> Option<&[String; 2]> {
        self.engine().armies.as_ref()
    }
    #[inline]
    pub fn pools(&self) -> Option<&[Vec<usize>; 2]> {
        self.engine().pools.as_ref()
    }
    #[inline]
    pub fn arena_first(&self) -> Option<Side> {
        self.engine().arena_first
    }
    #[inline]
    pub fn combat_rng(&self) -> u32 {
        self.engine().combat_rng
    }
}

impl<'catalog> BattleSession<'catalog> {
    pub fn continue_campaign(&self, selected: Option<&[UnitId]>) -> Result<Self, CommandError> {
        self.0.engine().continue_campaign(selected).map(Self::owned)
    }
}

impl<'catalog> BattleSession<'catalog> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn ensure_recording(&mut self) {
        if self.0.engine().recordings.is_none() {
            self.0.engine_mut().recordings = Some(Records::default());
        }
    }
    pub fn declare_outcome(&mut self, side: Side) {
        self.0.engine_mut().winner = Some(side);
    }
    pub fn refresh_mission(&mut self) {
        self.0.engine_mut().refresh_mission();
    }
}
impl<'catalog> ActionForecast<'catalog> {
    #[inline]
    pub fn action_forecasts(&self) -> ActionForecastSource<'_, 'catalog> {
        ActionForecastSource { view: &self.view }
    }
}
impl BattleView<'_, '_> {
    #[inline]
    pub fn board_handle(&self) -> Rc<Board> {
        self.engine().board.clone()
    }
}

// Host presentation is intentionally available only from the actual owner.
impl BattleSession<'_> {
    pub fn log(&self) -> &[String] {
        &self.0.engine().log
    }
    pub fn fog(&self) -> &[Option<std::sync::Arc<ConditionRule>>] {
        &self.0.engine().fog
    }
    pub fn public_units(&self) -> Vec<PublicUnit> {
        self.0.engine().public_units()
    }
    pub fn public_unit(&self, unit: &Unit) -> PublicUnit {
        self.0.engine().public_unit(unit)
    }
    pub fn tiles(&self) -> Vec<Tile> {
        self.0.engine().tiles()
    }
    pub fn inspect(&self, revision: u32) -> BattleInspection {
        self.0.engine().inspect(revision)
    }
    pub fn inspect_board(&self) -> BoardInspection {
        self.0.engine().inspect_board()
    }
    #[cfg(feature = "wasm")]
    pub fn patch_tiles(&self, tiles: &mut [Tile]) -> Vec<usize> {
        self.0.engine().patch_tiles(tiles)
    }
    pub fn presentation(&self) -> BattlefieldPresentation {
        self.0.engine().presentation()
    }
}

impl BattleSession<'_> {
    pub fn condition_memory(&self) -> &[String] {
        &self.0.engine().condition_memory
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn event_goal_progress(&self) -> Vec<Progress> {
        self.0.engine().event_goal_progress()
    }
}
