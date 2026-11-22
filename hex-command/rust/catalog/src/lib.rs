//! Authored definitions and pure composition. Battle state and scope selection belong to battle.
mod catalogue;
mod composition;
#[cfg(feature = "simulation")]
pub use composition::diagnostics;
mod loader;
mod meta;
mod schema;
mod simulation;
#[cfg(feature = "wasm")]
pub mod wiki;
mod world;

pub use catalogue::{
    AbilityDefinition, AttackHandle, Catalogue, EffectHandle, ItemDefinition, RaceDefinition,
    TerrainDefinition, UnitDefinition,
};
pub use composition::{
    AttackDescription, AttackOutcomes, DamageOutcome, NeuralAttackFeatures, NeuralUnitFeatures,
    OwnedAssessment, ProtectionBreakdown, ResolvedSight, SelectedEffects, StrikeDistributionKey,
    UnitCapabilities, UnitInputs,
};
pub use meta::{CatalogueMeta, RecruiterWeights, RecruitmentWeights};
pub use world::{
    CampaignDefinition, ChapterDefinition, ConcealmentDefinition, EnvironmentDefinition,
    FullHealth, InitialArmy, InitialHealth, InitialRoster, InitialUnit, InitialVisionGroup,
    MapDefinition, MapTile, StageScripts, StageStory,
};
// Authored vocabulary and contribution values, not source records or prepared tables.
pub use schema::{
    AllegianceChange, BattlefieldLight, CampaignCarryover, Category, Character, CompanionArrival,
    ConcealmentChange, ConcealmentKind, ConcealmentMask, ConcealmentRemoval,
    ConcealmentRequirements, ConcealmentRule, ConcealmentVisual, Condition, ConditionRule,
    ConditionSide, ConditionalHint, ConditionalText, ContentError, Controller, DamageModifiers,
    DamageType, DeathSelection, DialogueBranch, EffectCombine, EffectScope, EffectValue,
    EnemyGroup, EnvironmentEffect, EnvironmentalModifiers, EventAction, EventIntent,
    EventObjective, FactionPolicy, FactionRecruit, FactionSetup, FactionTransfer, Family,
    FlagUpdate, GiveItem, GoldGrant, GroupOrder, HealingTargets, Hex, HexDirection, HoldSelection,
    Illumination, JsonBundle, LightEmission, LoopSelection, MapLighting, MarkLoop, MarkSelection,
    MarkSpec, OwnSelection, OwnerSelection, PerceptionArea, PerceptionEffect, PerceptionGrant,
    PerceptionRemoval, PlaceItem, PlayerMarker, ProximitySelection, Race, RecruitmentUnlock,
    Relation, Role, ScopedEffect, ScriptEvent, ScriptLine, ScriptSpawn, SightContribution,
    SightModifiers, SiteSelection, SpawnEntry, SpawnMember, Speech, StackingRule, StatModifiers,
    SurvivalSelection, Terrain, TerrainChange, TerrainReveal, TerrainRole, TerrainStatModifiers,
    Text, TextSlice, TextVariant, TileBinding, TileLight, TileRef, TimeOfDay, TurnPhase,
    TurnSelection, UnitLoop, UnitMatch, UnitReference, UnitRole, UnitTypeId, VeteranWeights,
    VisionGroupAssignment, When,
};

#[cfg(test)]
mod tests;

pub use simulation::{
    AdaptiveSamplingSettings, ArenaForces, ArenaScenario, ArenaSettings, ArenaVariant, ArmyRoster,
    CampaignSettings, CampaignSimulation, PretrainImitator, SimulationObjective,
    SimulationSettings, SimulationStageObjective, TrainingArena, TrainingArmies,
    TrainingCampaignVariant, TrainingSettings,
};

mod prepared_combat;
pub use prepared_combat::PreparedCombat;
