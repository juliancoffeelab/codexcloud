//! Static-backed catalog collections and deferred owned snapshots.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Cow, ops::Index};

/// Ordered immutable entries. Baked tables borrow a static sorted slice.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "bake", derive(databake::Bake))]
#[cfg_attr(feature = "bake", databake(path = hex_command_catalog::storage))]
pub struct Table<K: Clone + 'static, V: Clone + 'static>(pub Cow<'static, [(K, V)]>);
pub trait Lookup<K> {
    fn compare(&self, key: &K) -> std::cmp::Ordering;
}
impl<K: Ord> Lookup<K> for K {
    fn compare(&self, key: &K) -> std::cmp::Ordering {
        self.cmp(key)
    }
}
impl Lookup<Cow<'static, str>> for str {
    fn compare(&self, key: &Cow<'static, str>) -> std::cmp::Ordering {
        self.cmp(key.as_ref())
    }
}
impl Lookup<Cow<'static, str>> for String {
    fn compare(&self, key: &Cow<'static, str>) -> std::cmp::Ordering {
        self.as_str().cmp(key.as_ref())
    }
}
impl Lookup<(Cow<'static, str>, u32)> for (String, u32) {
    fn compare(&self, key: &(Cow<'static, str>, u32)) -> std::cmp::Ordering {
        self.0
            .as_str()
            .cmp(key.0.as_ref())
            .then_with(|| self.1.cmp(&key.1))
    }
}
impl<K: Clone + Ord, V: Clone> Table<K, V> {
    pub fn get<Q: Lookup<K> + ?Sized>(&self, key: &Q) -> Option<&V> {
        self.0
            .binary_search_by(|(k, _)| key.compare(k).reverse())
            .ok()
            .map(|i| &self.0[i].1)
    }
    pub fn contains_key<Q: Lookup<K> + ?Sized>(&self, key: &Q) -> bool {
        self.get(key).is_some()
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.0.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(i) => Some(std::mem::replace(&mut self.0.to_mut()[i].1, value)),
            Err(i) => {
                self.0.to_mut().insert(i, (key, value));
                None
            }
        }
    }
    pub fn remove<Q: Lookup<K> + ?Sized>(&mut self, key: &Q) -> Option<V> {
        let i = self
            .0
            .binary_search_by(|(k, _)| key.compare(k).reverse())
            .ok()?;
        Some(self.0.to_mut().remove(i).1)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.0.iter().map(|(k, v)| (k, v))
    }
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl<K: Clone + Ord, V: Clone> Default for Table<K, V> {
    fn default() -> Self {
        Self(Cow::Borrowed(&[]))
    }
}
impl<K: Clone + Ord, V: Clone> FromIterator<(K, V)> for Table<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        // BTreeMap also gives deserialization the usual last-value-wins map semantics.
        Self(Cow::Owned(
            iter.into_iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ))
    }
}
impl<K: Clone + Ord, V: Clone, Q: Lookup<K> + ?Sized> Index<&Q> for Table<K, V> {
    type Output = V;
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("catalog key")
    }
}
impl<K: Clone + Ord + Serialize, V: Clone + Serialize> Serialize for Table<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.iter())
    }
}
impl<'de, K: Clone + Ord + Deserialize<'de>, V: Clone + Deserialize<'de>> Deserialize<'de>
    for Table<K, V>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        std::collections::BTreeMap::<K, V>::deserialize(deserializer)
            .map(|m| m.into_iter().collect())
    }
}
/// Conversion between owned snapshots and immutable, bakeable templates.
pub trait Snapshot: Clone + 'static {
    type Template: Clone + std::fmt::Debug + Sync + 'static;
    fn freeze(&self) -> Self::Template;
    fn thaw(template: &Self::Template) -> Self;
}
/// Owned snapshots are materialized only when requested, never during catalog startup.
#[derive(Clone, Debug)]
pub enum Seed<T: Snapshot> {
    Owned(T),
    Baked(&'static T::Template),
}
impl<T: Snapshot> Seed<T> {
    pub fn instantiate(&self) -> T {
        match self {
            Self::Owned(value) => value.clone(),
            Self::Baked(template) => T::thaw(template),
        }
    }
}
#[cfg(feature = "bake")]
impl<T: Snapshot> databake::Bake for Seed<T>
where
    T::Template: databake::Bake,
{
    fn bake(&self, ctx: &databake::CrateEnv) -> databake::TokenStream {
        let Self::Owned(value) = self else {
            panic!("cannot rebake a static snapshot")
        };
        let template = value.freeze();
        let template = template.bake(ctx);
        databake::quote! { hex_command_catalog::storage::Seed::Baked(&#template) }
    }
}
