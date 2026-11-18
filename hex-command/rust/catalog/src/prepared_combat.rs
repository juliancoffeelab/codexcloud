//! Opaque, reusable numeric combat composition. Battle retains all eligibility.
use crate::*;
use std::sync::Arc;

#[derive(PartialEq, Eq, Hash)]
pub(crate) struct CombatPreparationKey {
    unit: usize,
    equipment: Vec<usize>,
    map: StatModifiers,
    selected: Vec<StatModifiers>,
}
impl CombatPreparationKey {
    pub(crate) fn new(
        unit: usize,
        equipment: Vec<usize>,
        map: StatModifiers,
        selected: Vec<StatModifiers>,
    ) -> Self {
        Self {
            unit,
            equipment,
            map,
            selected,
        }
    }
}
struct PreparedAttack {
    outcomes: AttackOutcomes,
    category: Category,
    damage: DamageType,
    range: u8,
}
/// Final composed protection for each Catalogue terrain and owned attack handles.
/// No source numeric tables are exposed. A prepared value owns all its storage.
pub struct PreparedCombat {
    instance_identity: usize,
    attacks: Vec<PreparedAttack>,
    protection: Vec<Option<[[ProtectionBreakdown; 3]; 2]>>,
}
impl PreparedCombat {
    #[inline]
    pub fn attacks(&self) -> impl Iterator<Item = (Category, u8)> + '_ {
        self.attacks.iter().map(|a| (a.category, a.range))
    }
    /// Reflection is the already resolved incoming per-strike fact for this
    /// hypothetical exchange. Battle selects its providers through normal queries.
    #[inline]
    pub fn attack_outcomes(
        &self,
        weapon: usize,
        defender: &Self,
        terrain: &TerrainDefinition,
        reflection: i32,
    ) -> AttackOutcomes {
        assert_eq!(
            self.instance_identity, defender.instance_identity,
            "prepared combatants belong to one Catalogue"
        );
        assert_eq!(
            self.instance_identity, terrain.instance_identity,
            "prepared terrain belongs to Catalogue"
        );
        let attack = &self.attacks[weapon];
        let protection = defender.protection[terrain.id() as usize]
            .as_ref()
            .expect("terrain belongs to prepared Catalogue")[attack.category.index()]
            [attack.damage.index()];
        attack.outcomes.with_protection(protection, reflection)
    }
}
impl Catalogue {
    /// Retains exact current composition, including selected stat occurrences.
    /// The Catalogue-owned cache is bounded; returned handles survive eviction.
    pub fn prepare_combat(&self, capabilities: &UnitCapabilities<'_>) -> Arc<PreparedCombat> {
        let key = capabilities.combat_preparation_key(self.instance_identity);
        let mut cache = self
            .combat_preparations
            .lock()
            .expect("combat preparation cache");
        if let Some(prepared) = cache.get(&key) {
            return Arc::clone(prepared);
        }
        let rows = self
            .terrains()
            .map(|t| t.id() as usize)
            .max()
            .expect("Catalogue contains terrain")
            + 1;
        let mut protection = vec![None; rows];
        for terrain in self.terrains() {
            protection[terrain.id() as usize] =
                Some([Category::Melee, Category::Ranged].map(|category| {
                    [DamageType::Blade, DamageType::Pierce, DamageType::Impact]
                        .map(|kind| capabilities.protection(terrain, category, kind))
                }));
        }
        let attacks = capabilities
            .attacks()
            .map(|attack| PreparedAttack {
                outcomes: capabilities.unprotected_attack(attack),
                category: attack.category(),
                damage: attack.damage_type(),
                range: attack.range(),
            })
            .collect();
        let prepared = Arc::new(PreparedCombat {
            instance_identity: self.instance_identity,
            attacks,
            protection,
        });
        if cache.len() >= 512 {
            cache.clear();
        }
        cache.insert(key, Arc::clone(&prepared));
        prepared
    }
}
