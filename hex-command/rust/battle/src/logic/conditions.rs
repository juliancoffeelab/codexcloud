//! Live condition queries borrow battlefield facts; owned inputs are for devtools.
use super::Game;
pub use crate::data::presentation::{PresentationInput, matches, resolve};
use crate::data::{
    presentation::{self, ConditionContext, ConditionUnit, PresentationContext},
    *,
};

pub(crate) struct LiveContext<'a, 'catalog> {
    rules: &'catalog Rules,
    board: &'a super::battle::Board,
    units: &'a [Unit],
    factions: &'a [Faction],
    owners: &'a [Option<String>],
    mission: Option<&'a Mission>,
    flags: Option<&'a FlagHistory>,
    marks: &'a [String],
    fog: &'a [Option<std::sync::Arc<ConditionRule>>],
    light: &'a [Option<std::sync::Arc<TileLight>>],
    fog_memory: &'a [u64],
    map_id: &'a str,
    round: u32,
    position: Option<(u32, usize)>,
}
impl<'a, 'catalog> LiveContext<'a, 'catalog> {
    fn new(game: &'a Game<'catalog>, position: Option<(u32, usize)>) -> Self {
        Self {
            rules: game.rules,
            board: &game.board,
            units: &game.units,
            factions: &game.factions,
            owners: &game.owners,
            mission: game.mission.as_deref().map(|m| &**m),
            flags: game.campaign.as_deref().map(|s| &s.flags),
            marks: game
                .campaign
                .as_deref()
                .map(|s| s.marks.as_slice())
                .unwrap_or_default(),
            fog: &game.fog,
            light: &game.light,
            fog_memory: &game.fog_memory,
            map_id: &game.map_id,
            round: game.round,
            position,
        }
    }
}
impl ConditionContext for LiveContext<'_, '_> {
    fn factions(&self) -> &[Faction] {
        self.factions
    }
    fn owns(&self, id: &TileRef, side: &str) -> bool {
        self.board
            .resolve(id)
            .is_some_and(|i| self.owners[i].as_deref() == Some(side))
    }
    fn position(&self, id: &TileRef) -> Option<(i32, i32)> {
        self.board.resolve(id).map(|i| self.board.positions[i])
    }
    fn visible_at(&self, position: (i32, i32)) -> bool {
        self.board
            .positions
            .iter()
            .position(|&p| p == position)
            .is_some_and(|tile| {
                self.fog[tile].as_ref().is_none_or(|rule| {
                    (rule.remember && self.fog_remembered(&self.board.ids[tile]))
                        || crate::data::presentation::matches_fog(
                            &rule.condition,
                            self,
                            Some(&TileRef::Hex(self.board.ids[tile])),
                        )
                })
            })
    }
    #[inline(always)]
    fn any_unit(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        self.units.iter().any(|unit| {
            let tile = self
                .position
                .filter(|(actor, _)| *actor == unit.id)
                .map(|(_, tile)| tile)
                .unwrap_or(unit.tile);
            predicate(ConditionUnit {
                faction: unit.faction.as_deref(),
                side: unit.side,
                kind: &self.rules.units[unit.kind].id,
                character: unit.character.as_deref(),
                ai_group: unit.ai_group.as_deref(),
                position: Some(self.board.positions[tile]),
            })
        })
    }
    #[inline(always)]
    fn any_pending(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        let Some(mission) = self.mission.filter(|_| self.flags.is_some()) else {
            return false;
        };
        mission
            .events
            .iter()
            .filter(|event| {
                self.marks
                    .iter()
                    .any(|m| m.strip_prefix("event:") == Some(event.id.as_str()))
            })
            .any(|event| {
                event.spawn.iter().any(|group| {
                    group.units.iter().any(|unit| {
                        let spawned = self.marks.iter().any(|m| {
                            m.strip_prefix("spawn:")
                                .and_then(|s| s.strip_prefix(event.id.as_str()))
                                .and_then(|s| s.strip_prefix(':'))
                                == Some(unit.id.as_str())
                        });
                        !spawned
                            && predicate(ConditionUnit {
                                faction: unit.faction.as_deref().or(group.faction.as_deref()),
                                side: unit.side,
                                kind: &unit.r#type,
                                character: unit.character.as_deref(),
                                ai_group: group.ai_group.as_deref(),
                                position: None,
                            })
                    })
                })
            })
    }
    fn flag(&self, id: &str) -> bool {
        self.flags.is_some_and(|f| flag_is_set(f, id))
    }
    fn marks(&self) -> &[String] {
        self.marks
    }
    fn round(&self) -> u32 {
        self.round
    }
}
impl PresentationContext for LiveContext<'_, '_> {
    fn fog_index(&self, id: &Hex) -> usize {
        self.board.indices[id]
    }
    fn fog_remembered(&self, id: &Hex) -> bool {
        let i = self.fog_index(id);
        self.fog_memory
            .get(i / 64)
            .is_some_and(|word| word & (1u64 << (i % 64)) != 0)
    }
    fn lighting(&self) -> &MapLighting {
        static DEFAULT: MapLighting = MapLighting {
            time: TimeOfDay::Day,
        };
        self.rules
            .map_definitions
            .iter()
            .find(|map| map.id == self.map_id)
            .map(|m| &m.lighting)
            .unwrap_or(&DEFAULT)
    }
    fn visit_lights(&self, mut visitor: impl FnMut(&Hex, i32, i32, &TileLight)) {
        for (i, light) in self.light.iter().enumerate() {
            if let Some(light) = light {
                let (col, row) = self.board.positions[i];
                visitor(&self.board.ids[i], col, row, light);
            }
        }
    }
    fn visit_fog(&self, mut visitor: impl FnMut(&Hex, &ConditionRule)) {
        for (i, rule) in self.fog.iter().enumerate() {
            if let Some(rule) = rule {
                visitor(&self.board.ids[i], rule);
            }
        }
    }
}
impl<'catalog> Game<'catalog> {
    pub(crate) fn condition_context(&self) -> LiveContext<'_, 'catalog> {
        LiveContext::new(self, None)
    }
    pub(crate) fn event_condition_met(&self, condition: &Condition) -> bool {
        if matches(condition, &self.condition_context(), None) {
            return true;
        }
        if let Some((actor, path)) = &self.traversal {
            if self.units.iter().any(|u| u.id == *actor) {
                return path.iter().any(|&tile| {
                    matches(
                        condition,
                        &LiveContext::new(self, Some((*actor, tile))),
                        None,
                    )
                });
            }
        }
        false
    }
    pub(crate) fn group_active(&self, group: &EnemyGroup) -> bool {
        group.when.as_ref().is_none_or(|rule| {
            (rule.remember
                && self
                    .condition_memory
                    .iter()
                    .any(|m| m.strip_prefix("group:") == Some(group.id.as_str())))
                || matches(&rule.condition, &self.condition_context(), None)
        })
    }
    pub(crate) fn presentation(&self) -> BattlefieldPresentation {
        let mut result = presentation::render(&self.condition_context(), &self.condition_memory);
        result.lighting.lights.extend(self.script_mark_lights());
        result
    }
    pub(crate) fn remember_conditions(&mut self) {
        let groups = self.mission.as_ref().is_some_and(|m| {
            m.enemy_groups
                .iter()
                .any(|g| g.when.as_ref().is_some_and(|r| r.remember))
        });
        let lights = self
            .light
            .iter()
            .flatten()
            .any(|l| l.when.as_ref().is_some_and(|r| r.remember));
        if !groups
            && !lights
            && !self
                .fog
                .iter()
                .any(|f| f.as_ref().is_some_and(|r| r.remember))
        {
            return;
        }
        let mut memory = std::mem::take(&mut self.condition_memory);
        let mut known = std::mem::take(&mut self.fog_memory);
        let mut visit = |position| {
            let input = LiveContext::new(self, position);
            let added = presentation::discover(&input, &memory, &mut known);
            if !added.is_empty() {
                std::rc::Rc::make_mut(&mut memory).extend(added);
            }
            if let Some(mission) = &self.mission {
                for group in &mission.enemy_groups {
                    if group
                        .when
                        .as_ref()
                        .is_some_and(|r| r.remember && matches(&r.condition, &input, None))
                        && !memory
                            .iter()
                            .any(|m| m.strip_prefix("group:") == Some(group.id.as_str()))
                    {
                        std::rc::Rc::make_mut(&mut memory).push(format!("group:{}", group.id));
                    }
                }
            }
        };
        visit(None);
        if let Some((actor, path)) = &self.traversal {
            if self.units.iter().any(|u| u.id == *actor) {
                for &tile in path {
                    visit(Some((*actor, tile)));
                }
            }
        }
        self.condition_memory = memory;
        self.fog_memory = known;
    }
}
