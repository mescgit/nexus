// src/game_state.rs

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

// --- Data Structs ---

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ResourceType {
    FerrocreteOre,
    NutrientPaste,
    CuprumDeposits,
    Power,
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
    pub current_resources: HashMap<ResourceType, f32>,
    pub building_costs: HashMap<BuildingType, HashMap<ResourceType, f32>>,
    pub unlocked_techs: HashSet<Tech>,
    pub research_progress: Option<(Tech, f32)>,
    pub tech_costs: HashMap<Tech, f32>,
}

// Holds the current calculated stats for the entire colony.
#[derive(Resource, Default, Clone, Copy)]
pub struct ColonyStats {
    pub total_housing: u32,
    pub total_jobs: u32,
    pub health_capacity: u32,
    pub police_capacity: u32,
}

// Stores a history of stats for graphing.
#[derive(Resource, Default)]
pub struct GraphData {
    pub history: VecDeque<ColonyStats>,
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
        tech_costs.insert(Tech::BasicConstructionProtocols, 10.0);
        tech_costs.insert(Tech::EfficientExtraction, 25.0);

        let mut current_resources = HashMap::new();
        current_resources.insert(ResourceType::NutrientPaste, 50.0);
        current_resources.insert(ResourceType::FerrocreteOre, 200.0);
        current_resources.insert(ResourceType::CuprumDeposits, 50.0);
        current_resources.insert(ResourceType::Power, 100.0);

        Self {
            current_resources,
            building_costs,
            unlocked_techs: HashSet::new(),
            research_progress: None,
            tech_costs,
        }
    }
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
                update_colony_stats_system.after(game_tick_system),
                update_graph_data_system.after(update_colony_stats_system),
            ));
    }
}

fn game_tick_system(
    mut game_state: ResMut<GameState>,
    power_relays: Query<&PowerRelay>,
    storage_silos: Query<&StorageSilo>,
    extractors: Query<&Extractor>,
    bio_domes: Query<&BioDome>,
    research_institutes: Query<&ResearchInstitute>,
    fabricators: Query<&Fabricator>, // Added Fabricator query
) {
    let total_power_generation: u32 = power_relays.iter().map(|pr| pr.power_output).sum();
    let mut total_power_consumption: u32 = extractors.iter().map(|e| e.power_consumption).sum::<u32>()
        + bio_domes.iter().map(|b| b.power_consumption).sum::<u32>()
        + research_institutes.iter().map(|ri| ri.power_consumption).sum::<u32>()
        + fabricators.iter().map(|f| f.power_consumption).sum::<u32>(); // Added Fabricator power consumption
    // Note: ProcessingPlant power consumption will be added if/when it has active processing.

    let net_power = total_power_generation as f32 - total_power_consumption as f32;
    let stored_power_entry = game_state.current_resources.entry(ResourceType::Power).or_insert(0.0);
    *stored_power_entry = (*stored_power_entry + net_power).max(0.0); // Ensure stored power doesn't go below zero

    let mut has_sufficient_power = false;

    if total_power_generation >= total_power_consumption {
        has_sufficient_power = true;
    } else {
        let power_deficit = total_power_consumption - total_power_generation;
        let current_stored_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0);

        if current_stored_power >= power_deficit as f32 {
            has_sufficient_power = true;
            *game_state.current_resources.entry(ResourceType::Power).or_insert(0.0) -= power_deficit as f32;
        } else {
            has_sufficient_power = false;
            // Optional: Set stored power to 0 if partially consumed.
            // For now, we'll let the earlier max(0.0) handle ensuring it's not negative.
            // If we want to ensure it's fully depleted if deficit is greater:
            *game_state.current_resources.entry(ResourceType::Power).or_insert(0.0) = 0.0;
        }
    }

    if has_sufficient_power {
        let capacity = storage_silos.iter().map(|s| s.capacity).sum::<u32>() as f32; // Consider if capacity should be per resource type
        for dome in &bio_domes {
            let amount = game_state.current_resources.entry(ResourceType::NutrientPaste).or_insert(0.0);
            *amount = (*amount + dome.production_rate).min(capacity);
        }
        for extractor in &extractors {
            let amount = game_state.current_resources.entry(extractor.resource_type).or_insert(0.0);
            *amount = (*amount + extractor.extraction_rate).min(capacity);
        }

        // Fabricator Logic
        for fabricator in &fabricators {
            // Check for sufficient input resources
            let current_input_val_entry = game_state.current_resources.entry(fabricator.input_resource);
            let current_input_val = *current_input_val_entry.or_insert(0.0);

            let required_input_amount = fabricator.input_amount * fabricator.conversion_rate;

            if current_input_val >= required_input_amount {
                // Consume input resources
                *game_state.current_resources.entry(fabricator.input_resource).or_insert(0.0) -= required_input_amount;

                // Produce output resources
                let current_output_val = game_state.current_resources.entry(fabricator.output_resource).or_insert(0.0);
                *current_output_val += fabricator.output_amount * fabricator.conversion_rate;
                *current_output_val = (*current_output_val).min(capacity); // Apply storage capacity limit
            }
        }
    }
}

fn update_colony_stats_system(
    mut stats: ResMut<ColonyStats>,
    dwellings: Query<&BasicDwelling>,
    wellness_posts: Query<&WellnessPost>,
    security_stations: Query<&SecurityStation>,
) {
    stats.total_housing = dwellings.iter().map(|d| d.housing_capacity).sum();
    stats.health_capacity = wellness_posts.iter().map(|p| p.health_capacity).sum();
    stats.police_capacity = security_stations.iter().map(|p| p.police_capacity).sum();
    stats.total_jobs = wellness_posts.iter().map(|p| p.jobs_provided).sum::<u32>()
        + security_stations.iter().map(|p| p.jobs_provided).sum::<u32>();
}

fn research_system(mut game_state: ResMut<GameState>, query: Query<&ResearchInstitute>) {
    if query.is_empty() { return; }
    let mut completed_tech: Option<Tech> = None;
    if let Some((tech, progress)) = &game_state.research_progress {
        let required_progress = game_state.tech_costs[tech];
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