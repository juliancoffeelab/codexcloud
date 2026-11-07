//! Terrain-dependent attack geometry shared by forecasts of the same battle.
use super::Game;
use crate::data::Category;
use rustc_hash::FxHashMap as HashMap;
use std::{cell::RefCell, rc::Rc};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TileSet {
    Small(u128),
    Large(Box<[u64]>),
}
impl TileSet {
    #[inline]
    pub fn first_intersection(&self, other: &Self) -> Option<usize> {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => {
                let bits = a & b;
                (bits != 0).then(|| bits.trailing_zeros() as usize)
            }
            (Self::Large(a), Self::Large(b)) => {
                a.iter()
                    .zip(b.iter())
                    .enumerate()
                    .find_map(|(word, (a, b))| {
                        let bits = a & b;
                        (bits != 0).then(|| word * 64 + bits.trailing_zeros() as usize)
                    })
            }
            _ => None,
        }
    }
    #[inline]
    pub fn new(tiles: usize) -> Self {
        if tiles <= 128 {
            Self::Small(0)
        } else {
            Self::Large(vec![0; tiles.div_ceil(64)].into_boxed_slice())
        }
    }
    #[inline]
    pub fn insert(&mut self, tile: usize) {
        match self {
            Self::Small(bits) => *bits |= 1u128 << tile,
            Self::Large(bits) => bits[tile / 64] |= 1 << (tile % 64),
        }
    }
    #[inline]
    pub fn remove(&mut self, tile: usize) {
        match self {
            Self::Small(bits) => *bits &= !(1u128 << tile),
            Self::Large(bits) => bits[tile / 64] &= !(1 << (tile % 64)),
        }
    }
    #[inline]
    pub fn contains(&self, tile: usize) -> bool {
        match self {
            Self::Small(bits) => bits & (1u128 << tile) != 0,
            Self::Large(bits) => bits[tile / 64] & (1 << (tile % 64)) != 0,
        }
    }
    #[cfg(test)]
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a & b != 0,
            (Self::Large(a), Self::Large(b)) => {
                debug_assert_eq!(a.len(), b.len());
                a.iter().zip(b).any(|(a, b)| a & b != 0)
            }
            _ => unreachable!("same-map threat masks"),
        }
    }
    #[inline]
    pub fn union_with(&mut self, other: &Self) {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => *a |= b,
            (Self::Large(a), Self::Large(b)) => {
                debug_assert_eq!(a.len(), b.len());
                for (a, b) in a.iter_mut().zip(b) {
                    *a |= b;
                }
            }
            _ => unreachable!("same-map threat masks"),
        }
    }
    #[inline]
    pub fn for_each(&self, mut visit: impl FnMut(usize)) {
        let mut word = |mut bits: u64, offset: usize| {
            while bits != 0 {
                visit(offset + bits.trailing_zeros() as usize);
                bits &= bits - 1;
            }
        };
        match self {
            Self::Small(bits) => {
                word(*bits as u64, 0);
                word((bits >> 64) as u64, 64);
            }
            Self::Large(bits) => {
                for (index, bits) in bits.iter().enumerate() {
                    word(*bits, index * 64);
                }
            }
        }
    }
}
pub(super) type AttackGeometry = Rc<RefCell<HashMap<(usize, u8), Rc<Vec<TileSet>>>>>;
pub(super) type AttackMasks = Rc<RefCell<HashMap<(usize, u8), Rc<Vec<u128>>>>>;
impl Game<'_> {
    pub fn attack_masks(&self, category: Category, range: u8) -> Rc<Vec<u128>> {
        let key = (category.index(), range);
        if let Some(masks) = self.attack_masks.borrow().get(&key) {
            return masks.clone();
        }
        let masks: Rc<Vec<u128>> = Rc::new(
            self.attack_origins(category, range)
                .iter()
                .map(|set| {
                    let TileSet::Small(bits) = set else {
                        unreachable!("small-map attack masks")
                    };
                    *bits
                })
                .collect(),
        );
        self.attack_masks.borrow_mut().insert(key, masks.clone());
        masks
    }
    pub fn in_attack_range(
        &self,
        from: usize,
        target: usize,
        category: Category,
        range: u8,
    ) -> bool {
        let gap = self.board.distance(from, target);
        if gap < 1 || gap > range as u16 {
            return false;
        }
        category != Category::Ranged
            || gap != 2
            || self.board.adjacent[from].iter().any(|&between| {
                self.board.distance(between, target) == 1
                    && !self.terrain_role(between).blocks_shots()
            })
    }
    pub fn attack_origins(&self, category: Category, range: u8) -> Rc<Vec<TileSet>> {
        let key = (category.index(), range);
        if let Some(fields) = self.attack_geometry.borrow().get(&key) {
            return fields.clone();
        }
        let fields = Rc::new(
            (0..self.terrain.len())
                .map(|target| {
                    let mut origins = TileSet::new(self.terrain.len());
                    for from in 0..self.terrain.len() {
                        if self.in_attack_range(from, target, category, range) {
                            origins.insert(from);
                        }
                    }
                    origins
                })
                .collect::<Vec<_>>(),
        );
        self.attack_geometry
            .borrow_mut()
            .insert(key, fields.clone());
        fields
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Hex;
    #[test]
    fn threat_masks_cover_machine_word_boundaries_and_large_maps() {
        for size in [64, 128, 129, 257] {
            let mut first = TileSet::new(size);
            let mut second = TileSet::new(size);
            for tile in [0, 63, 64, 127, 128, 256].into_iter().filter(|&t| t < size) {
                first.insert(tile);
                assert!(first.contains(tile));
            }
            assert!(!first.intersects(&second));
            assert_eq!(first.first_intersection(&second), None);
            second.insert(size - 1);
            first.insert(size - 1);
            assert!(first.intersects(&second));
            assert_eq!(first.first_intersection(&second), Some(size - 1));
            second.insert(0);
            assert_eq!(first.first_intersection(&second), Some(0));
            second.remove(0);
            second.remove(size - 1);
            assert!(!first.intersects(&second));
        }
    }
    #[test]
    fn terrain_changes_detach_forecast_geometry_and_preserve_the_parent() {
        let rules = crate::data::Catalog::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        for tile in 0..game.terrain.len() {
            game.set_terrain(tile, rules.terrains["plains"]);
        }
        let from = game.site(&Hex::new(3, 4));
        let target = game.site(&Hex::new(5, 4));
        let original = game.attack_origins(Category::Ranged, 2);
        assert!(original[target].contains(from));
        let mut forecast = game.fork();
        assert!(Rc::ptr_eq(
            &original,
            &forecast.attack_origins(Category::Ranged, 2)
        ));
        for tile in game.board.adjacent[from]
            .iter()
            .copied()
            .filter(|&t| game.board.distance(t, target) == 1)
        {
            forecast.set_terrain(tile, rules.terrains["forest"]);
        }
        assert!(!forecast.attack_origins(Category::Ranged, 2)[target].contains(from));
        assert!(game.attack_origins(Category::Ranged, 2)[target].contains(from));
    }
    #[test]
    fn attack_geometry_is_reciprocal_including_blocked_ranged_paths() {
        let rules = crate::data::Catalog::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        for terrain in ["plains", "forest", "hill", "grove"] {
            for tile in (0..game.terrain.len()).step_by(3) {
                game.set_terrain(tile, rules.terrains[terrain]);
            }
            for category in [Category::Melee, Category::Ranged] {
                for range in 1..=3 {
                    let geometry = game.attack_origins(category, range);
                    let masks = game.attack_masks(category, range);
                    for a in 0..game.terrain.len() {
                        for b in 0..game.terrain.len() {
                            assert_eq!(geometry[a].contains(b), geometry[b].contains(a));
                            assert_eq!(geometry[a].contains(b), masks[a] & (1u128 << b) != 0);
                        }
                    }
                }
            }
        }
    }
}
