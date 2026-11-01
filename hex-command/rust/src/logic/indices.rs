//! Direct lookup for ordinary sequential IDs, with bounded storage for sparse saves.
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub(super) enum UnitIndices {
    Dense { slots: Vec<usize>, length: usize },
    Sparse(FxHashMap<u32, usize>),
}
impl Clone for UnitIndices {
    fn clone(&self) -> Self {
        match self {
            Self::Dense { slots, length } => Self::Dense {
                slots: slots.clone(),
                length: *length,
            },
            Self::Sparse(map) => Self::Sparse(map.clone()),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (
                Self::Dense { slots, length },
                Self::Dense {
                    slots: from,
                    length: count,
                },
            ) => {
                slots.clone_from(from);
                *length = *count;
            }
            (Self::Sparse(map), Self::Sparse(from)) => map.clone_from(from),
            (target, source) => *target = source.clone(),
        }
    }
}
impl Default for UnitIndices {
    fn default() -> Self {
        Self::Dense {
            slots: Vec::new(),
            length: 0,
        }
    }
}
impl UnitIndices {
    pub fn clear(&mut self) {
        match self {
            Self::Dense { slots, length } => {
                slots.clear();
                *length = 0;
            }
            Self::Sparse(map) => map.clear(),
        }
    }
    #[inline]
    pub fn get(&self, id: &u32) -> Option<&usize> {
        match self {
            Self::Dense { slots, .. } => slots
                .get(*id as usize)
                .filter(|&&value| value != usize::MAX),
            Self::Sparse(map) => map.get(id),
        }
    }
    pub fn insert(&mut self, id: u32, value: usize) {
        if let Self::Dense { slots, length } = self {
            if id < 256 {
                let index = id as usize;
                if slots.len() <= index {
                    slots.resize(index + 1, usize::MAX);
                }
                if slots[index] == usize::MAX {
                    *length += 1;
                }
                slots[index] = value;
                return;
            }
            let mut sparse = FxHashMap::with_capacity_and_hasher(*length + 1, Default::default());
            for (id, &value) in slots.iter().enumerate() {
                if value != usize::MAX {
                    sparse.insert(id as u32, value);
                }
            }
            *self = Self::Sparse(sparse);
        }
        if let Self::Sparse(map) = self {
            map.insert(id, value);
        }
    }
}
