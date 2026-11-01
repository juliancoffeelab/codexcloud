//! Immutable goal sites share their backing storage across planner snapshots.
use super::orders::SiteKey;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::{cell::RefCell, ops::Deref, rc::Rc};
type Entries = SmallVec<[SiteKey; 8]>;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Sites(Rc<Entries>);
#[derive(Default)]
struct Pool {
    slots: Vec<Rc<Entries>>,
    cursor: usize,
}
thread_local! {static POOL:RefCell<Pool>=RefCell::new(Pool::default());}
impl Pool {
    fn acquire(&mut self, needed: usize) -> usize {
        let count = self.slots.len();
        let find = |fits: bool| {
            (0..count)
                .map(|offset| (self.cursor + offset) % count)
                .find(|&index| {
                    Rc::strong_count(&self.slots[index]) == 1
                        && (!fits || self.slots[index].capacity() >= needed)
                })
        };
        let index = find(true).or_else(|| find(false)).unwrap_or_else(|| {
            self.slots.push(Rc::new(Entries::new()));
            count
        });
        self.cursor = (index + 1) % self.slots.len();
        let entries = Rc::get_mut(&mut self.slots[index]).expect("unleased goal sites");
        entries.clear();
        entries.reserve_exact(needed);
        index
    }
}
#[cfg(target_arch = "wasm32")]
pub(super) fn prepare(tiles: usize) {
    POOL.with_borrow_mut(|pool| {
        pool.slots
            .reserve(144usize.saturating_sub(pool.slots.len()));
        while pool.slots.len() < 128 {
            pool.slots.push(Rc::new(Entries::new()));
        }
        let mut large = pool
            .slots
            .iter()
            .filter(|slot| slot.capacity() >= tiles)
            .count();
        for slot in &mut pool.slots {
            if large >= 16 {
                break;
            }
            if slot.capacity() < tiles {
                if let Some(entries) = Rc::get_mut(slot) {
                    entries.clear();
                    entries.reserve_exact(tiles);
                    large += 1;
                }
            }
        }
        while large < 16 {
            pool.slots.push(Rc::new(Entries::with_capacity(tiles)));
            large += 1;
        }
    });
}
impl Deref for Sites {
    type Target = [SiteKey];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl FromIterator<SiteKey> for Sites {
    fn from_iter<I: IntoIterator<Item = SiteKey>>(items: I) -> Self {
        #[cfg(target_arch = "wasm32")]
        let entries: crate::reuse::Vec<SiteKey> = items.into_iter().collect();
        #[cfg(not(target_arch = "wasm32"))]
        let entries: Entries = items.into_iter().collect();
        POOL.with_borrow_mut(|pool| {
            let index = pool.acquire(entries.len());
            Rc::get_mut(&mut pool.slots[index])
                .expect("unleased goal sites")
                .extend(entries);
            Self(pool.slots[index].clone())
        })
    }
}
impl<const N: usize> From<[SiteKey; N]> for Sites {
    fn from(values: [SiteKey; N]) -> Self {
        values.into_iter().collect()
    }
}
impl<'a> IntoIterator for &'a Sites {
    type Item = &'a SiteKey;
    type IntoIter = std::slice::Iter<'a, SiteKey>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
pub struct IntoIter {
    sites: Sites,
    index: usize,
}
impl Iterator for IntoIter {
    type Item = SiteKey;
    fn next(&mut self) -> Option<SiteKey> {
        let value = self.sites.get(self.index)?.clone();
        self.index += 1;
        Some(value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = self.sites.len() - self.index;
        (left, Some(left))
    }
}
impl ExactSizeIterator for IntoIter {}
impl IntoIterator for Sites {
    type Item = SiteKey;
    type IntoIter = IntoIter;
    fn into_iter(self) -> IntoIter {
        IntoIter {
            sites: self,
            index: 0,
        }
    }
}
