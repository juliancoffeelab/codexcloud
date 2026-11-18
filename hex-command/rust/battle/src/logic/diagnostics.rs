//! Map Catalogue diagnostic occurrences back to runtime sources without leaking IDs into Catalogue.
use super::BattlePosition;
use crate::data::*;

pub(super) struct SupportDiagnostic {
    pub amount: i32,
    sources: Vec<u32>,
}
impl SupportDiagnostic {
    pub(super) fn source_names(&self, exclude: Option<u32>) -> Vec<Text> {
        let mut names: Vec<_> = self
            .sources
            .iter()
            .filter(|&&id| Some(id) != exclude)
            .map(|id| format!("u{id}"))
            .collect();
        names.sort_unstable();
        names
    }
}
impl BattlePosition<'_> {
    pub(super) fn support_diagnostic(
        &self,
        target: &Unit,
        mut select: impl FnMut(Option<&Unit>, EffectHandle<'_>) -> bool,
        query: impl FnOnce(&UnitCapabilities<'_>) -> hex_command_catalog::diagnostics::EffectTotal,
    ) -> SupportDiagnostic {
        let mut occurrences = Vec::new();
        let calculation = self.with_selected_capabilities_at(
            target,
            target.tile,
            &self.units,
            &[],
            |source, _, effect| {
                if !select(source, effect) {
                    return false;
                }
                occurrences.push(source.map(|unit| unit.id));
                true
            },
            query,
        );
        let mut sources: Vec<_> = calculation
            .contributors
            .into_iter()
            .filter_map(|index| occurrences[index])
            .collect();
        sources.sort_unstable();
        sources.dedup();
        SupportDiagnostic {
            amount: calculation.amount,
            sources,
        }
    }
}
