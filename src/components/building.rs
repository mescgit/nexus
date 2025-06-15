use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::game_state::{DevelopmentPhase, ResourceType, ServiceType, Tech, ZoneType}; // Assuming these are still in game_state or moved to a shared location

// --------------- Core Building Structs ---------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Building {
    pub id: String, // Consider using a UUID crate later if IDs need to be more robust
    pub variant: BuildingVariant,
    pub position: Option<(f32, f32)>, // (x, y) coordinates
    pub current_tier_index: usize,
    pub is_active: bool,
    pub assigned_workforce: u32,
    // pub current_hp: u32, // Future consideration: damage and repair
    // pub max_hp: u32,     // Future consideration
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BuildingVariant {
    Extractor {
        // Specific data for Extractors
        // Most data is likely in its tier, e.g., which resource it extracts.
        // For now, assuming ExtractorTier will define output resource and rate.
        available_tiers: Vec<ExtractorTier>,
    },
    BioDome {
        // Specific data for BioDomes
        // Tier will define nutrient paste output, workforce needs etc.
        available_tiers: Vec<BioDomeTier>,
    },
    PowerRelay {
        // Specific data for PowerRelays
        // Tier will define power generation
        available_tiers: Vec<PowerRelayTier>,
    },
    ResearchInstitute {
        // Specific data for ResearchInstitutes
        // Tier will define research point generation, workforce etc.
        available_tiers: Vec<ResearchInstituteTier>,
    },
    StorageSilo {
        // Specific data for StorageSilos
        // Tier will define storage capacity bonus
        available_tiers: Vec<StorageSiloTier>,
    },
    Habitation {
        // Specific data for Habitation
        current_inhabitants: u32, // This is stateful for an instance
        available_tiers: Vec<HabitationTier>,
    },
    Service {
        // Specific data for Service Buildings
        service_type: ServiceType,
        available_tiers: Vec<ServiceTier>,
    },
    Zone {
        // Specific data for Zones
        zone_type: ZoneType,
        available_tiers: Vec<ZoneTier>,
        // Zones might have less direct 'assigned_workforce' and more 'jobs_provided' by tier.
        // The main 'assigned_workforce' on Building could be 0 for zones,
        // or represent something else like construction/maintenance crew.
        // For now, specialist_jobs_provided is in ZoneTier.
    },
    Fabricator {
        // Specific data for Fabricators
        production_progress_secs: f32, // Stateful for an instance
        available_tiers: Vec<FabricatorTier>,
    },
    ProcessingPlant {
        // Specific data for ProcessingPlants
        processing_progress: f32, // Stateful for an instance
        available_tiers: Vec<ProcessingPlantTier>,
    },
    AdministrativeSpire {
        // Spire is unique, but can fit the model if we have one 'Building' instance for it
        available_tiers: Vec<AdministrativeSpireTier>,
    },
    LegacyStructure {
        // Legacy Structure is unique, but can fit the model
        available_tiers: Vec<LegacyStructureTier>,
    },
}

// --------------- Tier Definitions ---------------
// These now include common fields.

// --- Tier for currently simple buildings (can be expanded later) ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractorTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32,
    pub workforce_requirement: u32, // e.g., 5 for current extractors
    pub required_tech: Option<Tech>,
    // Specific to Extractor:
    pub resource_type: ResourceType, // The resource it extracts
    pub extraction_rate_per_sec: f32, // Units per second per fully staffed extractor
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BioDomeTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32,
    pub workforce_requirement: u32, // e.g., 10 for current biodomes
    pub required_tech: Option<Tech>,
    // Specific to BioDome:
    pub nutrient_paste_output_per_sec: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PowerRelayTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32, // Typically 0 for relays unless they have advanced features
    pub power_requirement: u32, // Could be non-zero for advanced relays (e.g. smart grid features)
    pub workforce_requirement: u32, // Typically 0
    pub required_tech: Option<Tech>,
    // Specific to PowerRelay:
    pub power_generation: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResearchInstituteTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32,
    pub workforce_requirement: u32, // e.g., 15 for current institutes
    pub required_tech: Option<Tech>,
    // Specific to ResearchInstitute:
    pub research_points_per_sec: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageSiloTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32, // Typically 0, unless smart silos
    pub workforce_requirement: u32, // Typically 0
    pub required_tech: Option<Tech>,
    // Specific to StorageSilo:
    pub storage_capacity_increase: u32, // For relevant resources
}


// --- Relocated and Adjusted Tier Structs ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HabitationTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32, // Habitation might have upkeep
    pub power_requirement: u32, // Habitation might consume power
    pub workforce_requirement: u32, // For maintenance, not the specialists it houses
    pub required_tech: Option<Tech>,
    // Specific to Habitation:
    pub housing_capacity: u32,
    pub specialist_slots: u32, // Slots for specialists to live/work, distinct from building's own workforce_requirement
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32, // Services likely consume power
    pub workforce_requirement: u32, // This is the specialist_requirement from old struct
    pub required_tech: Option<Tech>,
    // Specific to Service:
    pub service_capacity: u32,
    pub service_radius: f32,
    pub civic_index_contribution: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ZoneTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32, // Zones might consume some base power
    pub workforce_requirement: u32, // Represents specialist jobs provided
    pub required_tech: Option<Tech>,
    // Specific to Zone:
    // specialist_jobs_provided is now workforce_requirement in common fields
    pub civic_index_contribution: u32,
    pub income_generation: u32, // For commercial zones
    // LightIndustry zones might have output_resource/input_resource if we make them more complex
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FabricatorTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32,
    pub workforce_requirement: u32, // This is the specialist_requirement from old struct
    pub required_tech: Option<Tech>,
    // Specific to Fabricator:
    pub input_resources: HashMap<ResourceType, u32>,
    pub output_product: ResourceType,
    pub output_quantity: u32,
    pub production_time_secs: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessingPlantTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32,
    pub power_requirement: u32,
    pub workforce_requirement: u32, // This is the specialist_requirement from old struct
    pub required_tech: Option<Tech>,
    // Specific to ProcessingPlant:
    pub unlocks_resource: Option<ResourceType>, // Resource type it makes available for gathering globally
    pub input_resource: Option<(ResourceType, u32)>, // Type and amount per batch
    pub output_resource: Option<(ResourceType, u32)>,// Type and amount per batch
    pub processing_rate_per_sec: Option<f32>, // Batches per second
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdministrativeSpireTier {
    pub name: String,
    pub construction_credits_cost: u32, // Tier 0 has this
    pub upgrade_credits_cost: u32, // For Tiers > 0
    pub upkeep_cost: u32, // Spire should have upkeep
    pub power_requirement: u32,
    pub workforce_requirement: u32, // Spire might require some core staff
    pub required_tech: Option<Tech>, // Higher tiers might need tech
    // Specific to AdministrativeSpire:
    pub unlocks_phase: DevelopmentPhase,
    pub nutrient_paste_link_required: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LegacyStructureTier {
    pub name: String,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32, // Legacy structures might have upkeep
    pub power_requirement: u32, // Legacy structures might consume power
    pub workforce_requirement: u32, // Or maybe they are automated
    pub required_tech: Option<Tech>,
    // Specific to LegacyStructure:
    pub happiness_bonus: f32,
    pub income_bonus: f64,
}

// TODO: Need to define default tier lists for each building type,
// similar to get_habitation_tiers(), get_fabricator_tiers() etc.
// These will likely be static data, perhaps in a new `src/game_data/building_definitions.rs`
// or directly here for simpler cases. For this subtask, defining the structs is the priority.

// Helper for generating IDs - will eventually move to a utility module
// For now, keeping it simple. If already defined in game_state, this would be removed.
// use std::sync::atomic::{AtomicU32, Ordering};
// static NEXT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);
// pub fn generate_building_id() -> String {
//    format!("bldg_{}", NEXT_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
// }
