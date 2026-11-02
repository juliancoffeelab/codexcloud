//! Static storage for owned catalog models. Only the host compiler freezes data.
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    hash::Hash,
};
pub trait Freeze: Sized {
    type Frozen: Clone + Debug + Sync + 'static;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen;
    fn thaw(value: &Self::Frozen) -> Self;
}
macro_rules! scalar{($($t:ty),*)=>{$(impl Freeze for $t{type Frozen=Self;#[cfg(feature="bake")]fn freeze(&self)->Self{*self}fn thaw(value:&Self)->Self{*value}})*};}
scalar!(
    bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);
impl Freeze for String {
    type Frozen = Cow<'static, str>;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        Cow::Owned(self.clone())
    }
    fn thaw(value: &Self::Frozen) -> Self {
        value.to_string()
    }
}
impl<T: Freeze> Freeze for Vec<T> {
    type Frozen = Cow<'static, [T::Frozen]>;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        Cow::Owned(self.iter().map(T::freeze).collect())
    }
    fn thaw(value: &Self::Frozen) -> Self {
        value.iter().map(T::thaw).collect()
    }
}
impl<T: Freeze> Freeze for Option<T> {
    type Frozen = Option<T::Frozen>;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        self.as_ref().map(T::freeze)
    }
    fn thaw(value: &Self::Frozen) -> Self {
        value.as_ref().map(T::thaw)
    }
}
#[derive(Clone, Debug)]
pub struct FrozenArray<T, const N: usize>(pub [T; N]);
#[cfg(feature = "bake")]
impl<T: databake::Bake, const N: usize> databake::Bake for FrozenArray<T, N> {
    fn bake(&self, ctx: &databake::CrateEnv) -> databake::TokenStream {
        let items = self.0.iter().map(|v| v.bake(ctx));
        databake::quote! {hex_command_catalog::frozen::FrozenArray([#(#items,)*])}
    }
}
impl<T: Freeze, const N: usize> Freeze for [T; N] {
    type Frozen = FrozenArray<T::Frozen, N>;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        FrozenArray(std::array::from_fn(|i| self[i].freeze()))
    }
    fn thaw(value: &Self::Frozen) -> Self {
        std::array::from_fn(|i| T::thaw(&value.0[i]))
    }
}
impl<A: Freeze, B: Freeze> Freeze for (A, B) {
    type Frozen = (A::Frozen, B::Frozen);
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        (self.0.freeze(), self.1.freeze())
    }
    fn thaw(value: &Self::Frozen) -> Self {
        (A::thaw(&value.0), B::thaw(&value.1))
    }
}
impl<K: Freeze + Eq + Hash, V: Freeze> Freeze for HashMap<K, V> {
    type Frozen = Cow<'static, [(K::Frozen, V::Frozen)]>;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        let mut entries: Vec<_> = self.iter().map(|(k, v)| (k.freeze(), v.freeze())).collect();
        entries.sort_by_cached_key(|(k, _)| format!("{k:?}"));
        Cow::Owned(entries)
    }
    fn thaw(value: &Self::Frozen) -> Self {
        value
            .iter()
            .map(|(k, v)| (K::thaw(k), V::thaw(v)))
            .collect()
    }
}
impl<K: Freeze + Ord, V: Freeze> Freeze for BTreeMap<K, V> {
    type Frozen = Cow<'static, [(K::Frozen, V::Frozen)]>;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        Cow::Owned(self.iter().map(|(k, v)| (k.freeze(), v.freeze())).collect())
    }
    fn thaw(value: &Self::Frozen) -> Self {
        value
            .iter()
            .map(|(k, v)| (K::thaw(k), V::thaw(v)))
            .collect()
    }
}

// Immutable authored subtrees keep their wire shape while snapshots share ownership.
impl<T: Freeze> Freeze for std::sync::Arc<T> {
    type Frozen = T::Frozen;
    #[cfg(feature = "bake")]
    fn freeze(&self) -> Self::Frozen {
        T::freeze(self)
    }
    fn thaw(value: &Self::Frozen) -> Self {
        std::sync::Arc::new(T::thaw(value))
    }
}
