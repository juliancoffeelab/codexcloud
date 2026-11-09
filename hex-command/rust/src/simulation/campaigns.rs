//! Campaign definitions prepared once and shared by all trials and phase requests.
use super::{SimulationError, types::MatchConfig};
use crate::data::{CampaignRules, Rules};
use std::{borrow::Cow, collections::HashMap};

/// Own changed definitions and borrow unchanged authored campaigns from the catalog.
/// Workers only borrow this table after workload preparation has finished.
pub struct CampaignDefinitions<'catalog> {
    rules: &'catalog Rules,
    definitions: HashMap<String, HashMap<String, Cow<'catalog, CampaignRules>>>,
}
impl<'catalog> CampaignDefinitions<'catalog> {
    pub fn new(rules: &'catalog Rules) -> Self {
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
        let definition = self.rules.prepare_simulation_campaign(campaign, route)?;
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
    pub fn get(&self, campaign: &str, route: &str) -> Result<&CampaignRules, SimulationError> {
        self.definitions
            .get(campaign)
            .and_then(|routes| routes.get(route))
            .map(Cow::as_ref)
            .ok_or_else(|| {
                SimulationError::InvalidConfig(format!(
                    "campaign route was not prepared: {campaign}/{route}"
                ))
            })
    }
}
