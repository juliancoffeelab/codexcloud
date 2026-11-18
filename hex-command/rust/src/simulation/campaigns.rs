//! Campaign definitions prepared once and shared by all trials and phase requests.
use super::{SimulationError, types::MatchConfig};
use crate::data::Catalogue;
use hex_command_battle::CampaignProgram;
use std::collections::HashMap;

/// Own changed definitions and borrow unchanged authored campaigns from the catalog.
/// Workers only borrow this table after workload preparation has finished.
pub struct CampaignDefinitions<'catalog> {
    rules: &'catalog Catalogue,
    definitions: HashMap<String, HashMap<String, CampaignProgram>>,
}
impl<'catalog> CampaignDefinitions<'catalog> {
    pub fn new(rules: &'catalog Catalogue) -> Self {
        Self {
            rules,
            definitions: HashMap::new(),
        }
    }
    pub fn prepare(&mut self, campaign: &str, route: &str) -> Result<(), SimulationError> {
        if self
            .definitions
            .get(campaign)
            .is_some_and(|routes| routes.contains_key(route))
        {
            return Ok(());
        }
        let definition = self.rules.campaign(campaign);
        let changes = if route == "authored" {
            None
        } else {
            Some(
                self.rules
                    .meta()
                    .simulation
                    .campaign
                    .campaigns
                    .get(campaign)
                    .and_then(|config| config.routes.get(route))
                    .ok_or_else(|| {
                        SimulationError::InvalidConfig(format!(
                            "unknown campaign route {campaign}/{route}"
                        ))
                    })?,
            )
        };
        let definition = CampaignProgram::compile(self.rules, definition, changes);
        self.definitions
            .entry(campaign.to_owned())
            .or_default()
            .insert(route.to_owned(), definition);
        Ok(())
    }
    pub fn prepare_match(&mut self, config: &MatchConfig) -> Result<(), SimulationError> {
        if let MatchConfig::Campaign(config) = config {
            self.prepare(&config.campaign, &config.objective)?;
        }
        Ok(())
    }
    pub fn get(&self, campaign: &str, route: &str) -> Result<&CampaignProgram, SimulationError> {
        self.definitions
            .get(campaign)
            .and_then(|routes| routes.get(route))
            .ok_or_else(|| {
                SimulationError::InvalidConfig(format!(
                    "campaign route was not prepared: {campaign}/{route}"
                ))
            })
    }
}
