//! Terrain-dependent attack geometry shared by forecasts of the same battle.
use super::Game;
use crate::data::{Category, Terrain};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TileSet {
    Small(u128),
    Large(Box<[u64]>),
}
impl TileSet {
    pub(crate) fn new(tiles: usize) -> Self {
        if tiles <= 128 {
            Self::Small(0)
        } else {
            Self::Large(vec![0; tiles.div_ceil(64)].into_boxed_slice())
        }
    }
    pub(crate) fn insert(&mut self, tile: usize) {
        match self {
            Self::Small(bits) => *bits |= 1u128 << tile,
            Self::Large(bits) => bits[tile / 64] |= 1 << (tile % 64),
        }
    }
    pub(crate) fn contains(&self, tile: usize) -> bool {
        match self {
            Self::Small(bits) => bits & (1u128 << tile) != 0,
            Self::Large(bits) => bits[tile / 64] & (1 << (tile % 64)) != 0,
        }
    }
    pub(crate) fn intersects(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Small(a), Self::Small(b)) => a & b != 0,
            (Self::Large(a), Self::Large(b)) => {
                debug_assert_eq!(a.len(), b.len());
                a.iter().zip(b).any(|(a, b)| a & b != 0)
            }
            _ => unreachable!("same-map threat masks"),
        }
    }
}
pub(super) type AttackGeometry = Rc<RefCell<HashMap<(usize, u8), Rc<Vec<TileSet>>>>>;
impl Game<'_> {
    pub(crate) fn in_attack_range(
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
                    && !self.terrain_is(between, Terrain::Forest)
                    && !self.terrain_is(between, Terrain::Hill)
                    && !self.terrain_is(between, Terrain::Grove)
            })
    }
    pub(crate) fn attack_origins(&self, category: Category, range: u8) -> Rc<Vec<TileSet>> {
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
            second.insert(size - 1);
            first.insert(size - 1);
            assert!(first.intersects(&second));
        }
    }
    #[test]
    fn terrain_changes_detach_forecast_geometry_and_preserve_the_parent() {
        let rules = crate::data::Catalog::load().unwrap();
        let mut game = Game::new(&rules, "mirror");
        for tile in 0..game.terrain.len() {
            game.set_terrain(tile, rules.terrains["plains"]);
        }
        let from = game.site("3,4");
        let target = game.site("5,4");
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
}
