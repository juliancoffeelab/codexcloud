//! Borrowed rule queries and explicitly granted mechanical projections.
//! Query delegates are inlineable across the crate boundary, including native
//! simulation builds without LTO. Large engine algorithms retain their own bodies.
use crate::{
    data::*,
    logic::battle::Board,
    logic::geometry::TileSet,
    logic::{BattlePosition, Game},
    model::*,
};
use std::{cell::OnceCell, rc::Rc};

pub struct BattleRules<'a> {
    pub units: &'a [UnitRule],
    pub recruitment: &'a RecruitmentProfiles,
    pub mechanics: &'a UnitMechanics,
}

enum EngineAccess<'a, 'catalog> {
    Borrowed(&'a BattlePosition<'catalog>),
    Draft(BattlePosition<'catalog>),
}

/// Read-only mechanical view. It exposes neither authored goals nor campaign
/// counters, and has no Deref or raw-engine accessor outside this module.
pub struct BattleView<'a, 'catalog> {
    engine: EngineAccess<'a, 'catalog>,
    observer: Option<NameKey>,
    observed_units: OnceCell<Vec<Unit>>,
    inactive: Rc<rustc_hash::FxHashSet<u32>>,
    scoped: Rc<Vec<crate::model::ScopedOrder>>,
}
impl<'a, 'catalog> BattleView<'a, 'catalog> {
    /// Borrow from an existing owner; this cannot manufacture an owner from a view.
    #[inline]
    pub fn new(session: &'a BattleSession<'catalog>) -> Self {
        Self {
            engine: EngineAccess::Borrowed(session.view().engine()),
            observer: Some(session.view().observer_key()),
            observed_units: OnceCell::new(),
            inactive: session.inactive.clone(),
            scoped: session.scoped.clone(),
        }
    }
    #[inline]
    pub(crate) fn engine(&self) -> &BattlePosition<'catalog> {
        match &self.engine {
            EngineAccess::Borrowed(game) => game,
            EngineAccess::Draft(game) => game,
        }
    }
    #[inline]
    fn engine_mut(&mut self) -> &mut BattlePosition<'catalog> {
        self.observed_units.take();
        match &mut self.engine {
            EngineAccess::Draft(game) => game,
            EngineAccess::Borrowed(_) => {
                panic!("read-only battle view")
            }
        }
    }

    #[inline]
    pub fn rules(&self) -> BattleRules<'_> {
        BattleRules {
            units: &self.engine().rules.units,
            mechanics: self.engine().mechanics,
            recruitment: &self.engine().rules.recruitment,
        }
    }
    #[inline]
    pub fn name(&self, key: Option<NameKey>) -> Option<&str> {
        self.engine().name(key)
    }
    #[inline]
    pub fn inventory(&self, unit: &Unit) -> impl ExactSizeIterator<Item = &Item> + Clone {
        self.engine().inventory(unit)
    }
    #[inline]
    pub fn source_weapon(&self, weapon: &WeaponRules) -> &Weapon {
        self.engine().source_weapon(weapon)
    }
    #[inline]
    pub fn weapon_id(&self, key: WeaponKey) -> String {
        self.engine().weapon_id(key)
    }
    #[inline]
    pub fn weapons<'b>(
        &'b self,
        unit: &'b Unit,
    ) -> std::iter::Chain<std::slice::Iter<'b, WeaponRules>, std::slice::Iter<'b, WeaponRules>>
    {
        self.engine().weapons(unit)
    }
    #[inline]
    pub fn weapon<'b>(&'b self, unit: &'b Unit, index: usize) -> &'b WeaponRules {
        self.engine().weapon(unit, index)
    }
    #[inline]
    pub fn board(&self) -> &Board {
        &self.engine().runtime.board
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
    pub fn permits_runtime(&self, command: &crate::RuntimeCommand) -> bool {
        if self.engine().check_command_indices(command).is_err() {
            return false;
        }
        match command {
            crate::RuntimeCommand::Move { unit, tile } => {
                self.can_act(unit.0)
                    && self.unit(unit.0).is_some_and(|actor| {
                        self.scoped
                            .iter()
                            .filter(|order| order.actor == *unit)
                            .all(|order| order.allows_move(self, actor, tile.0))
                    })
            }
            crate::RuntimeCommand::Attack { unit, target, .. } => {
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
            crate::RuntimeCommand::Recruit { unit, .. }
            | crate::RuntimeCommand::Advance { unit, .. }
            | crate::RuntimeCommand::SetUnitIntent { unit, .. } => self.can_act(unit.0),
            _ => true,
        }
    }
    #[inline]
    pub fn available_combatant(&self, unit: &Unit) -> bool {
        self.engine().available_combatant(unit)
    }
    #[inline]
    pub fn unit_relation(&self, first: &Unit, second: &Unit) -> Relation {
        self.engine().unit_relation(first, second)
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
    pub fn hostile_to_faction(&self, unit: &Unit, faction: &str) -> bool {
        self.engine().hostile_to_faction(unit, faction)
    }
    #[inline]
    pub fn control_roster(&self) -> Vec<(u32, TurnPhase, Option<NameKey>, Option<Role>)> {
        self.engine().control_roster()
    }
    #[inline]
    pub fn belongs_to_faction(&self, unit: &Unit, faction: &str) -> bool {
        self.engine().belongs_to_faction(unit, faction)
    }
    #[inline]
    fn observer_key(&self) -> NameKey {
        self.observer.unwrap_or_else(|| {
            self.engine()
                .runtime
                .names
                .find(
                    self.engine()
                        .runtime
                        .active_faction
                        .as_deref()
                        .unwrap_or("player"),
                )
                .expect("interned faction")
        })
    }
    #[inline]
    pub fn observer_faction(&self) -> &str {
        self.engine().runtime.names.get(self.observer_key())
    }
    #[inline]
    pub fn tile_in_sight(&self, tile: usize) -> bool {
        self.engine()
            .tile_in_sight_to(self.observer_faction(), tile)
    }
    #[inline]
    pub fn tile_visible(&self, tile: usize) -> bool {
        self.engine().tile_visible_to(self.observer_faction(), tile)
    }
    #[inline]
    pub fn units(&self) -> &Vec<Unit> {
        self.observed_units
            .get_or_init(|| self.engine().runtime.units.clone())
    }

    #[inline]
    pub fn terrain(&self) -> &Vec<usize> {
        &self.engine().runtime.terrain
    }
    #[inline]
    pub fn owners(&self) -> &Vec<Option<String>> {
        &self.engine().runtime.owners
    }
    #[inline]
    pub fn map_id(&self) -> &str {
        &self.engine().runtime.map_id
    }
    #[inline]
    pub fn turn(&self) -> TurnPhase {
        self.engine().runtime.turn
    }
    #[inline]
    pub fn round(&self) -> u32 {
        self.engine().runtime.round
    }
    #[inline]
    pub fn gold_for_faction(&self, faction: &str) -> i32 {
        self.engine().gold_for_faction(faction)
    }
    #[inline]
    pub fn gold_for_unit(&self, unit: &Unit) -> i32 {
        self.engine().gold_for_unit(unit)
    }
    #[inline]
    pub fn owns(&self, tile: usize, faction: &str) -> bool {
        self.engine().owns(tile, faction)
    }
    #[inline]
    pub fn captures(&self, tile: usize, faction: &str) -> bool {
        self.engine().captures(tile, faction)
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
    pub fn winner(&self) -> Option<BattleResult> {
        self.engine().runtime.winner
    }
    #[inline]
    pub fn next_id(&self) -> u32 {
        self.engine().runtime.next_id
    }
    #[inline]
    pub fn occupied_tiles(&self) -> &TileSet {
        &self.engine().runtime.occupied_tiles
    }

    #[inline]
    pub fn perception_sensitive(&self) -> bool {
        self.engine().perception_sensitive()
    }
    /// Current-position detection over this view's known physical position.
    #[inline]
    pub fn detection_facts(&self, observer: &Unit, target: &Unit) -> DetectionFacts {
        self.engine()
            .perception(None)
            .detection_facts(observer, Some(target), target.tile)
    }
    /// Player presentation and planning clients receive the same detection facts.
    /// This view contains no unseen authoritative units or sight sources.
    pub fn unit_appearance(&self, unit: &Unit) -> UnitAppearance {
        let perception = self.engine().perception(None);
        let (eligible, active) = perception.visual_cover(unit);
        let mut detection = Vec::new();
        if eligible {
            for observer in self
                .units()
                .iter()
                .filter(|observer| self.hostile(unit, observer))
            {
                detection.push(KnownDetection {
                    observer: DetectionObserver::Unit(format!("u{}", observer.id)),
                    facts: self.detection_facts(observer, unit),
                });
            }
            for (origin, group, profile) in self.engine().fixed_sights.iter() {
                if self.engine().factions.iter().any(|faction| {
                    self.engine().vision_group(&faction.id) == *group
                        && self
                            .engine()
                            .faction_relation(self.unit_faction(unit), &faction.id)
                            == Relation::Hostile
                }) {
                    detection.push(KnownDetection {
                        observer: DetectionObserver::Terrain(self.board().ids[*origin]),
                        facts: perception.fixed_detection_facts(*group, *origin, *profile, unit),
                    });
                }
            }
        }
        UnitAppearance {
            unit: format!("u{}", unit.id),
            concealment: if !eligible {
                UnitConcealmentAppearance::Normal
            } else if !active || detection.iter().any(|d| d.facts.detected()) {
                UnitConcealmentAppearance::Exposed
            } else {
                UnitConcealmentAppearance::Concealed
            },
            detection,
        }
    }
    #[inline]
    pub fn can_threaten_from(
        &self,
        attacker: &Unit,
        target: &Unit,
        weapon: usize,
        origins: &TileSet,
    ) -> bool {
        self.engine()
            .can_threaten_from(attacker, target, weapon, origins)
    }
    #[inline]
    pub fn attack_origins(&self, category: Category, range: u8) -> Rc<Vec<TileSet>> {
        self.engine().attack_origins(category, range)
    }
    #[inline]
    pub fn attack_origin_runs(
        &self,
        kind: usize,
        from: usize,
        category: Category,
        range: u8,
        target: usize,
    ) -> Rc<Vec<crate::AttackOriginRun>> {
        self.engine()
            .attack_origin_runs(kind, from, category, range, target)
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
    pub fn thorn_damage(&self, attacker: &Unit, target: &Unit, weapon: usize, from: usize) -> i32 {
        self.engine().thorn_damage(attacker, target, weapon, from)
    }
    #[inline]
    pub fn combat_matchup(&self, attacker: &Unit, target: &Unit) -> crate::CombatMatchup<'_> {
        self.engine().combat_matchup(attacker, target)
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
        self.engine()
            .unit(id)
            .filter(|unit| self.units().iter().any(|seen| seen.id == unit.id))
    }
    #[inline]
    pub fn at(&self, tile: usize) -> Option<&Unit> {
        self.engine()
            .at(tile)
            .filter(|unit| self.units().iter().any(|seen| seen.id == unit.id))
    }
    #[inline]
    pub fn index(&self, id: u32) -> Option<usize> {
        self.units().iter().position(|unit| unit.id == id)
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
    pub fn recruitment_options_for_faction(&self, faction: &str) -> Vec<usize> {
        self.engine().recruitment_options_for_faction(faction)
    }
    #[inline]
    pub fn placement_tiles(&self, leader: &Unit, kind: usize, price: bool) -> PlacementTiles {
        if !self.can_act(leader.id) {
            return PlacementTiles::default();
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
    pub fn reachability(&self, kind: usize, start: usize) -> crate::Reachability {
        self.engine().reachability(kind, start)
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
    pub fn income(&self, faction: &str) -> i32 {
        self.engine().faction_income(faction)
    }
}

/// An opaque live battle. Only its owner can mint mechanical forecast sources.
pub struct BattleSession<'catalog> {
    live: Game<'catalog>,
    known: OnceCell<BattleView<'catalog, 'catalog>>,
    inactive: Rc<rustc_hash::FxHashSet<u32>>,
    scoped: Rc<Vec<crate::model::ScopedOrder>>,
}
impl<'catalog> std::ops::Deref for BattleSession<'catalog> {
    type Target = BattleView<'catalog, 'catalog>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.view()
    }
}
impl<'catalog> BattleSession<'catalog> {
    fn owned(live: Game<'catalog>) -> Self {
        Self {
            live,
            known: OnceCell::new(),
            inactive: Rc::default(),
            scoped: Rc::default(),
        }
    }
    fn live_mut(&mut self) -> &mut Game<'catalog> {
        self.known.take();
        &mut self.live
    }
    pub fn from_state(rules: &'catalog Rules, state: State) -> Self {
        Self::owned(Game::from_state(rules, state))
    }
    pub fn from_state_with_campaign(
        rules: &'catalog Rules,
        state: State,
        definition: Option<&CampaignRules>,
    ) -> Self {
        Self::owned(Game::from_state_with_campaign(rules, state, definition))
    }
    pub fn new(rules: &'catalog Rules, map: &str) -> Self {
        Self::owned(Game::new_with_campaign(rules, map, None))
    }
    pub fn new_with_campaign(
        rules: &'catalog Rules,
        map: &str,
        definition: Option<&CampaignRules>,
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
        definition: &CampaignRules,
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
    fn known_view(&self, observer: &str) -> BattleView<'catalog, 'catalog> {
        BattleView {
            engine: EngineAccess::Draft(self.live.position.known_to(observer)),
            observer: Some(
                self.live
                    .position
                    .names
                    .find(observer)
                    .expect("interned observer faction"),
            ),
            observed_units: OnceCell::new(),
            inactive: self.inactive.clone(),
            scoped: self.scoped.clone(),
        }
    }
    #[inline]
    pub fn view(&self) -> &BattleView<'catalog, 'catalog> {
        self.known.get_or_init(|| {
            self.known_view(
                self.live
                    .position
                    .active_faction
                    .as_deref()
                    .unwrap_or("player"),
            )
        })
    }
    #[inline]
    pub fn campaign_view(&self) -> CampaignView<'_, 'catalog> {
        CampaignView {
            view: self.view(),
            game: &self.live,
        }
    }
    #[inline]
    pub fn scope(&self) -> crate::BattleScope {
        self.live.runtime.scope.clone()
    }
    #[inline]
    pub fn in_scope(&self, scope: &crate::BattleScope) -> bool {
        self.live.runtime.scope.same_session(scope)
    }
    pub fn command_snapshot(
        &self,
        command: &crate::RuntimeCommand,
    ) -> Result<Command, CommandError> {
        self.live.command_snapshot(command)
    }
    /// Bind the original catalog to its owned live data; no unit/board conversion.
    /// The rules must be the same immutable catalog used to create this runtime.
    #[inline]
    pub fn from_runtime(rules: &'catalog Rules, runtime: crate::BattleState) -> Self {
        Self::owned(Game {
            position: BattlePosition {
                rules,
                mechanics: rules.units.mechanics(),
                runtime: runtime.position,
            },
            story: runtime.story,
        })
    }
    #[inline]
    pub fn into_runtime(self) -> crate::BattleState {
        crate::BattleState {
            position: self.live.position.runtime,
            story: self.live.story,
        }
    }
    pub fn state_with_tiles(&self, tiles: Vec<Tile>) -> State {
        self.live.state_with_tiles(tiles)
    }
    pub fn state(&self) -> State {
        self.live.state()
    }
    /// Execution invalidation token; carries no hidden event definitions.
    pub fn planning_revision(&self) -> u64 {
        self.live.story.planning_revision
    }
    pub fn campaign_snapshot(&self) -> Option<CampaignState> {
        self.live.campaign_snapshot()
    }
    #[inline]
    pub fn execute(&mut self, command: Command) -> Result<(), CommandError> {
        self.live_mut().execute(command)
    }
    #[inline]
    pub fn execute_runtime(&mut self, command: crate::RuntimeCommand) -> Result<(), CommandError> {
        self.live_mut().execute_runtime(command)
    }
    /// Admit each proposal against the actual game, then run all real events.
    /// Changed circumstances stop the sequence; invalid unchanged proposals are
    /// still errors and must not be silently retried.
    pub fn execute_proposal(
        &mut self,
        proposal: crate::ActionProposal,
    ) -> Result<crate::ProposalOutcome, CommandError> {
        if self.live.winner.is_some() {
            return Ok(crate::ProposalOutcome::Finished);
        }
        let before = self.live.story.planning_revision;
        self.execute_runtime(proposal.into_command())?;
        Ok(if self.live.winner.is_some() {
            crate::ProposalOutcome::Finished
        } else if self.live.story.planning_revision != before {
            crate::ProposalOutcome::Replan
        } else {
            crate::ProposalOutcome::Applied
        })
    }
    pub fn begin_turn(&mut self, side: TurnPhase) {
        self.live_mut().begin_turn(side);
    }
    pub fn end_turn(&mut self) {
        self.live_mut().end_turn();
    }
    pub fn record(&mut self, message: String) {
        self.live_mut().record(message);
    }
    pub fn fork(&self) -> Self {
        let mut fork = Self::owned(self.live.fork());
        fork.inactive = self.inactive.clone();
        fork.scoped = self.scoped.clone();
        fork
    }
    pub fn planning_fork(&self) -> Self {
        let mut fork = Self::owned(self.live.planning_fork());
        fork.inactive = self.inactive.clone();
        fork.scoped = self.scoped.clone();
        fork
    }
    #[inline]
    pub fn action_forecasts(&self) -> ActionForecastSource<'_, 'catalog> {
        ActionForecastSource { view: self.view() }
    }
    #[inline]
    pub fn phase_forecasts(&self) -> PhaseForecasts<'_, 'catalog> {
        PhaseForecasts { view: self.view() }
    }
    pub fn restrict_positions(&mut self, scoped: &[ScopedOrder]) {
        self.known.take();
        self.scoped = Rc::new(scoped.to_vec());
    }
    pub fn restrict_actors(&mut self, inactive: &[UnitId]) {
        self.known.take();
        self.inactive = Rc::new(inactive.iter().map(|id| id.0).collect());
    }
    pub fn set_active_faction(&mut self, faction: Option<String>) {
        if let Some(id) = &faction {
            let declared = self
                .live
                .position
                .faction(id)
                .expect("declared acting faction");
            assert_eq!(
                declared.phase, self.live.turn,
                "acting faction must be scheduled in this phase"
            );
        }
        self.live_mut().runtime.active_faction = faction;
    }
    pub fn set_combat_rng(&mut self, seed: u32) {
        self.live_mut().runtime.combat_rng = seed;
    }
    pub fn configure_arena(&mut self, pools: Option<[Vec<usize>; 2]>, first: Option<TurnPhase>) {
        self.live_mut().runtime.pools = pools;
        self.live_mut().runtime.arena_first = first;
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_recording(&mut self) {
        self.live_mut().runtime.recordings = Some(Records::default().into());
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn take_recordings(&mut self) -> Option<Records> {
        self.live_mut()
            .runtime
            .recordings
            .take()
            .map(|r| r.into_inner())
    }
    pub fn clear_log(&mut self) {
        self.live_mut().runtime.log.clear();
    }
    pub fn dismiss_all_dialogue(&mut self) {
        if let Some(campaign) = self.live_mut().story.campaign.as_mut() {
            campaign.dialogue_index = campaign.dialogue.len();
        }
    }
    #[cfg(feature = "wasm")]
    pub fn take_combat_animation(&mut self) -> Option<CombatAnimation> {
        self.live_mut().runtime.combat_animation.take()
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

/// Coarse path progress, available only to Campaigner. The target location,
/// hidden roster and absolute distances remain private.
pub struct SearchGuidance {
    routes: Rc<Vec<u16>>,
    origin_cost: u16,
}
impl SearchGuidance {
    #[inline]
    pub fn progress(&self, tile: usize) -> std::cmp::Ordering {
        self.routes[tile].cmp(&self.origin_cost)
    }
}

/// Authored battle/campaign observations, without AI decisions or mutable state.
#[derive(Clone, Copy)]
pub struct CampaignView<'a, 'catalog> {
    game: &'a Game<'catalog>,
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
    /// A gameplay rule for strategic searching, not a sighting. Guide patrols
    /// loosely toward the hostile army's centre without disclosing its roster.
    pub fn search_guidance(&self, origin: &Unit) -> Option<SearchGuidance> {
        let faction = self.game.unit_faction(origin);
        let mut count = 0i64;
        let mut q = 0i64;
        let mut r = 0i64;
        for unit in self.game.position.units.iter().filter(|unit| {
            self.game
                .faction_relation(faction, self.game.unit_faction(unit))
                == Relation::Hostile
        }) {
            let (col, row) = self.game.position.board.positions[unit.tile];
            q += i64::from(col - row.div_euclid(2));
            r += i64::from(row);
            count += 1;
        }
        if count == 0 {
            return None;
        }
        // Project the same hostile centre onto reachable terrain. Connected
        // regions and reverse cost fields are cached by movement class.
        let reachable = self.view.reachability(origin.kind, origin.tile);
        let goal = self
            .view
            .board()
            .positions
            .iter()
            .enumerate()
            .filter(|&(tile, _)| {
                reachable.contains(tile) && self.view.cost(origin.kind, tile) != u16::MAX
            })
            .min_by_key(|&(tile, &(col, row))| {
                let dq = count * i64::from(col - row.div_euclid(2)) - q;
                let dr = count * i64::from(row) - r;
                (dq.abs() + dr.abs() + (dq + dr).abs(), tile)
            })?
            .0;
        if goal == origin.tile {
            return None;
        }
        let routes = self.view.routes(origin.kind, &[goal]);
        let origin_cost = routes[origin.tile];
        (origin_cost != u16::MAX).then_some(SearchGuidance {
            routes,
            origin_cost,
        })
    }

    #[inline]
    pub fn in_campaign(&self) -> bool {
        self.game.story.campaign.is_some()
    }
    #[inline]
    pub fn planning_objectives(self) -> Vec<CampaignObjective<'a>> {
        let mut objectives = self.game.planning_objectives(self.view.observer_faction());
        {
            for objective in &mut objectives {
                if objective.intent == CampaignIntent::Defeat {
                    objective
                        .tiles
                        .retain(|tile| self.view.at(self.view.site(tile)).is_some());
                }
            }
        }
        objectives
    }
    #[inline]
    pub fn requirements_disclosed(&self) -> bool {
        self.game
            .requirements_disclosed(self.view.observer_faction())
    }
    #[inline]
    pub fn deadline_round(&self) -> Option<u32> {
        self.game.planning_deadline_round()
    }
    #[inline]
    pub fn pending_arrivals(&self) -> bool {
        self.game.script_pending_arrivals()
    }
    #[inline]
    pub fn required_companions(&self) -> &[String] {
        self.game
            .story
            .mission
            .as_ref()
            .map_or(&[], |m| m.required_companions.as_slice())
    }
    #[inline]
    pub fn enemy_leader_post(&self) -> Option<Hex> {
        if self
            .view
            .faction_relation("player", self.view.observer_faction())
            != Relation::Hostile
        {
            return None;
        }
        self.game
            .story
            .mission
            .as_ref()
            .and_then(|m| m.enemy_leader_post.as_ref())
            .map(|target| self.game.runtime.board.ids[self.game.reference_site(target)])
    }
    #[inline]
    pub fn defend_objectives(&self) -> bool {
        if self
            .view
            .faction_relation("player", self.view.observer_faction())
            != Relation::Hostile
        {
            return false;
        }
        self.game
            .story
            .mission
            .as_ref()
            .is_some_and(|m| m.defend_objectives)
    }
    pub fn groups(self) -> Vec<CampaignGroup<'a>> {
        let game = self.game;
        if game.turn != TurnPhase::Ai {
            return vec![];
        }
        game.story
            .mission
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
        self.game
            .objective_unit_matches(self.view.observer_faction(), select, unit)
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
                observer: Some(view.observer_key()),
                observed_units: OnceCell::new(),
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
    #[inline]
    pub fn execute_runtime(&mut self, command: crate::RuntimeCommand) -> Result<(), CommandError> {
        self.view.engine_mut().execute_runtime(command)
    }
    pub fn project_intent(
        &mut self,
        id: u32,
        retreating: Option<bool>,
        rebuilding: Option<bool>,
        attack_spent: bool,
    ) {
        if let Some(index) = self.view.engine().index(id) {
            let unit = &mut self.view.engine_mut().runtime.units[index];
            unit.retreating = retreating;
            unit.rebuilding = rebuilding;
            unit.attack_spent = attack_spent;
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
                observer: Some(self.view.observer_key()),
                observed_units: OnceCell::new(),
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
                observer: Some(self.view.observer_key()),
                observed_units: OnceCell::new(),
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
        &self.engine().runtime.factions
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
    pub fn faction_identity(&self, id: &str) -> NameKey {
        self.engine().runtime.names.find(id).expect("known faction")
    }
    #[inline]
    pub fn vision_group_identity(&self, id: &str) -> VisionGroupKey {
        self.engine().vision_group(id)
    }
    #[inline]
    pub fn unit_faction<'u>(&'u self, unit: &Unit) -> &'u str {
        self.engine().unit_faction(unit)
    }
    #[inline]
    pub fn may_act(&self, unit: &Unit) -> bool {
        self.engine().may_act(unit)
    }
    #[inline]
    pub fn active_faction(&self) -> Option<&str> {
        self.engine().runtime.active_faction.as_deref()
    }
    #[inline]
    pub fn automatic_factions(&self) -> Vec<(String, Option<FactionPolicy>)> {
        self.engine().automatic_factions()
    }
    #[inline]
    pub fn gold(&self) -> &Gold {
        &self.engine().runtime.gold
    }
    #[inline]
    pub fn ground_items(&self) -> impl Iterator<Item = GroundItem> + '_ {
        self.engine()
            .runtime
            .ground_items
            .iter()
            .map(|ground| GroundItem {
                tile: self.board().ids[ground.tile].clone(),
                item: self.engine().runtime.inventories.item(ground.item).clone(),
            })
    }
    #[inline]
    pub fn map_revision(&self) -> u32 {
        self.engine().runtime.map_revision
    }
    #[inline]
    pub fn cols(&self) -> i32 {
        self.engine().runtime.cols
    }
    #[inline]
    pub fn rows(&self) -> i32 {
        self.engine().runtime.rows
    }
    #[inline]
    pub fn is_campaign(&self) -> bool {
        self.engine().runtime.campaign_battle
    }
    #[inline]
    pub fn has_pools(&self) -> bool {
        self.engine().runtime.pools.is_some()
    }
    #[inline]
    pub fn site_at(&self, col: i32, row: i32) -> usize {
        self.engine()
            .runtime
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
    pub fn threat_reach(&self, unit: &Unit, occupancy: Option<&[Option<u32>]>) -> TileSet {
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
        self.game.mediation_at_risk(target)
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
        self.live.story.campaign.as_deref()
    }
    pub fn event_condition_met(&self, condition: &Condition) -> bool {
        self.live.event_condition_met(condition)
    }
}
impl<'catalog> BattleSession<'catalog> {
    #[inline]
    pub fn catalogue(&self) -> &'catalog Rules {
        self.live.rules
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn next_campaign(&self) -> Self {
        Self::owned(self.live.next_campaign())
    }
    #[cfg(feature = "wasm")]
    pub fn take_last_combat(&mut self) -> Option<LastCombat> {
        self.live_mut().runtime.last_combat.take()
    }
}

impl BattleView<'_, '_> {
    #[inline]
    pub fn last_combat(&self) -> Option<&LastCombat> {
        self.engine().runtime.last_combat.as_ref()
    }
    #[inline]
    pub fn combat_present(&self) -> bool {
        self.engine().runtime.combat_present
    }
    #[inline]
    pub fn armies(&self) -> Option<&[String; 2]> {
        self.engine().runtime.armies.as_ref()
    }
    #[inline]
    pub fn pools(&self) -> Option<&[Vec<usize>; 2]> {
        self.engine().runtime.pools.as_ref()
    }
    #[inline]
    pub fn arena_first(&self) -> Option<TurnPhase> {
        self.engine().runtime.arena_first
    }
    #[inline]
    pub fn combat_rng(&self) -> u32 {
        self.engine().runtime.combat_rng
    }
}

impl<'catalog> BattleSession<'catalog> {
    pub fn continue_campaign(&self, selected: Option<&[UnitId]>) -> Result<Self, CommandError> {
        self.live.continue_campaign(selected).map(Self::owned)
    }
}

impl<'catalog> BattleSession<'catalog> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn ensure_recording(&mut self) {
        if self.live.runtime.recordings.is_none() {
            self.live_mut().runtime.recordings = Some(Records::default().into());
        }
    }
    pub fn declare_outcome(&mut self, outcome: BattleResult) {
        self.live_mut().runtime.winner = Some(outcome);
    }
    pub fn refresh_mission(&mut self) {
        self.live_mut().refresh_mission();
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
        self.engine().runtime.board.clone()
    }
}

// Host presentation is intentionally available only from the actual owner.
impl BattleSession<'_> {
    pub fn log(&self) -> &[String] {
        &self.live.runtime.log
    }
    pub fn tile_perception(&self) -> &[TilePerception] {
        &self.live.runtime.tile_perception
    }
    /// Authoritative owner diagnostics. Policies only borrow BattleView, which
    /// deliberately has no equivalent access to hidden units or their facts.
    pub fn authoritative_units(&self) -> &[Unit] {
        &self.live.units
    }
    pub fn authoritative_unit(&self, id: u32) -> Option<&Unit> {
        self.live.unit(id)
    }
    pub fn authoritative_perception(&self, unit: &Unit) -> &UnitPerceptionFacts {
        &self.live.unit_perception[unit.perception.0]
    }
    pub fn public_units(&self) -> Vec<PublicUnit> {
        self.live.public_units()
    }
    pub fn public_unit(&self, unit: &Unit) -> PublicUnit {
        self.live.public_unit(unit)
    }
    pub fn tiles(&self) -> Vec<Tile> {
        self.live.tiles()
    }
    pub fn inspect(&self, revision: u32) -> BattleInspection {
        self.live.inspect(revision)
    }
    pub fn inspect_board(&self) -> BoardInspection {
        self.live.inspect_board()
    }
    #[cfg(feature = "wasm")]
    pub fn patch_tiles(&self, tiles: &mut [Tile]) -> Vec<usize> {
        self.live.patch_tiles(tiles)
    }
    pub fn presentation(&self) -> BattlefieldPresentation {
        self.presentation_for("player")
    }
    pub fn presentation_for(&self, faction: &str) -> BattlefieldPresentation {
        if faction
            == self
                .live
                .position
                .active_faction
                .as_deref()
                .unwrap_or("player")
        {
            self.live.presentation_for(faction, self.view())
        } else {
            self.live
                .presentation_for(faction, &self.known_view(faction))
        }
    }
}

impl BattleSession<'_> {
    pub fn condition_memory(&self) -> &[String] {
        &self.live.runtime.condition_memory
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn event_goal_progress(&self) -> Vec<Progress> {
        self.live.event_goal_progress()
    }
}
