// src/game_state.rs

use bevy::prelude::*;
use rand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU32, Ordering};

// --- Data Structs ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

// --- Administrative Spire Logic ---

pub fn construct_administrative_spire(game_state: &mut GameState) {
    if game_state.administrative_spire.is_none() {
        let all_tiers = vec![
            AdministrativeSpireTier { name: "Command Post".to_string(), power_requirement: 10, unlocks_phase: DevelopmentPhase::DP1, nutrient_paste_link_required: false, construction_credits_cost: 1000, upgrade_credits_cost: 0 },
            AdministrativeSpireTier { name: "Integrated Command".to_string(), power_requirement: 25, unlocks_phase: DevelopmentPhase::DP2, nutrient_paste_link_required: true, construction_credits_cost: 0, upgrade_credits_cost: 2500 },
            AdministrativeSpireTier { name: "Planetary Nexus".to_string(), power_requirement: 50, unlocks_phase: DevelopmentPhase::DP3, nutrient_paste_link_required: true, construction_credits_cost: 0, upgrade_credits_cost: 5000 },
        ];
        
        let initial_tier_def = &all_tiers[0];

        if game_state.credits < initial_tier_def.construction_credits_cost as f64 {
            add_notification(&mut game_state.notifications, format!("Insufficient credits for Spire."), 0.0);
            return;
        }
        game_state.credits -= initial_tier_def.construction_credits_cost as f64;
        add_notification(&mut game_state.notifications, format!("Constructed Administrative Spire."), 0.0);

        let spire = AdministrativeSpire {
            current_tier_index: 0,
            available_tiers: all_tiers,
        };
        game_state.administrative_spire = Some(spire);
        game_state.current_development_phase = DevelopmentPhase::DP1;
    }
}

pub fn upgrade_administrative_spire(game_state: &mut GameState) {
    if let Some(spire) = &mut game_state.administrative_spire {
        if spire.current_tier_index >= spire.available_tiers.len() - 1 {
            add_notification(&mut game_state.notifications, "Spire already at maximum tier.".to_string(), 0.0);
            return;
        }

        let next_tier_index = spire.current_tier_index + 1;
        let next_tier_info = &spire.available_tiers[next_tier_index];
        let current_tier_info = &spire.available_tiers[spire.current_tier_index];

        if game_state.credits < next_tier_info.upgrade_credits_cost as f64 {
            add_notification(&mut game_state.notifications, "Insufficient credits to upgrade Spire.".to_string(), 0.0);
            return;
        }
        
        let current_spire_consumption = current_tier_info.power_requirement;
        let power_consumed_by_others = game_state.total_consumed_power - current_spire_consumption as f32;
        let available_power_for_spire_upgrade = game_state.total_generated_power - power_consumed_by_others;
        
        if available_power_for_spire_upgrade < next_tier_info.power_requirement as f32 {
            add_notification(&mut game_state.notifications, "Insufficient power to upgrade Spire.".to_string(), 0.0);
            return;
        }

        if next_tier_info.nutrient_paste_link_required && game_state.current_resources.get(&ResourceType::NutrientPaste).unwrap_or(&0.0) <= &0.0 {
            add_notification(&mut game_state.notifications, "Nutrient Paste link required for Spire upgrade.".to_string(), 0.0);
            return;
        }
        
        game_state.credits -= next_tier_info.upgrade_credits_cost as f64;
        spire.current_tier_index = next_tier_index;
        game_state.current_development_phase = next_tier_info.unlocks_phase;
        
        add_notification(&mut game_state.notifications, format!("Upgraded Spire to {}.", next_tier_info.name), 0.0);

    } else {
        add_notification(&mut game_state.notifications, "Administrative Spire has not been constructed yet.".to_string(), 0.0);
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
}

// --- Habitation Data Structures ---

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
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
            unlocks_resource: Some(ResourceType::RawXylos), 
            input_resource: Some((ResourceType::RawXylos, 2)), 
            output_resource: Some((ResourceType::RefinedXylos, 1)), 
            processing_rate_per_sec: Some(0.5), 
            power_requirement: 20,
            specialist_requirement: 2,
            construction_credits_cost: 150, 
            upkeep_cost: 10,                
        },
        ProcessingPlantTier {
            name: "Quantium Resonator".to_string(),
            unlocks_resource: Some(ResourceType::RawQuantium),
            input_resource: None, 
            output_resource: None,
            processing_rate_per_sec: None,
            power_requirement: 25,
            specialist_requirement: 3,
            construction_credits_cost: 200,
            upkeep_cost: 15,
        },
         ProcessingPlantTier {
            name: "Advanced Material Synthesizer".to_string(),
            unlocks_resource: None, 
            input_resource: Some((ResourceType::CuprumDeposits, 3)), 
            output_resource: Some((ResourceType::ProcessedQuantium, 1)), 
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
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);

    let new_plant = ProcessingPlantData {
        id: generate_unique_id(),
        tier_index,
        available_tiers: all_tiers.clone(), 
        assigned_specialists: 0,
        is_active: false, 
        processing_progress: 0.0,
    };
    
    if let Some(unlocked_res) = tier_info.unlocks_resource { 
        game_state.unlocked_raw_materials.insert(unlocked_res);
    }
    game_state.processing_plants.push(new_plant);
}

pub fn upgrade_processing_plant(game_state: &mut GameState, plant_id: &str) {
    if let Some(plant) = game_state.processing_plants.iter_mut().find(|p| p.id == plant_id) {
        if plant.tier_index < plant.available_tiers.len() - 1 {
            let next_tier_index = plant.tier_index + 1;
            let next_tier_info = &plant.available_tiers[next_tier_index];
            
            let upgrade_cost = next_tier_info.construction_credits_cost; 

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
        plant.is_active = false; 
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
                plant.processing_progress = 0.0; 
                continue;
            }

            if let (Some((input_type, input_amount_per_batch)), Some((output_type, output_amount_per_batch)), Some(rate)) = 
                (tier.input_resource, tier.output_resource, tier.processing_rate_per_sec) {
                
                let potential_batches_this_tick = rate * time_delta_secs;
                plant.processing_progress += potential_batches_this_tick;

                if plant.processing_progress >= 1.0 { 
                    let num_batches_to_process = plant.processing_progress.floor();
                    let total_input_needed = input_amount_per_batch as f32 * num_batches_to_process;
                    let current_input_available = *game_state.current_resources.get(&input_type).unwrap_or(&0.0);

                    if current_input_available >= total_input_needed {
                        *game_state.current_resources.entry(input_type).or_insert(0.0) -= total_input_needed;
                        let total_output_produced = output_amount_per_batch as f32 * num_batches_to_process;
                        *game_state.current_resources.entry(output_type).or_insert(0.0) += total_output_produced;
                        
                        plant.processing_progress -= num_batches_to_process; 
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
    ManufacturedGoods,    
    AdvancedComponents,   
    RefinedXylos,         
    ProcessedQuantium,    
    RawXylos,             
    RawQuantium,          
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    Extractor,
    BioDome,
    PowerRelay,
    StorageSilo,
    ResearchInstitute,
    Fabricator, 
    ProcessingPlant, 
}

pub const ALL_BUILDING_TYPES: &[BuildingType] = &[
    BuildingType::Extractor,
    BuildingType::BioDome,
    BuildingType::PowerRelay,
    BuildingType::StorageSilo,
    BuildingType::ResearchInstitute,
    BuildingType::Fabricator,
    BuildingType::ProcessingPlant,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tech {
    BasicConstructionProtocols,
    EfficientExtraction,
}

#[derive(Component)]
pub struct Extractor {
    pub power_consumption: u32,
    pub resource_type: ResourceType,
    pub extraction_rate: f32,
    pub workforce_required: u32,
    pub is_staffed: bool,
}
#[derive(Component)]
pub struct BioDome {
    pub power_consumption: u32,
    pub production_rate: f32,
    pub workforce_required: u32,
    pub is_staffed: bool,
}
#[derive(Component)]
pub struct PowerRelay {
    pub power_output: u32,
}
#[derive(Component)]
pub struct StorageSilo {
    pub capacity: u32,
}
#[derive(Component)]
pub struct ResearchInstitute {
    pub power_consumption: u32,
    pub workforce_required: u32,
    pub is_staffed: bool,
}
#[derive(Component)]
pub struct WellnessPost {
    pub health_capacity: u32,
    pub jobs_provided: u32,
    pub workforce_required: u32,
    pub is_staffed: bool,
}
#[derive(Component)]
pub struct SecurityStation {
    pub police_capacity: u32,
    pub jobs_provided: u32,
    pub workforce_required: u32,
    pub is_staffed: bool,
}

#[derive(Component)]
pub struct Fabricator {
    pub power_consumption: u32,
    pub input_resource: ResourceType,
    pub input_amount: f32,
    pub output_resource: ResourceType,
    pub output_amount: f32,
    pub conversion_rate: f32, 
}

#[derive(Component)]
pub struct ProcessingPlant {
    pub power_consumption: u32,
}

#[derive(Resource)]
pub struct GameState {
    pub administrative_spire: Option<AdministrativeSpire>,
    pub current_development_phase: DevelopmentPhase,
    pub current_resources: HashMap<ResourceType, f32>,
    pub building_costs: HashMap<BuildingType, HashMap<ResourceType, f32>>,
    pub unlocked_techs: HashSet<Tech>,
    pub research_progress: Option<(Tech, f32)>,
    pub tech_costs: HashMap<Tech, u32>, 
    pub habitation_structures: Vec<HabitationStructure>,
    pub total_inhabitants: u32,
    pub assigned_workforce: u32,
    pub available_housing_capacity: u32,
    pub total_specialist_slots: u32, 
    pub assigned_specialists_total: u32,
    pub service_buildings: Vec<ServiceBuilding>,
    pub zones: Vec<Zone>,
    pub civic_index: u32, 
    pub colony_happiness: f32, 
    pub legacy_structure_happiness_bonus: f32,
    pub simulated_has_sufficient_nutrient_paste: bool, 
    pub fabricators: Vec<FabricatorData>, 
    pub processing_plants: Vec<ProcessingPlantData>, 
    pub unlocked_raw_materials: HashSet<ResourceType>,
    pub credits: f64,
    pub legacy_structure_income_bonus: f64,
    pub total_generated_power: f32,
    pub total_consumed_power: f32,
    pub notifications: VecDeque<NotificationEvent>,
    // Transient fields to bridge ECS queries with GameState logic
    pub transient_wellness_supply: f32,
    pub transient_security_supply: f32,
}

#[derive(Clone, Debug, PartialEq)] 
pub struct NotificationEvent {
    pub message: String,
    pub timestamp: f64, 
}

#[derive(Clone)]
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

pub struct FabricatorData { 
    pub id: String,
    pub tier_index: usize,
    pub available_tiers: Vec<FabricatorTier>,
    pub assigned_specialists: u32,
    pub is_active: bool,
    pub production_progress_secs: f32,
}

#[derive(Clone)]
pub struct ProcessingPlantTier {
    pub name: String,
    pub unlocks_resource: Option<ResourceType>,
    pub input_resource: Option<(ResourceType, u32)>,  
    pub output_resource: Option<(ResourceType, u32)>, 
    pub processing_rate_per_sec: Option<f32>, 
    pub power_requirement: u32,
    pub specialist_requirement: u32,
    pub construction_credits_cost: u32,
    pub upkeep_cost: u32, 
}

pub struct ProcessingPlantData {
    pub id: String,
    pub tier_index: usize,
    pub available_tiers: Vec<ProcessingPlantTier>,
    pub assigned_specialists: u32,
    pub is_active: bool,
    pub processing_progress: f32, 
}

#[derive(Resource, Default, Clone, Copy)]
pub struct ColonyStats {
    pub total_housing: u32,
    pub total_jobs: u32,
    pub health_capacity: u32,
    pub police_capacity: u32,
    pub happiness: f32,
    pub credits: f64,
    pub net_power: f32,
    pub nutrient_paste: f32,
}

#[derive(Resource, Default)]
pub struct GraphData {
    pub history: VecDeque<ColonyStats>, 
}

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
        
        let mut tech_costs = HashMap::new();
        tech_costs.insert(Tech::BasicConstructionProtocols, 100_u32); 
        tech_costs.insert(Tech::EfficientExtraction, 250_u32);

        let mut current_resources = HashMap::new();
        current_resources.insert(ResourceType::NutrientPaste, 50.0);
        current_resources.insert(ResourceType::FerrocreteOre, 200.0);
        current_resources.insert(ResourceType::CuprumDeposits, 50.0);
        current_resources.insert(ResourceType::Power, 100.0);
        current_resources.insert(ResourceType::ManufacturedGoods, 0.0);
        current_resources.insert(ResourceType::AdvancedComponents, 0.0);
        current_resources.insert(ResourceType::RefinedXylos, 0.0);
        current_resources.insert(ResourceType::ProcessedQuantium, 0.0);
        current_resources.insert(ResourceType::RawXylos, 0.0); 
        current_resources.insert(ResourceType::RawQuantium, 0.0);

        let mut new_state = Self {
            administrative_spire: None,
            current_development_phase: DevelopmentPhase::default(),
            current_resources,
            building_costs,
            unlocked_techs: HashSet::new(),
            research_progress: None,
            tech_costs,
            habitation_structures: Vec::new(),
            total_inhabitants: 5,
            assigned_workforce: 0,
            available_housing_capacity: 0,
            total_specialist_slots: 0, 
            assigned_specialists_total: 0,
            service_buildings: Vec::new(),
            zones: Vec::new(),
            civic_index: 0, 
            colony_happiness: 50.0, 
            legacy_structure_happiness_bonus: 0.0,
            simulated_has_sufficient_nutrient_paste: true, 
            fabricators: Vec::new(),
            processing_plants: Vec::new(),
            unlocked_raw_materials: HashSet::new(),
            credits: 10000.0, 
            legacy_structure_income_bonus: 0.0,
            total_generated_power: 0.0,
            total_consumed_power: 0.0,
            notifications: VecDeque::new(), 
            transient_wellness_supply: 0.0,
            transient_security_supply: 0.0,
        };

        Self::add_notification_internal(&mut new_state.notifications, "Colony established. Welcome, Commander!".to_string(), 0.0);
        Self::add_notification_internal(&mut new_state.notifications, "Low power levels detected early in the simulation.".to_string(), 0.1);
        Self::add_notification_internal(&mut new_state.notifications, "A strange signal was briefly detected from the nearby asteroid belt.".to_string(), 0.2);

        new_state
    }
}

impl GameState {
    fn add_notification_internal(notifications_vec: &mut VecDeque<NotificationEvent>, message: String, timestamp: f64) {
        notifications_vec.push_front(NotificationEvent {
            message,
            timestamp,
        });
        if notifications_vec.len() > 20 {
            notifications_vec.pop_back();
        }
    }
}

pub fn add_notification(notifications: &mut VecDeque<NotificationEvent>, message: String, current_time_seconds: f64) {
    GameState::add_notification_internal(notifications, message, current_time_seconds);
}

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

fn check_fabricator_inputs(
    current_resources: &HashMap<ResourceType, f32>,
    fabricator_tier: &FabricatorTier 
) -> bool {
    for (resource_type, required_amount) in &fabricator_tier.input_resources {
        if current_resources.get(resource_type).unwrap_or(&0.0) < &(*required_amount as f32) {
            return false; 
        }
    }
    true 
}


pub fn add_fabricator(game_state: &mut GameState, tier_index: usize) {
    let all_tiers = get_fabricator_tiers();
    if tier_index >= all_tiers.len() {
        println!("Error: Invalid tier index for fabricator.");
        return;
    }
    let tier_info = &all_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);

    let new_fabricator = FabricatorData {
        id: generate_unique_id(),
        tier_index,
        available_tiers: all_tiers.clone(),
        assigned_specialists: 0,
        is_active: false, 
        production_progress_secs: 0.0,
    };
    game_state.fabricators.push(new_fabricator);
}

pub fn upgrade_fabricator(game_state: &mut GameState, fabricator_id: &str) {
    if let Some(fab) = game_state.fabricators.iter_mut().find(|f| f.id == fabricator_id) {
        if fab.tier_index < fab.available_tiers.len() - 1 {
            let next_tier_index = fab.tier_index + 1;
            let next_tier_info = &fab.available_tiers[next_tier_index];
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
        fab.is_active = false; 
        fab.production_progress_secs = 0.0; 
        println!("Unassigned {} specialists from Fabricator {}.", actual_unassign, fab_id);
    } else {
        println!("Fabricator {} not found.", fab_id);
    }
}

pub fn fabricator_production_system(game_state: &mut GameState, time_delta_secs: f32) {
    for fab in game_state.fabricators.iter_mut() {
        if let Some(tier) = fab.available_tiers.get(fab.tier_index) {
            let has_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0) >= tier.power_requirement as f32;
            let has_specialists = fab.assigned_specialists >= tier.specialist_requirement;
            let has_inputs = check_fabricator_inputs(&game_state.current_resources, tier);

            fab.is_active = has_power && has_specialists && has_inputs;

            if fab.is_active {
                fab.production_progress_secs += time_delta_secs;
                if fab.production_progress_secs >= tier.production_time_secs {
                    for (resource_type, required_amount) in &tier.input_resources {
                        *game_state.current_resources.entry(*resource_type).or_insert(0.0) -= *required_amount as f32;
                    }
                    *game_state.current_resources.entry(tier.output_product).or_insert(0.0) += tier.output_quantity as f32;
                    // println!("Fabricator {} produced {} {} (total now: {}).", fab.id, tier.output_quantity, format!("{:?}", tier.output_product), game_state.current_resources.get(&tier.output_product).unwrap());
                    fab.production_progress_secs = 0.0; 
                }
            }
        }
    }
}

pub fn calculate_colony_happiness(game_state: &mut GameState) {
    let mut happiness_score = 50.0; 

    // --- POSITIVE MODIFIERS ---
    // Food Bonus
    if game_state.simulated_has_sufficient_nutrient_paste {
        happiness_score += 10.0; 
    }

    // Housing Bonus
    if game_state.available_housing_capacity > 0 && game_state.total_inhabitants > 0 {
        let occupancy_ratio = game_state.total_inhabitants as f32 / game_state.available_housing_capacity as f32;
        if occupancy_ratio <= 0.9 { 
            happiness_score += 5.0; // Plenty of space
        } else if occupancy_ratio < 1.0 {
            happiness_score += 2.0; // A bit tight
        }
    }
    
    // Employment Bonus
    if game_state.total_inhabitants > 0 {
        let employment_ratio = game_state.assigned_workforce as f32 / game_state.total_inhabitants as f32;
        if employment_ratio > 0.7 { // High employment
            happiness_score += 5.0;
        }
    }
    
    // Civic & Legacy Bonuses
    happiness_score += game_state.legacy_structure_happiness_bonus;
    happiness_score += (game_state.civic_index as f32 / 10.0).min(5.0);


    // --- NEGATIVE MODIFIERS ---
    // Food Penalty
    if !game_state.simulated_has_sufficient_nutrient_paste {
        happiness_score -= 25.0; 
    }
    
    // Housing Penalty (Homelessness)
    if game_state.total_inhabitants > game_state.available_housing_capacity {
        let homeless = game_state.total_inhabitants - game_state.available_housing_capacity;
        happiness_score -= (homeless as f32) * 2.0; // Harsh penalty for each homeless person
    }

    // --- NEEDS-BASED MODIFIERS (HEALTH & SECURITY) ---
    let service_types_to_check = [ServiceType::Wellness, ServiceType::Security];
    for service_type in service_types_to_check.iter() {
        // Each building provides 1 "unit" of service coverage
        let demand = (game_state.total_inhabitants as f32 / 50.0).ceil(); // 1 service building needed per 50 inhabitants
        let mut supply = 0.0;

        // Add supply from staffed legacy component buildings
        match service_type {
            ServiceType::Wellness => supply += game_state.transient_wellness_supply,
            ServiceType::Security => supply += game_state.transient_security_supply,
            _ => {}
        }

        // Add supply from staffed service buildings (newer struct-based system)
        for building in &game_state.service_buildings {
            if building.service_type == *service_type && building.is_active {
                if let Some(tier) = building.available_tiers.get(building.current_tier_index) {
                    if building.assigned_specialists >= tier.specialist_requirement {
                        supply += 1.0; 
                    }
                }
            }
        }

        // Apply happiness modifier based on supply vs. demand
        if supply >= demand && demand > 0.0 {
            happiness_score += 10.0; // Needs fully met
        } else if demand > 0.0 {
            // Needs not met, apply penalty
            let coverage_ratio = supply / demand;
            happiness_score -= (1.0 - coverage_ratio) * 15.0; // Penalty scales with lack of coverage
        }
    }
    
    // Clamp final score
    game_state.colony_happiness = happiness_score.clamp(0.0, 100.0);
}


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
}

static NEXT_ID: AtomicU32 = AtomicU32::new(0);

fn generate_unique_id() -> String {
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
        return;
    }
    
    let tier_info = &all_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 {
        add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0);
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    
    let new_structure = HabitationStructure {
        id: generate_unique_id(),
        tier_index, 
        available_tiers: all_tiers.clone(), 
        current_inhabitants: 0, 
        assigned_specialists: 0,
    };
    game_state.habitation_structures.push(new_structure);
    update_housing_and_specialist_slots(game_state);
}

pub fn upgrade_habitation_structure(game_state: &mut GameState, structure_id: &str) {
    if let Some(structure) = game_state.habitation_structures.iter_mut().find(|s| s.id == structure_id) {
        if structure.tier_index < structure.available_tiers.len() - 1 {
            let next_tier_index = structure.tier_index + 1;
            let next_tier_info = &structure.available_tiers[next_tier_index];
            
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
    if let Some(index) = game_state.habitation_structures.iter().position(|s| s.id == structure_id) {
        let removed_structure = game_state.habitation_structures.remove(index);
        println!("Removed Habitation Structure with ID {}", structure_id);

        update_housing_and_specialist_slots(game_state); 

        game_state.total_inhabitants = game_state.total_inhabitants.saturating_sub(removed_structure.current_inhabitants);
        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(removed_structure.assigned_specialists);

    } else {
        println!("Habitation Structure with ID {} not found for removal.", structure_id);
        return;
    }

    if game_state.total_inhabitants > game_state.available_housing_capacity {
        game_state.total_inhabitants = game_state.available_housing_capacity;
    }
    game_state.assigned_specialists_total = game_state.assigned_specialists_total.min(game_state.total_inhabitants).min(game_state.total_specialist_slots);
}

pub fn grow_inhabitants(game_state: &mut GameState) {
    // Growth is probabilistic, based on happiness.
    // A score of 50 is neutral. Below 50 risks population decline (not implemented yet).
    // Above 50 increases growth chance.
    let happiness_factor = (game_state.colony_happiness - 50.0) / 50.0; // Range from -1.0 to 1.0

    if happiness_factor <= 0.0 {
        return; // No growth if happiness is 50 or less
    }

    // The chance to gain an inhabitant per second.
    // At 100 happiness, chance is 0.1 (or 10%) per second.
    // At 75 happiness, chance is 0.05 (or 5%) per second.
    let growth_chance_per_sec = happiness_factor * 0.1; 

    if rand::thread_rng().gen::<f32>() < growth_chance_per_sec {
        if game_state.total_inhabitants < game_state.available_housing_capacity {
            let growth_amount = 1; 
            game_state.total_inhabitants += growth_amount;
        }
    }
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
            if structure.assigned_specialists + num_to_assign > structure.current_inhabitants {
                 println!("Cannot assign more specialists than current inhabitants in the structure.");
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

pub fn get_service_building_tiers(service_type: ServiceType) -> Vec<ServiceBuildingTier> {
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
    let tier_info = &all_tiers[tier_index]; 
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
        is_active: true, 
        position,
    };
    game_state.service_buildings.push(new_building);
    update_civic_index(game_state); 
    println!("Added Service Building: {}", tier_info.name);
}

pub fn upgrade_service_building(game_state: &mut GameState, building_id: &str) {
    let mut upgraded = false;
    if let Some(building) = game_state.service_buildings.iter_mut().find(|b| b.id == building_id) {
        if building.current_tier_index < building.available_tiers.len() - 1 {
            let next_tier_index = building.current_tier_index + 1;
            let upgrade_cost = building.available_tiers[next_tier_index].construction_credits_cost;
            let next_tier_name = building.available_tiers[next_tier_index].name.clone();
            let specialist_requirement_next_tier = building.available_tiers[next_tier_index].specialist_requirement;

            if game_state.credits < upgrade_cost as f64 {
                println!("Not enough credits to upgrade Service Building {} to {}. Required: {}, Available: {:.2}", building_id, next_tier_name, upgrade_cost, game_state.credits);
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
    } else {
        println!("Service Building {} not found for specialist unassignment.", building_id);
    }
}

pub fn get_zone_tiers(zone_type: ZoneType) -> Vec<ZoneTier> {
    match zone_type {
        ZoneType::Commercial => vec![
            ZoneTier { name: "Small Market Stalls".to_string(), specialist_jobs_provided: 5, civic_index_contribution: 3, upkeep_cost: 10, construction_credits_cost: 100, income_generation: 50 },
            ZoneTier { name: "Shopping Plaza".to_string(), specialist_jobs_provided: 15, civic_index_contribution: 10, upkeep_cost: 30, construction_credits_cost: 300, income_generation: 200 },
        ],
        ZoneType::LightIndustry => vec![
            ZoneTier { name: "Workshops".to_string(), specialist_jobs_provided: 8, civic_index_contribution: 2, upkeep_cost: 15, construction_credits_cost: 120, income_generation: 0 }, 
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
    let tier_info = &all_tiers[tier_index]; 
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
        is_active: true, 
    };
    game_state.zones.push(new_zone);
    update_total_specialist_slots(game_state); 
    update_civic_index(game_state);
    println!("Added Zone: {}", tier_info.name);
}

pub fn upgrade_zone(game_state: &mut GameState, zone_id: &str) {
    let mut upgraded = false;
    if let Some(zone) = game_state.zones.iter_mut().find(|z| z.id == zone_id) {
        if zone.current_tier_index < zone.available_tiers.len() - 1 {
            let next_tier_index = zone.current_tier_index + 1;
            let next_tier_info = &zone.available_tiers[next_tier_index];

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
            println!("Upgraded Zone {} to {}", zone_id, next_tier_info.name); 
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
    } else {
        println!("Zone {} not found for specialist unassignment.", zone_id);
    }
}

pub fn update_total_specialist_slots(game_state: &mut GameState) {
    let mut total_slots = 0;
    for structure in &game_state.habitation_structures {
        if let Some(tier) = structure.available_tiers.get(structure.tier_index) {
            total_slots += tier.specialist_slots; 
        }
    }
    for zone in &game_state.zones {
        if zone.is_active {
            if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                total_slots += tier.specialist_jobs_provided; 
            }
        }
    }
    game_state.total_specialist_slots = total_slots;
}

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .init_resource::<ColonyStats>()
            .init_resource::<GraphData>()
            .add_systems(FixedUpdate, (
                workforce_assignment_system,
                game_tick_system.after(workforce_assignment_system),
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

    let initial_credits = game_state.credits; 

    let mut potential_total_upkeep = 0.0;
    for building in &game_state.service_buildings { if building.is_active { if let Some(tier) = building.available_tiers.get(building.current_tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }
    for zone in &game_state.zones { if zone.is_active { if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }
    for fab in &game_state.fabricators { if fab.is_active { if let Some(tier) = fab.available_tiers.get(fab.tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }
    for plant in &game_state.processing_plants { if plant.is_active { if let Some(tier) = plant.available_tiers.get(plant.tier_index) { potential_total_upkeep += tier.upkeep_cost as f64; } } }

    if initial_credits >= potential_total_upkeep {
        total_upkeep_to_deduct_this_tick = potential_total_upkeep;
    } else {
        println!("Warning: Insufficient credits ({:.2}) to cover all upkeep ({:.2}). Deactivating buildings.", initial_credits, potential_total_upkeep);
        
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
    }

    if total_upkeep_to_deduct_this_tick > 0.0 {
        game_state.credits -= total_upkeep_to_deduct_this_tick; 
        if game_state.credits < 0.0 { 
            println!("Critical Warning: Credits still went negative ({:.2}) after deactivations!", game_state.credits);
            game_state.credits = 0.0; 
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

fn processing_plant_operations_tick_system(mut game_state: ResMut<GameState>, time: Res<Time>) {
    processing_plant_operations_system(&mut game_state, time.delta_seconds());
}

fn fabricator_production_tick_system(mut game_state: ResMut<GameState>, time: Res<Time>) {
    fabricator_production_system(&mut game_state, time.delta_seconds());
}

fn calculate_colony_happiness_system(
    mut game_state: ResMut<GameState>,
    wellness_posts: Query<&WellnessPost>,
    security_stations: Query<&SecurityStation>,
) {
    // Bridge the ECS component data to the GameState resource before calculation
    game_state.transient_wellness_supply = wellness_posts.iter().filter(|p| p.is_staffed).count() as f32;
    game_state.transient_security_supply = security_stations.iter().filter(|s| s.is_staffed).count() as f32;

    calculate_colony_happiness(&mut game_state);
}


fn grow_inhabitants_system(mut game_state: ResMut<GameState>) {
    grow_inhabitants(&mut game_state);
}

fn workforce_assignment_system(
    mut game_state: ResMut<GameState>,
    mut extractors: Query<&mut Extractor>,
    mut bio_domes: Query<&mut BioDome>,
    mut research_institutes: Query<&mut ResearchInstitute>,
    mut wellness_posts: Query<&mut WellnessPost>,
    mut security_stations: Query<&mut SecurityStation>,
) {
    // This system only handles the "legacy" workforce buildings.
    // Newer systems like FabricatorData handle their specialist assignment separately.
    let required_workforce: u32 = extractors.iter().map(|e| e.workforce_required).sum::<u32>()
        + bio_domes.iter().map(|b| b.workforce_required).sum::<u32>()
        + research_institutes.iter().map(|ri| ri.workforce_required).sum::<u32>()
        + wellness_posts.iter().map(|wp| wp.workforce_required).sum::<u32>()
        + security_stations.iter().map(|ss| ss.workforce_required).sum::<u32>();
    
    game_state.assigned_workforce = required_workforce.min(game_state.total_inhabitants);
    let mut available_workforce = game_state.assigned_workforce;

    // This is a simple greedy assignment. A more complex system could prioritize.
    for mut extractor in &mut extractors {
        if available_workforce >= extractor.workforce_required {
            extractor.is_staffed = true;
            available_workforce -= extractor.workforce_required;
        } else {
            extractor.is_staffed = false;
        }
    }
    for mut dome in &mut bio_domes {
        if available_workforce >= dome.workforce_required {
            dome.is_staffed = true;
            available_workforce -= dome.workforce_required;
        } else {
            dome.is_staffed = false;
        }
    }
    for mut institute in &mut research_institutes {
        if available_workforce >= institute.workforce_required {
            institute.is_staffed = true;
            available_workforce -= institute.workforce_required;
        } else {
            institute.is_staffed = false;
        }
    }
    for mut post in &mut wellness_posts {
        if available_workforce >= post.workforce_required {
            post.is_staffed = true;
            available_workforce -= post.workforce_required;
        } else {
            post.is_staffed = false;
        }
    }
    for mut station in &mut security_stations {
        if available_workforce >= station.workforce_required {
            station.is_staffed = true;
            available_workforce -= station.workforce_required;
        } else {
            station.is_staffed = false;
        }
    }
}


fn game_tick_system(
    mut game_state: ResMut<GameState>,
    power_relays: Query<&PowerRelay>,
    storage_silos: Query<&StorageSilo>,
    extractors: Query<&Extractor>,
    bio_domes: Query<&BioDome>,
    research_institutes: Query<&ResearchInstitute>,
) {
    // --- Power Calculation ---
    let total_power_generation: u32 = power_relays.iter().map(|pr| pr.power_output).sum();
    
    let mut total_power_consumption: u32 = 0;
    total_power_consumption += extractors.iter().filter(|e| e.is_staffed).map(|e| e.power_consumption).sum::<u32>();
    total_power_consumption += bio_domes.iter().filter(|b| b.is_staffed).map(|b| b.power_consumption).sum::<u32>();
    total_power_consumption += research_institutes.iter().filter(|ri| ri.is_staffed).map(|ri| ri.power_consumption).sum::<u32>();
    
    // Power for administrative spire
    if let Some(spire) = &game_state.administrative_spire {
        if let Some(tier) = spire.available_tiers.get(spire.current_tier_index) {
            total_power_consumption += tier.power_requirement;
        }
    }

    // Power for active fabricators and plants
    for fab_data in &game_state.fabricators {
        if fab_data.is_active {
            if let Some(tier) = fab_data.available_tiers.get(fab_data.tier_index) {
                total_power_consumption += tier.power_requirement;
            }
        }
    }
    for plant_data in &game_state.processing_plants {
        if plant_data.is_active {
            if let Some(tier) = plant_data.available_tiers.get(plant_data.tier_index) {
                total_power_consumption += tier.power_requirement;
            }
        }
    }

    let net_power = total_power_generation as f32 - total_power_consumption as f32;
    game_state.total_generated_power = total_power_generation as f32;
    game_state.total_consumed_power = total_power_consumption as f32;
    
    // Determine if there's a power deficit that needs to be covered by stored power
    let power_deficit = if net_power < 0.0 { -net_power } else { 0.0 };
    let stored_power = game_state.current_resources.entry(ResourceType::Power).or_insert(0.0);
    
    let has_sufficient_power = if net_power >= 0.0 {
        *stored_power += net_power; // Add surplus to storage
        true
    } else {
        if *stored_power >= power_deficit {
            *stored_power -= power_deficit; // Cover deficit from storage
            true
        } else {
            *stored_power = 0.0; // Drain all stored power
            false // Power outage!
        }
    };
    
    // --- Resource Production (only if powered) ---
    if has_sufficient_power {
        let capacity: u32 = storage_silos.iter().map(|s| s.capacity).sum();
        let total_capacity = 1000 + capacity; // Base capacity + silo capacity
        for dome in bio_domes.iter().filter(|d| d.is_staffed) {
            let amount = game_state.current_resources.entry(ResourceType::NutrientPaste).or_insert(0.0);
            *amount = (*amount + dome.production_rate).min(total_capacity as f32);
        }
        for extractor in extractors.iter().filter(|e| e.is_staffed) {
            let amount = game_state.current_resources.entry(extractor.resource_type).or_insert(0.0);
            *amount = (*amount + extractor.extraction_rate).min(total_capacity as f32);
        }
    }
    
    // Update food status for happiness calculation
    game_state.simulated_has_sufficient_nutrient_paste = game_state.current_resources.get(&ResourceType::NutrientPaste).unwrap_or(&0.0) > &0.0;
}


fn update_colony_stats_system(
    mut stats: ResMut<ColonyStats>,
    game_state: Res<GameState>, 
    wellness_posts: Query<&WellnessPost>,
    security_stations: Query<&SecurityStation>,
) {
    stats.total_housing = game_state.available_housing_capacity; 
    stats.health_capacity = wellness_posts.iter().map(|p| p.health_capacity).sum();
    stats.police_capacity = security_stations.iter().map(|p| p.police_capacity).sum();
    stats.total_jobs = game_state.assigned_workforce; 
    stats.happiness = game_state.colony_happiness;
    stats.credits = game_state.credits;
    stats.net_power = game_state.total_generated_power - game_state.total_consumed_power;
    stats.nutrient_paste = *game_state.current_resources.get(&ResourceType::NutrientPaste).unwrap_or(&0.0);
}

fn research_system(mut game_state: ResMut<GameState>, query: Query<&ResearchInstitute>) {
    if query.iter().all(|ri| !ri.is_staffed) { return; }
    let mut completed_tech: Option<Tech> = None;
    if let Some((tech, progress)) = &game_state.research_progress {
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