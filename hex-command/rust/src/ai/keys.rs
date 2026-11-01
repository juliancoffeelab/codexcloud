//! Immutable planner labels share their storage across orders and snapshots.
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, ops::Deref, rc::Rc};
type Text = smallstr::SmallString<[u8; 64]>;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GoalKey(Rc<Text>);
#[derive(Default)]
struct Pool {
    slots: Vec<Rc<Text>>,
    cursor: usize,
}
thread_local! {static POOL:RefCell<Pool>=RefCell::new(Pool::default());}
pub(super) fn prepare(tiles: usize) {
    POOL.with_borrow_mut(|pool| {
        let count = 64 + tiles * 4;
        let mut free = pool
            .slots
            .iter()
            .filter(|slot| Rc::strong_count(slot) == 1)
            .count();
        pool.slots.reserve(count.saturating_sub(free));
        while free < count {
            pool.slots.push(Rc::new(Text::new()));
            free += 1;
        }
    });
}
impl GoalKey {
    pub(super) fn formatted(args: std::fmt::Arguments<'_>) -> Self {
        use std::fmt::Write;
        POOL.with_borrow_mut(|pool| {
            let count = pool.slots.len();
            let index = (0..count)
                .map(|offset| (pool.cursor + offset) % count)
                .find(|&i| Rc::strong_count(&pool.slots[i]) == 1)
                .unwrap_or_else(|| {
                    pool.slots.push(Rc::new(Text::new()));
                    count
                });
            pool.cursor = (index + 1) % pool.slots.len();
            let text = Rc::get_mut(&mut pool.slots[index]).expect("unleased planner label");
            text.clear();
            text.write_fmt(args).expect("formatting a planner label");
            Self(pool.slots[index].clone())
        })
    }
}
impl From<&str> for GoalKey {
    fn from(value: &str) -> Self {
        Self::formatted(format_args!("{value}"))
    }
}
impl From<String> for GoalKey {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}
impl Deref for GoalKey {
    type Target = Text;
    fn deref(&self) -> &Text {
        &self.0
    }
}
impl AsRef<str> for GoalKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::borrow::Borrow<str> for GoalKey {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for GoalKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_str(), f)
    }
}
