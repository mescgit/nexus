// src/game_state.rs

use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

// --- Data Structs ---

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ResourceType {
    FerrocreteOre,
    NutrientPaste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    // Operations Hub
    Extractor,
    BioDome,
    PowerRelay,
    StorageSilo,
    ResearchInstitute,
    // Habitation Sector
    BasicDwelling,
    WellnessPost,
    SecurityStation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tech {
    BasicConstructionProtocols,
}

// --- Building Components ---
#[derive(Component)]
pub struct Extractor { pub power_required: u32, pub resource_type: ResourceType, pub extraction_rate: f32 }
#[derive(Component)]
pub struct BioDome { pub power_required: u32, pub production_rate: f32 }
#[derive(Component)]
pub struct PowerRelay { pub power_output: u32 }
#[derive(Component)]
pub struct StorageSilo { pub capacity: u32 }
#[derive(Component)]
pub struct ResearchInstitute;
#[derive(Component)]
pub struct BasicDwelling { pub housing_capacity: u32 }
#[derive(Component)]
pub struct WellnessPost { pub health_capacity: u32, pub jobs_provided: u32 }
#[derive(Component)]
pub struct SecurityStation { pub police_capacity: u32, pub jobs_provided: u32 }


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
        building_costs.insert(BuildingType::BasicDwelling, HashMap::from([(ResourceType::FerrocreteOre, 100.0)]));
        building_costs.insert(BuildingType::WellnessPost, HashMap::from([(ResourceType::FerrocreteOre, 120.0)]));
        building_costs.insert(BuildingType::SecurityStation, HashMap::from([(ResourceType::FerrocreteOre, 120.0)]));
        
        let mut tech_costs = HashMap::new();
        tech_costs.insert(Tech::BasicConstructionProtocols, 10.0);

        let mut current_resources = HashMap::new();
        current_resources.insert(ResourceType::NutrientPaste, 50.0);
        current_resources.insert(ResourceType::FerrocreteOre, 200.0);

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
) {
    let power_gen: u32 = power_relays.iter().map(|pr| pr.power_output).sum();
    let power_con: u32 = extractors.iter().map(|e| e.power_required).sum::<u32>() + bio_domes.iter().map(|b| b.power_required).sum::<u32>();
    
    if power_gen >= power_con {
        let capacity = storage_silos.iter().map(|s| s.capacity).sum::<u32>() as f32;
        for dome in &bio_domes {
            let amount = game_state.current_resources.entry(ResourceType::NutrientPaste).or_insert(0.0);
            *amount = (*amount + dome.production_rate).min(capacity);
        }
        for extractor in &extractors {
            let amount = game_state.current_resources.entry(extractor.resource_type).or_insert(0.0);
            *amount = (*amount + extractor.extraction_rate).min(capacity);
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