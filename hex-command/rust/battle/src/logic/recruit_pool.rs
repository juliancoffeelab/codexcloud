//! Borrowed pool selection centralizes ordering and fallback without allocating.
use crate::data::*;
#[derive(Clone)]
pub(crate) enum PoolEntries<'a> {
    Kinds(std::slice::Iter<'a, usize>),
    Names(std::slice::Iter<'a, String>),
    Recruits(std::slice::Iter<'a, FactionRecruit>),
}
#[derive(Clone)]
pub(crate) struct RecruitPool<'a> {
    pub rules: &'a Rules,
    pub entries: PoolEntries<'a>,
}
impl Iterator for RecruitPool<'_> {
    type Item = usize;
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.entries {
            PoolEntries::Kinds(v) => v.size_hint(),
            PoolEntries::Names(v) => v.size_hint(),
            PoolEntries::Recruits(v) => v.size_hint(),
        }
    }
    #[inline]
    fn next(&mut self) -> Option<usize> {
        match &mut self.entries {
            PoolEntries::Kinds(v) => v.next().copied(),
            PoolEntries::Names(v) => v.next().map(|id| self.rules.types[id]),
            PoolEntries::Recruits(v) => v.next().map(|r| self.rules.types[&r.unit]),
        }
    }
}

impl ExactSizeIterator for RecruitPool<'_> {}
