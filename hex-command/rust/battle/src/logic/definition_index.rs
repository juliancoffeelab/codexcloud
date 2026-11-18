//! Battle-local keys and prepared owned handle lists; numeric stacking remains in Catalogue.
use crate::data::{
    AttackHandle, Catalogue, EffectHandle, Inventories, ItemDefinition, OwnedAssessment,
    PreparedCombat, StatModifiers, Terrain, TerrainDefinition, Unit, UnitDefinition, WeaponKey,
};
use rustc_hash::FxHashMap;
use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

pub(crate) struct DefinitionIndex<'catalog> {
    units: Vec<&'catalog UnitDefinition>,
    terrain: Vec<&'catalog TerrainDefinition>,
    unit_ids: FxHashMap<String, usize>,
    terrain_ids: FxHashMap<String, usize>,
    items: FxHashMap<&'catalog str, &'catalog ItemDefinition>,
    unequipped: Vec<OnceCell<Rc<OwnedHandles<'catalog>>>>,
    equipped: RefCell<FxHashMap<(usize, usize), Rc<OwnedHandles<'catalog>>>>,
}
pub(crate) struct OwnedHandles<'catalog> {
    /// Valid only through the position-independent combat guard. This retains
    /// Catalogue's opaque result, never source numbers or a stacking formula.
    pub combat: RefCell<Option<(Rc<StatModifiers>, std::sync::Arc<PreparedCombat>)>>,
    pub assessment: OwnedAssessment,
    pub attacks: Vec<AttackHandle<'catalog>>,
    pub equipment: Vec<&'catalog ItemDefinition>,
    pub effects: Vec<EffectHandle<'catalog>>,
}
impl<'catalog> DefinitionIndex<'catalog> {
    pub(crate) fn new(catalogue: &'catalog Catalogue) -> Self {
        let units: Vec<_> = catalogue.units().collect();
        let terrain: Vec<_> = catalogue.terrains().collect();
        let unit_ids = units
            .iter()
            .enumerate()
            .map(|(i, u)| (u.id().to_owned(), i))
            .collect();
        let terrain_ids = terrain
            .iter()
            .enumerate()
            .map(|(i, t)| (t.id().as_str().to_owned(), i))
            .collect();
        let unequipped = (0..units.len()).map(|_| OnceCell::new()).collect();
        let items = catalogue.items().map(|item| (item.id(), item)).collect();
        Self {
            items,
            unequipped,
            equipped: RefCell::new(FxHashMap::default()),
            units,
            terrain,
            unit_ids,
            terrain_ids,
        }
    }
    /// Definition handles have Catalogue lifetime. Runtime inventory row IDs can
    /// collide between forks, so equipped lists use immutable row identities.
    pub(crate) fn owned(
        &self,
        unit: &Unit,
        inventories: &Inventories,
    ) -> Rc<OwnedHandles<'catalog>> {
        if unit.inventory.0 == 0 {
            return Rc::clone(self.unequipped[unit.kind].get_or_init(|| {
                Rc::new(OwnedHandles {
                    combat: RefCell::new(None),
                    assessment: self.units[unit.kind].assess(&[], &[]),
                    equipment: Vec::new(),
                    attacks: self.units[unit.kind].attacks(&[]).collect(),
                    effects: self.units[unit.kind].effects(&[], &[]).collect(),
                })
            }));
        }
        let identity = (unit.kind, inventories.identity(unit.inventory));
        if let Some(owned) = self.equipped.borrow().get(&identity) {
            return Rc::clone(owned);
        }
        let equipment: Vec<_> = inventories
            .items(unit.inventory)
            .map(|id| self.items[id])
            .collect();
        let owned = Rc::new(OwnedHandles {
            combat: RefCell::new(None),
            assessment: self.units[unit.kind].assess(&equipment, &[]),
            attacks: self.units[unit.kind].attacks(&equipment).collect(),
            effects: self.units[unit.kind].effects(&equipment, &[]).collect(),
            equipment,
        });
        // Eviction bounds derived storage; active queries keep their own Rc.
        if self.equipped.borrow().len() >= 4096 {
            self.equipped.borrow_mut().clear();
        }
        self.equipped
            .borrow_mut()
            .insert(identity, Rc::clone(&owned));
        owned
    }
    pub(crate) fn find_kind(&self, id: &str) -> Option<usize> {
        self.unit_ids.get(id).copied()
    }
    #[inline]
    pub(crate) fn unit(&self, kind: usize) -> &'catalog UnitDefinition {
        self.units[kind]
    }
    pub(crate) fn kind(&self, id: &str) -> usize {
        *self
            .unit_ids
            .get(id)
            .expect("unit type exists in catalogue")
    }
    #[inline]
    pub(crate) fn terrain(&self, kind: usize) -> &'catalog TerrainDefinition {
        self.terrain[kind]
    }
    pub(crate) fn terrain_name(&self, id: &str) -> usize {
        self.terrain_ids[id]
    }
    pub(crate) fn terrain_kind(&self, id: Terrain) -> usize {
        self.terrain_ids[id.as_str()]
    }
}

impl std::fmt::Debug for DefinitionIndex<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefinitionIndex")
            .field("units", &self.units.len())
            .field("terrain", &self.terrain.len())
            .finish()
    }
}

/// Runtime identities survive temporary reattachment to borrowed Catalogue handles.
/// They are search state, never persisted in saves.
#[derive(Default, Debug)]
pub(crate) struct WeaponIdentities((FxHashMap<String, WeaponKey>, Vec<String>));
impl WeaponIdentities {
    pub(crate) fn weapon_key(&mut self, id: &str) -> WeaponKey {
        let weapons = &mut self.0;
        if let Some(key) = weapons.0.get(id) {
            return *key;
        }
        let key = WeaponKey(u32::try_from(weapons.1.len()).expect("weapon identity space"));
        weapons.1.push(id.to_owned());
        weapons.0.insert(id.to_owned(), key);
        key
    }
    pub(crate) fn weapon_id(&self, key: WeaponKey) -> Option<String> {
        self.0.1.get(key.0 as usize).cloned()
    }
}
