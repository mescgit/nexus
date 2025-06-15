// src/game_state.rs

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU32, Ordering};

// Import new building components
// Use the tier types from components::building directly
use crate::components::building::{
    AdministrativeSpireTier, Building, BuildingVariant, LegacyStructureTier,
    BioDomeTier, ExtractorTier, FabricatorTier, HabitationTier, PowerRelayTier,
    ProcessingPlantTier, ResearchInstituteTier, ServiceTier, StorageSiloTier, ZoneTier,
};

// --- Core Enums & Basic Structs ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DevelopmentPhase { DP1, DP2, DP3 }
impl Default for DevelopmentPhase { fn default() -> Self { DevelopmentPhase::DP1 } }

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ResourceType {
    FerrocreteOre, NutrientPaste, CuprumDeposits, Power,
    ManufacturedGoods, AdvancedComponents, RefinedXylos, ProcessedQuantium,
    RawXylos, RawQuantium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tech {
    BasicConstructionProtocols, EfficientExtraction, AdvancedFabrication,
    IndustrialProcessing, ZoningOrdinances, ArcologyConstruction,
}

// Enums that will be used by BuildingVariant and specific Tier data
// These are defined in components::building.rs but also needed here for some systems if they use them directly.
// For now, keep them here if systems depend on them directly.
// If not, they can be removed from here and used via components::building::ServiceType etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType { Wellness, Security, Education, Recreation, Spiritual, }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneType { Commercial, LightIndustry, }


// Local definitions of LegacyStructureTier and AdministrativeSpireTier are removed.
// They are now imported directly from crate::components::building.

// These structs still need to be defined locally if their fields are different from
// what BuildingVariant would hold, or if they are accessed directly on GameState.
// For now, we ensure their `available_tiers` field uses the imported Tier types.
#[derive(Serialize, Deserialize, Clone)]
pub struct LegacyStructure {
    pub current_tier_index: usize,
    pub available_tiers: Vec<LegacyStructureTier>, // Uses imported LegacyStructureTier
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AdministrativeSpire {
    pub current_tier_index: usize,
    pub available_tiers: Vec<AdministrativeSpireTier>, // Uses imported AdministrativeSpireTier
}

// --- GameState Struct Definition ---
#[derive(Resource, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub administrative_spire: Option<AdministrativeSpire>,
    pub legacy_structure: Option<LegacyStructure>,
    pub current_development_phase: DevelopmentPhase,
    pub current_resources: HashMap<ResourceType, f32>,
    pub unlocked_techs: HashSet<Tech>,
    pub research_progress: Option<(Tech, f32)>,
    pub tech_costs: HashMap<Tech, u32>,
    pub total_inhabitants: u32,
    pub assigned_workforce: u32,
    pub available_housing_capacity: u32,
    pub total_specialist_slots: u32,
    pub assigned_specialists_total: u32,
    pub civic_index: u32,
    pub colony_happiness: f32,
    pub simulated_has_sufficient_nutrient_paste: bool,
    pub unlocked_raw_materials: HashSet<ResourceType>,
    pub credits: f64,
    pub total_generated_power: f32,
    pub total_consumed_power: f32,
    pub notifications: VecDeque<NotificationEvent>,
    pub buildings: Vec<Building>,
}

impl Default for GameState {
    fn default() -> Self {
        let mut tech_costs = HashMap::new();
        tech_costs.insert(Tech::BasicConstructionProtocols, 100);
        tech_costs.insert(Tech::EfficientExtraction, 250);
        tech_costs.insert(Tech::AdvancedFabrication, 500);
        tech_costs.insert(Tech::IndustrialProcessing, 500);
        tech_costs.insert(Tech::ZoningOrdinances, 400);
        tech_costs.insert(Tech::ArcologyConstruction, 1000);

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
            legacy_structure: None,
            current_development_phase: DevelopmentPhase::default(),
            current_resources,
            unlocked_techs: HashSet::new(),
            research_progress: None,
            tech_costs,
            total_inhabitants: 5,
            assigned_workforce: 0,
            available_housing_capacity: 0,
            total_specialist_slots: 0,
            assigned_specialists_total: 0,
            civic_index: 0,
            colony_happiness: 50.0,
            simulated_has_sufficient_nutrient_paste: true,
            unlocked_raw_materials: HashSet::new(),
            credits: 10000.0,
            total_generated_power: 0.0,
            total_consumed_power: 0.0,
            notifications: VecDeque::new(),
            buildings: Vec::new(),
        };
        Self::add_notification_internal(&mut new_state.notifications, "Colony established. Welcome, Commander!".to_string(), 0.0);
        new_state
    }
}

pub fn toggle_building_active_state_by_id(game_state: &mut GameState, building_id: &str) {
    let mut building_found = false;
    let mut new_active_state = false;
    let mut requires_civic_index_update = false;
    let mut requires_job_slot_update = false;
    let mut building_name_for_notification = "Building".to_string();


    if let Some(building) = game_state.buildings.iter_mut().find(|b| b.id == building_id) {
        building_found = true;
        building.is_active = !building.is_active;
        new_active_state = building.is_active;

        building_name_for_notification = match &building.variant {
            BuildingVariant::Extractor { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Extractor".to_string(), |t| t.name.clone()),
            BuildingVariant::BioDome { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "BioDome".to_string(), |t| t.name.clone()),
            BuildingVariant::PowerRelay { .. } => "Power Relay".to_string(), // Simple name, no specific tier name needed for notification
            BuildingVariant::ResearchInstitute { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Research Institute".to_string(), |t| t.name.clone()),
            BuildingVariant::StorageSilo { .. } => "Storage Silo".to_string(), // Simple name
            BuildingVariant::Habitation { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Habitation".to_string(), |t| t.name.clone()),
            BuildingVariant::Service { available_tiers, service_type } => format!("{:?} {}", service_type, available_tiers.get(building.current_tier_index).map_or_else(|| "".to_string(), |t| t.name.clone())),
            BuildingVariant::Zone { available_tiers, zone_type } => format!("{:?} {}", zone_type, available_tiers.get(building.current_tier_index).map_or_else(|| "".to_string(), |t| t.name.clone())),
            BuildingVariant::Fabricator { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Fabricator".to_string(), |t| t.name.clone()),
            BuildingVariant::ProcessingPlant { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Processing Plant".to_string(), |t| t.name.clone()),
            // Admin Spire and Legacy Structures are not in the `buildings` Vec, so not handled here.
            _ => "Building".to_string(),
        };

        // Determine if updates are needed based on building type
        match &building.variant {
            BuildingVariant::Service { .. } | BuildingVariant::Zone { .. } => {
                requires_civic_index_update = true;
                requires_job_slot_update = true;
            }
            BuildingVariant::Extractor { .. } |
            BuildingVariant::BioDome { .. } |
            BuildingVariant::ResearchInstitute { .. } |
            BuildingVariant::Fabricator { .. } |
            BuildingVariant::ProcessingPlant { .. } |
            BuildingVariant::Habitation { .. } => { // Habitation provides specialist slots which are part of total_specialist_slots
                requires_job_slot_update = true;
            }
            // PowerRelay and StorageSilo changes in active state don't directly affect civic index or job slots
            // but might affect power calculation or storage capacity, handled elsewhere.
            _ => {}
        }
    }

    if building_found {
        let status_message = if new_active_state { "activated" } else { "deactivated" };
        add_notification(&mut game_state.notifications, format!("{} {} {}.", building_name_for_notification, building_id, status_message), 0.0);

        if requires_civic_index_update {
            update_civic_index(game_state);
        }
        if requires_job_slot_update {
            // If a building is deactivated, its workforce should be unassigned.
            // If it's activated, workforce can be assigned later.
            // For simplicity here, we ensure assigned workforce is 0 if deactivated.
            // More complex logic could remember assigned workforce for reactivation.
            if !new_active_state {
                 if let Some(building) = game_state.buildings.iter_mut().find(|b| b.id == building_id) {
                    if building.assigned_workforce > 0 {
                        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(building.assigned_workforce);
                        building.assigned_workforce = 0;
                         add_notification(&mut game_state.notifications, format!("Workforce unassigned from deactivated {}.", building_name_for_notification), 0.0);
                    }
                 }
            }
            update_total_specialist_slots(game_state); // Recalculate total slots based on active buildings
        }
        // Note: Power update (total_generated_power, total_consumed_power) should be triggered globally
        // if a PowerRelay or any power consumer/producer changes state. This function doesn't do it directly.
    } else {
        add_notification(&mut game_state.notifications, format!("Building ID {} not found for toggling active state.", building_id), 0.0);
    }
}

// --- Notification System ---
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub message: String,
    pub timestamp: f64,
}

impl GameState {
    fn add_notification_internal(notifications_vec: &mut VecDeque<NotificationEvent>, message: String, timestamp: f64) {
        notifications_vec.push_front(NotificationEvent { message, timestamp });
        if notifications_vec.len() > 20 { notifications_vec.pop_back(); }
    }
}

pub fn add_notification(notifications: &mut VecDeque<NotificationEvent>, message: String, current_time_seconds: f64) {
    GameState::add_notification_internal(notifications, message, current_time_seconds);
}

// --- ID Generation ---
static NEXT_ID: AtomicU32 = AtomicU32::new(0);
pub fn generate_unique_id() -> String {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    format!("struct_{}", id)
}

// --- Utility Structs for Stats and Graphing ---
#[derive(Resource, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ColonyStats { pub total_housing: u32, pub total_jobs: u32, pub happiness: f32, pub credits: f64, pub net_power: f32, pub nutrient_paste: f32 }

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct ServiceCoverage { pub coverage: HashMap<ServiceType, f32>, }
#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct GraphData { pub history: VecDeque<ColonyStats>, }

// --- Game Logic Plugin (systems heavily commented out) ---
pub struct GameLogicPlugin;
impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .init_resource::<ServiceCoverage>()
            .init_resource::<ColonyStats>()
            .init_resource::<GraphData>()
            .add_event::<SaveGameEvent>()
            .add_event::<LoadGameEvent>()
            // .add_systems( FixedUpdate, ( /* ... systems ... */ ) )
            .add_systems(Update, (save_game_system, load_game_system));
    }
}

// --- Save/Load Logic (kept for now) ---
#[derive(Event)]
pub struct SaveGameEvent;
#[derive(Event)]
pub struct LoadGameEvent;
const SAVE_PATH: &str = "save.json";

fn save_game_system(game_state: Res<GameState>, mut save_event_reader: EventReader<SaveGameEvent>) {
    if save_event_reader.read().last().is_some() {
        match File::create(SAVE_PATH) {
            Ok(mut file) => {
                let state_json = serde_json::to_string_pretty(&*game_state).unwrap();
                if let Err(e) = file.write_all(state_json.as_bytes()) { println!("Error writing to save file: {}", e); }
                else { println!("Game Saved."); }
            }
            Err(e) => { println!("Error creating save file: {}", e); }
        }
    }
}
fn load_game_system(mut commands: Commands, mut load_event_reader: EventReader<LoadGameEvent>) {
    if load_event_reader.read().last().is_some() {
        match File::open(SAVE_PATH) {
            Ok(mut file) => {
                let mut json_str = String::new();
                if file.read_to_string(&mut json_str).is_ok() {
                    match serde_json::from_str::<GameState>(&json_str) {
                        Ok(mut loaded_state) => {
                            add_notification(&mut loaded_state.notifications, "Game Loaded.".to_string(), 0.0);
                            commands.insert_resource(loaded_state);
                            println!("Game loaded successfully from {}", SAVE_PATH);
                        }
                        Err(e) => println!("Error deserializing game state: {}", e),
                    }
                }
            }
            Err(e) => println!("Error opening save file: {}", e),
        }
    }
}

// --- Placeholder for functions and systems to be added back ---

// --- Habitation Structure Functions ---
pub fn get_new_habitation_tiers() -> Vec<HabitationTier> {
    vec![
        HabitationTier {
            name: "Basic Dwellings".to_string(),
            construction_credits_cost: 100,
            upkeep_cost: 5,
            power_requirement: 2,
            workforce_requirement: 1, // Maintenance staff
            required_tech: None,
            housing_capacity: 10,
            specialist_slots: 1,
        },
        HabitationTier {
            name: "Community Blocks".to_string(),
            construction_credits_cost: 250,
            upkeep_cost: 10,
            power_requirement: 5,
            workforce_requirement: 2,
            required_tech: None,
            housing_capacity: 25,
            specialist_slots: 3,
        },
        HabitationTier {
            name: "Arcology Spires".to_string(),
            construction_credits_cost: 1000,
            upkeep_cost: 25,
            power_requirement: 10,
            workforce_requirement: 5,
            required_tech: Some(Tech::ArcologyConstruction),
            housing_capacity: 100,
            specialist_slots: 10,
        },
    ]
}

pub fn add_habitation_structure(
    game_state: &mut GameState,
    tier_index: usize,
    position: Option<(f32, f32)>,
) {
    let all_available_tiers = get_new_habitation_tiers();
    if tier_index >= all_available_tiers.len() {
        println!("Error: Invalid tier index {} for habitation structure.", tier_index);
        return;
    }

    let tier_info = &all_available_tiers[tier_index];

    if game_state.credits < tier_info.construction_credits_cost as f64 {
        add_notification(
            &mut game_state.notifications,
            format!("Not enough credits to build {}.", tier_info.name),
            0.0,
        );
        return;
    }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification(
        &mut game_state.notifications,
        format!("Built {}.", tier_info.name),
        0.0,
    );

    let new_building = Building {
        id: generate_unique_id(),
        variant: BuildingVariant::Habitation {
            current_inhabitants: 0,
            available_tiers: all_available_tiers,
        },
        position,
        current_tier_index: tier_index,
        is_active: true,
        assigned_workforce: 0,
    };
    game_state.buildings.push(new_building);
    update_housing_and_specialist_slots(game_state);
}

// --- Service Building Functions ---
pub fn get_new_service_tiers(service_type: ServiceType) -> Vec<ServiceTier> {
    match service_type {
        ServiceType::Wellness => vec![
            ServiceTier { name: "Clinic".to_string(), construction_credits_cost: 150, upkeep_cost: 10, power_requirement: 5, workforce_requirement: 2, required_tech: None, service_capacity: 50, service_radius: 50.0, civic_index_contribution: 5, },
            ServiceTier { name: "Hospital".to_string(), construction_credits_cost: 400, upkeep_cost: 30, power_requirement: 15, workforce_requirement: 5, required_tech: None, service_capacity: 250, service_radius: 75.0, civic_index_contribution: 15, },
        ],
        ServiceType::Security => vec![
            ServiceTier { name: "Security Post".to_string(), construction_credits_cost: 150, upkeep_cost: 15, power_requirement: 5, workforce_requirement: 3, required_tech: None, service_capacity: 50, service_radius: 50.0, civic_index_contribution: 5, },
            ServiceTier { name: "Precinct".to_string(), construction_credits_cost: 450, upkeep_cost: 40, power_requirement: 10, workforce_requirement: 7, required_tech: None, service_capacity: 250, service_radius: 75.0, civic_index_contribution: 15, },
        ],
        ServiceType::Education => vec![ServiceTier { name: "School".to_string(), construction_credits_cost: 300, upkeep_cost: 25, power_requirement: 8, workforce_requirement: 4, required_tech: None, service_capacity: 100, service_radius: 60.0, civic_index_contribution: 10, }],
        ServiceType::Recreation => vec![ServiceTier { name: "Rec Center".to_string(), construction_credits_cost: 250, upkeep_cost: 20, power_requirement: 7, workforce_requirement: 3, required_tech: None, service_capacity: 100, service_radius: 60.0, civic_index_contribution: 8, }],
        ServiceType::Spiritual => vec![ServiceTier { name: "Sanctum".to_string(), construction_credits_cost: 200, upkeep_cost: 10, power_requirement: 3, workforce_requirement: 2, required_tech: None, service_capacity: 100, service_radius: 60.0, civic_index_contribution: 3, }],
    }
}

pub fn add_service_building( game_state: &mut GameState, service_type: ServiceType, tier_index: usize, position: Option<(f32, f32)>) {
    let all_available_tiers = get_new_service_tiers(service_type);
    if tier_index >= all_available_tiers.len() { println!("Error: Invalid tier index {} for service building type {:?}.", tier_index, service_type); return; }
    let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification( &mut game_state.notifications, format!("Not enough credits to build {:?} - {}.", service_type, tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification( &mut game_state.notifications, format!("Built {:?} - {}.", service_type, tier_info.name), 0.0);
    let new_building = Building {
        id: generate_unique_id(),
        variant: BuildingVariant::Service { service_type, available_tiers: all_available_tiers },
        position, current_tier_index: tier_index, is_active: false, assigned_workforce: 0,
    };
    game_state.buildings.push(new_building);
    update_civic_index(game_state);
    update_total_specialist_slots(game_state);
}

// --- Zone Functions ---
pub fn get_new_zone_tiers(zone_type: ZoneType) -> Vec<ZoneTier> {
    match zone_type {
        ZoneType::Commercial => vec![
            ZoneTier { name: "Small Market Stalls".to_string(), construction_credits_cost: 100, upkeep_cost: 10, power_requirement: 3, workforce_requirement: 5, required_tech: None, civic_index_contribution: 3, income_generation: 50, },
            ZoneTier { name: "Shopping Plaza".to_string(), construction_credits_cost: 300, upkeep_cost: 30, power_requirement: 8, workforce_requirement: 15, required_tech: Some(Tech::ZoningOrdinances), civic_index_contribution: 10, income_generation: 200, },
        ],
        ZoneType::LightIndustry => vec![
            ZoneTier { name: "Workshops".to_string(), construction_credits_cost: 120, upkeep_cost: 15, power_requirement: 5, workforce_requirement: 8, required_tech: None, civic_index_contribution: 2, income_generation: 0, },
            ZoneTier { name: "Assembly Lines".to_string(), construction_credits_cost: 350, upkeep_cost: 40, power_requirement: 12, workforce_requirement: 20, required_tech: Some(Tech::ZoningOrdinances), civic_index_contribution: 8, income_generation: 0, },
        ],
    }
}

pub fn add_zone( game_state: &mut GameState, zone_type: ZoneType, tier_index: usize, position: Option<(f32,f32)> ) {
    let all_available_tiers = get_new_zone_tiers(zone_type);
    if tier_index >= all_available_tiers.len() { println!("Error: Invalid tier index {} for zone type {:?}.", tier_index, zone_type); return; }
    let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification( &mut game_state.notifications, format!("Not enough credits to build {:?} - {}.", zone_type, tier_info.name),0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification( &mut game_state.notifications, format!("Constructed Zone: {:?} - {}.", zone_type, tier_info.name), 0.0);
    let new_building = Building {
        id: generate_unique_id(),
        variant: BuildingVariant::Zone { zone_type, available_tiers: all_available_tiers },
        position, current_tier_index: tier_index, is_active: false, assigned_workforce: 0,
    };
    game_state.buildings.push(new_building);
    update_total_specialist_slots(game_state);
    update_civic_index(game_state);
}

// --- Fabricator Functions ---
pub fn get_new_fabricator_tiers() -> Vec<FabricatorTier> {
    vec![
        FabricatorTier { name: "Basic Fabricator".to_string(), construction_credits_cost: 200, upkeep_cost: 10, power_requirement: 15, workforce_requirement: 1, required_tech: None, input_resources: HashMap::from([(ResourceType::FerrocreteOre, 2),(ResourceType::CuprumDeposits, 1),]), output_product: ResourceType::ManufacturedGoods, output_quantity: 1, production_time_secs: 10.0, },
        FabricatorTier { name: "Advanced Fabricator".to_string(), construction_credits_cost: 500, upkeep_cost: 25, power_requirement: 30, workforce_requirement: 2, required_tech: Some(Tech::AdvancedFabrication), input_resources: HashMap::from([(ResourceType::ManufacturedGoods, 2),(ResourceType::RefinedXylos, 1), ]), output_product: ResourceType::AdvancedComponents, output_quantity: 1, production_time_secs: 20.0, },
    ]
}

pub fn add_fabricator( game_state: &mut GameState, tier_index: usize, position: Option<(f32,f32)>) {
    let all_available_tiers = get_new_fabricator_tiers();
    if tier_index >= all_available_tiers.len() { println!("Error: Invalid tier index {} for fabricator.", tier_index); return; }
    let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification( &mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification( &mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    let new_building = Building {
        id: generate_unique_id(),
        variant: BuildingVariant::Fabricator { production_progress_secs: 0.0, available_tiers: all_available_tiers, },
        position, current_tier_index: tier_index, is_active: false, assigned_workforce: 0,
    };
    game_state.buildings.push(new_building);
    update_total_specialist_slots(game_state);
}

// --- Processing Plant Functions ---
pub fn get_new_processing_plant_tiers() -> Vec<ProcessingPlantTier> {
    vec![
        ProcessingPlantTier { name: "Xylos Purifier".to_string(), construction_credits_cost: 150, upkeep_cost: 10, power_requirement: 20, workforce_requirement: 2, required_tech: Some(Tech::IndustrialProcessing), unlocks_resource: Some(ResourceType::RawXylos), input_resource: Some((ResourceType::RawXylos, 2)), output_resource: Some((ResourceType::RefinedXylos, 1)), processing_rate_per_sec: Some(0.5), },
        ProcessingPlantTier { name: "Quantium Resonator".to_string(), construction_credits_cost: 200, upkeep_cost: 15, power_requirement: 25, workforce_requirement: 3, required_tech: Some(Tech::IndustrialProcessing), unlocks_resource: Some(ResourceType::RawQuantium), input_resource: None, output_resource: None, processing_rate_per_sec: None, },
        ProcessingPlantTier { name: "Advanced Material Synthesizer".to_string(), construction_credits_cost: 300, upkeep_cost: 20, power_requirement: 40, workforce_requirement: 4, required_tech: Some(Tech::IndustrialProcessing), unlocks_resource: None, input_resource: Some((ResourceType::CuprumDeposits, 3)), output_resource: Some((ResourceType::ProcessedQuantium, 1)), processing_rate_per_sec: Some(0.2), },
    ]
}

pub fn add_processing_plant( game_state: &mut GameState, tier_index: usize, position: Option<(f32,f32)>) {
    let all_available_tiers = get_new_processing_plant_tiers();
    if tier_index >= all_available_tiers.len() { println!("Error: Invalid tier index {} for processing plant.", tier_index); return; }
    let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification( &mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64;
    add_notification( &mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    let new_building = Building {
        id: generate_unique_id(),
        variant: BuildingVariant::ProcessingPlant { processing_progress: 0.0, available_tiers: all_available_tiers, },
        position, current_tier_index: tier_index, is_active: false, assigned_workforce: 0,
    };
    if let Some(unlocked_res) = tier_info.unlocks_resource { game_state.unlocked_raw_materials.insert(unlocked_res); }
    game_state.buildings.push(new_building);
    update_total_specialist_slots(game_state);
}

// --- Functions for "Simple" Buildings (Extractor, BioDome, etc.) ---
pub fn get_default_extractor_tiers() -> Vec<ExtractorTier> {
    vec![ExtractorTier { name: "Ferrocrete Extractor".to_string(), construction_credits_cost: 75, upkeep_cost: 5, power_requirement: 15, workforce_requirement: 5, required_tech: None, resource_type: ResourceType::FerrocreteOre, extraction_rate_per_sec: 2.5, }]
}
pub fn add_extractor(game_state: &mut GameState, position: Option<(f32, f32)>) {
    let all_available_tiers = get_default_extractor_tiers(); let tier_index = 0; let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64; add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    game_state.buildings.push(Building { id: generate_unique_id(), variant: BuildingVariant::Extractor { available_tiers: all_available_tiers }, position, current_tier_index: tier_index, is_active: false, assigned_workforce: 0 });
    update_total_specialist_slots(game_state);
}
pub fn get_default_biodome_tiers() -> Vec<BioDomeTier> {
    vec![BioDomeTier { name: "Bio-Dome".to_string(), construction_credits_cost: 50, upkeep_cost: 3, power_requirement: 10, workforce_requirement: 10, required_tech: None, nutrient_paste_output_per_sec: 5.0, }]
}
pub fn add_bio_dome(game_state: &mut GameState, position: Option<(f32, f32)>) {
    let all_available_tiers = get_default_biodome_tiers(); let tier_index = 0; let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64; add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    game_state.buildings.push(Building { id: generate_unique_id(), variant: BuildingVariant::BioDome { available_tiers: all_available_tiers }, position, current_tier_index: tier_index, is_active: false, assigned_workforce: 0 });
    update_total_specialist_slots(game_state);
}
pub fn get_default_power_relay_tiers() -> Vec<PowerRelayTier> {
    vec![PowerRelayTier { name: "Power Relay".to_string(), construction_credits_cost: 60, upkeep_cost: 1, power_requirement: 0, workforce_requirement: 0, required_tech: None, power_generation: 50, }]
}
pub fn add_power_relay(game_state: &mut GameState, position: Option<(f32, f32)>) {
    let all_available_tiers = get_default_power_relay_tiers(); let tier_index = 0; let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64; add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    game_state.buildings.push(Building { id: generate_unique_id(), variant: BuildingVariant::PowerRelay { available_tiers: all_available_tiers }, position, current_tier_index: tier_index, is_active: true, assigned_workforce: 0 });
}
pub fn get_default_research_institute_tiers() -> Vec<ResearchInstituteTier> {
    vec![ResearchInstituteTier { name: "Research Institute".to_string(), construction_credits_cost: 150, upkeep_cost: 10, power_requirement: 5, workforce_requirement: 15, required_tech: None, research_points_per_sec: 0.5, }]
}
pub fn add_research_institute(game_state: &mut GameState, position: Option<(f32, f32)>) {
    let all_available_tiers = get_default_research_institute_tiers(); let tier_index = 0; let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64; add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    game_state.buildings.push(Building { id: generate_unique_id(), variant: BuildingVariant::ResearchInstitute { available_tiers: all_available_tiers }, position, current_tier_index: tier_index, is_active: false, assigned_workforce: 0 });
    update_total_specialist_slots(game_state);
}
pub fn get_default_storage_silo_tiers() -> Vec<StorageSiloTier> {
    vec![StorageSiloTier { name: "Storage Silo".to_string(), construction_credits_cost: 100, upkeep_cost: 2, power_requirement: 1, workforce_requirement: 0, required_tech: None, storage_capacity_increase: 500, }]
}
pub fn add_storage_silo(game_state: &mut GameState, position: Option<(f32, f32)>) {
    let all_available_tiers = get_default_storage_silo_tiers(); let tier_index = 0; let tier_info = &all_available_tiers[tier_index];
    if game_state.credits < tier_info.construction_credits_cost as f64 { add_notification(&mut game_state.notifications, format!("Not enough credits to build {}.", tier_info.name), 0.0); return; }
    game_state.credits -= tier_info.construction_credits_cost as f64; add_notification(&mut game_state.notifications, format!("Built {}.", tier_info.name), 0.0);
    game_state.buildings.push(Building { id: generate_unique_id(), variant: BuildingVariant::StorageSilo { available_tiers: all_available_tiers }, position, current_tier_index: tier_index, is_active: true, assigned_workforce: 0 });
}

// --- Building Removal, Upgrade, Workforce Assignment (Generic Functions) ---

pub fn remove_building_by_id(game_state: &mut GameState, building_id_to_remove: &str) {
    let mut building_found_and_removed = false;
    let mut unassigned_workforce_from_removed_building = 0;
    let mut requires_housing_update = false;
    let mut requires_civic_index_update = false;
    let mut requires_job_slot_update = false;

    if let Some(index) = game_state.buildings.iter().position(|b| b.id == building_id_to_remove) {
        let removed_building = game_state.buildings.remove(index);
        building_found_and_removed = true;
        unassigned_workforce_from_removed_building = removed_building.assigned_workforce;

        match removed_building.variant {
            BuildingVariant::Habitation { .. } => {
                requires_housing_update = true;
                requires_job_slot_update = true;
            }
            BuildingVariant::Service { .. } | BuildingVariant::Zone { .. } => {
                requires_civic_index_update = true;
                requires_job_slot_update = true;
            }
            BuildingVariant::Extractor { .. } |
            BuildingVariant::BioDome { .. } |
            BuildingVariant::ResearchInstitute { .. } |
            BuildingVariant::Fabricator { .. } |
            BuildingVariant::ProcessingPlant { .. } => {
                requires_job_slot_update = true;
            }
            _ => {}
        }
        add_notification(&mut game_state.notifications, format!("Building {} removed.", removed_building.id), 0.0);
    } else {
        add_notification(&mut game_state.notifications, format!("Building ID {} not found for removal.", building_id_to_remove), 0.0);
    }

    if building_found_and_removed {
        game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(unassigned_workforce_from_removed_building);
        if requires_housing_update { update_housing_and_specialist_slots(game_state); }
        if requires_civic_index_update { update_civic_index(game_state); }
        if requires_job_slot_update && !requires_housing_update { update_total_specialist_slots(game_state); }

        if requires_housing_update {
             if game_state.total_inhabitants > game_state.available_housing_capacity {
                game_state.total_inhabitants = game_state.total_inhabitants.min(game_state.available_housing_capacity);
            }
             game_state.assigned_specialists_total = game_state.assigned_specialists_total.min(game_state.total_inhabitants).min(game_state.total_specialist_slots);
        }
    }
}

pub fn upgrade_building_by_id(game_state: &mut GameState, building_id_to_upgrade: &str) {
    let mut building_found = false;
    let mut new_tier_index_opt: Option<usize> = None;
    let mut cost = 0.0;
    let mut new_tier_name = "".to_string();
    let mut old_workforce_max = 0;
    let mut new_workforce_max = 0;
    // Flags to call update functions later
    let mut requires_housing_update = false;
    let mut requires_civic_index_update = false;
    let mut requires_job_slot_update = false;


    if let Some(building) = game_state.buildings.iter_mut().find(|b| b.id == building_id_to_upgrade) {
        building_found = true;
        let current_tier_idx = building.current_tier_index;

        let (available_tiers_len, next_tier_cost_opt, next_tier_tech_req_opt, next_tier_w_req_opt, current_tier_w_req_opt, name_of_next_tier_opt, variant_type_for_updates) = match &building.variant {
            BuildingVariant::Habitation { available_tiers, .. } => {
                (available_tiers.len(),
                 available_tiers.get(current_tier_idx + 1).map(|t: &HabitationTier| t.construction_credits_cost),
                 available_tiers.get(current_tier_idx + 1).and_then(|t: &HabitationTier| t.required_tech),
                 available_tiers.get(current_tier_idx + 1).map(|t: &HabitationTier| t.workforce_requirement),
                 available_tiers.get(current_tier_idx).map(|t: &HabitationTier| t.workforce_requirement),
                 available_tiers.get(current_tier_idx + 1).map(|t: &HabitationTier| t.name.clone()),
                 "Habitation"
                )
            }
            BuildingVariant::Service { available_tiers, .. } => {
                (available_tiers.len(), available_tiers.get(current_tier_idx + 1).map(|t: &ServiceTier| t.construction_credits_cost), available_tiers.get(current_tier_idx + 1).and_then(|t: &ServiceTier| t.required_tech), available_tiers.get(current_tier_idx + 1).map(|t: &ServiceTier| t.workforce_requirement), available_tiers.get(current_tier_idx).map(|t: &ServiceTier| t.workforce_requirement), available_tiers.get(current_tier_idx + 1).map(|t: &ServiceTier| t.name.clone()), "Service")
            }
            BuildingVariant::Zone { available_tiers, .. } => {
                (available_tiers.len(), available_tiers.get(current_tier_idx + 1).map(|t: &ZoneTier| t.construction_credits_cost), available_tiers.get(current_tier_idx + 1).and_then(|t: &ZoneTier| t.required_tech), available_tiers.get(current_tier_idx + 1).map(|t: &ZoneTier| t.workforce_requirement), available_tiers.get(current_tier_idx).map(|t: &ZoneTier| t.workforce_requirement), available_tiers.get(current_tier_idx + 1).map(|t: &ZoneTier| t.name.clone()), "Zone")
            }
            BuildingVariant::Fabricator { available_tiers, .. } => {
                 (available_tiers.len(), available_tiers.get(current_tier_idx + 1).map(|t: &FabricatorTier| t.construction_credits_cost), available_tiers.get(current_tier_idx + 1).and_then(|t: &FabricatorTier| t.required_tech), available_tiers.get(current_tier_idx + 1).map(|t: &FabricatorTier| t.workforce_requirement), available_tiers.get(current_tier_idx).map(|t: &FabricatorTier| t.workforce_requirement), available_tiers.get(current_tier_idx + 1).map(|t: &FabricatorTier| t.name.clone()), "Production")
            }
            BuildingVariant::ProcessingPlant { available_tiers, .. } => {
                 (available_tiers.len(), available_tiers.get(current_tier_idx + 1).map(|t: &ProcessingPlantTier| t.construction_credits_cost), available_tiers.get(current_tier_idx + 1).and_then(|t: &ProcessingPlantTier| t.required_tech), available_tiers.get(current_tier_idx + 1).map(|t: &ProcessingPlantTier| t.workforce_requirement), available_tiers.get(current_tier_idx).map(|t: &ProcessingPlantTier| t.workforce_requirement), available_tiers.get(current_tier_idx + 1).map(|t: &ProcessingPlantTier| t.name.clone()), "Production")
            }
            // Add other upgradable types if they become so
            _ => {
                add_notification(&mut game_state.notifications, format!("Building ID {} is not upgradable.", building_id_to_upgrade), 0.0);
                return;
            }
        };

        if current_tier_idx < available_tiers_len - 1 {
            if let Some(tech_req) = next_tier_tech_req_opt {
                if !game_state.unlocked_techs.contains(&tech_req) {
                    add_notification(&mut game_state.notifications, format!("Tech {:?} required for upgrade.", tech_req), 0.0);
                    return;
                }
            }

            if let Some(next_cost) = next_tier_cost_opt {
                cost = next_cost as f64;
                if game_state.credits < cost {
                    add_notification(&mut game_state.notifications, "Not enough credits for upgrade.".to_string(), 0.0);
                    return;
                }
            } else { return; }

            new_tier_index_opt = Some(current_tier_idx + 1);
            new_tier_name = name_of_next_tier_opt.unwrap_or_default();
            old_workforce_max = current_tier_w_req_opt.unwrap_or(0);
            new_workforce_max = next_tier_w_req_opt.unwrap_or(0);

            match variant_type_for_updates {
                "Habitation" => { requires_housing_update = true; requires_job_slot_update = true; }
                "Service" | "Zone" => { requires_civic_index_update = true; requires_job_slot_update = true; }
                "Production" => { requires_job_slot_update = true; }
                _ => {}
            }

        } else {
            add_notification(&mut game_state.notifications, "Already at maximum tier.".to_string(), 0.0);
            return;
        }
    }

    if building_found && new_tier_index_opt.is_some() {
        game_state.credits -= cost;
        let new_tier_idx = new_tier_index_opt.unwrap();

        if let Some(building) = game_state.buildings.iter_mut().find(|b| b.id == building_id_to_upgrade) {
            building.current_tier_index = new_tier_idx;
            add_notification(&mut game_state.notifications, format!("Upgraded {} to {}.", building.id, new_tier_name), 0.0);

            if new_workforce_max < old_workforce_max {
                if building.assigned_workforce > new_workforce_max {
                    let to_unassign = building.assigned_workforce - new_workforce_max;
                    building.assigned_workforce = new_workforce_max;
                    game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(to_unassign);
                }
            }
            if let BuildingVariant::Fabricator { production_progress_secs, .. } = &mut building.variant {
                *production_progress_secs = 0.0;
            }
            if let BuildingVariant::ProcessingPlant { processing_progress, .. } = &mut building.variant {
                *processing_progress = 0.0;
            }
        }

        if requires_housing_update { update_housing_and_specialist_slots(game_state); }
        if requires_civic_index_update { update_civic_index(game_state); }
        if requires_job_slot_update && !requires_housing_update { update_total_specialist_slots(game_state); }

    } else if building_found {
        add_notification(&mut game_state.notifications, "Upgrade not performed.".to_string(), 0.0);
    } else {
         add_notification(&mut game_state.notifications, format!("Building ID {} not found for upgrade.", building_id_to_upgrade), 0.0);
    }
}

pub fn assign_workforce_to_building_by_id(game_state: &mut GameState, building_id: &str, num_to_assign: u32) {
    let mut success = false;
    if let Some(building) = game_state.buildings.iter_mut().find(|b| b.id == building_id) {
        let (max_workforce_for_tier, building_name) = match &building.variant {
            BuildingVariant::Extractor { available_tiers } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &ExtractorTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "Extractor".to_string(), |t: &ExtractorTier| t.name.clone())),
            BuildingVariant::BioDome { available_tiers } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &BioDomeTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "BioDome".to_string(), |t: &BioDomeTier| t.name.clone())),
            BuildingVariant::ResearchInstitute { available_tiers } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &ResearchInstituteTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "Research Institute".to_string(), |t: &ResearchInstituteTier| t.name.clone())),
            BuildingVariant::Service { available_tiers, service_type } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &ServiceTier| t.workforce_requirement), format!("{:?} {}", service_type, available_tiers.get(building.current_tier_index).map_or_else(|| "".to_string(), |t: &ServiceTier| t.name.clone()))),
            BuildingVariant::Zone { available_tiers, zone_type } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &ZoneTier| t.workforce_requirement), format!("{:?} {}", zone_type, available_tiers.get(building.current_tier_index).map_or_else(|| "".to_string(), |t: &ZoneTier| t.name.clone()))),
            BuildingVariant::Fabricator { available_tiers, .. } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &FabricatorTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "Fabricator".to_string(), |t: &FabricatorTier| t.name.clone())),
            BuildingVariant::ProcessingPlant { available_tiers, .. } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &ProcessingPlantTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "Processing Plant".to_string(), |t: &ProcessingPlantTier| t.name.clone())),
            BuildingVariant::Habitation { available_tiers, .. } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &HabitationTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "Habitation Structure".to_string(), |t: &HabitationTier| t.name.clone())),
            BuildingVariant::AdministrativeSpire { available_tiers } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &AdministrativeSpireTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "Admin Spire".to_string(), |t: &AdministrativeSpireTier| t.name.clone())), // Changed NewAdministrativeSpireTier
            BuildingVariant::LegacyStructure { available_tiers } => (available_tiers.get(building.current_tier_index).map_or(0, |t: &LegacyStructureTier| t.workforce_requirement), available_tiers.get(building.current_tier_index).map_or_else(|| "Legacy Structure".to_string(), |t: &LegacyStructureTier| t.name.clone())), // Changed NewLegacyStructureTier
            _ => (0, "Unknown Building".to_string()),
        };

        if max_workforce_for_tier == 0 {
            add_notification(&mut game_state.notifications, format!("{} does not require workforce.", building_name), 0.0);
            return;
        }
        let unassigned_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
        if unassigned_inhabitants < num_to_assign {
            add_notification(&mut game_state.notifications, format!("Not enough unassigned inhabitants for {}.", building_name), 0.0);
            return;
        }
        if building.assigned_workforce + num_to_assign > max_workforce_for_tier {
            add_notification(&mut game_state.notifications, format!("Cannot assign more workforce than required for {}. Max: {}", building_name, max_workforce_for_tier), 0.0);
            return;
        }
        building.assigned_workforce += num_to_assign;
        game_state.assigned_specialists_total += num_to_assign;
        if building.assigned_workforce >= max_workforce_for_tier { /* building.is_active = true; */ }
        success = true;
        add_notification(&mut game_state.notifications, format!("Assigned {} workforce to {}.", num_to_assign, building_name),0.0);
    } else {
        add_notification(&mut game_state.notifications, format!("Building ID {} not found for workforce assignment.", building_id),0.0);
    }
    if success { /* Potentially call update_total_specialist_slots or other global updates if needed */ }
}

pub fn unassign_workforce_from_building_by_id(game_state: &mut GameState, building_id: &str, num_to_unassign: u32) {
    if let Some(building) = game_state.buildings.iter_mut().find(|b| b.id == building_id) {
        let building_name = match &building.variant {
             BuildingVariant::Extractor { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Extractor".to_string(), |t: &ExtractorTier| t.name.clone()),
             BuildingVariant::BioDome { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "BioDome".to_string(), |t: &BioDomeTier| t.name.clone()),
             BuildingVariant::ResearchInstitute { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Research Institute".to_string(), |t: &ResearchInstituteTier| t.name.clone()),
             BuildingVariant::Service { available_tiers, service_type } => format!("{:?} {}", service_type, available_tiers.get(building.current_tier_index).map_or_else(|| "".to_string(), |t: &ServiceTier| t.name.clone())),
             BuildingVariant::Zone { available_tiers, zone_type } => format!("{:?} {}", zone_type, available_tiers.get(building.current_tier_index).map_or_else(|| "".to_string(), |t: &ZoneTier| t.name.clone())),
             BuildingVariant::Fabricator { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Fabricator".to_string(), |t: &FabricatorTier| t.name.clone()),
             BuildingVariant::ProcessingPlant { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Processing Plant".to_string(), |t: &ProcessingPlantTier| t.name.clone()),
             BuildingVariant::Habitation { available_tiers, .. } => available_tiers.get(building.current_tier_index).map_or_else(|| "Habitation Structure".to_string(), |t: &HabitationTier| t.name.clone()),
             _ => "Building".to_string(),
        };

        let actual_unassign = num_to_unassign.min(building.assigned_workforce);
        if actual_unassign > 0 {
            building.assigned_workforce -= actual_unassign;
            game_state.assigned_specialists_total = game_state.assigned_specialists_total.saturating_sub(actual_unassign);
            match &mut building.variant {
                BuildingVariant::Fabricator { production_progress_secs, .. } => *production_progress_secs = 0.0,
                BuildingVariant::ProcessingPlant { processing_progress, .. } => *processing_progress = 0.0,
                _ => {}
            }
            add_notification(&mut game_state.notifications, format!("Unassigned {} workforce from {}.", actual_unassign, building_name),0.0);
        }
    } else {
        add_notification(&mut game_state.notifications, format!("Building ID {} not found for workforce unassignment.", building_id),0.0);
    }
}

// --- Helper Update Functions ---
pub fn update_housing_and_specialist_slots(game_state: &mut GameState) {
    let mut total_housing = 0;
    let mut total_slots = 0; // For specialists housed in Habitation, distinct from general job slots.
    for building in &game_state.buildings {
        if let BuildingVariant::Habitation { ref available_tiers, .. } = building.variant {
            if let Some(tier) = available_tiers.get(building.current_tier_index) {
                total_housing += tier.housing_capacity;
                total_slots += tier.specialist_slots;
            }
        }
    }
    game_state.available_housing_capacity = total_housing;
    game_state.total_specialist_slots = total_slots;
    // Note: total_specialist_slots might need to be a sum from other job providers too,
    // this function now focuses on Habitation's contribution.
    // Or, rename this to total_habitation_specialist_slots.
    // For now, this matches the old direct summation from habitation_structures.
}

pub fn update_civic_index(game_state: &mut GameState) {
    let mut new_civic_index = 0;
    for building in &game_state.buildings {
        if building.is_active { // Only active buildings contribute
            match &building.variant {
                BuildingVariant::Service { ref available_tiers, .. } => {
                    if let Some(tier) = available_tiers.get(building.current_tier_index) {
                        new_civic_index += tier.civic_index_contribution;
                    }
                }
                BuildingVariant::Zone { ref available_tiers, .. } => {
                    if let Some(tier) = available_tiers.get(building.current_tier_index) {
                        new_civic_index += tier.civic_index_contribution;
                    }
                }
                _ => {} // Other building types might not contribute to civic index
            }
        }
    }
    game_state.civic_index = new_civic_index;
}

pub fn update_total_specialist_slots(game_state: &mut GameState) {
    let mut total_slots = 0;
    for building in &game_state.buildings {
        if !building.is_active { // Consider only active buildings for job slots
            continue;
        }
        match &building.variant {
            BuildingVariant::Habitation { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.specialist_slots; // These are explicit specialist housing slots
                }
            }
            BuildingVariant::Zone { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.workforce_requirement; // For Zones, workforce_requirement = jobs
                }
            }
            BuildingVariant::Service { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.workforce_requirement;
                }
            }
            BuildingVariant::Fabricator { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.workforce_requirement;
                }
            }
            BuildingVariant::ProcessingPlant { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.workforce_requirement;
                }
            }
            BuildingVariant::ResearchInstitute { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.workforce_requirement;
                }
            }
            BuildingVariant::Extractor { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.workforce_requirement;
                }
            }
            BuildingVariant::BioDome { ref available_tiers, .. } => {
                if let Some(tier) = available_tiers.get(building.current_tier_index) {
                    total_slots += tier.workforce_requirement;
                }
            }
            _ => {}
        }
    }
    game_state.total_specialist_slots = total_slots;
}

// --- Administrative Spire Functions ---

// Placeholder: Actual tier data should be defined (e.g., in GameState::default or loaded)
// Ensure this function returns Vec<crate::components::building::AdministrativeSpireTier>
// and that the fields match the definition in components/building.rs
pub fn get_administrative_spire_tiers() -> Vec<AdministrativeSpireTier> { // Return type now uses imported struct
    vec![
        AdministrativeSpireTier { // Fields must match crate::components::building::AdministrativeSpireTier
            name: "Spire Core".to_string(),
            construction_credits_cost: 1000,
            upgrade_credits_cost: 1500,
            upkeep_cost: 50, // Added common field
            power_requirement: 50,
            workforce_requirement: 5, // Added common field
            required_tech: None, // Added common field
            unlocks_phase: DevelopmentPhase::DP1,
            nutrient_paste_link_required: true,
        },
        AdministrativeSpireTier {
            name: "Spire Nexus".to_string(),
            construction_credits_cost: 0,
            upgrade_credits_cost: 2500,
            upkeep_cost: 100, // Added common field
            power_requirement: 100,
            workforce_requirement: 10, // Added common field
            required_tech: Some(Tech::EfficientExtraction), // Example
            unlocks_phase: DevelopmentPhase::DP2,
            nutrient_paste_link_required: true,
        },
        AdministrativeSpireTier {
            name: "Spire Apex".to_string(),
            construction_credits_cost: 0,
            upgrade_credits_cost: 0,
            upkeep_cost: 200, // Added common field
            power_requirement: 200,
            workforce_requirement: 20, // Added common field
            required_tech: Some(Tech::ArcologyConstruction), // Example
            unlocks_phase: DevelopmentPhase::DP3,
            nutrient_paste_link_required: false,
        },
    ]
}

pub fn construct_administrative_spire(game_state: &mut GameState) {
    if game_state.administrative_spire.is_some() {
        add_notification(&mut game_state.notifications, "Administrative Spire already constructed.".to_string(), 0.0);
        return;
    }
    let tiers = get_administrative_spire_tiers();
    if tiers.is_empty() {
        add_notification(&mut game_state.notifications, "No Administrative Spire tiers defined.".to_string(), 0.0);
        return;
    }
    let initial_tier = &tiers[0];
    if game_state.credits < initial_tier.construction_credits_cost as f64 {
        add_notification(&mut game_state.notifications, format!("Not enough credits to construct {}.", initial_tier.name), 0.0);
        return;
    }
    game_state.credits -= initial_tier.construction_credits_cost as f64;
    game_state.administrative_spire = Some(AdministrativeSpire {
        current_tier_index: 0,
        available_tiers: tiers,
    });
    game_state.current_development_phase = initial_tier.unlocks_phase; // Set initial phase
    add_notification(&mut game_state.notifications, format!("{} constructed.", initial_tier.name), 0.0);
}

pub fn upgrade_administrative_spire(game_state: &mut GameState) {
    if let Some(spire) = &mut game_state.administrative_spire {
        if spire.current_tier_index < spire.available_tiers.len() - 1 {
            let current_tier = &spire.available_tiers[spire.current_tier_index];
            let next_tier = &spire.available_tiers[spire.current_tier_index + 1];
            let upgrade_cost = current_tier.upgrade_credits_cost; // Cost is on the current tier to upgrade to next

            if game_state.credits < upgrade_cost as f64 {
                add_notification(&mut game_state.notifications, format!("Not enough credits to upgrade Spire to {}.", next_tier.name), 0.0);
                return;
            }
            // Placeholder: Check other requirements like tech if necessary

            game_state.credits -= upgrade_cost as f64;
            spire.current_tier_index += 1;
            game_state.current_development_phase = next_tier.unlocks_phase; // Update phase
            add_notification(&mut game_state.notifications, format!("Administrative Spire upgraded to {}.", next_tier.name), 0.0);
        } else {
            add_notification(&mut game_state.notifications, "Administrative Spire is already at maximum tier.".to_string(), 0.0);
        }
    } else {
        add_notification(&mut game_state.notifications, "Administrative Spire not constructed yet.".to_string(), 0.0);
    }
}

// --- Legacy Structure Functions ---

// Ensure this function returns Vec<crate::components::building::LegacyStructureTier>
// and that the fields match the definition in components/building.rs
pub fn get_legacy_structure_tiers() -> Vec<LegacyStructureTier> { // Return type now uses imported struct
    vec![
        LegacyStructureTier { // Fields must match crate::components::building::LegacyStructureTier
            name: "Ancient Monolith".to_string(),
            construction_credits_cost: 2000,
            upkeep_cost: 20, // Added common field
            power_requirement: 10, // Added common field
            workforce_requirement: 0, // Added common field
            required_tech: None, // Added common field
            happiness_bonus: 5.0,
            income_bonus: 100.0,
        },
        LegacyStructureTier {
            name: "Restored Monument".to_string(),
            construction_credits_cost: 5000,
            upkeep_cost: 50, // Added common field
            power_requirement: 25, // Added common field
            workforce_requirement: 0, // Added common field
            required_tech: Some(Tech::ZoningOrdinances), // Example
            happiness_bonus: 10.0,
            income_bonus: 250.0,
        },
        LegacyStructureTier {
            name: "Nexus Beacon".to_string(),
            construction_credits_cost: 10000,
            upkeep_cost: 100, // Added common field
            power_requirement: 50, // Added common field
            workforce_requirement: 0, // Added common field
            required_tech: Some(Tech::ArcologyConstruction), // Example
            happiness_bonus: 20.0,
            income_bonus: 500.0,
        },
    ]
}

pub fn construct_legacy_structure(game_state: &mut GameState) {
    if game_state.legacy_structure.is_some() {
        add_notification(&mut game_state.notifications, "Legacy Structure already constructed.".to_string(), 0.0);
        return;
    }
    if game_state.current_development_phase < DevelopmentPhase::DP3 {
         add_notification(&mut game_state.notifications, "Legacy Structure requires Development Phase 3.".to_string(), 0.0);
        return;
    }
    let tiers = get_legacy_structure_tiers();
    if tiers.is_empty() {
         add_notification(&mut game_state.notifications, "No Legacy Structure tiers defined.".to_string(), 0.0);
        return;
    }
    let initial_tier = &tiers[0];
    if game_state.credits < initial_tier.construction_credits_cost as f64 {
        add_notification(&mut game_state.notifications, format!("Not enough credits to construct {}.", initial_tier.name), 0.0);
        return;
    }
    game_state.credits -= initial_tier.construction_credits_cost as f64;
    game_state.legacy_structure = Some(LegacyStructure {
        current_tier_index: 0,
        available_tiers: tiers,
    });
    // Apply initial bonuses
    game_state.colony_happiness += initial_tier.happiness_bonus;
    // Income bonus might be applied per tick in another system
    add_notification(&mut game_state.notifications, format!("{} constructed.", initial_tier.name), 0.0);
}

pub fn upgrade_legacy_structure(game_state: &mut GameState) {
    if let Some(structure) = &mut game_state.legacy_structure {
        if structure.current_tier_index < structure.available_tiers.len() - 1 {
            let next_tier_index = structure.current_tier_index + 1;
            let next_tier = &structure.available_tiers[next_tier_index];
            // Cost to upgrade is the construction_cost of the target tier.
            let upgrade_cost = next_tier.construction_credits_cost;

            if game_state.credits < upgrade_cost as f64 {
                add_notification(&mut game_state.notifications, format!("Not enough credits to upgrade Legacy Structure to {}.", next_tier.name), 0.0);
                return;
            }
            // Placeholder: Check other requirements if necessary

            // Remove previous tier's bonus before applying new one
            let previous_tier = &structure.available_tiers[structure.current_tier_index];
            game_state.colony_happiness -= previous_tier.happiness_bonus;

            game_state.credits -= upgrade_cost as f64;
            structure.current_tier_index = next_tier_index;

            // Apply new tier's bonus
            game_state.colony_happiness += next_tier.happiness_bonus;
            // Income bonus might be applied per tick in another system

            add_notification(&mut game_state.notifications, format!("Legacy Structure upgraded to {}.", next_tier.name), 0.0);
        } else {
            add_notification(&mut game_state.notifications, "Legacy Structure is already at maximum tier.".to_string(), 0.0);
        }
    } else {
        add_notification(&mut game_state.notifications, "Legacy Structure not constructed yet.".to_string(), 0.0);
    }
}

// pub fn get_legacy_structure_tiers() -> Vec<LegacyStructureTier> { ... } // These are already present
// pub fn construct_legacy_structure(game_state: &mut GameState) { ... }
