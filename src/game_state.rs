// src/game_state.rs

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};

// --- Data Structs ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DevelopmentPhase {
    DP1,
    DP2,
    DP3,
}

impl Default for DevelopmentPhase {
    fn default() -> Self {
        DevelopmentPhase::DP1
    }
}

pub fn link_spire_to_hub(game_state: &mut GameState) {
    if let Some(spire) = &mut game_state.administrative_spire {
        // In a more detailed implementation, we would check if an OperationsHub exists
        // and has Nutrient Paste production/storage.
        // For now, simply toggle the flag.
        spire.is_linked_to_hub = true;
        println!("Administrative Spire linked to Operations Hub.");
    } else {
        println!("Administrative Spire has not been constructed yet.");
    }
}

// --- Administrative Spire Logic ---
// (This section might be better placed after GameState or in its own module if it grows larger)

pub fn construct_administrative_spire(game_state: &mut GameState) {
    if game_state.administrative_spire.is_none() {
        // Tiers should be defined consistently, perhaps loaded from a config or a dedicated function.
        // For now, ensure all fields are present.
        let all_tiers = vec![
            AdministrativeSpireTier { name: "Command Post".to_string(), power_requirement: 10, unlocks_phase: DevelopmentPhase::DP1, nutrient_paste_link_required: false, construction_credits_cost: 1000, upgrade_credits_cost: 0 },
            AdministrativeSpireTier { name: "Integrated Command".to_string(), power_requirement: 25, unlocks_phase: DevelopmentPhase::DP2, nutrient_paste_link_required: true, construction_credits_cost: 0, upgrade_credits_cost: 2500 },
            AdministrativeSpireTier { name: "Planetary Nexus".to_string(), power_requirement: 50, unlocks_phase: DevelopmentPhase::DP3, nutrient_paste_link_required: true, construction_credits_cost: 0, upgrade_credits_cost: 5000 },
        ];
        
        let initial_tier_def = &all_tiers[0];

        if game_state.credits < initial_tier_def.construction_credits_cost as f64 {
            println!("Not enough credits to construct Administrative Spire. Required: {}, Available: {:.2}", initial_tier_def.construction_credits_cost, game_state.credits);
            return;
        }
        game_state.credits -= initial_tier_def.construction_credits_cost as f64;
        println!("Constructed Administrative Spire for {} credits. Remaining credits: {:.2}", initial_tier_def.construction_credits_cost, game_state.credits);

        let spire = AdministrativeSpire {
            current_tier_index: 0,
            available_tiers: all_tiers,
            is_linked_to_hub: false,
        };
        game_state.administrative_spire = Some(spire);
        // Note: Resource costs (non-credit) for construction will be handled by a separate building system if any.
    }
}

pub fn upgrade_administrative_spire(game_state: &mut GameState) {
    if let Some(spire) = &mut game_state.administrative_spire {
        if spire.current_tier_index < spire.available_tiers.len() - 1 {
            let next_tier_index = spire.current_tier_index + 1;
            let next_tier_info = &spire.available_tiers[next_tier_index];
            let current_tier_info = &spire.available_tiers[spire.current_tier_index];

            // Check credit cost for upgrading
            if game_state.credits < next_tier_info.upgrade_credits_cost as f64 {
                println!("Not enough credits to upgrade Administrative Spire to {}. Required: {}, Available: {:.2}", next_tier_info.name, next_tier_info.upgrade_credits_cost, game_state.credits);
                return;
            }

            // Check power requirement
            // Available power = total generated - total consumed by *other* buildings + current spire consumption (if any)
            // The check is whether this available power can cover the *new* tier's requirement.
            // Effectively: total_generated_power - (total_consumed_power - current_tier_power) >= next_tier_power
            
            let current_spire_consumption = current_tier_info.power_requirement;
            // total_consumed_power already includes current_spire_consumption if spire is built.
            let power_consumed_by_others = game_state.total_consumed_power - current_spire_consumption as f32;
            let available_power_for_spire_upgrade = game_state.total_generated_power - power_consumed_by_others;
            
            if available_power_for_spire_upgrade < next_tier_info.power_requirement as f32 {
                println!(
                    "Not enough power to upgrade Administrative Spire to {}. Required: {}, Effectively Available for Spire: {:.2} (Generated: {:.2}, Others Consume: {:.2})",
                    next_tier_info.name,
                    next_tier_info.power_requirement,
                    available_power_for_spire_upgrade,
                    game_state.total_generated_power,
                    power_consumed_by_others
                );
                return;
            }

            // Check Nutrient Paste link requirement for the new tier
            if next_tier_info.nutrient_paste_link_required && !spire.is_linked_to_hub {
                println!("Administrative Spire upgrade to {} requires Nutrient Paste link to Operations Hub, but it is not linked.", next_tier_info.name);
                return;
            }
            
            game_state.credits -= next_tier_info.upgrade_credits_cost as f64;
            println!("Upgraded Administrative Spire to {} for {} credits. Remaining credits: {:.2}", next_tier_info.name, next_tier_info.upgrade_credits_cost, game_state.credits);

            spire.current_tier_index = next_tier_index;
            game_state.current_development_phase = next_tier_info.unlocks_phase;
            
            // Power consumption will be updated automatically by game_tick_system in the next tick.
            println!("Administrative Spire upgraded to: {}. Development Phase unlocked: {:?}", next_tier_info.name, next_tier_info.unlocks_phase);

        } else {
            println!("Administrative Spire is already at its maximum tier.");
        }
    } else {
        println!("Administrative Spire has not been constructed yet.");
    }
}

pub struct AdministrativeSpireTier {
    pub name: String,
    pub power_requirement: u32,
    pub unlocks_phase: DevelopmentPhase,
    pub nutrient_paste_link_required: bool,
    pub construction_credits_cost: u32,
    pub upgrade_credits_cost: u32,
}

pub struct AdministrativeSpire {
    pub current_tier_index: usize,
    pub available_tiers: Vec<AdministrativeSpireTier>,
    pub is_linked_to_hub: bool,
}

// --- Habitation Data Structures ---

pub struct HabitationStructureTier {
    pub name: String,
    pub housing_capacity: u32,
    pub specialist_slots: u32,
    pub construction_credits_cost: u32, 
}

pub struct HabitationStructure {
    pub id: String, // Unique identifier
    pub tier_index: usize,
    pub available_tiers: Vec<HabitationStructureTier>,
    pub current_inhabitants: u32,
    pub assigned_specialists: u32,
}

// --- Service Building Data Structures ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceType {
    Wellness,
    Security,
    Education,
    Recreation,
    Spiritual,
}

pub struct ServiceBuildingTier {
    pub name: String,
    pub specialist_requirement: u32,
    pub influence_radius: f32, 
    pub upkeep_cost: u32,      
    pub civic_index_contribution: u32,
    pub construction_credits_cost: u32,
}

pub struct ServiceBuilding {
    pub id: String,
    pub service_type: ServiceType,
    pub current_tier_index: usize,
    pub available_tiers: Vec<ServiceBuildingTier>,
    pub assigned_specialists: u32,
    pub is_active: bool,
    pub position: Option<(f32, f32)>, // For spatial simulation if used
}

// --- Zone Data Structures ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZoneType {
    Commercial,
    LightIndustry,
}

pub struct ZoneTier {
    pub name: String,
    pub specialist_jobs_provided: u32,
    pub civic_index_contribution: u32,
    pub upkeep_cost: u32, 
    pub construction_credits_cost: u32,
    pub income_generation: u32,
}

pub struct Zone {
    pub id: String,
    pub zone_type: ZoneType,
    pub current_tier_index: usize,
    pub available_tiers: Vec<ZoneTier>,
    pub assigned_specialists: u32,
    pub is_active: bool,
}


// --- Processing Plant Logic ---

pub fn get_processing_plant_tiers() -> Vec<ProcessingPlantTier> {
    vec![
        ProcessingPlantTier {
            name: "Xylos Purifier".to_string(),
            unlocks_resource: Some(ResourceType::RawXylos), // This plant allows RawXylos to be gathered
            input_resource: Some((ResourceType::RawXylos, 2)), // Consumes 2 RawXylos
            output_resource: Some((ResourceType::RefinedXylos, 1)), // Produces 1 RefinedXylos
            processing_rate_per_sec: Some(0.5), // Processes 0.5 batches (2 input -> 1 output) per second
            power_requirement: 20,
            specialist_requirement: 2,
            construction_credits_cost: 150, 
            upkeep_cost: 10,                
        },
        ProcessingPlantTier {
            name: "Quantium Resonator".to_string(),
            unlocks_resource: Some(ResourceType::RawQuantium),
            input_resource: None, // This one might just unlock, not process directly
            output_resource: None,
            processing_rate_per_sec: None,
            power_requirement: 25,
            specialist_requirement: 3,
            construction_credits_cost: 200,
            upkeep_cost: 15,
        },
         ProcessingPlantTier {
            name: "Advanced Material Synthesizer".to_string(),
            unlocks_resource: None, // Doesn't unlock new raw types
            input_resource: Some((ResourceType::CuprumDeposits, 3)), 
            output_resource: Some((ResourceType::ProcessedQuantium, 1)), // Example output
            processing_rate_per_sec: Some(0.2), 
            power_requirement: 40,
            specialist_requirement: 4,
            construction_credits_cost: 300,
            upkeep_cost: 20,
        },
    ]
}

pub fn add_processing_plant(game_state: &mut GameState, tier_index: usize) {
    let all_tiers = get_processing_plant_tiers();
    if tier_index >= all_tiers.len() {
        println!("Error: Invalid tier index for processing plant.");
        return;
    }
    let tier_info = &all_tiers[tier_index];
    // Credit Check for Construction
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        println!("Not enough credits to build {}. Required: {}, Available: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    println!("Built {} for {} credits. Remaining credits: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);

    let new_plant = ProcessingPlantData {
        id: generate_unique_id(),
        tier_index,
        available_tiers: all_tiers.clone(), 
        assigned_specialists: 0,
        is_active: false, 
        processing_progress: 0.0,
    };
    
    // Initial resource unlock check, also using the correct tier_info
    if let Some(unlocked_res) = tier_info.unlocks_resource { 
        game_state.unlocked_raw_materials.insert(unlocked_res);
        println!("Processing Plant {} (type: {}) unlocked gathering of {:?}", new_plant.id, tier_info.name, unlocked_res);
    }
    game_state.processing_plants.push(new_plant);
    println!("Added Processing Plant: {}", tier_info.name); // Using tier_info.name for consistency
}

pub fn upgrade_processing_plant(game_state: &mut GameState, plant_id: &str) {
    if let Some(plant) = game_state.processing_plants.iter_mut().find(|p| p.id == plant_id) {
        if plant.tier_index < plant.available_tiers.len() - 1 {
            let next_tier_index = plant.tier_index + 1;
            let next_tier_info = &plant.available_tiers[next_tier_index];
            
            let upgrade_cost = next_tier_info.construction_credits_cost; // Assuming upgrade cost is construction cost of next tier

            if game_state.credits < upgrade_cost as f64 {
                println!("Not enough credits to upgrade Processing Plant {} to {}. Required: {}, Available: {:.2}", plant_id, next_tier_info.name, upgrade_cost, game_state.credits);
                return;
            }
            game_state.credits -= upgrade_cost as f64;
            println!("Upgraded Processing Plant {} to {} for {} credits. Remaining credits: {:.2}", plant_id, next_tier_info.name, upgrade_cost, game_state.credits);
            
            plant.tier_index = next_tier_index;
            plant.processing_progress = 0.0;

            if plant.assigned_specialists > next_tier_info.specialist_requirement {
                let to_unassign = plant.assigned_specialists - next_tier_info.specialist_requirement;
                plant.assigned_specialists -= to_unassign;
                game_state.assigned_specialists_total -= to_unassign;
            }
            
            if let Some(unlocked_res) = next_tier_info.unlocks_resource {
                if game_state.unlocked_raw_materials.insert(unlocked_res) { 
                    println!("Processing Plant {} upgrade unlocked gathering of {:?}", plant.id, unlocked_res);
                }
            }
            println!("Upgraded Processing Plant {} to {}", plant_id, next_tier_info.name);
        } else {
            println!("Processing Plant {} is already at max tier.", plant_id);
        }
    } else {
        println!("Processing Plant with ID {} not found.", plant_id);
    }
}

pub fn remove_processing_plant(game_state: &mut GameState, plant_id: &str) {
    if let Some(index) = game_state.processing_plants.iter().position(|p| p.id == plant_id) {
        let removed_plant = game_state.processing_plants.remove(index);
        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(removed_plant.assigned_specialists);
        // Note: Unlocking resources is permanent for now, even if plant is removed. Could be complex to revert.
        println!("Removed Processing Plant with ID {}", plant_id);
    } else {
        println!("Processing Plant with ID {} not found for removal.", plant_id);
    }
}

pub fn assign_specialists_to_processing_plant(game_state: &mut GameState, plant_id: &str, num_to_assign: u32) {
    if let Some(plant) = game_state.processing_plants.iter_mut().find(|p| p.id == plant_id) {
        if let Some(tier) = plant.available_tiers.get(plant.tier_index) {
            let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
            if available_general_inhabitants < num_to_assign {
                println!("Not enough unassigned inhabitants for Processing Plant {}.", plant_id); return;
            }
            if plant.assigned_specialists + num_to_assign > tier.specialist_requirement {
                println!("Cannot assign more specialists than required for Processing Plant {}. Max: {}", plant_id, tier.specialist_requirement); return;
            }
            plant.assigned_specialists += num_to_assign;
            game_state.assigned_specialists_total += num_to_assign;
            println!("Assigned {} specialists to Processing Plant {}.", num_to_assign, plant_id);
        }
    } else {
        println!("Processing Plant {} not found.", plant_id);
    }
}

pub fn unassign_specialists_from_processing_plant(game_state: &mut GameState, plant_id: &str, num_to_unassign: u32) {
     if let Some(plant) = game_state.processing_plants.iter_mut().find(|p| p.id == plant_id) {
        let actual_unassign = num_to_unassign.min(plant.assigned_specialists);
        plant.assigned_specialists -= actual_unassign;
        game_state.assigned_specialists_total -= actual_unassign;
        plant.is_active = false; // Deactivate, will be checked next tick
        plant.processing_progress = 0.0;
        println!("Unassigned {} specialists from Processing Plant {}.", actual_unassign, plant_id);
    } else {
        println!("Processing Plant {} not found.", plant_id);
    }
}

pub fn processing_plant_operations_system(game_state: &mut GameState, time_delta_secs: f32) {
    for plant in game_state.processing_plants.iter_mut() {
        if let Some(tier) = plant.available_tiers.get(plant.tier_index) {
            let has_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0) >= tier.power_requirement as f32;
            let has_specialists = plant.assigned_specialists >= tier.specialist_requirement;
            
            plant.is_active = has_power && has_specialists;

            if !plant.is_active {
                plant.processing_progress = 0.0; // Reset progress if inactive
                continue;
            }

            // Resource Unlocking (already handled in add/upgrade, but could be double-checked here if needed)
            // if let Some(unlocked_res) = tier.unlocks_resource {
            //     game_state.unlocked_raw_materials.insert(unlocked_res);
            // }

            // Resource Conversion
            if let (Some((input_type, input_amount_per_batch)), Some((output_type, output_amount_per_batch)), Some(rate)) = 
                (tier.input_resource, tier.output_resource, tier.processing_rate_per_sec) {
                
                // Continuous processing based on rate
                let potential_batches_this_tick = rate * time_delta_secs;
                plant.processing_progress += potential_batches_this_tick;

                if plant.processing_progress >= 1.0 { // At least one full batch can be processed
                    let num_batches_to_process = plant.processing_progress.floor();
                    let total_input_needed = input_amount_per_batch as f32 * num_batches_to_process;
                    let current_input_available = *game_state.current_resources.get(&input_type).unwrap_or(&0.0);

                    if current_input_available >= total_input_needed {
                        // Consume input
                        *game_state.current_resources.entry(input_type).or_insert(0.0) -= total_input_needed;
                        // Produce output
                        let total_output_produced = output_amount_per_batch as f32 * num_batches_to_process;
                        *game_state.current_resources.entry(output_type).or_insert(0.0) += total_output_produced;
                        
                        plant.processing_progress -= num_batches_to_process; // Reduce progress by processed batches
                        
                        println!("Processing Plant {} processed {}x{:.0} {:?} into {:.0} {:?}. Input Left: {:.1}, Output Total: {:.1}", 
                            plant.id, num_batches_to_process, input_amount_per_batch, input_type, output_amount_per_batch, output_type,
                            game_state.current_resources.get(&input_type).unwrap_or(&0.0),
                            game_state.current_resources.get(&output_type).unwrap_or(&0.0)
                        );
                    } else {
                        // Not enough input for the accumulated progress, pause or reset progress partially
                        // For simplicity, let's just pause and wait for more input.
                        // plant.processing_progress = 0.0; // Or some partial value if preferred
                    }
                }
            }
        }
    }
}


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ResourceType {
    FerrocreteOre,
    NutrientPaste,
    CuprumDeposits,
    Power,
    // New resources for Fabricators & Processing Plants
    ManufacturedGoods,    // Output of Fabricator
    AdvancedComponents,   // Output of Fabricator
    RefinedXylos,         // Example processed material
    ProcessedQuantium,    // Example processed material
    RawXylos,             // Example raw material unlocked by Processing Plant
    RawQuantium,          // Example raw material unlocked by Processing Plant
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    // Operations Hub
    Extractor,
    BioDome,
    PowerRelay,
    StorageSilo,
    ResearchInstitute,
    Fabricator, // Added Fabricator
    ProcessingPlant, // Added ProcessingPlant
    // Habitation Sector
    BasicDwelling,
    WellnessPost,
    SecurityStation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tech {
    BasicConstructionProtocols,
    EfficientExtraction,
}

// --- Building Components ---
#[derive(Component)]
pub struct Extractor { pub power_consumption: u32, pub resource_type: ResourceType, pub extraction_rate: f32 } // Renamed power_required
#[derive(Component)]
pub struct BioDome { pub power_consumption: u32, pub production_rate: f32 } // Renamed power_required
#[derive(Component)]
pub struct PowerRelay { pub power_output: u32 }
#[derive(Component)]
pub struct StorageSilo { pub capacity: u32 }
#[derive(Component)]
pub struct ResearchInstitute { pub power_consumption: u32 } // Added power_consumption
#[derive(Component)]
pub struct BasicDwelling { pub housing_capacity: u32 }
#[derive(Component)]
pub struct WellnessPost { pub health_capacity: u32, pub jobs_provided: u32 }
#[derive(Component)]
pub struct SecurityStation { pub police_capacity: u32, pub jobs_provided: u32 }

#[derive(Component)]
pub struct Fabricator {
    pub power_consumption: u32,
    pub input_resource: ResourceType,
    pub input_amount: f32,
    pub output_resource: ResourceType,
    pub output_amount: f32,
    pub conversion_rate: f32, // Assuming 1.0 for now, effectively cycles per tick
}

#[derive(Component)]
pub struct ProcessingPlant {
    pub power_consumption: u32,
    // Fields for input/output/rate can be added later when mechanics are defined
}


// --- Game State Resources ---

// FIXED: Removed 'Default' from the derive macro to resolve the conflict.
#[derive(Resource)]
pub struct GameState {
    pub administrative_spire: Option<AdministrativeSpire>,
    pub current_development_phase: DevelopmentPhase,
    pub current_resources: HashMap<ResourceType, f32>,
    pub building_costs: HashMap<BuildingType, HashMap<ResourceType, f32>>,
    pub unlocked_techs: HashSet<Tech>,
    pub research_progress: Option<(Tech, f32)>,
    pub tech_costs: HashMap<Tech, u32>, // Changed to u32 for Credit costs
    // Habitation and Population
    pub habitation_structures: Vec<HabitationStructure>,
    pub total_inhabitants: u32,
    pub available_housing_capacity: u32,
    pub total_specialist_slots: u32, // From Habitation + Zones + Services that *provide* slots (not require)
    pub assigned_specialists_total: u32,
    // Services and Zones
    pub service_buildings: Vec<ServiceBuilding>,
    pub zones: Vec<Zone>,
    pub civic_index: u32,
    // Happiness
    pub colony_happiness: f32,
    pub legacy_structure_happiness_bonus: f32,
    pub simulated_has_sufficient_nutrient_paste: bool, // Placeholder
    // Fabricators & Processing Plants
    pub fabricators: Vec<FabricatorData>, 
    pub processing_plants: Vec<ProcessingPlantData>, 
    pub unlocked_raw_materials: HashSet<ResourceType>,
    // Economy
    pub credits: f64,
    pub legacy_structure_income_bonus: f64,
    // Power grid totals
    pub total_generated_power: f32,
    pub total_consumed_power: f32,
}

// --- Fabricator Data Structures ---

pub struct FabricatorTier {
    pub name: String,
    pub input_resources: HashMap<ResourceType, u32>,
    pub output_product: ResourceType,
    pub output_quantity: u32,
    pub production_time_secs: f32,
    pub power_requirement: u32,
    pub specialist_requirement: u32,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32, 
}

// Renamed from Fabricator to FabricatorData to avoid conflict with Bevy Component
pub struct FabricatorData { 
    pub id: String,
    pub tier_index: usize,
    pub available_tiers: Vec<FabricatorTier>,
    pub assigned_specialists: u32,
    pub is_active: bool,
    pub production_progress_secs: f32,
}

// --- Processing Plant Data Structures ---

pub struct ProcessingPlantTier {
    pub name: String,
    pub unlocks_resource: Option<ResourceType>,
    pub input_resource: Option<(ResourceType, u32)>,  // Type and amount per second/batch
    pub output_resource: Option<(ResourceType, u32)>, // Type and amount per second/batch
    pub processing_rate_per_sec: Option<f32>, 
    pub power_requirement: u32,
    pub specialist_requirement: u32,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32, 
}

// Renamed from ProcessingPlant to ProcessingPlantData
pub struct ProcessingPlantData {
    pub id: String,
    pub tier_index: usize,
    pub available_tiers: Vec<ProcessingPlantTier>,
    pub assigned_specialists: u32,
    pub is_active: bool,
    pub processing_progress: f32, // For batch processing if not continuous
}


// Holds the current calculated stats for the entire colony.
#[derive(Resource, Default, Clone, Copy)]
pub struct ColonyStats {
    pub total_housing: u32,
    pub total_jobs: u32,
    pub health_capacity: u32,
    pub police_capacity: u32,
    pub happiness: f32, 
    pub credits: f64, // Added credits field
}

// Stores a history of stats for graphing.
#[derive(Resource, Default)]
pub struct GraphData {
    pub history: VecDeque<ColonyStats>, // Consider adding happiness to graph history
}

// This manual implementation is now the only one, which is correct.
impl Default for GameState {
    fn default() -> Self {
        let mut building_costs = HashMap::new();
        building_costs.insert(BuildingType::Extractor, HashMap::from([(ResourceType::FerrocreteOre, 75.0)]));
        building_costs.insert(BuildingType::BioDome, HashMap::from([(ResourceType::FerrocreteOre, 50.0)]));
        building_costs.insert(BuildingType::PowerRelay, HashMap::from([(ResourceType::FerrocreteOre, 60.0)]));
        building_costs.insert(BuildingType::StorageSilo, HashMap::from([(ResourceType::FerrocreteOre, 100.0)]));
        building_costs.insert(BuildingType::ResearchInstitute, HashMap::from([(ResourceType::FerrocreteOre, 150.0)]));
        building_costs.insert(BuildingType::Fabricator, HashMap::from([(ResourceType::FerrocreteOre, 200.0)]));
        building_costs.insert(BuildingType::ProcessingPlant, HashMap::from([(ResourceType::FerrocreteOre, 180.0)]));
        building_costs.insert(BuildingType::BasicDwelling, HashMap::from([(ResourceType::FerrocreteOre, 100.0)]));
        building_costs.insert(BuildingType::WellnessPost, HashMap::from([(ResourceType::FerrocreteOre, 120.0)]));
        building_costs.insert(BuildingType::SecurityStation, HashMap::from([(ResourceType::FerrocreteOre, 120.0)]));
        
        let mut tech_costs = HashMap::new();
        // Updated to u32 Credit costs
        tech_costs.insert(Tech::BasicConstructionProtocols, 100_u32); 
        tech_costs.insert(Tech::EfficientExtraction, 250_u32);

        let mut current_resources = HashMap::new();
        current_resources.insert(ResourceType::NutrientPaste, 50.0);
        current_resources.insert(ResourceType::FerrocreteOre, 200.0);
        current_resources.insert(ResourceType::CuprumDeposits, 50.0);
        current_resources.insert(ResourceType::Power, 100.0);
        // Initialize new resources
        current_resources.insert(ResourceType::ManufacturedGoods, 0.0);
        current_resources.insert(ResourceType::AdvancedComponents, 0.0);
        current_resources.insert(ResourceType::RefinedXylos, 0.0);
        current_resources.insert(ResourceType::ProcessedQuantium, 0.0);
        current_resources.insert(ResourceType::RawXylos, 0.0); // Typically starts at 0 until unlocked
        current_resources.insert(ResourceType::RawQuantium, 0.0);


        Self {
            administrative_spire: None,
            current_development_phase: DevelopmentPhase::default(),
            current_resources,
            building_costs,
            unlocked_techs: HashSet::new(),
            research_progress: None,
            tech_costs,
            // Habitation and Population
            habitation_structures: Vec::new(),
            total_inhabitants: 5, // Start with a few inhabitants
            available_housing_capacity: 0,
            total_specialist_slots: 0, // Will be updated by structures
            assigned_specialists_total: 0,
            // Services and Zones
            service_buildings: Vec::new(),
            zones: Vec::new(),
            civic_index: 0, // Starting civic index
            // Happiness
            colony_happiness: 50.0, // Start at a neutral base
            legacy_structure_happiness_bonus: 0.0,
            simulated_has_sufficient_nutrient_paste: true, // Assume true for now
            // Fabricators & Processing Plants
            fabricators: Vec::new(),
            processing_plants: Vec::new(),
            unlocked_raw_materials: HashSet::new(),
            // Economy
            credits: 10000.0, // Starting credits
            legacy_structure_income_bonus: 0.0,
            // Power grid totals
            total_generated_power: 0.0,
            total_consumed_power: 0.0,
        }
    }
}

// --- Happiness Logic ---

// ... (calculate_colony_happiness function remains here)


// --- Fabricator Logic ---

pub fn get_fabricator_tiers() -> Vec<FabricatorTier> {
    vec![
        FabricatorTier {
            name: "Basic Fabricator".to_string(),
            input_resources: HashMap::from([
                (ResourceType::FerrocreteOre, 2),
                (ResourceType::CuprumDeposits, 1),
            ]),
            output_product: ResourceType::ManufacturedGoods,
            output_quantity: 1,
            production_time_secs: 10.0,
            power_requirement: 15,
            specialist_requirement: 1,
            construction_credits_cost: 200,
            upkeep_cost: 10,
        },
        FabricatorTier {
            name: "Advanced Fabricator".to_string(),
            input_resources: HashMap::from([
                (ResourceType::ManufacturedGoods, 2),
                (ResourceType::RefinedXylos, 1), 
            ]),
            output_product: ResourceType::AdvancedComponents,
            output_quantity: 1,
            production_time_secs: 20.0,
            power_requirement: 30,
            specialist_requirement: 2,
            construction_credits_cost: 500,
            upkeep_cost: 25,
        },
    ]
}

// Modified signature and logic for check_fabricator_inputs
fn check_fabricator_inputs(
    current_resources: &HashMap<ResourceType, f32>,
    fabricator_tier: &FabricatorTier 
) -> bool {
    for (resource_type, required_amount) in &fabricator_tier.input_resources {
        if current_resources.get(resource_type).unwrap_or(&0.0) < &(*required_amount as f32) {
            return false; // Not enough of this input resource
        }
    }
    true // All inputs are sufficient
}


pub fn add_fabricator(game_state: &mut GameState, tier_index: usize) {
    let all_tiers = get_fabricator_tiers();
    if tier_index >= all_tiers.len() {
        println!("Error: Invalid tier index for fabricator.");
        return;
    }
    let tier_info = &all_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        println!("Not enough credits to build {}. Required: {}, Available: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    println!("Built {} for {} credits. Remaining credits: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);

    let new_fabricator = FabricatorData {
        id: generate_unique_id(),
        tier_index,
        available_tiers: all_tiers.clone(),
        assigned_specialists: 0,
        is_active: false, 
        production_progress_secs: 0.0,
    };
    game_state.fabricators.push(new_fabricator);
    println!("Added Fabricator: {}", tier_info.name);
}

pub fn upgrade_fabricator(game_state: &mut GameState, fabricator_id: &str) {
    if let Some(fab) = game_state.fabricators.iter_mut().find(|f| f.id == fabricator_id) {
        if fab.tier_index < fab.available_tiers.len() - 1 {
            let next_tier_index = fab.tier_index + 1;
            let next_tier_info = &fab.available_tiers[next_tier_index];
            // Assuming upgrade cost is construction cost of next tier if not specified otherwise
            // In a more complex system, FabricatorTier could have an `upgrade_credits_cost` field.
            let upgrade_cost = next_tier_info.construction_credits_cost; 

            if game_state.credits < upgrade_cost as f64 {
                println!("Not enough credits to upgrade Fabricator {} to {}. Required: {}, Available: {:.2}", fabricator_id, next_tier_info.name, upgrade_cost, game_state.credits);
                return;
            }
            game_state.credits -= upgrade_cost as f64;
            println!("Upgraded Fabricator {} to {} for {} credits. Remaining credits: {:.2}", fabricator_id, next_tier_info.name, upgrade_cost, game_state.credits);
            
            fab.tier_index = next_tier_index;
            fab.production_progress_secs = 0.0; 

            if fab.assigned_specialists > next_tier_info.specialist_requirement {
                let to_unassign = fab.assigned_specialists - next_tier_info.specialist_requirement;
                fab.assigned_specialists -= to_unassign;
                game_state.assigned_specialists_total -= to_unassign;
            }
            println!("Upgraded Fabricator {} to {}", fabricator_id, next_tier_info.name);
        } else {
            println!("Fabricator {} is already at max tier.", fabricator_id);
        }
    } else {
        println!("Fabricator with ID {} not found.", fabricator_id);
    }
}

pub fn remove_fabricator(game_state: &mut GameState, fabricator_id: &str) {
    if let Some(index) = game_state.fabricators.iter().position(|f| f.id == fabricator_id) {
        let removed_fab = game_state.fabricators.remove(index);
        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(removed_fab.assigned_specialists);
        println!("Removed Fabricator with ID {}", fabricator_id);
    } else {
        println!("Fabricator with ID {} not found for removal.", fabricator_id);
    }
}

pub fn assign_specialists_to_fabricator(game_state: &mut GameState, fab_id: &str, num_to_assign: u32) {
    if let Some(fab) = game_state.fabricators.iter_mut().find(|f| f.id == fab_id) {
        if let Some(tier) = fab.available_tiers.get(fab.tier_index) {
            let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
            if available_general_inhabitants < num_to_assign {
                println!("Not enough unassigned inhabitants for Fabricator {}.", fab_id); return;
            }
            if fab.assigned_specialists + num_to_assign > tier.specialist_requirement {
                println!("Cannot assign more specialists than required for Fabricator {}. Max: {}", fab_id, tier.specialist_requirement); return;
            }
            fab.assigned_specialists += num_to_assign;
            game_state.assigned_specialists_total += num_to_assign;
            println!("Assigned {} specialists to Fabricator {}.", num_to_assign, fab_id);
        }
    } else {
        println!("Fabricator {} not found.", fab_id);
    }
}

pub fn unassign_specialists_from_fabricator(game_state: &mut GameState, fab_id: &str, num_to_unassign: u32) {
    if let Some(fab) = game_state.fabricators.iter_mut().find(|f| f.id == fab_id) {
        let actual_unassign = num_to_unassign.min(fab.assigned_specialists);
        fab.assigned_specialists -= actual_unassign;
        game_state.assigned_specialists_total -= actual_unassign;
        fab.is_active = false; // Deactivate when specialists are removed, will be checked next tick.
        fab.production_progress_secs = 0.0; // Reset progress
        println!("Unassigned {} specialists from Fabricator {}.", actual_unassign, fab_id);
    } else {
        println!("Fabricator {} not found.", fab_id);
    }
}

pub fn fabricator_production_system(game_state: &mut GameState, time_delta_secs: f32) {
    for fab in game_state.fabricators.iter_mut() {
        if let Some(tier) = fab.available_tiers.get(fab.tier_index) {
            // Check power (external system should update game_state.current_resources[ResourceType::Power])
            let has_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0) >= tier.power_requirement as f32;
            // Check specialists
            let has_specialists = fab.assigned_specialists >= tier.specialist_requirement;
            // Check input resources - Updated call to check_fabricator_inputs
            let has_inputs = check_fabricator_inputs(&game_state.current_resources, tier);

            fab.is_active = has_power && has_specialists && has_inputs;

            if fab.is_active {
                fab.production_progress_secs += time_delta_secs;
                if fab.production_progress_secs >= tier.production_time_secs {
                    // Consume inputs
                    for (resource_type, required_amount) in &tier.input_resources {
                        *game_state.current_resources.entry(*resource_type).or_insert(0.0) -= *required_amount as f32;
                    }
                    // Add output
                    *game_state.current_resources.entry(tier.output_product).or_insert(0.0) += tier.output_quantity as f32;
                    println!("Fabricator {} produced {} {} (total now: {}).", fab.id, tier.output_quantity, format!("{:?}", tier.output_product), game_state.current_resources.get(&tier.output_product).unwrap());
                    fab.production_progress_secs = 0.0; // Reset progress
                }
            } else {
                // Optional: Decay progress if inactive, or just pause
                 if !has_inputs && fab.production_progress_secs > 0.0 {
                    // fab.production_progress_secs = (fab.production_progress_secs - time_delta_secs * 0.5).max(0.0); // Slow decay if no input
                    // For now, just pause if not active
                 }
            }
        }
    }
}


// --- Happiness Logic ---

pub fn calculate_colony_happiness(game_state: &mut GameState) {
    let mut happiness_score = 50.0; // Base happiness

    // 1. Nutrient Paste Supply
    if game_state.simulated_has_sufficient_nutrient_paste {
        happiness_score += 10.0; // Positive modifier for food
    } else {
        happiness_score -= 20.0; // Negative modifier for lack of food
    }

    // 2. Service Coverage (Simplified global check)
    // This is a very basic model. A real system would be more nuanced.
    let desired_services_per_100_inhabitants = 1.0; // Arbitrary demand factor
    let service_types_to_check = [
        ServiceType::Wellness, ServiceType::Security, 
        // ServiceType::Education, ServiceType::Recreation, ServiceType::Spiritual, // TODO: Add once tiers are defined
    ];

    for service_type in service_types_to_check.iter() {
        let demand = (game_state.total_inhabitants as f32 / 100.0) * desired_services_per_100_inhabitants;
        let mut supply = 0.0;
        for building in &game_state.service_buildings {
            if building.service_type == *service_type && building.is_active {
                if let Some(tier) = building.available_tiers.get(building.current_tier_index) {
                    // Assume each staffed building tier provides a certain amount of "service units"
                    // For simplicity, let's say a staffed building provides 1 unit of service supply.
                    // A more complex model would use specialist count vs requirement, tier capacity, etc.
                    if building.assigned_specialists >= tier.specialist_requirement {
                        supply += 1.0; // Simplified: 1 active, staffed building = 1 unit of supply
                    }
                }
            }
        }

        if supply >= demand && demand > 0.0 {
            happiness_score += 5.0; // Max +5 per fully covered service type
        } else if supply > 0.0 && demand > 0.0 { // Partially covered
            happiness_score += 2.5 * (supply / demand); // Proportional bonus
        } else if demand > 0.0 { // No supply for existing demand
            happiness_score -= 5.0;
        }
        // If demand is 0 (e.g. no inhabitants), no change for this service.
    }
    
    // Add specific checks for missing Education, Recreation, Spiritual when their tiers are defined
    if game_state.service_buildings.iter().any(|b| b.service_type == ServiceType::Education && b.is_active && b.assigned_specialists > 0) { /* check if staffed */ } else if game_state.total_inhabitants > 20 { happiness_score -= 2.0; } // Small penalty if basic education missing for larger pop
    if game_state.service_buildings.iter().any(|b| b.service_type == ServiceType::Recreation && b.is_active && b.assigned_specialists > 0) { /* check if staffed */ } else if game_state.total_inhabitants > 15 { happiness_score -= 2.0; }
    if game_state.service_buildings.iter().any(|b| b.service_type == ServiceType::Spiritual && b.is_active && b.assigned_specialists > 0) { /* check if staffed */ } else if game_state.total_inhabitants > 25 { happiness_score -= 1.0; }


    // 3. Housing
    if game_state.available_housing_capacity > 0 {
        let occupancy_ratio = game_state.total_inhabitants as f32 / game_state.available_housing_capacity as f32;
        if occupancy_ratio > 1.0 { // Overcrowded
            happiness_score -= (occupancy_ratio - 1.0) * 30.0; // Significant penalty, e.g. 20% over = -6
        } else if occupancy_ratio > 0.9 { // Near capacity
            happiness_score += 1.0;
        } else { // Plenty of space
            happiness_score += 3.0;
        }
    } else if game_state.total_inhabitants > 0 { // Inhabitants exist but no housing
        happiness_score -= 25.0; // Major penalty for homelessness
    }

    // 4. Employment
    if game_state.total_specialist_slots > 0 {
        // Unemployment among potential specialists
        let potential_specialists = game_state.total_inhabitants; // Simplification: any inhabitant could be a specialist
        let unemployment_rate = if potential_specialists > game_state.assigned_specialists_total {
            (potential_specialists - game_state.assigned_specialists_total) as f32 / potential_specialists as f32
        } else { 0.0 };
        
        if unemployment_rate > 0.5 { // High unemployment
            happiness_score -= unemployment_rate * 10.0; // Penalty up to -5 if 100% unemployed
        } else { // Low unemployment or all jobs filled by available people
            happiness_score += 3.0;
        }

        // Separately, consider if there are not enough jobs for the population
        if game_state.total_inhabitants > game_state.total_specialist_slots {
            let job_shortage_ratio = (game_state.total_inhabitants - game_state.total_specialist_slots) as f32 / game_state.total_inhabitants as f32;
            happiness_score -= job_shortage_ratio * 10.0; // Penalty if not enough jobs for everyone
        }

    } else if game_state.total_inhabitants > 5 { // No jobs at all, but population exists
        happiness_score -= 5.0;
    }
    
    // 5. Legacy Structure Bonus
    happiness_score += game_state.legacy_structure_happiness_bonus;

    // 6. Civic Index (Optional Influence)
    // Let's say every 10 civic index points add 1 to happiness, max of +5
    happiness_score += (game_state.civic_index as f32 / 10.0).min(5.0);

    // Final Calculation
    game_state.colony_happiness = happiness_score.clamp(0.0, 100.0);
    println!("Colony Happiness updated to: {:.2}%", game_state.colony_happiness);
}


// --- Civic Index and Services/Zones Logic ---

pub fn update_civic_index(game_state: &mut GameState) {
    let mut new_civic_index = 0;
    for service_building in &game_state.service_buildings {
        if service_building.is_active {
            if let Some(tier) = service_building.available_tiers.get(service_building.current_tier_index) {
                new_civic_index += tier.civic_index_contribution;
            }
        }
    }
    for zone in &game_state.zones {
        if zone.is_active {
            if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                new_civic_index += tier.civic_index_contribution;
            }
        }
    }
    game_state.civic_index = new_civic_index;
    println!("Civic Index updated to: {}", game_state.civic_index);
}

// --- Population and Habitation Logic ---
// (This section already exists, new Service/Zone functions will be added below or in a new module)

// Static atomic counter for generating unique IDs
static NEXT_ID: AtomicU32 = AtomicU32::new(0);

// (Helper for generating unique IDs, simple for now)
fn generate_unique_id() -> String {
    // Fetch the current value and increment.
    // Ordering::Relaxed is fine for a simple counter.
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("struct_{}", id)
}

pub fn get_habitation_tiers() -> Vec<HabitationStructureTier> {
    vec![
        HabitationStructureTier {
            name: "Basic Dwellings".to_string(),
            housing_capacity: 10,
            specialist_slots: 1,
            construction_credits_cost: 100, 
        },
        HabitationStructureTier {
            name: "Community Blocks".to_string(),
            housing_capacity: 25,
            specialist_slots: 3,
            construction_credits_cost: 250, 
        },
        HabitationStructureTier {
            name: "Arcology Spires".to_string(),
            housing_capacity: 100,
            specialist_slots: 10,
            construction_credits_cost: 1000, 
        },
    ]
}

pub fn update_housing_and_specialist_slots(game_state: &mut GameState) {
    let mut total_housing = 0;
    let mut total_slots = 0;
    for structure in &game_state.habitation_structures {
        if let Some(tier) = structure.available_tiers.get(structure.tier_index) {
            total_housing += tier.housing_capacity;
            total_slots += tier.specialist_slots;
        }
    }
    game_state.available_housing_capacity = total_housing;
    game_state.total_specialist_slots = total_slots;
}

pub fn add_habitation_structure(game_state: &mut GameState, tier_index: usize) {
    let all_tiers = get_habitation_tiers();
    if tier_index >= all_tiers.len() {
        println!("Error: Invalid tier index for habitation structure.");
        return;
    }
    // Only take the tiers up to and including the specified one for the new structure's available_tiers
    // This is a simplification. A real game might allow upgrading through all defined tiers.
    // For now, we assume a structure is built *as* a certain set of possible tiers.
    // Or, more simply, just give it all defined tiers and let `tier_index` control current.

    let tier_info = &all_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        println!("Not enough credits to build {}. Required: {}, Available: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    println!("Built {} for {} credits. Remaining credits: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);
    
    let new_structure = HabitationStructure {
        id: generate_unique_id(),
        tier_index, // Starts at the specified tier
        available_tiers: all_tiers.clone(), 
        current_inhabitants: 0, // Will be filled by grow_inhabitants
        assigned_specialists: 0,
    };
    game_state.habitation_structures.push(new_structure);
    update_housing_and_specialist_slots(game_state);
    println!("Added Habitation Structure: {}", tier_info.name);
}

pub fn upgrade_habitation_structure(game_state: &mut GameState, structure_id: &str) {
    if let Some(structure) = game_state.habitation_structures.iter_mut().find(|s| s.id == structure_id) {
        if structure.tier_index < structure.available_tiers.len() - 1 {
            let next_tier_index = structure.tier_index + 1;
            let next_tier_info = &structure.available_tiers[next_tier_index];
            
            // Assume upgrade cost is the construction cost of the next tier for now
            let upgrade_cost = next_tier_info.construction_credits_cost; 

            if game_state.credits < upgrade_cost as f64 {
                println!("Not enough credits to upgrade Habitation Structure {} to {}. Required: {}, Available: {:.2}", structure_id, next_tier_info.name, upgrade_cost, game_state.credits);
                return;
            }
            game_state.credits -= upgrade_cost as f64;
            println!("Upgraded Habitation Structure {} to {} for {} credits. Remaining credits: {:.2}", structure_id, next_tier_info.name, upgrade_cost, game_state.credits);

            structure.tier_index = next_tier_index;
            update_housing_and_specialist_slots(game_state);
        } else {
            println!("Habitation Structure {} is already at max tier.", structure_id);
        }
    } else {
        println!("Habitation Structure with ID {} not found.", structure_id);
    }
}

pub fn remove_habitation_structure(game_state: &mut GameState, structure_id: &str) {
    // Removed: let mut inhabitants_to_reassign = 0;
    // Removed: let mut specialists_to_unassign = 0;

    if let Some(index) = game_state.habitation_structures.iter().position(|s| s.id == structure_id) {
        let removed_structure = game_state.habitation_structures.remove(index);
        // inhabitants_to_reassign = removed_structure.current_inhabitants; // Directly use removed_structure fields
        // specialists_to_unassign = removed_structure.assigned_specialists; // Directly use removed_structure fields
        println!("Removed Habitation Structure with ID {}", structure_id);

        update_housing_and_specialist_slots(game_state); // Call this before adjusting global counts

        // Adjust global counts based on the removed structure's population
        game_state.total_inhabitants = game_state.total_inhabitants.saturating_sub(removed_structure.current_inhabitants);
        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(removed_structure.assigned_specialists);

    } else {
        println!("Habitation Structure with ID {} not found for removal.", structure_id);
        return;
    }

    // These checks should be done after potential removal and adjustments
    // If total inhabitants now exceed housing, clamp it.
    if game_state.total_inhabitants > game_state.available_housing_capacity {
        game_state.total_inhabitants = game_state.available_housing_capacity;
    }
    // Ensure assigned specialists don't exceed total inhabitants or new total slots
    game_state.assigned_specialists_total = game_state.assigned_specialists_total.min(game_state.total_inhabitants).min(game_state.total_specialist_slots);
    
    // Distribute remaining inhabitants (very basic, just ensures they are housed if possible)
    // and re-check specialist assignments across remaining structures (beyond scope for now).
    // For now, the removed specialists are just returned to the general pool.
    // Individual structure current_inhabitants and assigned_specialists would need careful recalculation
    // if we were to perfectly re-distribute. This is simplified.
}

pub fn grow_inhabitants(game_state: &mut GameState) {
    // Placeholder Growth Logic:
    if game_state.total_inhabitants < game_state.available_housing_capacity {
        let growth_amount = 1; // Fixed amount for now
        game_state.total_inhabitants += growth_amount;
        println!("Inhabitants grew by {}. Total: {}", growth_amount, game_state.total_inhabitants);

        // Distribute new inhabitants to structures (simple fill-up logic)
        // This is also where we'd assign them to `current_inhabitants` of specific structures.
        // For now, we just track totals. A more detailed assignment is needed for per-structure inhabitant counts.
        // Let's try a simple distribution for now:
        for structure in game_state.habitation_structures.iter_mut() {
            if game_state.total_inhabitants > structure.current_inhabitants { // Check against global total for simplicity here
                 if let Some(tier) = structure.available_tiers.get(structure.tier_index) {
                    let can_house = tier.housing_capacity - structure.current_inhabitants;
                    if can_house > 0 {
                        // This logic is still a bit off as it adds to *each* structure if there's global space
                        // A better way would be to iterate, assign one by one until global growth is met or all full.
                        // For now, let's assume growth fills the *first available* slots.
                        // This requires a more complex loop if `growth_amount` > 1.
                        // Simplified: If any structure has space, and global total allows, add one.
                        // This is still problematic. The problem is distributing a *total* growth amount
                        // into individual structures.
                        // Corrected simplified logic:
                        // Iterate structures, if a structure has space (tier.cap > struct.current)
                        // and global total_inhabitants has not yet reached global available_housing_capacity,
                        // increment struct.current_inhabitants. Decrement remaining_growth_today.
                        // This will be done in a more robust way when inhabitant assignment is refined.
                        // For now, the `current_inhabitants` field in `HabitationStructure` is not strictly enforced by this function.
                    }
                 }
            }
        }

    }
    // Future Refinements:
    // - Growth rate influenced by happiness.
    // - Growth rate influenced by food availability (Nutrient Paste).
    // - Growth rate influenced by Legacy Structure bonuses.
    // - Ensure `total_inhabitants` does not exceed `available_housing_capacity` (already handled by the condition).
}

pub fn assign_specialists_to_structure(game_state: &mut GameState, structure_id: &str, num_to_assign: u32) {
    if let Some(structure) = game_state.habitation_structures.iter_mut().find(|s| s.id == structure_id) {
        if let Some(tier) = structure.available_tiers.get(structure.tier_index) {
            let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
            if available_general_inhabitants < num_to_assign {
                println!("Not enough unassigned inhabitants to become specialists.");
                return;
            }
            if structure.assigned_specialists + num_to_assign > tier.specialist_slots {
                println!("Not enough specialist slots in this structure.");
                return;
            }
            // Also ensure structure's current_inhabitants can support these specialists
            // (assuming specialists are also inhabitants of that structure)
            if structure.assigned_specialists + num_to_assign > structure.current_inhabitants {
                 println!("Cannot assign more specialists than current inhabitants in the structure.");
                 // This check assumes specialists live in the structure they work in, which might be refined.
                 // For now, let's also ensure the structure has enough total inhabitants to cover the new specialists
                 // This might require assigning inhabitants first.
                 // For simplicity, we assume if total_inhabitants is high enough, they can be sourced.
                 // A better model would explicitly move an inhabitant to a specialist role.
            }


            structure.assigned_specialists += num_to_assign;
            game_state.assigned_specialists_total += num_to_assign;
            println!("Assigned {} specialists to structure {}. Total assigned: {}", num_to_assign, structure_id, game_state.assigned_specialists_total);
        } else {
            println!("Error: Structure {} tier data missing.", structure_id);
        }
    } else {
        println!("Habitation Structure with ID {} not found.", structure_id);
    }
}

pub fn unassign_specialists_from_structure(game_state: &mut GameState, structure_id: &str, num_to_unassign: u32) {
    if let Some(structure) = game_state.habitation_structures.iter_mut().find(|s| s.id == structure_id) {
        let actual_unassign = num_to_unassign.min(structure.assigned_specialists);
        structure.assigned_specialists -= actual_unassign;
        game_state.assigned_specialists_total -= actual_unassign;
        println!("Unassigned {} specialists from structure {}. Total assigned: {}", actual_unassign, structure_id, game_state.assigned_specialists_total);
    } else {
        println!("Habitation Structure with ID {} not found.", structure_id);
    }
}

// --- Service Building Management ---

pub fn get_service_building_tiers(service_type: ServiceType) -> Vec<ServiceBuildingTier> {
    // Example tiers, customize per service type
    match service_type {
        ServiceType::Wellness => vec![
            ServiceBuildingTier { name: "Clinic".to_string(), specialist_requirement: 2, influence_radius: 10.0, upkeep_cost: 10, civic_index_contribution: 5, construction_credits_cost: 150 },
            ServiceBuildingTier { name: "Hospital".to_string(), specialist_requirement: 5, influence_radius: 20.0, upkeep_cost: 30, civic_index_contribution: 15, construction_credits_cost: 400 },
        ],
        ServiceType::Security => vec![
            ServiceBuildingTier { name: "Security Post".to_string(), specialist_requirement: 3, influence_radius: 15.0, upkeep_cost: 15, civic_index_contribution: 5, construction_credits_cost: 150 },
            ServiceBuildingTier { name: "Precinct".to_string(), specialist_requirement: 7, influence_radius: 30.0, upkeep_cost: 40, civic_index_contribution: 15, construction_credits_cost: 450 },
        ],
        ServiceType::Education => vec![ ServiceBuildingTier { name: "School".to_string(), specialist_requirement: 4, influence_radius: 20.0, upkeep_cost: 25, civic_index_contribution: 10, construction_credits_cost: 300 } ],
        ServiceType::Recreation => vec![ ServiceBuildingTier { name: "Rec Center".to_string(), specialist_requirement: 3, influence_radius: 15.0, upkeep_cost: 20, civic_index_contribution: 8, construction_credits_cost: 250 } ],
        ServiceType::Spiritual => vec![ ServiceBuildingTier { name: "Sanctum".to_string(), specialist_requirement: 2, influence_radius: 10.0, upkeep_cost: 10, civic_index_contribution: 3, construction_credits_cost: 200 } ],
    }
}

pub fn add_service_building(game_state: &mut GameState, service_type: ServiceType, tier_index: usize, position: Option<(f32, f32)>) {
    let all_tiers = get_service_building_tiers(service_type);
    if tier_index >= all_tiers.len() {
        println!("Error: Invalid tier index for service building type {:?}.", service_type);
        return;
    }
    let tier_info = &all_tiers[tier_index]; // tier_info is correctly defined here
    // Credit Check for Construction
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        println!("Not enough credits to build {}. Required: {}, Available: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    println!("Built {} for {} credits. Remaining credits: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);

    let new_building = ServiceBuilding {
        id: generate_unique_id(),
        service_type,
        current_tier_index: tier_index,
        available_tiers: all_tiers.clone(),
        assigned_specialists: 0,
        is_active: true, // Activate on build for now, upkeep might deactivate later
        position,
    };
    game_state.service_buildings.push(new_building);
    update_civic_index(game_state); // Civic index might depend on active buildings
    println!("Added Service Building: {}", tier_info.name);
}

pub fn upgrade_service_building(game_state: &mut GameState, building_id: &str) {
    let mut upgraded = false;
    if let Some(building) = game_state.service_buildings.iter_mut().find(|b| b.id == building_id) {
        if building.current_tier_index < building.available_tiers.len() - 1 {
            let next_tier_index = building.current_tier_index + 1;
            // Clone necessary information from next_tier_info to avoid borrow issues later
            // if the original next_tier_info reference becomes invalid after building mutation.
            let upgrade_cost = building.available_tiers[next_tier_index].construction_credits_cost;
            let next_tier_name = building.available_tiers[next_tier_index].name.clone();
            let specialist_requirement_next_tier = building.available_tiers[next_tier_index].specialist_requirement;

            if game_state.credits < upgrade_cost as f64 {
                println!("Not enough credits to upgrade Service Building {} to {}. Required: {}, Available: {:.2}", building_id, next_tier_name, upgrade_cost, game_state.credits);
                // Note: No early return here; let the function complete to potentially call update_civic_index if other logic paths existed.
                // However, for this specific structure, an early return is fine.
                // For consistency with the boolean flag pattern, we avoid early return here.
            } else {
                game_state.credits -= upgrade_cost as f64;
                println!("Upgraded Service Building {} to {} for {} credits. Remaining credits: {:.2}", building_id, next_tier_name, upgrade_cost, game_state.credits);

                building.current_tier_index = next_tier_index;
                if building.assigned_specialists > specialist_requirement_next_tier {
                    let to_unassign = building.assigned_specialists - specialist_requirement_next_tier;
                    building.assigned_specialists -= to_unassign;
                    game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(to_unassign); 
                }
                println!("Service Building {} successfully upgraded to: {}", building_id, next_tier_name);
                upgraded = true;
            }
        } else {
            println!("Service Building {} is already at max tier.", building_id);
        }
    } else {
        println!("Service Building with ID {} not found.", building_id);
    }

    if upgraded {
        update_civic_index(game_state);
    }
}

pub fn remove_service_building(game_state: &mut GameState, building_id: &str) {
    if let Some(index) = game_state.service_buildings.iter().position(|b| b.id == building_id) {
        let removed_building = game_state.service_buildings.remove(index);
        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(removed_building.assigned_specialists);
        update_civic_index(game_state);
        // update_total_specialist_slots(game_state);
        println!("Removed Service Building with ID {}", building_id);
    } else {
        println!("Service Building with ID {} not found for removal.", building_id);
    }
}

pub fn assign_specialists_to_service_building(game_state: &mut GameState, building_id: &str, num_to_assign: u32) {
    if let Some(building) = game_state.service_buildings.iter_mut().find(|b| b.id == building_id) {
        if let Some(tier) = building.available_tiers.get(building.current_tier_index) {
            let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
            if available_general_inhabitants < num_to_assign {
                println!("Not enough unassigned inhabitants to become specialists for service building {}.", building_id);
                return;
            }
            if building.assigned_specialists + num_to_assign > tier.specialist_requirement {
                println!("Cannot assign more specialists than required for service building {}. Max: {}", building_id, tier.specialist_requirement);
                return;
            }
            building.assigned_specialists += num_to_assign;
            game_state.assigned_specialists_total += num_to_assign;
            println!("Assigned {} specialists to service building {}. Total assigned globally: {}", num_to_assign, building_id, game_state.assigned_specialists_total);
            // Potentially activate building if requirements met.
            // building.is_active = building.assigned_specialists >= tier.specialist_requirement;
            // update_civic_index(game_state); // Called if is_active changes state
        }
    } else {
        println!("Service Building {} not found for specialist assignment.", building_id);
    }
}

pub fn unassign_specialists_from_service_building(game_state: &mut GameState, building_id: &str, num_to_unassign: u32) {
    if let Some(building) = game_state.service_buildings.iter_mut().find(|b| b.id == building_id) {
        let actual_unassign = num_to_unassign.min(building.assigned_specialists);
        building.assigned_specialists -= actual_unassign;
        game_state.assigned_specialists_total -= actual_unassign;
        println!("Unassigned {} specialists from service building {}. Total assigned globally: {}", actual_unassign, building_id, game_state.assigned_specialists_total);
        // Potentially deactivate building if requirements no longer met.
        // if let Some(tier) = building.available_tiers.get(building.current_tier_index) {
        //     building.is_active = building.assigned_specialists >= tier.specialist_requirement;
        //     update_civic_index(game_state); // Called if is_active changes state
        // }
    } else {
        println!("Service Building {} not found for specialist unassignment.", building_id);
    }
}

// --- Zone Management ---

pub fn get_zone_tiers(zone_type: ZoneType) -> Vec<ZoneTier> {
    match zone_type {
        ZoneType::Commercial => vec![
            ZoneTier { name: "Small Market Stalls".to_string(), specialist_jobs_provided: 5, civic_index_contribution: 3, upkeep_cost: 10, construction_credits_cost: 100, income_generation: 50 },
            ZoneTier { name: "Shopping Plaza".to_string(), specialist_jobs_provided: 15, civic_index_contribution: 10, upkeep_cost: 30, construction_credits_cost: 300, income_generation: 200 },
        ],
        ZoneType::LightIndustry => vec![
            ZoneTier { name: "Workshops".to_string(), specialist_jobs_provided: 8, civic_index_contribution: 2, upkeep_cost: 15, construction_credits_cost: 120, income_generation: 0 }, // Light Industry might not generate direct credits
            ZoneTier { name: "Assembly Lines".to_string(), specialist_jobs_provided: 20, civic_index_contribution: 8, upkeep_cost: 40, construction_credits_cost: 350, income_generation: 0 },
        ],
    }
}

pub fn add_zone(game_state: &mut GameState, zone_type: ZoneType, tier_index: usize) {
    let all_tiers = get_zone_tiers(zone_type);
    if tier_index >= all_tiers.len() {
        println!("Error: Invalid tier index for zone type {:?}.", zone_type);
        return;
    }
    let tier_info = &all_tiers[tier_index]; // tier_info is correctly defined here
    // Credit Check for Construction
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        println!("Not enough credits to build {}. Required: {}, Available: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    println!("Built {} for {} credits. Remaining credits: {:.2}", tier_info.name, tier_info.construction_credits_cost, game_state.credits);
    
    let new_zone = Zone {
        id: generate_unique_id(),
        zone_type,
        current_tier_index: tier_index,
        available_tiers: all_tiers.clone(),
        assigned_specialists: 0,
        is_active: true, // Activate on build for now, upkeep might deactivate
    };
    game_state.zones.push(new_zone);
    update_total_specialist_slots(game_state); 
    update_civic_index(game_state);
    println!("Added Zone: {}", tier_info.name);
}

pub fn upgrade_zone(game_state: &mut GameState, zone_id: &str) {
    if let Some(zone) = game_state.zones.iter_mut().find(|z| z.id == zone_id) {
        if zone.current_tier_index < zone.available_tiers.len() - 1 {
            let next_tier_index = zone.current_tier_index + 1;
            let next_tier_info = &zone.available_tiers[next_tier_index];

            // Assume upgrade cost is construction cost of the next tier
            let upgrade_cost = next_tier_info.construction_credits_cost;

            if game_state.credits < upgrade_cost as f64 {
                println!("Not enough credits to upgrade Zone {} to {}. Required: {}, Available: {:.2}", zone_id, next_tier_info.name, upgrade_cost, game_state.credits);
                return;
            }
            game_state.credits -= upgrade_cost as f64;
            println!("Upgraded Zone {} to {} for {} credits. Remaining credits: {:.2}", zone_id, next_tier_info.name, upgrade_cost, game_state.credits);
            
            zone.current_tier_index = next_tier_index;
            if zone.assigned_specialists > next_tier_info.specialist_jobs_provided {
                let to_unassign = zone.assigned_specialists - next_tier_info.specialist_jobs_provided;
                zone.assigned_specialists -= to_unassign;
                game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(to_unassign);
            }
            println!("Upgraded Zone {} to {}", zone_id, next_tier_info.name); // Original println
            upgraded = true;
        } else {
            println!("Zone {} is already at max tier.", zone_id);
        }
    } else {
        println!("Zone with ID {} not found.", zone_id);
    }

    if upgraded {
        update_total_specialist_slots(game_state);
        update_civic_index(game_state);
    }
}

pub fn remove_zone(game_state: &mut GameState, zone_id: &str) {
    if let Some(index) = game_state.zones.iter().position(|z| z.id == zone_id) {
        let removed_zone = game_state.zones.remove(index);
        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(removed_zone.assigned_specialists);
        update_total_specialist_slots(game_state);
        update_civic_index(game_state);
        println!("Removed Zone with ID {}", zone_id);
    } else {
        println!("Zone with ID {} not found for removal.", zone_id);
    }
}

pub fn assign_specialists_to_zone(game_state: &mut GameState, zone_id: &str, num_to_assign: u32) {
    if let Some(zone) = game_state.zones.iter_mut().find(|z| z.id == zone_id) {
        if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
            let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
            if available_general_inhabitants < num_to_assign {
                println!("Not enough unassigned inhabitants to become specialists for zone {}.", zone_id);
                return;
            }
            if zone.assigned_specialists + num_to_assign > tier.specialist_jobs_provided {
                println!("Cannot assign more specialists than available jobs in zone {}. Max: {}", zone_id, tier.specialist_jobs_provided);
                return;
            }
            zone.assigned_specialists += num_to_assign;
            game_state.assigned_specialists_total += num_to_assign;
            println!("Assigned {} specialists to zone {}. Total assigned globally: {}", num_to_assign, zone_id, game_state.assigned_specialists_total);
            // zone.is_active = zone.assigned_specialists > 0; // Example activation, could be more complex
            // update_civic_index(game_state);
        }
    } else {
        println!("Zone {} not found for specialist assignment.", zone_id);
    }
}

pub fn unassign_specialists_from_zone(game_state: &mut GameState, zone_id: &str, num_to_unassign: u32) {
    if let Some(zone) = game_state.zones.iter_mut().find(|z| z.id == zone_id) {
        let actual_unassign = num_to_unassign.min(zone.assigned_specialists);
        zone.assigned_specialists -= actual_unassign;
        game_state.assigned_specialists_total -= actual_unassign;
        println!("Unassigned {} specialists from zone {}. Total assigned globally: {}", actual_unassign, zone_id, game_state.assigned_specialists_total);
        // if zone.assigned_specialists == 0 { zone.is_active = false; }
        // update_civic_index(game_state);
    } else {
        println!("Zone {} not found for specialist unassignment.", zone_id);
    }
}

// Helper function to update total specialist slots from all sources
pub fn update_total_specialist_slots(game_state: &mut GameState) {
    let mut total_slots = 0;
    for structure in &game_state.habitation_structures {
        if let Some(tier) = structure.available_tiers.get(structure.tier_index) {
            total_slots += tier.specialist_slots; // Habitation provides some base slots
        }
    }
    for zone in &game_state.zones {
        if zone.is_active {
            if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                total_slots += tier.specialist_jobs_provided; // Zones provide jobs/slots
            }
        }
    }
    // Service buildings typically *require* specialists, not provide slots, so they are not added here.
    // If some service buildings also provided jobs, that logic would be added.
    game_state.total_specialist_slots = total_slots;
    println!("Total specialist slots updated to: {}", game_state.total_specialist_slots);
}


// --- Game Logic Plugin ---

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .init_resource::<ColonyStats>()
            .init_resource::<GraphData>()
            .add_systems(FixedUpdate, (
                game_tick_system,
                research_system,
                grow_inhabitants_system.after(game_tick_system),
                fabricator_production_tick_system.after(game_tick_system),
                processing_plant_operations_tick_system.after(game_tick_system),
                upkeep_income_tick_system.after(processing_plant_operations_tick_system), 
                calculate_colony_happiness_system.after(upkeep_income_tick_system), 
                update_colony_stats_system.after(calculate_colony_happiness_system),
                update_graph_data_system.after(update_colony_stats_system),
            ));
    }
}

// --- Economic Systems ---

fn generate_income_system(game_state: &mut GameState) {
    let mut total_income_this_period = 0.0;

    for zone in &game_state.zones {
        if zone.is_active && zone.zone_type == ZoneType::Commercial {
            if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                total_income_this_period += tier.income_generation as f64;
            }
        }
    }
    total_income_this_period += game_state.legacy_structure_income_bonus;
    if total_income_this_period > 0.0 {
        game_state.credits += total_income_this_period;
    }
}

fn deduct_upkeep_system(game_state: &mut GameState) {
    let mut total_upkeep_to_deduct_this_tick = 0.0;
    let mut civic_index_needs_update = false;

    let initial_credits = game_state.credits; // Credits before attempting any upkeep payment

    // Calculate total potential upkeep first
    let mut potential_total_upkeep = 0.0;
    for building in &game_state.service_buildings { if building.is_active { if let Some(tier) = building.available_tiers.get(building.current_tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }
    for zone in &game_state.zones { if zone.is_active { if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }
    for fab in &game_state.fabricators { if fab.is_active { if let Some(tier) = fab.available_tiers.get(fab.tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }
    for plant in &game_state.processing_plants { if plant.is_active { if let Some(tier) = plant.available_tiers.get(plant.tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }
    // TODO: Add Admin Spire upkeep to potential_total_upkeep

    if initial_credits >= potential_total_upkeep {
        // Sufficient credits for all upkeep
        total_upkeep_to_deduct_this_tick = potential_total_upkeep;
    } else {
        // Insufficient credits, deactivate buildings one by one until affordable or all are off
        println!("Warning: Insufficient credits ({:.2}) to cover all upkeep ({:.2}). Deactivating buildings.", initial_credits, potential_total_upkeep);
        
        // Deactivate less critical buildings first (example order, can be refined)
        for fab in game_state.fabricators.iter_mut() {
            if fab.is_active { if let Some(tier) = fab.available_tiers.get(fab.tier_index) {
                if initial_credits < (total_upkeep_to_deduct_this_tick + tier.upkeep_cost as f64) { fab.is_active = false; println!("Deactivated Fabricator {}", fab.id); } 
                else { total_upkeep_to_deduct_this_tick += tier.upkeep_cost as f64;}
            }}
        }
        for plant in game_state.processing_plants.iter_mut() {
            if plant.is_active { if let Some(tier) = plant.available_tiers.get(plant.tier_index) {
                if initial_credits < (total_upkeep_to_deduct_this_tick + tier.upkeep_cost as f64) { plant.is_active = false; println!("Deactivated Processing Plant {}", plant.id); }
                else { total_upkeep_to_deduct_this_tick += tier.upkeep_cost as f64; }
            }}
        }
        for zone in game_state.zones.iter_mut() {
            if zone.is_active { if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                if initial_credits < (total_upkeep_to_deduct_this_tick + tier.upkeep_cost as f64) { zone.is_active = false; civic_index_needs_update = true; println!("Deactivated Zone {}", zone.id); }
                else { total_upkeep_to_deduct_this_tick += tier.upkeep_cost as f64; }
            }}
        }
        for building in game_state.service_buildings.iter_mut() {
            if building.is_active { if let Some(tier) = building.available_tiers.get(building.current_tier_index) {
                if initial_credits < (total_upkeep_to_deduct_this_tick + tier.upkeep_cost as f64) { building.is_active = false; civic_index_needs_update = true; println!("Deactivated Service Building {}", building.id); }
                else { total_upkeep_to_deduct_this_tick += tier.upkeep_cost as f64; }
            }}
        }
        // TODO: Deactivate Admin Spire last if necessary
    }

    if total_upkeep_to_deduct_this_tick > 0.0 {
        game_state.credits -= total_upkeep_to_deduct_this_tick; 
        if game_state.credits < 0.0 { // Should ideally not happen if deactivation logic is perfect
            println!("Critical Warning: Credits still went negative ({:.2}) after deactivations!", game_state.credits);
            game_state.credits = 0.0; // Prevent further negative spiral from this error
        }
    }

    if civic_index_needs_update {
        update_civic_index(game_state); 
    }
}

fn upkeep_income_tick_system(mut game_state: ResMut<GameState>) {
    generate_income_system(&mut game_state);
    deduct_upkeep_system(&mut game_state);
}

// System wrapper for Processing Plant operations
fn processing_plant_operations_tick_system(mut game_state: ResMut<GameState>, time: Res<Time>) {
    processing_plant_operations_system(&mut game_state, time.delta_seconds());
}

// System wrapper for fabricator production
fn fabricator_production_tick_system(mut game_state: ResMut<GameState>, time: Res<Time>) {
    fabricator_production_system(&mut game_state, time.delta_seconds());
}

// System wrapper for calculate_colony_happiness
fn calculate_colony_happiness_system(mut game_state: ResMut<GameState>) {
    calculate_colony_happiness(&mut game_state);
}

// System wrapper for grow_inhabitants
fn grow_inhabitants_system(mut game_state: ResMut<GameState>) {
    grow_inhabitants(&mut game_state);
    // Note: We also need to manage game_state.habitation_structures[X].current_inhabitants
    // This is currently not fully handled by grow_inhabitants and would need a separate pass
    // or more complex logic within grow_inhabitants to distribute people into physical structures.
    // For now, `total_inhabitants` acts as the primary count, and `available_housing_capacity` as the limit.
    // Also, `update_housing_and_specialist_slots` should be called if structures change, not per tick.
}


fn game_tick_system(
    mut game_state: ResMut<GameState>,
    power_relays: Query<&PowerRelay>,
    storage_silos: Query<&StorageSilo>,
    extractors: Query<&Extractor>,
    bio_domes: Query<&BioDome>,
    research_institutes: Query<&ResearchInstitute>,
    _fabricators: Query<&Fabricator>, // Prefixed with underscore as it's unused here
) {
    let total_power_generation: u32 = power_relays.iter().map(|pr| pr.power_output).sum();
    let mut total_power_consumption: u32 = extractors.iter().map(|e| e.power_consumption).sum::<u32>()
        + bio_domes.iter().map(|b| b.power_consumption).sum::<u32>()
        + research_institutes.iter().map(|ri| ri.power_consumption).sum::<u32>();
        // Note: Bevy ECS fabricators query was removed as we are using GameState internal list for now.
        // + fabricators.iter().map(|f| f.power_consumption).sum::<u32>(); 

    // Add Administrative Spire power consumption
    if let Some(spire) = &game_state.administrative_spire {
        if spire.current_tier_index < spire.available_tiers.len() {
            total_power_consumption += spire.available_tiers[spire.current_tier_index].power_requirement;
        }
    }
    // Note: ProcessingPlant power consumption will be added if/when it has active processing.

    // Add Fabricator power consumption from GameState list
    for fab_data in &game_state.fabricators {
        if fab_data.is_active {
            if let Some(tier) = fab_data.available_tiers.get(fab_data.tier_index) {
                total_power_consumption += tier.power_requirement;
            }
        }
    }
    // Add ProcessingPlant power consumption from GameState list
    for plant_data in &game_state.processing_plants {
        if plant_data.is_active {
            if let Some(tier) = plant_data.available_tiers.get(plant_data.tier_index) {
                total_power_consumption += tier.power_requirement;
            }
        }
    }

    let net_power = total_power_generation as f32 - total_power_consumption as f32;
    
    // Store totals in GameState for UI
    game_state.total_generated_power = total_power_generation as f32;
    game_state.total_consumed_power = total_power_consumption as f32;

    let stored_power_entry = game_state.current_resources.entry(ResourceType::Power).or_insert(0.0);
    *stored_power_entry = (*stored_power_entry + net_power).max(0.0); // Ensure stored power doesn't go below zero

    // Determine if there's enough power for grid consumers (Extractors, BioDomes)
    let enough_power_for_grid_consumers = if total_power_generation >= total_power_consumption {
        true
    } else {
        let power_deficit = total_power_consumption - total_power_generation; // Deficit is positive
        let current_stored_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0);

        if current_stored_power >= power_deficit as f32 {
            // Deduct from stored power if deficit can be covered
            *game_state.current_resources.entry(ResourceType::Power).or_insert(0.0) -= power_deficit as f32;
            true 
        } else {
            // Not enough stored power to cover deficit, deplete all stored power
            *game_state.current_resources.entry(ResourceType::Power).or_insert(0.0) = 0.0;
            false 
        }
    };

    if enough_power_for_grid_consumers {
        let capacity = storage_silos.iter().map(|s| s.capacity).sum::<u32>() as f32; // Consider if capacity should be per resource type
        for dome in &bio_domes {
            let amount = game_state.current_resources.entry(ResourceType::NutrientPaste).or_insert(0.0);
            *amount = (*amount + dome.production_rate).min(capacity);
        }
        for extractor in &extractors {
            let amount = game_state.current_resources.entry(extractor.resource_type).or_insert(0.0);
            *amount = (*amount + extractor.extraction_rate).min(capacity);
        }

        // Fabricator Logic (now handled by fabricator_production_system)
        // for fabricator in &fabricators {
        //     // ... old direct logic removed ...
        // }
    }
}

fn update_colony_stats_system(
    mut stats: ResMut<ColonyStats>,
    game_state: Res<GameState>, // Added game_state parameter
    dwellings: Query<&BasicDwelling>,
    wellness_posts: Query<&WellnessPost>,
    security_stations: Query<&SecurityStation>,
) {
    // stats.total_housing = dwellings.iter().map(|d| d.housing_capacity).sum(); // This will be replaced by game_state.available_housing_capacity
    stats.total_housing = game_state.available_housing_capacity; // This line should now work
    stats.health_capacity = wellness_posts.iter().map(|p| p.health_capacity).sum();
    stats.police_capacity = security_stations.iter().map(|p| p.police_capacity).sum();
    // stats.total_jobs = wellness_posts.iter().map(|p| p.jobs_provided).sum::<u32>() // Old way
    //     + security_stations.iter().map(|p| p.jobs_provided).sum::<u32>();
    stats.total_jobs = game_state.total_specialist_slots; // This line should now work
                                                       // Consider if this should be total_specialist_slots (potential)
                                                       // or game_state.assigned_specialists_total (filled)
    // Add civic index to ColonyStats if it's displayed often, or query GameState directly in UI.
    // For now, not adding to ColonyStats to keep it focused on Bevy component queries.
    // TODO: Add game_state.colony_happiness to ColonyStats if it's to be graphed or easily displayed.
    // For now, ColonyStats is primarily for things queried from Bevy components.
    // Let's add it directly for now for simplicity in testing, can be refactored.
    stats.happiness = game_state.colony_happiness; // This line should now work
    stats.credits = game_state.credits; // This line should now work
}

fn research_system(mut game_state: ResMut<GameState>, query: Query<&ResearchInstitute>) {
    if query.is_empty() { return; }
    let mut completed_tech: Option<Tech> = None;
    if let Some((tech, progress)) = &game_state.research_progress {
        // required_progress is now u32, cast to f32 for comparison with f32 progress
        let required_progress = game_state.tech_costs[tech] as f32; 
        if progress + 1.0 >= required_progress {
            completed_tech = Some(*tech);
        }
    }
    if let Some(tech) = completed_tech {
        game_state.unlocked_techs.insert(tech);
        game_state.research_progress = None;
    } else if let Some((_, progress)) = &mut game_state.research_progress {
        *progress += 1.0;
    }
}

fn update_graph_data_system(stats: Res<ColonyStats>, mut graph_data: ResMut<GraphData>) {
    graph_data.history.push_front(*stats);
    if graph_data.history.len() > 200 { 
        graph_data.history.pop_back();
    }
}