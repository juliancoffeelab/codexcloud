//! Borrowed pool selection centralizes ordering and fallback without allocating.
use crate::data::*;
#[derive(Clone)]
pub(crate) enum PoolEntries<'a> {
    Kinds(std::slice::Iter<'a, usize>),
    Names(std::slice::Iter<'a, String>),
    Recruits(std::slice::Iter<'a, FactionRecruit>),
}
#[derive(Clone)]
pub(crate) struct RecruitPool<'a, 'catalog> {
    pub definitions: &'a super::definition_index::DefinitionIndex<'catalog>,
    pub entries: PoolEntries<'a>,
}
impl Iterator for RecruitPool<'_, '_> {
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
            PoolEntries::Names(v) => v.next().map(|id| self.definitions.kind(id)),
            PoolEntries::Recruits(v) => v.next().map(|r| self.definitions.kind(&r.unit)),
        }
    }
}

impl ExactSizeIterator for RecruitPool<'_, '_> {}
