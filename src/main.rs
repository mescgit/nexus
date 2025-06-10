// src/main.rs

use bevy::prelude::*;
// REMOVED: `use bevy_prototype_lyon::prelude::*;`
use game_state::{
    BasicDwelling, BioDome, BuildingType, ColonyStats, Extractor, GameState, GraphData, PowerRelay,
    ResearchInstitute, ResourceType, SecurityStation, StorageSilo, Tech, WellnessPost,
};

mod game_state;

// --- UI Marker Components ---
#[derive(Component)]
struct PowerText;
#[derive(Component)]
struct ResourceText(ResourceType);
#[derive(Component)]
struct BuildButton(BuildingType);
#[derive(Component)]
struct ResearchButton(Tech);
#[derive(Component)]
struct MessageText;
#[derive(Component)]
struct ColonyStatText(StatType);
#[derive(Component)]
struct CreditsText; // New marker component for Credits
#[derive(Component)]
struct GraphArea; // Marker for the graph's background node

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StatType {
    Housing,
    Jobs,
    Health,
    Police,
}

// --- Constants ---
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

const HOUSING_COLOR: Color = Color::rgb(0.2, 0.7, 1.0);
const JOBS_COLOR: Color = Color::rgb(1.0, 0.7, 0.2);
const HEALTH_COLOR: Color = Color::rgb(0.2, 1.0, 0.7);
const POLICE_COLOR: Color = Color::rgb(1.0, 0.2, 0.2);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Nexus Core: Colony Manager".into(),
                    ..default()
                }),
                ..default()
            }),
            // REMOVED: ShapePlugin
            game_state::GameLogicPlugin,
            UiPlugin,
        ))
        .insert_resource(Time::<Fixed>::from_seconds(1.0))
        .run();
}

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MessageLog>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                button_interaction_system,
                research_button_system,
                update_text_display,
                draw_graph_gizmos, // REPLACED: draw_graph_system with a gizmo version
            ));
    }
}

#[derive(Resource, Default)]
struct MessageLog { message: String }

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, ..default() }, ..default()
    }).with_children(|parent| {
        // --- Top Bar ---
        parent.spawn(NodeBundle {
            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(10.0)), align_items: AlignItems::Center, flex_wrap: FlexWrap::Wrap, ..default() },
            background_color: Color::DARK_GRAY.into(), ..default()
        }).with_children(|parent| {
            parent.spawn((TextBundle::from_section("Power:", TextStyle { font_size: 20.0, ..default() }), PowerText));
            // Added CreditsText display
            parent.spawn((
                TextBundle::from_section(
                    "Credits: 0", // Initial text
                    TextStyle {
                        font_size: 18.0, // Consistent font size
                        color: Color::GOLD, // Distinct color for credits
                        ..default()
                    }
                )
                .with_style(Style {
                    margin: UiRect { left: Val::Px(15.0), ..default() }, // Consistent margin
                    ..default()
                }),
                CreditsText // Marker component
            ));
            parent.spawn((TextBundle::from_section("Nutrient Paste:", TextStyle { font_size: 20.0, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ResourceText(ResourceType::NutrientPaste)));
            parent.spawn((TextBundle::from_section("Ferrocrete Ore:", TextStyle { font_size: 20.0, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ResourceText(ResourceType::FerrocreteOre)));
            
            // Spawn TextBundles for new resources
            let new_resources_to_display = [
                ResourceType::CuprumDeposits,
                ResourceType::ManufacturedGoods,
                ResourceType::AdvancedComponents,
                ResourceType::RawXylos,
                ResourceType::RefinedXylos,
                ResourceType::RawQuantium,
                ResourceType::ProcessedQuantium,
            ];

            for resource_type in new_resources_to_display {
                parent.spawn((
                    TextBundle::from_section(
                        format!("{:?}: 0", resource_type), // Initial text
                        TextStyle {
                            font_size: 18.0, // Consistent font size
                            ..default() 
                        }
                    )
                    .with_style(Style {
                        margin: UiRect { left: Val::Px(15.0), ..default() }, // Consistent margin
                        ..default()
                    }),
                    ResourceText(resource_type) // Marker component
                ));
            }
            // TODO: Add UI elements for new resources like CuprumDeposits and display current stored Power resource. (Original TODO can be kept or removed as items are addressed)
            parent.spawn((TextBundle::from_section("Housing:", TextStyle { font_size: 20.0, color: HOUSING_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(40.0), ..default() }, ..default() }), ColonyStatText(StatType::Housing)));
            parent.spawn((TextBundle::from_section("Jobs:", TextStyle { font_size: 20.0, color: JOBS_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ColonyStatText(StatType::Jobs)));
            parent.spawn((TextBundle::from_section("Health:", TextStyle { font_size: 20.0, color: HEALTH_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ColonyStatText(StatType::Health)));
            parent.spawn((TextBundle::from_section("Police:", TextStyle { font_size: 20.0, color: POLICE_COLOR, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(20.0), ..default() }, ..default() }), ColonyStatText(StatType::Police)));
        });

        // --- Center Content (Graph Area) ---
        // This is now just a background panel. The gizmos will be drawn on top of it.
        parent.spawn((
            NodeBundle {
                style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), margin: UiRect::all(Val::Px(10.0)), ..default() },
                background_color: Color::rgba(0.1, 0.1, 0.1, 0.5).into(),
                ..default()
            },
            GraphArea, // Add marker to find its position and size
        ));

        // --- Bottom Bar ---
        parent.spawn(NodeBundle {
            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), align_items: AlignItems::Center, justify_content: JustifyContent::SpaceBetween, ..default() },
            background_color: Color::DARK_GRAY.into(), ..default()
        }).with_children(|parent| {
            parent.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, width: Val::Percent(75.0), ..default() }, ..default() })
                .with_children(|parent| {
                    let buildings = [
                        BuildingType::BioDome, BuildingType::Extractor, BuildingType::PowerRelay,
                        BuildingType::StorageSilo, BuildingType::ResearchInstitute,
                        BuildingType::Fabricator, BuildingType::ProcessingPlant, // Added
                        BuildingType::BasicDwelling, BuildingType::WellnessPost, BuildingType::SecurityStation,
                    ];
                    for building_type in buildings {
                        parent.spawn((ButtonBundle { style: Style { padding: UiRect::all(Val::Px(8.)), margin: UiRect::all(Val::Px(3.)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, background_color: NORMAL_BUTTON.into(), ..default() }, BuildButton(building_type)))
                                .with_children(|p| { p.spawn(TextBundle::from_section(format!("Build {:?}", building_type), TextStyle { font_size: 16.0, ..default() })); });
                    }
                });
            
            parent.spawn(NodeBundle { style: Style { align_items: AlignItems::Center, justify_content: JustifyContent::FlexEnd, flex_direction: FlexDirection::Row, ..default() }, ..default() })
                .with_children(|parent| {
                    // TODO: Add UI buttons for new research like EfficientExtraction.
                    parent.spawn((ButtonBundle { style: Style { padding: UiRect::all(Val::Px(8.)), margin: UiRect::horizontal(Val::Px(5.)), ..default() }, background_color: NORMAL_BUTTON.into(), ..default() }, ResearchButton(Tech::BasicConstructionProtocols)))
                            .with_children(|p| { p.spawn(TextBundle::from_section("Research Basic Construction", TextStyle { font_size: 16.0, ..default() })); });
                    parent.spawn((TextBundle::from_section("Welcome!", TextStyle { font_size: 20.0, ..default() }).with_style(Style{margin: UiRect::left(Val::Px(20.0)), ..default()}), MessageText));
                });
        });
    });
    // TODO: Spawn initial entities, e.g., a starting Operations Hub, some initial PowerRelays or a small amount of stored Power if not covered by GameState::default().

    // --- Test Administrative Spire functions ---
    // Example of how to call the functions (likely in a debug menu or specific game event later)
    // let mut game_state_res = world.resource_mut::<GameState>();
    // game_state::construct_administrative_spire(&mut game_state_res);
    // game_state::link_spire_to_hub(&mut game_state_res); // Link before certain upgrades
    // game_state::upgrade_administrative_spire(&mut game_state_res);
    // game_state::upgrade_administrative_spire(&mut game_state_res); // Try another upgrade
    // println!("Current Development Phase after tests: {:?}", game_state_res.current_development_phase);
    // if let Some(spire) = &game_state_res.administrative_spire {
    //     println!("Spire current tier index: {}, linked: {}", spire.current_tier_index, spire.is_linked_to_hub);
    // }
    // Note: To see the effects of these calls, you might need to run the game and observe console output,
    // or integrate these calls into UI buttons or game events that allow inspecting GameState.
    // The `game_state_res.current_development_phase` and `spire.current_tier_index` can be logged
    // as shown in the commented-out example.

    // --- Test Habitation and Population functions ---
    // (Access world directly as setup_ui is run once at startup)
    // let world = &mut app.world; // This line would need to be part of a system or a one-off setup.
                                // For direct calls in setup_ui, you'd pass `&mut Commands` and query `ResMut<GameState>`.
                                // The following are conceptual examples assuming `game_state_res` is available.
    // let mut game_state_res = world.resource_mut::<GameState>();
    // game_state::add_habitation_structure(&mut game_state_res, 0); // Add Basic Dwellings
    // game_state::add_habitation_structure(&mut game_state_res, 1); // Add Community Blocks
    // println!("Initial Housing: {}, Initial Inhabitants: {}", game_state_res.available_housing_capacity, game_state_res.total_inhabitants);
    //
    // // Simulate some game ticks for population growth
    // for _ in 0..10 {
    //     game_state::grow_inhabitants(&mut game_state_res);
    // }
    // println!("Inhabitants after 10 ticks: {}", game_state_res.total_inhabitants);
    //
    // if let Some(first_structure_id) = game_state_res.habitation_structures.first().map(|s| s.id.clone()) {
    //     game_state::assign_specialists_to_structure(&mut game_state_res, &first_structure_id, 1);
    //     game_state::upgrade_habitation_structure(&mut game_state_res, &first_structure_id);
    //     game_state::assign_specialists_to_structure(&mut game_state_res, &first_structure_id, 2); // Try to assign more after upgrade
    // }
    //
    // println!("Total Specialists: {}, Slots: {}", game_state_res.assigned_specialists_total, game_state_res.total_specialist_slots);
    // if let Some(first_structure) = game_state_res.habitation_structures.first() {
    //      println!("First structure: Tier {}, Inhabitants {}, Specialists {}", first_structure.tier_index, first_structure.current_inhabitants, first_structure.assigned_specialists);
    // }
    //
    // // Example of removing a structure
    // // if let Some(first_structure_id) = game_state_res.habitation_structures.first().map(|s| s.id.clone()) {
    // //     game_state::remove_habitation_structure(&mut game_state_res, &first_structure_id);
    // //     println!("After removal - Housing: {}, Inhabitants: {}, Specialists: {}", game_state_res.available_housing_capacity, game_state_res.total_inhabitants, game_state_res.assigned_specialists_total);
    // // }

    // --- Test Service Building, Zone, and Civic Index functions ---
    // (Conceptual examples, assuming `game_state_res` is available as ResMut<GameState> in a system or setup)
    // game_state::add_service_building(&mut game_state_res, game_state::ServiceType::Wellness, 0, Some((10.0, 20.0)));
    // game_state::add_zone(&mut game_state_res, game_state::ZoneType::Commercial, 0);
    // println!("Civic Index after adding buildings: {}", game_state_res.civic_index);
    // println!("Total Specialist Slots after adding zone: {}", game_state_res.total_specialist_slots);

    // if let Some(service_building_id) = game_state_res.service_buildings.first().map(|b| b.id.clone()) {
    //     // Need enough inhabitants first for specialists
    //     // Ensure game_state_res.total_inhabitants is sufficient by calling grow_inhabitants or setting it.
    //     // For testing, let's assume we have 10 inhabitants and 0 assigned specialists.
    //     // game_state_res.total_inhabitants = 10; game_state_res.assigned_specialists_total = 0;
    //     game_state::assign_specialists_to_service_building(&mut game_state_res, &service_building_id, 2);
    //     game_state::upgrade_service_building(&mut game_state_res, &service_building_id); // To Hospital (req 5)
    //     game_state::assign_specialists_to_service_building(&mut game_state_res, &service_building_id, 3); // Assign 3 more
    // }
    //
    // if let Some(zone_id) = game_state_res.zones.first().map(|z| z.id.clone()) {
    //     game_state::assign_specialists_to_zone(&mut game_state_res, &zone_id, 5);
    //     game_state::upgrade_zone(&mut game_state_res, &zone_id); // To Shopping Plaza (provides 15)
    //     game_state::assign_specialists_to_zone(&mut game_state_res, &zone_id, 10); // Assign 10 more
    // }
    //
    // println!("Civic Index after assignments/upgrades: {}", game_state_res.civic_index);
    // println!("Total assigned specialists: {}", game_state_res.assigned_specialists_total);
    // println!("Total specialist slots: {}", game_state_res.total_specialist_slots);
    //
    // // Example of removing a service building
    // // if let Some(service_building_id) = game_state_res.service_buildings.first().map(|b| b.id.clone()) {
    // //     game_state::remove_service_building(&mut game_state_res, &service_building_id);
    // //     println!("Civic Index after removing service: {}", game_state_res.civic_index);
    // //     println!("Total assigned specialists after removing service: {}", game_state_res.assigned_specialists_total);
    // // }

    // --- Test Happiness System ---
    // (Conceptual examples, assuming `game_state_res` is available as ResMut<GameState> in a system or setup)
    // // Initial state check (after default + 1 tick of calculation)
    // // game_state::calculate_colony_happiness(&mut game_state_res); // Call manually if not waiting for system tick
    // println!("Initial Colony Happiness: {:.2}%", game_state_res.colony_happiness);

    // // Scenario 1: Simulate food shortage
    // game_state_res.simulated_has_sufficient_nutrient_paste = false;
    // // game_state::calculate_colony_happiness(&mut game_state_res); // Recalculate
    // println!("Happiness after food shortage: {:.2}%", game_state_res.colony_happiness);
    // game_state_res.simulated_has_sufficient_nutrient_paste = true; // Reset

    // // Scenario 2: Add housing and grow population to be overcrowded
    // game_state::add_habitation_structure(&mut game_state_res, 0); // Basic Dwellings (Cap 10)
    // game_state_res.total_inhabitants = 15; // Overcrowd
    // // game_state::calculate_colony_happiness(&mut game_state_res);
    // println!("Happiness when overcrowded: {:.2}%", game_state_res.colony_happiness);
    // game_state_res.total_inhabitants = 5; // Reset population to normal for next test

    // // Scenario 3: Add a wellness service and staff it
    // game_state::add_service_building(&mut game_state_res, game_state::ServiceType::Wellness, 0, None); // Clinic (req 2 specialists)
    // if let Some(wellness_id) = game_state_res.service_buildings.last().map(|b| b.id.clone()) {
    //     // Ensure enough total_inhabitants and unassigned_specialists for this to succeed
    //     game_state_res.total_inhabitants = 10; // Ensure enough people exist
    //     game_state::assign_specialists_to_service_building(&mut game_state_res, &wellness_id, 2);
    // }
    // // game_state::calculate_colony_happiness(&mut game_state_res);
    // println!("Happiness with staffed wellness service: {:.2}%", game_state_res.colony_happiness);

    // // Scenario 4: Increase legacy bonus
    // game_state_res.legacy_structure_happiness_bonus = 10.0;
    // // game_state::calculate_colony_happiness(&mut game_state_res);
    // println!("Happiness with legacy bonus: {:.2}%", game_state_res.colony_happiness);
    // game_state_res.legacy_structure_happiness_bonus = 0.0; // Reset

    // // Note: To see these changes reflected by the system, you'd typically run the app and let
    // // the `calculate_colony_happiness_system` execute on its FixedUpdate schedule.
    // // Manual calls to `game_state::calculate_colony_happiness` are for immediate testing here.

    // --- Test Fabricator & Processing Plant functions ---
    // (Conceptual examples, assuming `game_state_res` is available as ResMut<GameState>)
    // // Add some starting resources for testing fabricators
    // *game_state_res.current_resources.entry(ResourceType::FerrocreteOre).or_insert(0.0) += 100.0;
    // *game_state_res.current_resources.entry(ResourceType::CuprumDeposits).or_insert(0.0) += 50.0;
    // *game_state_res.current_resources.entry(ResourceType::Power).or_insert(0.0) = 500.0; // Ensure enough power

    // // Add a Fabricator
    // game_state::add_fabricator(&mut game_state_res, 0); // Basic Fabricator
    // if let Some(fab_id) = game_state_res.fabricators.first().map(|f| f.id.clone()) {
    //     game_state_res.total_inhabitants = 10; // Ensure inhabitants for specialists
    //     game_state::assign_specialists_to_fabricator(&mut game_state_res, &fab_id, 1);
    //     println!("Fabricator {} created, assigned specialists.", fab_id);
    // }
    // // To test production, you'd let `fabricator_production_tick_system` run via App::update().
    // // Then check `game_state_res.current_resources.get(&ResourceType::ManufacturedGoods)`
    // // For example, after some ticks:
    // // println!("Manufactured Goods after some ticks: {}", game_state_res.current_resources.get(&ResourceType::ManufacturedGoods).unwrap_or(&0.0));

    // // Add a Processing Plant (Xylos Purifier)
    // game_state::add_processing_plant(&mut game_state_res, 0);
    // if let Some(plant_id) = game_state_res.processing_plants.first().map(|p| p.id.clone()) {
    //     game_state_res.total_inhabitants = 12; // More inhabitants for more specialists
    //     game_state::assign_specialists_to_processing_plant(&mut game_state_res, &plant_id, 2);
    //     println!("Processing Plant {} (Xylos Purifier) created, assigned specialists.", plant_id);
    //     // Check if RawXylos is unlocked
    //     if game_state_res.unlocked_raw_materials.contains(&ResourceType::RawXylos) {
    //         println!("RawXylos successfully unlocked for extraction!");
    //     }
    //     // Add some RawXylos to be processed
    //     *game_state_res.current_resources.entry(ResourceType::RawXylos).or_insert(0.0) += 20.0;
    // }
    // // To test processing, let `processing_plant_operations_tick_system` run.
    // // Then check `game_state_res.current_resources.get(&ResourceType::RefinedXylos)`
    // // For example, after some ticks:
    // // println!("Refined Xylos after some ticks: {}", game_state_res.current_resources.get(&ResourceType::RefinedXylos).unwrap_or(&0.0));
    // // println!("Raw Xylos remaining: {}", game_state_res.current_resources.get(&ResourceType::RawXylos).unwrap_or(&0.0));

    // --- Test Economic Model functions ---
    // (Conceptual examples, assuming `game_state_res` is available via `world.resource_mut::<GameState>()`)
    // println!("Initial Credits: {:.2}", game_state_res.credits);

    // // Simulate building something that costs credits
    // // (Assuming add_habitation_structure now deducts credits)
    // // game_state::add_habitation_structure(&mut game_state_res, 0); // Costs 100
    // // println!("Credits after building Habitation (Tier 0): {:.2}", game_state_res.credits);

    // // Simulate adding a commercial zone for income
    // // game_state::add_zone(&mut game_state_res, game_state::ZoneType::Commercial, 0); // Costs 100, Income 50
    // // if let Some(zone_id) = game_state_res.zones.first().map(|z| z.id.clone()) {
    // //     game_state::assign_specialists_to_zone(&mut game_state_res, &zone_id, 5); // Staff it
    // // }
    // // println!("Credits after building Commercial Zone (Tier 0): {:.2}", game_state_res.credits);

    // // Simulate adding a service building for upkeep
    // // game_state::add_service_building(&mut game_state_res, game_state::ServiceType::Wellness, 0, None); // Costs 150, Upkeep 10
    // // if let Some(service_id) = game_state_res.service_buildings.first().map(|b| b.id.clone()) {
    // //    game_state::assign_specialists_to_service_building(&mut game_state_res, &service_id, 2); // Staff it
    // // }
    // // println!("Credits after building Wellness Service (Tier 0): {:.2}", game_state_res.credits);
    
    // // To test income/upkeep systems, you would let the `upkeep_income_tick_system` run via App::update().
    // // For example, after a few "game days" (ticks, if running per tick):
    // // println!("Credits after some income/upkeep ticks: {:.2}", game_state_res.credits);

    // // Test building deactivation if upkeep isn't met
    // // game_state_res.credits = 5.0; // Set credits very low
    // // Manually call upkeep or let the system run: game_state::deduct_upkeep_system(&mut game_state_res);
    // // Then check `is_active` status of buildings.
    // // if let Some(zone) = game_state_res.zones.first() {
    // //     println!("Commercial zone active after low credits for upkeep: {}", zone.is_active);
    // // }
    // // if let Some(service) = game_state_res.service_buildings.first() {
    // //     println!("Wellness service active after low credits for upkeep: {}", service.is_active);
    // // }
}

// NEW: This system uses Gizmos to draw the graph, avoiding UI conflicts.
fn draw_graph_gizmos(
    mut gizmos: Gizmos,
    graph_data: Res<GraphData>,
    graph_area_query: Query<(&Node, &GlobalTransform), With<GraphArea>>,
) {
    if graph_data.history.is_empty() { return; }

    let (graph_node, transform) = graph_area_query.single();
    let graph_area = graph_node.size();
    
    // The Gizmos are drawn in world space, so we need the bottom-left corner of our UI node.
    let bottom_left = transform.translation().truncate() - graph_area / 2.0;

    let max_val = graph_data.history.iter().fold(100.0f32, |max, stats| {
        max.max(stats.total_housing as f32).max(stats.total_jobs as f32).max(stats.health_capacity as f32).max(stats.police_capacity as f32)
    });

    let stat_types = [
        (StatType::Housing, HOUSING_COLOR),
        (StatType::Jobs, JOBS_COLOR),
        (StatType::Health, HEALTH_COLOR),
        (StatType::Police, POLICE_COLOR),
    ];

    for (stat_type, color) in stat_types {
        let mut points: Vec<Vec2> = Vec::new();
        for (i, stats) in graph_data.history.iter().enumerate() {
            let value = match stat_type {
                StatType::Housing => stats.total_housing as f32,
                StatType::Jobs => stats.total_jobs as f32,
                StatType::Health => stats.health_capacity as f32,
                StatType::Police => stats.police_capacity as f32,
            };
            
            let x = graph_area.x - (i as f32 * (graph_area.x / 200.0));
            let y = (value / max_val) * graph_area.y;
            
            if x >= 0.0 && y >= 0.0 {
                // Add the panel's bottom-left corner to translate to world space
                points.push(bottom_left + Vec2::new(x, y));
            }
        }

        if points.len() > 1 {
            gizmos.linestrip_2d(points, color);
        }
    }
}

fn button_interaction_system(
    mut interaction_query: Query<(&Interaction, &BuildButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut log: ResMut<MessageLog>,
) {
    for (interaction, build_button, mut color) in &mut interaction_query {
        let building_type = build_button.0;
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if (building_type == BuildingType::BasicDwelling || building_type == BuildingType::WellnessPost || building_type == BuildingType::SecurityStation) 
                   && !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                    log.message = "Requires Basic Construction Protocols".to_string();
                    continue; 
                }
                
                let costs = game_state.building_costs.get(&building_type).unwrap().clone();
                if costs.iter().all(|(res, &cost)| game_state.current_resources.get(res).unwrap_or(&0.0) >= &cost) {
                    for (res, cost) in &costs { *game_state.current_resources.get_mut(res).unwrap() -= cost; }
                    log.message = format!("Construction started: {:?}", building_type);
                    // TODO: Add entity spawning logic for Fabricator and ProcessingPlant.
                    match building_type {
                        BuildingType::BioDome => { commands.spawn(BioDome { power_consumption: 10, production_rate: 5.0 }); }
                        BuildingType::Extractor => { commands.spawn(Extractor { power_consumption: 15, resource_type: ResourceType::FerrocreteOre, extraction_rate: 2.5 }); }
                        BuildingType::PowerRelay => { commands.spawn(PowerRelay { power_output: 50 }); }
                        BuildingType::StorageSilo => { commands.spawn(StorageSilo { capacity: 1000 }); }
                        BuildingType::ResearchInstitute => { commands.spawn(ResearchInstitute { power_consumption: 5 }); } // Added power_consumption
                        BuildingType::BasicDwelling => { commands.spawn(BasicDwelling { housing_capacity: 100 }); }
                        BuildingType::WellnessPost => { commands.spawn(WellnessPost { health_capacity: 50, jobs_provided: 5 }); }
                        BuildingType::SecurityStation => { commands.spawn(SecurityStation { police_capacity: 50, jobs_provided: 5 }); }
                        BuildingType::Fabricator => {
                            // GameState-managed, credit check is inside add_fabricator
                            game_state::add_fabricator(&mut game_state, 0); 
                            // The material cost check above is for Bevy component buildings.
                            // If Fabricator also had material costs, that would need to be reconciled.
                            // For now, assuming add_fabricator handles all its own prerequisites.
                            log.message = "Fabricator construction initiated.".to_string();
                        }
                        BuildingType::ProcessingPlant => {
                            game_state::add_processing_plant(&mut game_state, 0);
                            log.message = "Processing Plant construction initiated.".to_string();
                        }
                    }
                } else {
                    // This message is for material costs of Bevy Component buildings.
                    // Credit costs for GameState managed buildings are handled inside their add_* functions.
                    log.message = "Not enough material resources!".to_string();
                }
            }
            Interaction::Hovered => { *color = HOVERED_BUTTON.into(); }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}

fn research_button_system(
    mut interaction_query: Query<(&Interaction, &ResearchButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>,
    institute_q: Query<&ResearchInstitute>,
) {
    for (interaction, research_button, mut color) in &mut interaction_query {
        let tech = research_button.0;
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if institute_q.is_empty() {
                    log.message = "Must build a Research Institute first.".to_string();
                } else if game_state.research_progress.is_some() {
                    log.message = "Research already in progress.".to_string();
                } else if game_state.unlocked_techs.contains(&tech) {
                    log.message = "Technology already researched.".to_string();
                } else {
                    // Credit Check for Research
                    let credit_cost = *game_state.tech_costs.get(&tech).unwrap_or(&0_u32); // Get u32 cost
                    if game_state.credits >= credit_cost as f64 {
                        game_state.credits -= credit_cost as f64;
                        log.message = format!("Researching {:?} for {} Credits...", tech, credit_cost);
                        game_state.research_progress = Some((tech, 0.0));
                    } else {
                        log.message = format!("Not enough Credits to research {:?}. Cost: {} Credits, Available: {:.0}", tech, credit_cost, game_state.credits);
                    }
                    // TODO: Ensure new research Techs like EfficientExtraction can be selected and processed.
                }
            }
            Interaction::Hovered => { *color = HOVERED_BUTTON.into(); }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}

fn update_text_display(
    game_state: Res<GameState>,
    stats: Res<ColonyStats>,
    log: Res<MessageLog>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<PowerText>>,
        Query<(&mut Text, &ResourceText)>,
        Query<(&mut Text, &ColonyStatText)>,
        Query<&mut Text, With<MessageText>>,
        Query<&mut Text, With<CreditsText>>, // Added query for CreditsText
    )>,
    // Removed: power_q, extractor_q, biodome_q, research_institute_q
    // as power data will now come from game_state
) {
    // TODO: Add display for new resources like CuprumDeposits. (This specific TODO might be covered now)
    for (mut text, resource_marker) in text_queries.p1().iter_mut() {
        let amount = game_state.current_resources.get(&resource_marker.0).unwrap_or(&0.0);
        // Updated formatting to whole number
        text.sections[0].value = format!("{:?}: {:.0}", resource_marker.0, amount); 
    }
    
    // Updated power display to use GameState fields
    for mut text in text_queries.p0().iter_mut() {
        let stored_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0);
        let gen = game_state.total_generated_power;
        let con = game_state.total_consumed_power;
        let net = gen - con;
        text.sections[0].value = format!(
            "Power: {:.0} (Stored) | Gen: {:.0}, Con: {:.0}, Net: {:.0}",
            stored_power,
            gen,
            con,
            net
        );
    }

    for (mut text, stat_marker) in text_queries.p2().iter_mut() {
        text.sections[0].value = match stat_marker.0 {
            StatType::Housing => format!("Housing: {}", stats.total_housing),
            StatType::Jobs => format!("Jobs: {}", stats.total_jobs),
            StatType::Health => format!("Health: {}", stats.health_capacity),
            StatType::Police => format!("Police: {}", stats.police_capacity),
        };
    }
    
    if log.is_changed() {
        for mut text in text_queries.p3().iter_mut() {
            text.sections[0].value = log.message.clone();
        }
    }

    // Update Credits display
    for mut text in text_queries.p4().iter_mut() { // p4 is the new index for CreditsText
        text.sections[0].value = format!("Credits: {:.0}", game_state.credits);
    }
}