//! Live condition queries borrow battlefield facts; owned inputs are for devtools.
use super::{LiveBattle, event_queries::EventFacts};
#[cfg(feature = "wasm")]
pub use crate::data::presentation::PresentationInput;
pub use crate::data::presentation::matches;
use crate::data::{
    presentation::{self, ConditionContext, ConditionUnit, PresentationContext},
    *,
};

pub(crate) struct LiveContext<'a, 'catalog> {
    game: &'a LiveBattle<'catalog>,
    observer: Option<&'a str>,
    position: Option<(u32, usize)>,
}
impl<'a, 'catalog> LiveContext<'a, 'catalog> {
    pub(crate) fn new(
        game: &'a LiveBattle<'catalog>,
        observer: Option<&'a str>,
        position: Option<(u32, usize)>,
    ) -> Self {
        Self {
            game,
            observer,
            position,
        }
    }
    pub(crate) fn for_faction(
        game: &'a LiveBattle<'catalog>,
        observer: &'a str,
        position: Option<(u32, usize)>,
    ) -> Self {
        Self {
            game,
            observer: Some(observer),
            position,
        }
    }
    pub(crate) fn unit_visible(&self, unit: &Unit) -> bool {
        self.game.perception(self.position).sees_unit(
            self.game
                .vision_group(self.observer.expect("explicit presentation observer")),
            unit,
        )
    }
}
impl ConditionContext for LiveContext<'_, '_> {
    fn factions(&self) -> &[Faction] {
        &self.game.factions
    }
    fn observer_faction(&self) -> Option<&str> {
        self.observer
    }
    fn visible_unit(&self, unit: ConditionUnit<'_>, observer: &str) -> bool {
        self.game.units.iter().any(|u| {
            self.game.definitions.unit(u.kind).id() == unit.kind
                && self.game.board.positions[self
                    .position
                    .filter(|(id, _)| *id == u.id)
                    .map_or(u.tile, |(_, tile)| tile)]
                    == unit.position.unwrap_or((-1, -1))
                && self.game.unit_visible_to(observer, u, self.position)
        })
    }
    fn owns(&self, id: &TileRef, side: &str) -> bool {
        super::event_queries::static_tile(&self.game.board, id)
            .is_some_and(|i| self.game.owners[i].as_deref() == Some(side))
    }
    fn position(&self, id: &TileRef) -> Option<(i32, i32)> {
        super::event_queries::static_tile(&self.game.board, id)
            .map(|i| self.game.board.positions[i])
    }
    fn visible_at(&self, position: (i32, i32), observer: &str) -> bool {
        self.game
            .board
            .indices
            .get(&Hex::new(position.0, position.1))
            .is_some_and(|&tile| {
                self.game
                    .perception(self.position)
                    .terrain_knowledge(self.game.vision_group(observer), tile)
                    != TerrainKnowledge::Unexplored
            })
    }
    #[inline(always)]
    fn any_unit(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        self.game.units.iter().any(|unit| {
            let tile = self
                .position
                .filter(|(actor, _)| *actor == unit.id)
                .map(|(_, tile)| tile)
                .unwrap_or(unit.tile);
            predicate(ConditionUnit {
                faction: self.game.names.get(unit.faction),
                kind: self.game.definitions.unit(unit.kind).id(),
                character: self.game.names.resolve(unit.character),
                ai_group: self.game.names.resolve(unit.ai_group),
                position: Some(self.game.board.positions[tile]),
            })
        })
    }
    #[inline(always)]
    fn any_pending(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
        self.game.story.scripts.as_deref().is_some_and(|state| {
            state.saved().pending.iter().any(|group| {
                group.units.iter().any(|unit| {
                    predicate(ConditionUnit {
                        faction: &group.faction,
                        kind: &unit.unit_type,
                        character: unit.character.as_deref(),
                        ai_group: group.ai_group.as_deref(),
                        position: None,
                    })
                })
            })
        })
    }
    fn flag(&self, id: &str) -> bool {
        self.game
            .story
            .campaign
            .as_deref()
            .is_some_and(|s| flag_is_set(&s.flags, id))
    }
    fn marks(&self) -> &[String] {
        self.game
            .story
            .campaign
            .as_deref()
            .map(|s| s.marks.as_slice())
            .unwrap_or_default()
    }
    fn round(&self) -> u32 {
        self.game.round
    }
}
impl PresentationContext for LiveContext<'_, '_> {
    fn fog_hidden(&self, id: &Hex) -> bool {
        !self.visible_at(
            (id.col, id.row),
            self.observer.expect("explicit presentation observer"),
        )
    }
    fn unlit_tiles(&self) -> Vec<Hex> {
        self.game
            .ambient_illumination
            .iter()
            .enumerate()
            .filter_map(|(i, level)| {
                (*level != Illumination::Lit).then_some(self.game.board.ids[i])
            })
            .collect()
    }
    fn visit_lights(&self, mut visitor: impl FnMut(&Hex, i32, i32, &TileLight)) {
        for (i, light) in self.game.light.iter().enumerate() {
            if let Some(light) = light {
                let (col, row) = self.game.board.positions[i];
                visitor(&self.game.board.ids[i], col, row, light);
            }
        }
    }
    fn visit_tiles(&self, mut visitor: impl FnMut(&Hex)) {
        for id in &self.game.board.ids {
            visitor(id);
        }
    }
}
impl<'catalog> LiveBattle<'catalog> {
    pub(crate) fn condition_context(&self) -> LiveContext<'_, 'catalog> {
        LiveContext::new(self, None, None)
    }

    pub(crate) fn remember_conditions(&mut self) {
        let groups = self.story.mission.as_ref().is_some_and(|m| {
            m.enemy_groups
                .iter()
                .any(|g| g.when.as_ref().is_some_and(|r| r.remember))
        });
        let lights = self
            .light
            .iter()
            .flatten()
            .any(|l| l.when.as_ref().is_some_and(|r| r.remember));
        if !groups && !lights {
            self.remember_faction_sight();
            return;
        }
        let mut memory = std::mem::take(&mut self.position.runtime.condition_memory);

        let mut visit =
            |position| {
                let input = LiveContext::new(self, None, position);
                let added = presentation::discover(&input, &memory);
                if !added.is_empty() {
                    std::rc::Rc::make_mut(&mut memory).extend(added);
                }
                if let Some(mission) = &self.story.mission {
                    for group in &mission.enemy_groups {
                        if group.when.as_ref().is_some_and(|r| {
                            r.remember && presentation::rule_matches(r, &input, None)
                        }) && !memory
                            .iter()
                            .any(|m| m.strip_prefix("group:") == Some(group.id.as_str()))
                        {
                            std::rc::Rc::make_mut(&mut memory).push(format!("group:{}", group.id));
                        }
                    }
                }
            };
        visit(None);
        if let Some((actor, path)) = &self.position.runtime.traversal {
            if self.position.runtime.units.iter().any(|u| u.id == *actor) {
                for &tile in path {
                    visit(Some((*actor, tile)));
                }
            }
        }
        self.position.runtime.condition_memory = memory;
        self.remember_faction_sight();
    }
}

impl super::BattlePosition<'_> {
    pub(crate) fn unit_visible_to(
        &self,
        observer: &str,
        unit: &Unit,
        position: Option<(u32, usize)>,
    ) -> bool {
        self.perception(position)
            .sees_unit(self.vision_group(observer), unit)
    }
    pub(crate) fn tile_in_sight_to(&self, observer: &str, tile: usize) -> bool {
        self.perception(None)
            .sees_terrain(self.vision_group(observer), tile)
    }
    pub(crate) fn tile_visible_to(&self, observer: &str, tile: usize) -> bool {
        self.perception(None)
            .terrain_knowledge(self.vision_group(observer), tile)
            != TerrainKnowledge::Unexplored
    }
}

impl LiveContext<'_, '_> {
    pub(crate) fn event_condition_met(&self, condition: &Condition) -> bool {
        let game = self.game;
        let observer = self.observer;
        if matches(condition, &LiveContext::new(game, observer, None), None) {
            return true;
        }
        if let Some((actor, path)) = &game.position.runtime.traversal {
            if game.position.runtime.units.iter().any(|u| u.id == *actor) {
                return path.iter().any(|&tile| {
                    matches(
                        condition,
                        &LiveContext::new(game, observer, Some((*actor, tile))),
                        None,
                    )
                });
            }
        }
        false
    }
    pub(crate) fn group_active(&self, group: &EnemyGroup) -> bool {
        let game = self.game;
        group.when.as_ref().is_none_or(|rule| {
            (rule.remember
                && game
                    .condition_memory
                    .iter()
                    .any(|m| m.strip_prefix("group:") == Some(group.id.as_str())))
                || presentation::rule_matches(rule, &game.condition_context(), None)
        })
    }
    pub(crate) fn presentation(
        &self,
        knowledge: &crate::BattleView<'_, '_>,
    ) -> BattlefieldPresentation {
        let game = self.game;
        let observer = self.observer.expect("explicit presentation observer");
        let context = self;
        let mut result = BattlefieldPresentation {
            lighting: BattlefieldLighting {
                unlit_tiles: context.unlit_tiles(),
                lights: Vec::new(),
            },
            fog: game
                .board
                .ids
                .iter()
                .filter(|id| context.fog_hidden(id))
                .copied()
                .collect(),
            visible_units: Vec::new(),
            unit_appearances: Vec::new(),
        };
        result.visible_units = game
            .units
            .iter()
            .filter(|u| context.unit_visible(u))
            .map(|u| format!("u{}", u.id))
            .collect();
        // Only units are visibility-filtered. Illumination is a physical world fact.
        result.lighting.lights.clear();
        let perception = game.perception(None);
        perception.visit_light_sources(|_, tile, emission, appearance| {
            if let Some(appearance) = appearance {
                result.lighting.lights.push(appearance.clone());
            } else if let Some(emission) = emission {
                let (col, row) = game.board.positions[tile];
                result.lighting.lights.push(BattlefieldLight {
                    col,
                    row,
                    color: Some([255, 208, 128]),
                    fixture: Some(false),
                    dx: None,
                    dy: None,
                    radius: (f64::from(emission.radius) + 0.5) * 64.0,
                    power: 1.0,
                    when: None,
                });
            }
            false
        });
        let group = game.vision_group(observer);
        result.unit_appearances = knowledge
            .units()
            .iter()
            .filter(|u| u.vision_group == group)
            .map(|unit| knowledge.unit_appearance(unit))
            .collect();

        result
    }
    pub(crate) fn resolve_text(&self, text: &ConditionalText) -> Option<String> {
        let game = self.game;
        match text {
            ConditionalText::Text(text) => Some(text.clone()),
            ConditionalText::Variants(v) => v
                .iter()
                .find(|v| {
                    v.when.as_ref().is_none_or(|w| {
                        EventFacts::for_faction(game, "player").condition_matches(w)
                    })
                })
                .map(|v| v.text.clone()),
        }
    }
    pub(crate) fn resolved_story(&self) -> Option<ResolvedStory> {
        let game = self.game;
        let mission = game.story.mission.as_ref()?;
        Some(ResolvedStory {
            briefing: self.resolve_text(&mission.briefing).unwrap_or_default(),
            debrief: mission.debrief.as_ref().and_then(|t| self.resolve_text(t)),
            hints: mission
                .hints
                .iter()
                .filter_map(|hint| match hint {
                    ConditionalHint::Text(t) => Some(t.clone()),
                    ConditionalHint::Variant(v) => v
                        .when
                        .as_ref()
                        .is_none_or(|w| {
                            EventFacts::for_faction(game, "player").condition_matches(w)
                        })
                        .then(|| v.text.clone()),
                })
                .collect(),
        })
    }
    pub(crate) fn script_speech(&self, nodes: &[Speech], out: &mut Vec<Dialogue>) {
        let game = self.game;
        for node in nodes {
            match node {
                Speech::Branch(branch) => {
                    let present = game
                        .units
                        .iter()
                        .any(|u| game.name(u.character) == Some(branch.with.as_str()));
                    self.script_speech(
                        if present {
                            &branch.dialogue
                        } else {
                            &branch.alternative
                        },
                        out,
                    );
                }
                Speech::Line(line) => {
                    let present = line.character.as_ref().and_then(|id| {
                        game.runtime
                            .units
                            .iter()
                            .find(|u| game.name(u.character) == Some(id.as_str()))
                    });
                    if let Some(id) = &line.character {
                        let party =
                            game.campaign_definition().chapters().iter().any(|m| {
                                m.map.units.iter().any(|u| {
                                    u.faction == "player" && u.character.as_ref() == Some(id)
                                }) || m.arrivals.iter().any(|a| &a.character == id)
                            });
                        if party && present.is_none() {
                            continue;
                        }
                    }
                    let portrait = line
                        .portrait
                        .clone()
                        .or_else(|| {
                            present.map(|u| game.position.definitions.unit(u.kind).id().to_owned())
                        })
                        .unwrap_or_else(|| "worker".into());
                    out.push(Dialogue {
                        character: line.character.clone(),
                        speaker: line.speaker.clone().unwrap_or_default(),
                        portrait,
                        text: line.text.clone(),
                        sound: line.sound.clone(),
                    });
                }
            }
        }
    }
}

/// Devtool visibility uses the same capabilities, inventory and perception as live play.
#[cfg(feature = "wasm")]
pub fn resolve(catalogue: &Catalogue, input: &PresentationInput) -> BattlefieldPresentation {
    struct PreviewContext<'a, 'catalog> {
        live: LiveContext<'a, 'catalog>,
        input: &'a PresentationInput,
    }
    impl ConditionContext for PreviewContext<'_, '_> {
        fn factions(&self) -> &[Faction] {
            self.live.factions()
        }
        fn observer_faction(&self) -> Option<&str> {
            Some(&self.input.observer)
        }
        fn owns(&self, id: &TileRef, side: &str) -> bool {
            self.live.owns(id, side)
        }
        fn position(&self, id: &TileRef) -> Option<(i32, i32)> {
            self.live.position(id)
        }
        fn visible_at(&self, position: (i32, i32), observer: &str) -> bool {
            self.live.visible_at(position, observer)
        }
        fn visible_unit(&self, unit: ConditionUnit<'_>, observer: &str) -> bool {
            ConditionContext::visible_unit(&self.live, unit, observer)
        }
        fn any_unit(&self, predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
            self.live.any_unit(predicate)
        }
        fn any_pending(&self, mut predicate: impl FnMut(ConditionUnit<'_>) -> bool) -> bool {
            self.input.pending.iter().any(|unit| {
                predicate(ConditionUnit {
                    faction: &unit.faction,
                    kind: &unit.unit_type,
                    character: unit.character.as_deref(),
                    ai_group: unit.ai_group.as_deref(),
                    position: None,
                })
            })
        }
        fn flag(&self, id: &str) -> bool {
            self.input.choices.iter().any(|choice| choice == id)
        }
        fn marks(&self) -> &[Text] {
            &self.input.marks
        }
        fn round(&self) -> u32 {
            self.live.round()
        }
    }
    impl PresentationContext for PreviewContext<'_, '_> {
        fn unlit_tiles(&self) -> Vec<Hex> {
            self.live.unlit_tiles()
        }
        fn visit_lights(&self, visitor: impl FnMut(&Hex, i32, i32, &TileLight)) {
            self.live.visit_lights(visitor)
        }
        fn visit_tiles(&self, visitor: impl FnMut(&Hex)) {
            self.live.visit_tiles(visitor)
        }
        fn fog_hidden(&self, id: &Hex) -> bool {
            self.live.fog_hidden(id)
        }
    }
    let game = LiveBattle::from_state(catalogue, input.state.clone());
    let context = PreviewContext {
        live: LiveContext::for_faction(&game, &input.observer, None),
        input,
    };
    let mut memory = input.state.condition_memory.clone();
    memory.extend(presentation::discover(&context, &memory));
    let mut result = presentation::render(&context, &memory);
    let perception = game.perception(None);
    let group = game.vision_group(&input.observer);
    result.visible_units = game
        .units
        .iter()
        .filter(|unit| perception.sees_unit(group, unit))
        .map(|unit| format!("u{}", unit.id))
        .collect();
    result
}
