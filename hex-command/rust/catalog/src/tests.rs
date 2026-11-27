use crate::*;

#[test]
fn authored_catalogue_parses_and_resolves_all_definitions() {
    let group = neutral_modifiers().sight;
    let catalogue = Catalogue::load().expect("current authored catalogue parses");
    for profile in [
        &catalogue.meta().recruiter_weights.standard,
        &catalogue.meta().recruiter_weights.aggressive_predictor,
    ] {
        assert!(profile.endurance_reference_hp > 0.0);
        let values = serde_json::to_value(profile).unwrap();
        for value in values.as_object().unwrap().values() {
            if !value.is_null() {
                let value = value.as_f64().unwrap();
                assert!(
                    value.is_finite() && value >= 0.0,
                    "authored recruiter coefficients are finite and nonnegative"
                );
            }
        }
    }
    let simulation = &catalogue.meta().simulation;
    for roster in &simulation.rosters {
        catalogue.unit(&roster.leader);
        for id in &roster.pool {
            catalogue.unit(id);
        }
    }
    assert!(simulation.arena.default_max_rounds > 0 && simulation.campaign.default_max_rounds > 0);
    for scenario in &simulation.arena.scenarios {
        assert!(!scenario.variants.is_empty());
        for variant in &scenario.variants {
            let map = catalogue.map(&variant.map, variant.map_revision);
            for point in &scenario.territory {
                assert!(
                    map.tiles()
                        .iter()
                        .any(|tile| tile.col == point.col && tile.row == point.row),
                    "arena capture point belongs to every authored variant"
                );
            }
        }
    }
    for id in &simulation.arena.default_scenarios {
        assert!(simulation.arena.scenarios.iter().any(|s| &s.id == id));
    }
    for id in &simulation.campaign.default_campaigns {
        catalogue.campaign(id);
    }
    for (id, config) in &simulation.campaign.campaigns {
        let campaign = catalogue.campaign(id);
        for route in config.routes.values() {
            for (stage, overrides) in &route.stages {
                let chapter = campaign
                    .chapters()
                    .iter()
                    .find(|c| c.map_id() == stage)
                    .expect("configured stage exists in campaign");
                for event in overrides.script_conditions.keys() {
                    assert!(
                        chapter
                            .scripts()
                            .events
                            .iter()
                            .chain(&overrides.script_events)
                            .any(|e| &e.id == event)
                    );
                }
            }
        }
    }
    for arena in &simulation.training.arenas {
        assert!(simulation.arena.scenarios.iter().any(|s| s.id == arena.id));
        for pair in &arena.pairs {
            for id in [&pair.player, &pair.enemy] {
                assert!(simulation.rosters.iter().any(|r| &r.id == id));
            }
        }
    }
    for id in simulation.training.campaigns.keys() {
        catalogue.campaign(id);
    }
    assert!(catalogue.units().next().is_some());
    assert!(catalogue.maps().next().is_some());
    assert!(catalogue.campaigns().next().is_some());
    fn unique<'a>(ids: impl Iterator<Item = &'a str>) {
        let mut seen = std::collections::HashSet::new();
        for id in ids {
            assert!(seen.insert(id), "duplicate authored definition {id}");
        }
    }
    unique(catalogue.units().map(|u| u.id()));
    unique(catalogue.items().map(|u| u.id()));
    unique(catalogue.abilities().map(|u| u.id()));
    unique(catalogue.environment_effects().map(|u| u.id()));
    let mut maps = std::collections::HashSet::new();
    for map in catalogue.maps() {
        assert!(
            maps.insert((map.id(), map.revision())),
            "duplicate map identity and revision"
        );
        catalogue.map(map.id(), map.revision());
    }
    unique(catalogue.campaigns().map(|u| u.id()));
    for kind in ConcealmentKind::ALL {
        catalogue.concealment_kind(kind);
    }
    let mut groups = std::collections::HashMap::new();
    let effects = catalogue
        .abilities()
        .flat_map(|a| &a.source.effects)
        .chain(catalogue.terrains().flat_map(|t| &t.source.effects))
        .chain(
            catalogue
                .environment_effects()
                .flat_map(|e| &e.source.effects)
                .filter_map(|e| match e {
                    EnvironmentEffect::Unit { effect } => Some(effect),
                    _ => None,
                }),
        );
    for effect in effects {
        assert!(!effect.stacking.group.is_empty());
        let supported = match &effect.effect {
            EffectValue::Healing { power } => {
                assert!(*power > 0);
                matches!(
                    effect.stacking.combine,
                    EffectCombine::Add | EffectCombine::Max
                )
            }
            EffectValue::Thorns {
                damage_per_strike, ..
            } => {
                assert!(*damage_per_strike > 0);
                matches!(
                    effect.stacking.combine,
                    EffectCombine::Add | EffectCombine::Max
                )
            }
            EffectValue::Sight { .. } | EffectValue::StatModifiers { .. } => {
                effect.stacking.combine == EffectCombine::Add
            }
            EffectValue::Concealment { rule } => {
                assert!(!rule.requires.terrains.as_ref().is_some_and(Vec::is_empty));
                assert!(
                    !rule
                        .requires
                        .illumination
                        .as_ref()
                        .is_some_and(Vec::is_empty)
                );
                effect.stacking.combine == EffectCombine::Union
            }
            EffectValue::Light { radius } => {
                assert!(*radius > 0);
                effect.stacking.combine == EffectCombine::Union
            }
        };
        assert!(
            supported,
            "unsupported operation in {}",
            effect.stacking.group
        );
        let signature = (
            std::mem::discriminant(&effect.effect),
            &effect.stacking.combine,
        );
        if let Some(previous) = groups.insert(&effect.stacking.group, signature) {
            assert_eq!(
                previous, signature,
                "authored effect groups have consistent kind and operation"
            );
        }
    }
    for unit in catalogue.units() {
        assert!(unit.max_hp > 0, "{} has positive maximum HP", unit.id());
        catalogue.race(unit.race());
        let neutral = neutral_modifiers();
        let caps = unit.compose(UnitInputs {
            vision_group_sight: &group,
            equipment: &[],
            gained_abilities: &[],
            selected_effects: &[],
            map_modifiers: &neutral,
            perception_contributions: &[],
        });
        for terrain in catalogue.terrains() {
            caps.step_cost(terrain);
            assert!(
                terrain.matches_concealment_requirements(&ConcealmentRequirements {
                    terrains: Some(vec![terrain.source.role]),
                    illumination: Some(vec![Illumination::Darkness])
                })
            );
            assert!(
                terrain.matches_concealment_requirements(&ConcealmentRequirements {
                    terrains: None,
                    illumination: Some(vec![Illumination::Darkness])
                })
            );
            assert!(
                !terrain.matches_concealment_requirements(&ConcealmentRequirements {
                    terrains: Some(vec![]),
                    illumination: None
                })
            );
        }
        for illumination in [
            Illumination::Lit,
            Illumination::Unlit,
            Illumination::Darkness,
        ] {
            caps.sight(illumination);
        }
        for attack in caps.attacks() {
            assert!((1..=16).contains(&attack.source.count));
            assert!(attack.source.damage >= 0);
            assert!(attack.source.spread >= 0 && attack.source.spread <= attack.source.damage);
            assert!(attack.range() > 0);
            assert!(attack.source.drain <= 100);
        }

        for promotion in unit.promotion_ids() {
            catalogue.unit(promotion);
        }
    }
    for map in catalogue.maps() {
        for tile in map.tiles() {
            catalogue.terrain(tile.terrain);
            for effect in &tile.environment_effects {
                catalogue.environment_effect(effect);
            }
        }
        for army in &map.roster().factions {
            let group = map
                .roster()
                .vision_groups
                .iter()
                .find(|g| g.id == army.faction.vision_group)
                .expect("authored army vision group exists");
            for unit in &army.units {
                let definition = catalogue.unit(&unit.unit_type);
                let equipment: Vec<_> = unit.items.iter().map(|id| catalogue.item(id)).collect();
                let caps = definition.compose(UnitInputs {
                    vision_group_sight: &group.sight,
                    equipment: &equipment,
                    gained_abilities: &[],
                    selected_effects: &[],
                    map_modifiers: map.modifiers(),
                    perception_contributions: &[],
                });
                let sight = caps.sight(Illumination::Lit);
                if group.sight.unlimited {
                    assert!(
                        sight.range.is_none(),
                        "group unlimited sight enters composed capabilities"
                    );
                }
                assert!(
                    group
                        .sight
                        .penetrates
                        .iter()
                        .all(|kind| sight.penetrates.contains(kind)),
                    "group penetration enters composed capabilities"
                );
                for item in &unit.items {
                    catalogue.item(item);
                }
            }
        }
    }
    for campaign in catalogue.campaigns() {
        catalogue.unit(campaign.leader_id());
        for chapter in campaign.chapters() {
            catalogue.map(chapter.map_id(), chapter.map_revision());
        }
    }
}

fn neutral_modifiers() -> StatModifiers {
    StatModifiers {
        sight: SightContribution {
            unlimited: false,
            sight_modifiers: SightModifiers {
                lit: 0,
                unlit: 0,
                darkness: 0,
            },
            penetrates: vec![],
        },
        damage_modifiers: DamageModifiers {
            blade: 0,
            pierce: 0,
            impact: 0,
        },
        terrain_modifiers: std::collections::BTreeMap::new(),
    }
}
fn healing(power: i32, group: &str, combine: EffectCombine) -> ScopedEffect {
    ScopedEffect {
        scope: EffectScope {
            radius: 2,
            include_self: false,
            targets: HealingTargets::Allies,
        },
        stacking: StackingRule {
            group: group.into(),
            combine,
        },
        effect: EffectValue::Healing { power },
    }
}
#[test]
fn selected_occurrences_stack_without_becoming_owned_emissions() {
    let group = neutral_modifiers().sight;
    let catalogue = Catalogue::load().unwrap();
    let unit = catalogue.units().next().unwrap();
    let map = neutral_modifiers();
    let effects = [
        healing(2, "shared", EffectCombine::Max),
        healing(7, "shared", EffectCombine::Max),
        healing(3, "extra", EffectCombine::Add),
        healing(7, "shared", EffectCombine::Max),
    ];
    let selected: Vec<_> = effects.iter().map(EffectHandle::from_value).collect();
    let caps = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &[],
        gained_abilities: &[],
        selected_effects: &selected,
        map_modifiers: &map,
        perception_contributions: &[],
    });
    assert_eq!(caps.healing(), 10);
    #[cfg(feature = "simulation")]
    assert_eq!(
        diagnostics::healing(&caps),
        diagnostics::EffectTotal {
            amount: 10,
            contributors: vec![1, 2, 3]
        }
    );
    assert_eq!(
        caps.effects().count(),
        unit.effects(&[], &[]).count(),
        "incoming auras are not re-emitted"
    );
    let same_definition = healing(4, "two-emitters", EffectCombine::Add);
    let occurrences = [
        EffectHandle::from_value(&same_definition),
        EffectHandle::from_value(&same_definition),
    ];
    let caps = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &[],
        gained_abilities: &[],
        selected_effects: &occurrences,
        map_modifiers: &map,
        perception_contributions: &[],
    });
    assert_eq!(
        caps.healing(),
        8,
        "two emitting sources remain distinct occurrences"
    );
    #[cfg(feature = "simulation")]
    assert_eq!(
        diagnostics::healing(&caps),
        diagnostics::EffectTotal {
            amount: 8,
            contributors: vec![0, 1]
        }
    );
    let caps = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &[],
        gained_abilities: &[],
        selected_effects: &[],
        map_modifiers: &map,
        perception_contributions: &[],
    });
    assert_eq!(
        caps.healing(),
        0,
        "owned scoped effects require explicit battle selection"
    );
}
#[test]
fn equipment_and_reciprocal_combat_use_current_composed_inputs() {
    let group = neutral_modifiers().sight;
    let catalogue = Catalogue::load().unwrap();
    let unit = catalogue.units().next().unwrap();
    let map = neutral_modifiers();
    let naked = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &[],
        gained_abilities: &[],
        selected_effects: &[],
        map_modifiers: &map,
        perception_contributions: &[],
    });
    let armor = catalogue
        .items()
        .find(|i| i.source.armor.is_some())
        .expect("authored armor fixture");
    let equipment = [armor];
    let protected = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &equipment,
        gained_abilities: &[],
        selected_effects: &[],
        map_modifiers: &map,
        perception_contributions: &[],
    });
    let attack = naked.attacks().nth(0).expect("attack index is valid");
    let terrain = catalogue.terrains().next().unwrap();
    let ordinary = naked.attack_outcomes(attack, &naked, terrain);
    let armored = naked.attack_outcomes(attack, &protected, terrain);
    assert!(
        armored.mean() < ordinary.mean(),
        "authored armor changes received damage"
    );
    assert!(armored.bounds().0.damage <= ordinary.bounds().0.damage);
    assert!(armored.bounds().1.damage <= ordinary.bounds().1.damage);
    let retaliation = protected.attacks().nth(0).expect("attack index is valid");
    assert_eq!(
        protected
            .attack_outcomes(retaliation, &naked, terrain)
            .bounds(),
        ordinary.bounds(),
        "retaliation resolves the opposite recipient's current defenses"
    );
    let granted = catalogue
        .items()
        .find(|i| i.attacks().next().is_some())
        .expect("authored equipment attack fixture");
    let equipment = [granted];
    let equipped = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &equipment,
        gained_abilities: &[],
        selected_effects: &[],
        map_modifiers: &map,
        perception_contributions: &[],
    });
    assert_eq!(
        equipped.attacks().count(),
        naked.attacks().count() + granted.attacks().count()
    );
}

#[test]
fn gained_abilities_and_runtime_contributions_use_target_lighting() {
    let group = neutral_modifiers().sight;
    let catalogue = Catalogue::load().unwrap();
    let unit = catalogue.units().next().unwrap();
    let ability = catalogue
        .abilities()
        .find(|a| {
            a.effects().any(|e| match e.value() {
                EffectValue::Sight {
                    sight_modifiers, ..
                } => sight_modifiers.unlit > sight_modifiers.lit,
                _ => false,
            })
        })
        .expect("authored target-light sight modifier fixture");
    let gained = [ability];
    let selected: Vec<_> = ability.effects().collect();
    let neutral = neutral_modifiers();
    let base = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &[],
        gained_abilities: &[],
        selected_effects: &[],
        map_modifiers: &neutral,
        perception_contributions: &[],
    });
    let upgraded = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &[],
        gained_abilities: &gained,
        selected_effects: &selected,
        map_modifiers: &neutral,
        perception_contributions: &[],
    });
    assert_eq!(
        upgraded.effects().count(),
        base.effects().count() + selected.len()
    );
    assert!(upgraded.sight(Illumination::Unlit).range > upgraded.sight(Illumination::Lit).range);
    let mut weather = neutral_modifiers();
    weather.sight.sight_modifiers = SightModifiers {
        lit: -1,
        unlit: -1,
        darkness: -1,
    };
    let changed = unit.compose(UnitInputs {
        vision_group_sight: &group,
        equipment: &[],
        gained_abilities: &gained,
        selected_effects: &selected,
        map_modifiers: &weather,
        perception_contributions: &[],
    });
    assert!(
        changed.sight(Illumination::Unlit).range < upgraded.sight(Illumination::Unlit).range,
        "current weather input affects the new composition"
    );
}

#[test]
fn meta_scores_use_mean_strikes_and_owned_healing_not_incoming_healing() {
    let mut catalogue = Catalogue::load().unwrap();
    let mut unit = catalogue.units.remove(0);
    let mut single = unit.attacks[0].clone();
    single.id = "single".into();
    single.damage = 10;
    single.spread = 0;
    single.count = 1;
    let mut multi = single.clone();
    multi.id = "multi".into();
    multi.damage = 8;
    multi.spread = 2;
    multi.count = 3;
    unit.attacks = vec![single, multi];
    unit.racial_abilities.clear();
    unit.abilities = vec![crate::schema::AbilityDefinition {
        id: "outgoing-healing".into(),
        name: "Outgoing healing".into(),
        description: "Meta valuation fixture".into(),
        effects: vec![
            healing(4, "shared", EffectCombine::Max),
            healing(6, "shared", EffectCombine::Max),
            healing(2, "extra", EffectCombine::Add),
        ],
    }];
    let mut aura = healing(6, "thorn-aura", EffectCombine::Max);
    aura.scope.radius = 1;
    aura.scope.include_self = false;
    aura.effect = EffectValue::Thorns {
        damage_per_strike: 6,
        incoming_category: Category::Melee,
    };
    unit.abilities[0].effects.push(aura);
    let mut received_thorns = healing(7, "received-thorns", EffectCombine::Add);
    received_thorns.effect = EffectValue::Thorns {
        damage_per_strike: 7,
        incoming_category: Category::Melee,
    };
    let incoming = healing(99, "incoming", EffectCombine::Add);
    let selected = [
        EffectHandle::from_value(&incoming),
        EffectHandle::from_value(&received_thorns),
    ];
    let map = neutral_modifiers();
    let capabilities = unit.compose(UnitInputs {
        vision_group_sight: &map.sight,
        equipment: &[],
        gained_abilities: &[],
        selected_effects: &selected,
        map_modifiers: &map,
        perception_contributions: &[],
    });
    let protection = capabilities.neural_unit_features(
        &catalogue.terrain[0],
        Category::Melee,
        DamageType::Blade,
    );
    assert_eq!(protection.reflection_per_strike, 7);
    let normal = capabilities.protection(&catalogue.terrain[0], Category::Melee, DamageType::Blade);
    assert_eq!(
        (normal.armour, normal.terrain),
        (protection.armour, protection.terrain)
    );
    let description = capabilities.attack_description(
        capabilities
            .attacks()
            .nth(1)
            .expect("attack index is valid"),
    );
    assert_eq!(
        (description.damage, description.spread, description.count),
        (8, 2, 3)
    );
    assert_eq!(
        SelectedEffects::new(&selected).meta_thorn_score(Category::Melee),
        7
    );
    assert_eq!(
        SelectedEffects::new(&selected).meta_thorn_score(Category::Ranged),
        0
    );
    #[cfg(feature = "simulation")]
    {
        assert_eq!(
            diagnostics::thorns(&capabilities, Category::Melee),
            diagnostics::EffectTotal {
                amount: 7,
                contributors: vec![1]
            }
        );
        assert_eq!(
            diagnostics::thorns(&capabilities, Category::Ranged),
            diagnostics::EffectTotal {
                amount: 0,
                contributors: vec![]
            }
        );
        let diagnostic = diagnostics::attack_outcomes(
            capabilities
                .attacks()
                .nth(1)
                .expect("attack index is valid"),
            -2,
            -1,
        );
        assert_eq!(diagnostic.bounds().0.raw, 6);
        assert_eq!(diagnostic.bounds().1.raw, 10);
        assert_eq!(diagnostic.mean(), 5.0);
        assert_eq!(diagnostic.strike_count(), 3);
    }
    assert_eq!(unit.assess(&[], &[]).meta_total_attack_score(None), 24.0);
    assert_eq!(
        unit.assess(&[], &[])
            .meta_total_attack_score(Some(unit.attacks[0].category)),
        24.0
    );
    assert_eq!(protection.owned_thorn_aura_bound, 6);
    assert_eq!(
        protection.owned_healing_bound, 8,
        "incoming healing is excluded from owned emissions"
    );
    let attack = capabilities.neural_pure_attack_features(
        capabilities
            .attacks()
            .nth(1)
            .expect("attack index is valid"),
    );
    assert_eq!(
        (
            attack.mean_damage,
            attack.spread,
            attack.strike_count,
            attack.drain_percent
        ),
        (8.0, 2, 3, unit.attacks[0].drain)
    );
    assert_eq!(
        unit.assess(&[], &[]).meta_attack_score(),
        10.0,
        "legacy valuation compares mean damage per strike"
    );
    assert_eq!(
        unit.assess(&[], &[]).meta_healing_score(),
        8.0,
        "owned Max+Add emissions are valued without inventing a recipient"
    );
    assert_eq!(
        capabilities.healing(),
        99,
        "received healing remains an independent mechanical result"
    );
}
