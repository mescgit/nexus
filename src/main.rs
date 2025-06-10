// src/main.rs

use bevy::prelude::*;
// REMOVED: `use bevy_prototype_lyon::prelude::*;`
use game_state::{
    BasicDwelling, BioDome, BuildingType, ColonyStats, Extractor, GameState, GraphData, PowerRelay,
    ResearchInstitute, ResourceType, SecurityStation, StorageSilo, Tech, WellnessPost,
};

mod game_state;

// Define the trait
trait GraphableFn: Fn(&ColonyStats) -> f32 {}

// Implement the trait for all closures that fit the signature
impl<F: Fn(&ColonyStats) -> f32> GraphableFn for F {}

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
// #[derive(Component)] // Old HappinessText, replaced by ColonyHappinessText for ticker
// struct HappinessText; // Marker component for Happiness - Keep if used elsewhere, remove if only for old ticker
#[derive(Component)]
struct PopulationText; // For Status Ticker
#[derive(Component)]
struct CoreResourceText(ResourceType); // For Status Ticker
#[derive(Component)]
struct ColonyHappinessText; // For Status Ticker
#[derive(Component)]
struct ConstructSpireButton;
#[derive(Component)]
struct UpgradeSpireButton;
#[derive(Component)]
struct BuildHabitationButton(usize); // usize is the tier_index
#[derive(Component)]
struct BuildServiceBuildingButton {
    service_type: game_state::ServiceType, // Ensure game_state::ServiceType is in scope
    tier_index: usize,
}
#[derive(Component)]
struct BuildZoneButton {
    zone_type: game_state::ZoneType, // Ensure game_state::ZoneType is in scope
    tier_index: usize,
}
#[derive(Component)]
struct GraphArea; // Marker for the graph's background node

// --- App Panel Marker Components ---
#[derive(Component)]
struct DashboardPanel;
#[derive(Component)]
struct NotificationsPanel;
#[derive(Component)]
struct AnalyticsGraphPanel;
#[derive(Component)]
struct ConstructionPanel;
#[derive(Component)]
struct ColonyStatusPanel;
#[derive(Component)]
struct ResearchPanel;

#[derive(Component)] // Marker for App Drawer buttons
struct AppDrawerButton(AppType);

use crate::game_state::{BuildingType as GameBuildingType, DevelopmentPhase, ALL_BUILDING_TYPES}; // Renamed to avoid conflict
use std::collections::HashMap;

// Define BuildingMetadata and its getter function
#[derive(Clone, Copy, Debug)]
pub struct BuildingMetadata {
    pub name: &'static str,
    pub category: ConstructionCategory,
    pub required_tech: Option<Tech>,
    pub required_dp: Option<DevelopmentPhase>, // Placeholder for now
}

fn get_building_metadata() -> HashMap<GameBuildingType, BuildingMetadata> {
    let mut meta = HashMap::new();
    meta.insert(GameBuildingType::Extractor, BuildingMetadata { name: "Extractor", category: ConstructionCategory::Operations, required_tech: None, required_dp: None });
    meta.insert(GameBuildingType::BioDome, BuildingMetadata { name: "Bio-Dome", category: ConstructionCategory::Operations, required_tech: None, required_dp: None });
    meta.insert(GameBuildingType::PowerRelay, BuildingMetadata { name: "Power Relay", category: ConstructionCategory::Operations, required_tech: None, required_dp: None });
    meta.insert(GameBuildingType::StorageSilo, BuildingMetadata { name: "Storage Silo", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::ResearchInstitute, BuildingMetadata { name: "Research Institute", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    
    // GameState managed buildings - costs are primarily handled by their add_... functions
    // For this metadata, we mostly care about category and tech pre-requisites for showing them in the list.
    // Affordability check will be simplified for them initially.
    meta.insert(GameBuildingType::Fabricator, BuildingMetadata { name: "Fabricator", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::ProcessingPlant, BuildingMetadata { name: "Processing Plant", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });

    // Habitation category often refers to GameState managed structures
    meta.insert(GameBuildingType::BasicDwelling, BuildingMetadata { name: "Basic Dwelling", category: ConstructionCategory::Habitation, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    // Note: WellnessPost and SecurityStation are Bevy components, not directly GameState managed structures like Habitation tiers,
    // so their costs would ideally be in game_state.building_costs if they have material costs.
    // For now, grouping them under Habitation for UI purposes.
    meta.insert(GameBuildingType::WellnessPost, BuildingMetadata { name: "Wellness Post", category: ConstructionCategory::Habitation, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    meta.insert(GameBuildingType::SecurityStation, BuildingMetadata { name: "Security Station", category: ConstructionCategory::Habitation, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None });
    
    // Example for Legacy if any 'old' Bevy component buildings were to be listed here.
    // meta.insert(BuildingType::LegacyStructure, BuildingMetadata { name: "Old Extractor Mk1", category: ConstructionCategory::Legacy, required_tech: None, required_dp: None });
    meta
}


// --- Construction App Enums, Resources, and Components ---
#[derive(Resource, Default, Debug)] // Added Debug
pub struct SelectedBuilding(pub Option<GameBuildingType>);

#[derive(Component)]
struct ConfirmBuildButton(GameBuildingType);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConstructionCategory {
    #[default]
    Operations, // e.g. Extractors, Power Plants, Core GameState buildings
    Habitation, // Habitation structures from game_state
    Legacy,     // Original Bevy Component buildings (BioDome, old Extractor, etc if kept)
}

#[derive(Resource, Default)]
pub struct CurrentConstructionCategory(pub ConstructionCategory);

#[derive(Component)]
struct ConstructionCategoryTab(ConstructionCategory);

#[derive(Component)]
struct ConstructionItemListPanel; // Panel to hold the list of buildable items

#[derive(Component)]
struct ConstructionItemButton(GameBuildingType); // For items in the list (using GameBuildingType)

#[derive(Component)]
struct ConstructionItemDetailsPanel; // Panel for selected item's details


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppType {
    #[default]
    Dashboard,
    Construction,
    ColonyStatus,
    Research,
}

#[derive(Resource, Default)]
pub struct CurrentApp(pub AppType);

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

// New colors for additional graph lines
const CREDITS_COLOR: Color = Color::rgb(0.9, 0.8, 0.2); // Gold-ish
const NET_POWER_COLOR: Color = Color::rgb(0.4, 0.6, 1.0); // Light Blue
const HAPPINESS_COLOR: Color = Color::rgb(1.0, 0.5, 0.8); // Pink-ish
const NUTRIENT_PASTE_COLOR: Color = Color::rgb(0.5, 0.9, 0.3); // Green-ish

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
            .init_resource::<CurrentApp>() 
            .init_resource::<CurrentConstructionCategory>() 
            .init_resource::<SelectedBuilding>() // Initialize SelectedBuilding resource
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (
                app_drawer_button_system,
                manage_app_panels_visibility, 
                update_dashboard_notifications_system.after(manage_app_panels_visibility),
                construction_category_tab_system, 
                update_construction_list_system.after(construction_category_tab_system),
                construction_item_interaction_system.after(update_construction_list_system),
                update_construction_details_panel_system.after(construction_item_interaction_system),
                construction_interaction_system.after(update_construction_details_panel_system), 
                update_text_display,
                draw_graph_gizmos, 
                // Old systems fully removed
            ));
    }
}

// Old button interaction systems (habitation_button_system, zone_button_system, 
// service_building_button_system, button_interaction_system, research_button_system, admin_spire_button_system)
// have been removed from the file. Their functionality is replaced by the new Construction App UI.

#[derive(Resource, Default)]
struct MessageLog { message: String } // This is still used by update_text_display for the bottom bar message.
                                      // It will be replaced by new notification system eventually.

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::SpaceBetween, ..default() }, ..default()
    }).with_children(|parent| {
        // --- Top Bar ---
        // --- Top Bar (Status Ticker) ---
        parent.spawn(NodeBundle {
            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(10.0)), align_items: AlignItems::Center, flex_wrap: FlexWrap::Wrap, ..default() },
            background_color: Color::DARK_GRAY.into(), ..default()
        }).with_children(|parent| {
            // Clear old elements by not spawning them. New elements below:

            // Credits
            parent.spawn((TextBundle::from_section("Cr. 0", TextStyle { font_size: 18.0, color: Color::GOLD, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(15.0), ..default() }, ..default() }), CreditsText));
            
            // Power
            parent.spawn((TextBundle::from_section("Power - Net: +0 | Stored: 0/0", TextStyle { font_size: 18.0, color: Color::CYAN, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(15.0), ..default() }, ..default() }), PowerText));
            
            // Population
            parent.spawn((TextBundle::from_section("Inhabitants: 0 / 0", TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(15.0), ..default() }, ..default() }), PopulationText));
            
            // Core Resources
            let core_resources_to_display = [ResourceType::NutrientPaste, ResourceType::FerrocreteOre, ResourceType::CuprumDeposits];
            for resource_type in core_resources_to_display {
                parent.spawn((TextBundle::from_section(format!("{:?}: 0", resource_type), TextStyle { font_size: 18.0, color: Color::GRAY, ..default() }).with_style(Style { margin: UiRect { right: Val::Px(10.0), ..default() }, ..default() }), CoreResourceText(resource_type)));
            }
            
            // Colony Happiness (ensure this is at the end or use justify_content on parent if more items are added to push it right)
            parent.spawn((TextBundle::from_section("😐 0%", TextStyle { font_size: 18.0, color: Color::YELLOW, ..default() }).with_style(Style { margin: UiRect { left: Val::Px(15.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0) }, ..default() }), ColonyHappinessText));
        });

        // --- New Middle Area (App Drawer + Colony Viewport) ---
        parent.spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0), // Or flex_grow: 1.0 if TopBar has fixed height
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            // --- App Drawer (Left) ---
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(15.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                background_color: Color::rgb(0.1, 0.1, 0.1).into(),
                ..default()
            }).with_children(|parent| {
                let app_buttons = [AppType::Dashboard, AppType::Construction, AppType::ColonyStatus, AppType::Research];
                for app_type in app_buttons {
                    parent.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Percent(95.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                margin: UiRect::all(Val::Px(3.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        AppDrawerButton(app_type),
                    )).with_children(|p| {
                        p.spawn(TextBundle::from_section(format!("{:?}", app_type), TextStyle { font_size: 18.0, ..default() }));
                    });
                }
            });

            // --- Colony Viewport (Right) ---
            // This NodeBundle is the main content area for different "Apps"
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(85.0),
                    height: Val::Percent(100.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column, // Children panels will stack vertically
                    ..default()
                },
                ..default()
            }).with_children(|parent| {
                // --- Dashboard Panel ---
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            display: Display::None, // Initially hidden
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    },
                    DashboardPanel,
                )).with_children(|dashboard_content| {
                    // Notifications Panel (within Dashboard)
                    dashboard_content.spawn((
                        NodeBundle {
                            style: Style {
                                flex_grow: 0.3, // Takes 30% of DashboardPanel height
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            background_color: Color::rgba(0.0, 0.0, 0.0, 0.3).into(),
                            ..default()
                        },
                        NotificationsPanel,
                    )).with_children(|notifications_parent| {
                         notifications_parent.spawn(TextBundle::from_section("Notifications Appear Here", TextStyle {font_size: 16.0, color: Color::GRAY, ..default()}));
                    });

                    // Analytics Graph Panel (within Dashboard)
                    dashboard_content.spawn((
                        NodeBundle {
                            style: Style {
                                flex_grow: 0.7, // Takes 70% of DashboardPanel height
                                ..default()
                            },
                            ..default()
                        },
                        AnalyticsGraphPanel,
                    )).with_children(|graph_parent| {
                        // Move existing GraphArea here
                        graph_parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                background_color: Color::rgba(0.1, 0.1, 0.1, 0.5).into(), // Original GraphArea background
                                ..default()
                            },
                            GraphArea, 
                        ));
                    });
                });

                // --- Construction Panel ---
                parent.spawn((
                    NodeBundle {
                        style: Style { 
                            width: Val::Percent(100.0), 
                            height: Val::Percent(100.0), 
                            display: Display::None, // Initially hidden
                            flex_direction: FlexDirection::Column, // Main axis for this panel
                            ..default()
                        },
                        ..default()
                    },
                    ConstructionPanel,
                )).with_children(|construction_panel_content| {
                    // Remove placeholder: p.spawn(TextBundle::from_section("Construction App Content", TextStyle {font_size: 24.0, ..default()}));
                    
                    // Tabs Section
                    construction_panel_content.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(40.0), // Fixed height for tab bar
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    }).with_children(|tabs_bar| {
                        let categories = [ConstructionCategory::Operations, ConstructionCategory::Habitation, ConstructionCategory::Legacy];
                        for category in categories {
                            tabs_bar.spawn((
                                ButtonBundle {
                                    style: Style {
                                        padding: UiRect::all(Val::Px(8.0)),
                                        margin: UiRect::horizontal(Val::Px(5.0)),
                                        ..default()
                                    },
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                ConstructionCategoryTab(category),
                            )).with_children(|button_content| {
                                button_content.spawn(TextBundle::from_section(format!("{:?}", category), TextStyle {font_size: 16.0, ..default()}));
                            });
                        }
                    });

                    // Building List Section
                    construction_panel_content.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_grow: 1.0, // Takes remaining vertical space
                                flex_direction: FlexDirection::Column, // Items will list vertically
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            background_color: Color::rgba(0.0, 0.0, 0.0, 0.2).into(),
                            ..default()
                        },
                        ConstructionItemListPanel,
                    )).with_children(|list_panel| {
                        list_panel.spawn(TextBundle::from_section("Select a category to see buildable items.", TextStyle {font_size: 18.0, color: Color::GRAY, ..default()}));
                    });

                    // Item Details Section (Placeholder)
                    construction_panel_content.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(100.0), // Fixed height for now
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            background_color: Color::rgba(0.0, 0.0, 0.0, 0.4).into(),
                            ..default()
                        },
                        ConstructionItemDetailsPanel,
                    )).with_children(|details_panel| {
                        details_panel.spawn(TextBundle::from_section("Select an item to see details.", TextStyle {font_size: 16.0, color: Color::GRAY, ..default()}));
                    });
                });
                
                // --- ColonyStatus Panel ---
                parent.spawn((
                    NodeBundle {
                        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()},
                        ..default()
                    },
                    ColonyStatusPanel,
                )).with_children(|p| { p.spawn(TextBundle::from_section("Colony Status App Content", TextStyle {font_size: 24.0, ..default()})); });

                // --- Research Panel ---
                parent.spawn((
                    NodeBundle {
                        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), display: Display::None, justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default()},
                        ..default()
                    },
                    ResearchPanel,
                )).with_children(|p| { p.spawn(TextBundle::from_section("Research App Content", TextStyle {font_size: 24.0, ..default()})); });
            });
        });

        // --- Bottom Bar (Removed) ---
        // The entire bottom bar NodeBundle and its children are removed.
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

    let max_val = graph_data.history.iter().fold(100.0f32, |max_so_far, stats| {
        max_so_far
            .max(stats.total_housing as f32)
            .max(stats.total_jobs as f32)
            .max(stats.health_capacity as f32)
            .max(stats.police_capacity as f32)
            .max(stats.credits as f32)      // Add this
            .max(stats.net_power.abs())     // Add this (use abs for potential negative net_power for scaling)
            .max(stats.happiness as f32)    // Add this
            .max(stats.nutrient_paste as f32) // Add this
    });

    let graph_lines: [(Color, Box<dyn GraphableFn>); 8] = [
        (HOUSING_COLOR, Box::new(|stats: &ColonyStats| stats.total_housing as f32)),
        (JOBS_COLOR, Box::new(|stats: &ColonyStats| stats.total_jobs as f32)),
        (HEALTH_COLOR, Box::new(|stats: &ColonyStats| stats.health_capacity as f32)),
        (POLICE_COLOR, Box::new(|stats: &ColonyStats| stats.police_capacity as f32)),
        (CREDITS_COLOR, Box::new(|stats: &ColonyStats| stats.credits as f32)),
        (NET_POWER_COLOR, Box::new(|stats: &ColonyStats| stats.net_power as f32)),
        (HAPPINESS_COLOR, Box::new(|stats: &ColonyStats| stats.happiness as f32)),
        (NUTRIENT_PASTE_COLOR, Box::new(|stats: &ColonyStats| stats.nutrient_paste as f32)),
    ];

    for (color, get_value) in graph_lines.iter() { // Iterate with .iter()
        let mut points: Vec<Vec2> = Vec::new();
        for (i, stats) in graph_data.history.iter().enumerate() {
            let value = get_value(stats); // Calling the boxed closure

            // X-coordinate calculation (newest data on the right)
            let point_x_offset = (i as f32 / graph_data.history.len().max(1) as f32) * graph_area.x;
            let x = graph_area.x - point_x_offset;

            // Y-coordinate calculation with scaling
            let y_scaled = if max_val == 0.0 { 0.0 } else { (value / max_val) * graph_area.y };
            
            // Clamp y to be within the visible graph area [0, graph_area.y]
            // This means negative values (like for net_power) will be at the bottom line.
            // Ensure points are within the x-bounds before adding.
            if x >= 0.0 && x <= graph_area.x {
                 points.push(bottom_left + Vec2::new(x, y_scaled.clamp(0.0, graph_area.y)));
            }
        }

        if points.len() > 1 {
            gizmos.linestrip_2d(points, *color); // Dereference color
        }
    }
}

fn zone_button_system(
    mut interaction_query: Query<(&Interaction, &BuildZoneButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>,
) {
    for (interaction, button_data, mut color) in &mut interaction_query {
        let zone_type = button_data.zone_type;
        let tier_index = button_data.tier_index;

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                // Tech check (assuming BasicConstructionProtocols for all zones for now)
                if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                    log.message = "Requires Basic Construction Protocols.".to_string();
                    continue; 
                }

                let initial_credits = game_state.credits;
                let initial_zone_count = game_state.zones.len();

                game_state::add_zone(&mut game_state, zone_type, tier_index);
                
                if game_state.zones.len() > initial_zone_count && game_state.credits < initial_credits {
                    // Success: new zone added and credits deducted.
                    let all_tiers_for_type = game_state::get_zone_tiers(zone_type);
                    let name = all_tiers_for_type.get(tier_index).map_or_else(
                        || format!("{:?} Tier {}", zone_type, tier_index),
                        |t| t.name.clone()
                    );
                    log.message = format!("{} developed.", name);
                } else if game_state.credits == initial_credits && game_state.zones.len() == initial_zone_count {
                    // Failed, likely due to cost. game_state::add_zone prints to console.
                    let all_tiers_for_type = game_state::get_zone_tiers(zone_type);
                    if let Some(tier_info) = all_tiers_for_type.get(tier_index) {
                        log.message = format!("Failed: Need {} credits for {}.", tier_info.construction_credits_cost, tier_info.name);
                    } else {
                        log.message = format!("Failed to develop {:?} (Tier {}). Check credits/console.", zone_type, tier_index);
                    }
                } else {
                    log.message = format!("Action for {:?} (Tier {}). Check console.", zone_type, tier_index);
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                let tiers = game_state::get_zone_tiers(button_data.zone_type);
                if let Some(tier_info) = tiers.get(button_data.tier_index) {
                    let mut message = format!("{}: Cost {} Credits. Upkeep: {} Credits.", tier_info.name, tier_info.construction_credits_cost, tier_info.upkeep_cost);
                    if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                        message.push_str(" (Req: Basic Construction Protocols)");
                    }
                    log.message = message;
                } else {
                    log.message = format!("Hovering Zone {:?} Tier {}.", button_data.zone_type, button_data.tier_index);
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}

fn service_building_button_system(
    mut interaction_query: Query<(&Interaction, &BuildServiceBuildingButton, &mut BackgroundColor), Changed<Interaction>>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>,
) {
    for (interaction, button_data, mut color) in &mut interaction_query {
        let service_type = button_data.service_type;
        let tier_index = button_data.tier_index;

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                // Tech check (assuming BasicConstructionProtocols for all service buildings for now)
                if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                    log.message = "Requires Basic Construction Protocols.".to_string();
                    continue; 
                }

                let initial_credits = game_state.credits;
                let initial_building_count = game_state.service_buildings.len();

                // The add_service_building function in game_state takes an Option<(f32, f32)> for position.
                // For UI buttons, we'll pass None for now, meaning no specific position.
                game_state::add_service_building(&mut game_state, service_type, tier_index, None);
                
                if game_state.service_buildings.len() > initial_building_count && game_state.credits < initial_credits {
                    // Success: new building added and credits deducted.
                    let tiers_for_type = game_state::get_service_building_tiers(service_type);
                    let name = tiers_for_type.get(tier_index).map_or_else(
                        || format!("{:?} Tier {}", service_type, tier_index),
                        |t| t.name.clone()
                    );
                    log.message = format!("{} constructed.", name);
                } else if game_state.credits == initial_credits && game_state.service_buildings.len() == initial_building_count {
                    // Failed, likely due to cost. game_state::add_service_building prints to console.
                    let all_tiers_for_type = game_state::get_service_building_tiers(service_type);
                    if let Some(tier_info) = all_tiers_for_type.get(tier_index) {
                        log.message = format!("Failed: Need {} credits for {}.", tier_info.construction_credits_cost, tier_info.name);
                    } else {
                        log.message = format!("Failed to build {:?} (Tier {}). Check credits/console.", service_type, tier_index);
                    }
                } else {
                    log.message = format!("Action for {:?} (Tier {}). Check console.", service_type, tier_index);
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                let tiers = game_state::get_service_building_tiers(button_data.service_type);
                if let Some(tier_info) = tiers.get(button_data.tier_index) {
                    let mut message = format!("{}: Cost {} Credits. Upkeep: {} Credits.", tier_info.name, tier_info.construction_credits_cost, tier_info.upkeep_cost);
                    if !game_state.unlocked_techs.contains(&Tech::BasicConstructionProtocols) {
                        message.push_str(" (Req: Basic Construction Protocols)");
                    }
                    log.message = message;
                } else {
                    log.message = format!("Hovering Service {:?} Tier {}.", button_data.service_type, button_data.tier_index);
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
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
        Query<&mut Text, With<PowerText>>,           // p0 (Ticker Power)
        Query<(&mut Text, &ResourceText)>,           // p1 (Old general resources - review if needed)
        Query<(&mut Text, &ColonyStatText)>,         // p2 (Old stats for graph, not ticker UI - review if needed)
        Query<&mut Text, With<MessageText>>,         // p3 (Message log)
        Query<&mut Text, With<CreditsText>>,         // p4 (Ticker Credits)
        Query<&mut Text, With<PopulationText>>,     // p5 (New Ticker Population)
        Query<(&mut Text, &CoreResourceText)>,       // p6 (New Ticker Core Resources)
        Query<&mut Text, With<ColonyHappinessText>>, // p7 (New Ticker Colony Happiness)
        // Query<&mut Text, With<HappinessText>> // Removed old HappinessText query
    )>,
) {
    // Update Ticker Credits display (p4)
    for mut text in text_queries.p4().iter_mut() {
        text.sections[0].value = format!("Cr. {:.0}", game_state.credits);
        // Flashing logic can be added here later if needed
    }

    // Update Ticker Power display (p0)
    for mut text in text_queries.p0().iter_mut() {
        let net_power = game_state.total_generated_power - game_state.total_consumed_power;
        let stored_power = *game_state.current_resources.get(&ResourceType::Power).unwrap_or(&0.0);
        let max_power_storage = 5000.0; // Placeholder, as per plan. Ensure GameState might have this later.
        text.sections[0].value = format!("Power - Net: {:+.0} | Stored: {:.0}/{:.0}", net_power, stored_power, max_power_storage);
        text.sections[0].style.color = if net_power < 0.0 { Color::RED } else { Color::CYAN };
    }

    // Update Ticker Population display (p5)
    for mut text in text_queries.p5().iter_mut() {
        text.sections[0].value = format!("Inhabitants: {} / {}", game_state.total_inhabitants, game_state.available_housing_capacity);
        let housing_ratio = if game_state.available_housing_capacity > 0 {
            game_state.total_inhabitants as f32 / game_state.available_housing_capacity as f32
        } else if game_state.total_inhabitants > 0 { // More people than houses (0 houses)
            2.0 // Critical situation
        } else { // 0 people, 0 houses
            0.0 // Neutral or normal
        };
        text.sections[0].style.color = if housing_ratio >= 0.9 { Color::rgb(1.0, 0.9, 0.3) } else { Color::WHITE }; // Yellowish if 90% full or more
    }

    // Update Ticker Core Resources display (p6)
    for (mut text, core_resource_marker) in text_queries.p6().iter_mut() {
        let amount = game_state.current_resources.get(&core_resource_marker.0).unwrap_or(&0.0);
        text.sections[0].value = format!("{:?}: {:.0}", core_resource_marker.0, amount);
    }

    // Update Ticker Colony Happiness display (p7)
    for mut text in text_queries.p7().iter_mut() {
        let happiness_icon = match game_state.colony_happiness {
            h if h >= 85.0 => "😊",
            h if h >= 50.0 => "😐",
            _ => "☹️",
        };
        text.sections[0].value = format!("{} {:.0}%", happiness_icon, game_state.colony_happiness);
        text.sections[0].style.color = match game_state.colony_happiness {
            h if h >= 85.0 => Color::rgb(0.3, 1.0, 0.3), // Green
            h if h >= 50.0 => Color::rgb(1.0, 0.9, 0.3), // Yellowish
            _ => Color::rgb(1.0, 0.3, 0.3),              // Reddish
        };
    }
    
    // Update logic for general ResourceText (p1) - if these are not part of the new ticker, this section might be deprecated or only for other UI.
    // For now, we assume it's for other UI parts, or it will simply not find any matching entities if they were only in the old top bar.
    for (mut text, resource_marker) in text_queries.p1().iter_mut() {
        // This section should only update ResourceText entities NOT handled by CoreResourceText in the ticker.
        // If all resource displays are now CoreResourceText, this loop might become unused.
        let amount = game_state.current_resources.get(&resource_marker.0).unwrap_or(&0.0);
        text.sections[0].value = format!("{:?}: {:.0}", resource_marker.0, amount); 
    }
    
    // Update logic for ColonyStatText (p2) - these were in the old top bar (Housing, Jobs, etc.)
    // These specific TextBundles are removed from the top bar.
    // The ColonyStats resource is still updated by `update_colony_stats_system` and used by the graph.
    // This loop will no longer find the Text entities in the top bar.
    // If ColonyStatText is used elsewhere in the UI, that logic would remain.
    // For now, this loop will likely do nothing as the top bar elements are gone.
    for (mut text, stat_marker) in text_queries.p2().iter_mut() {
        text.sections[0].value = match stat_marker.0 {
            StatType::Housing => format!("Housing: {}", stats.total_housing), // Example, will not find entity if it was only in top bar
            StatType::Jobs => format!("Jobs: {}", stats.total_jobs),
            StatType::Health => format!("Health: {}", stats.health_capacity),
            StatType::Police => format!("Police: {}", stats.police_capacity),
        };
    }
    
    // Message Log update (p3)
    if log.is_changed() {
        if let Ok(mut text) = text_queries.p3().get_single_mut() { // Assuming MessageText is unique
            text.sections[0].value = log.message.clone();
        }
    }

    // Old HappinessText update logic (original p5) is removed as its query is removed.
    // If it were kept, its update logic for the old top bar element would be here and likely do nothing.
}

// --- App Panel Management Systems ---

fn manage_app_panels_visibility(
    current_app: Res<CurrentApp>,
    mut panel_queries: ParamSet<(
        Query<&mut Style, With<DashboardPanel>>,
        Query<&mut Style, With<ConstructionPanel>>,
        Query<&mut Style, With<ColonyStatusPanel>>,
        Query<&mut Style, With<ResearchPanel>>,
    )>,
) {
    // Hide all panels first
    for mut style in panel_queries.p0().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p1().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p2().iter_mut() { style.display = Display::None; }
    for mut style in panel_queries.p3().iter_mut() { style.display = Display::None; }

    // Show the current one
    match current_app.0 {
        AppType::Dashboard => {
            if let Ok(mut style) = panel_queries.p0().get_single_mut() {
                style.display = Display::Flex;
            }
        }
        AppType::Construction => {
            if let Ok(mut style) = panel_queries.p1().get_single_mut() {
                style.display = Display::Flex;
            }
        }
        AppType::ColonyStatus => {
            if let Ok(mut style) = panel_queries.p2().get_single_mut() {
                style.display = Display::Flex;
            }
        }
        AppType::Research => {
            if let Ok(mut style) = panel_queries.p3().get_single_mut() {
                style.display = Display::Flex;
            }
        }
    }
}

fn update_dashboard_notifications_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    notifications_panel_query: Query<Entity, With<NotificationsPanel>>,
    // Query for children of the notification panel to despawn them.
    // A more robust way might be to mark notification text entities with a component.
    // children_query: Query<(Entity, &Parent)>, // Removed this complex query
    mut commands: Commands,
) {
    if current_app.0 != AppType::Dashboard {
        return;
    }

    // Update only if game_state (which contains notifications) or current_app changed
    if game_state.is_changed() || current_app.is_changed() {
        if let Ok(panel_entity) = notifications_panel_query.get_single() {
            // Despawn all existing children of the notification panel.
            // This is simpler and usually preferred for dynamically populated UI lists.
            commands.entity(panel_entity).despawn_descendants();

            // Spawn new notifications
            for event in game_state.notifications.iter().take(10) { // Show latest 10
                commands.entity(panel_entity).with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("[{:.1}] {}", event.timestamp, event.message), 
                        TextStyle { font_size: 16.0, color: Color::WHITE, ..default() }
                    ).with_style(Style{ margin: UiRect::bottom(Val::Px(2.0)), ..default()}));
                });
            }
        }
    }
}


// --- App Drawer Button System ---
const ACTIVE_BUTTON_COLOR: Color = Color::rgb(0.2, 0.5, 0.2); // Greenish for active app
const ACTIVE_CONSTRUCTION_TAB_COLOR: Color = Color::rgb(0.2, 0.5, 0.2); // Similar active color for construction tabs

fn construction_category_tab_system(
    mut current_category: ResMut<CurrentConstructionCategory>,
    mut button_query: Query<(&Interaction, &ConstructionCategoryTab, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, tab, mut bg_color) in button_query.iter_mut() {
        // Determine base color based on whether this tab is the current category
        if tab.0 == current_category.0 {
            *bg_color = ACTIVE_CONSTRUCTION_TAB_COLOR.into();
        } else {
            *bg_color = NORMAL_BUTTON.into();
        }

        // Apply interaction effects on top of the base color
        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into();
                // If pressed, update the current category resource
                if current_category.0 != tab.0 {
                    current_category.0 = tab.0;
                    println!("Switched to construction category: {:?}", current_category.0);
                }
            }
            Interaction::Hovered => {
                // Only apply hover effect if it's not the currently active tab
                if tab.0 != current_category.0 {
                    *bg_color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {
                // The base color (active or normal) is already set, so nothing to do for None.
            }
        }
    }
}

fn update_construction_list_system(
    current_app: Res<CurrentApp>,
    current_category: Res<CurrentConstructionCategory>,
    mut item_list_panel_query: Query<Entity, With<ConstructionItemListPanel>>,
    game_state: Res<GameState>, 
    mut commands: Commands,
) {
    if current_app.0 != AppType::Construction {
        return;
    }

    if current_category.is_changed() || (current_app.is_changed() && current_app.0 == AppType::Construction) {
        if let Ok(panel_entity) = item_list_panel_query.get_single_mut() {
            commands.entity(panel_entity).despawn_descendants(); 

            let building_meta_map = get_building_metadata();
            // Iterate over ALL_BUILDING_TYPES from game_state.rs
            // Ensure game_state::ALL_BUILDING_TYPES is imported.
            
            let mut found_items = false;
            commands.entity(panel_entity).with_children(|parent| {
                for building_type_ref in ALL_BUILDING_TYPES.iter() {
                    let building_type = *building_type_ref; // Dereference to get GameBuildingType
                    if let Some(meta) = building_meta_map.get(&building_type) {
                        if meta.category == current_category.0 {
                            found_items = true;
                            
                            let tech_ok = meta.required_tech.map_or(true, |tech| game_state.unlocked_techs.contains(&tech));
                            // let dp_ok = meta.required_dp.map_or(true, |dp| game_state.current_development_phase >= dp); // Assuming DP is comparable
                            
                            if tech_ok { // Add dp_ok when ready
                                let (can_afford_credits, can_afford_materials) = check_affordability(&game_state, building_type);
                                let button_color = if can_afford_credits && can_afford_materials { NORMAL_BUTTON } else { Color::rgba(0.5, 0.15, 0.15, 0.8) };

                                parent.spawn((
                                    ButtonBundle {
                                        style: Style { 
                                            width: Val::Percent(90.0), 
                                            padding: UiRect::all(Val::Px(5.0)), 
                                            margin: UiRect::all(Val::Px(2.0)), 
                                            justify_content: JustifyContent::Center, 
                                            align_items: AlignItems::Center, // Center text vertically
                                            ..default() 
                                        },
                                        background_color: button_color.into(),
                                        ..default()
                                    },
                                    ConstructionItemButton(building_type) 
                                )).with_children(|p| {
                                    p.spawn(TextBundle::from_section(meta.name, TextStyle { font_size: 16.0, ..default() }));
                                    // TODO: Optionally, add cost text here too
                                });
                            }
                        }
                    }
                }
                if !found_items {
                    parent.spawn(TextBundle::from_section(
                        format!("No items currently available in {:?} category.", current_category.0), 
                        TextStyle { font_size: 16.0, color: Color::GRAY, ..default() }
                    ));
                }
            });
        }
    }
}

fn check_affordability(game_state: &Res<GameState>, building_type: GameBuildingType) -> (bool, bool) {
    let can_afford_credits = true; 
    let mut can_afford_materials = true;

    // For GameState-managed buildings, their `add_...` functions typically handle credit costs.
    // Material costs, if any, might be in `building_costs` or also handled internally.
    // This simplified check primarily looks at `building_costs` for material costs.
    // Credit checks for these types are often implicit (i.e., if the `add_...` function succeeds).
    // For this UI visual cue, we'll assume credits are OK for GameState types unless a specific rule is added.

    match building_type {
        GameBuildingType::Fabricator | GameBuildingType::ProcessingPlant | GameBuildingType::BasicDwelling => {
            // These types have their costs (especially credits) handled by their respective
            // game_state::add_fabricator, game_state::add_processing_plant, game_state::add_habitation_structure functions.
            // For visual cue, if they have entries in `building_costs` (e.g. for supplemental materials), check those.
            // Otherwise, assume credits are the main factor checked at construction time.
            if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                for (res, &required_amount) in material_costs {
                    if game_state.current_resources.get(res).unwrap_or(&0.0) < &required_amount {
                        can_afford_materials = false;
                        break;
                    }
                }
            }
            // Credit affordability for these is complex to determine here without calling cost-calculating functions
            // from game_state.rs that might exist for each tier. For now, assume true for UI cue.
        }
        _ => { // For other Bevy component buildings (Extractor, BioDome, etc.)
            if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                for (res, &required_amount) in material_costs {
                    if game_state.current_resources.get(res).unwrap_or(&0.0) < &required_amount {
                        can_afford_materials = false;
                        break;
                    }
                }
                // Example: If credits were part of building_costs map under a special ResourceType::Credits
                // if let Some(credit_cost) = material_costs.get(&ResourceType::Credits) {
                //     can_afford_credits = game_state.credits >= *credit_cost;
                // }
            } else {
                // If a Bevy component building is NOT in building_costs, it implies no material cost or missing data.
                // For this simplified check, assume no material cost if not listed.
                // can_afford_materials = false; // Or true, depending on desired behavior for missing data
            }
        }
    }
    (can_afford_credits, can_afford_materials)
}

fn construction_item_interaction_system(
    mut selected_building_res: ResMut<SelectedBuilding>,
    game_state: Res<GameState>, // Ensure game_state is passed as Res<GameState>
    mut button_query: Query<(&Interaction, &ConstructionItemButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, item_button, mut bg_color) in button_query.iter_mut() {
        let building_type = item_button.0;
        let is_selected = selected_building_res.0 == Some(building_type);

        // 1. Determine base color
        if is_selected {
            *bg_color = Color::rgb(0.2, 0.6, 0.2).into(); // Selected color
        } else {
            // Ensure check_affordability is correctly called.
            // It was defined as: check_affordability(game_state: &Res<GameState>, building_type: GameBuildingType)
            let (can_afford_credits, can_afford_materials) = check_affordability(&game_state, building_type);
            if can_afford_credits && can_afford_materials {
                *bg_color = NORMAL_BUTTON.into();
            } else {
                *bg_color = Color::rgba(0.5, 0.15, 0.15, 0.8).into(); // Unaffordable color
            }
        }

        // 2. Apply interaction effects on top of the base color
        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into(); // Set pressed color
                if is_selected {
                    selected_building_res.0 = None; // Deselect if clicked again
                    // println!("Deselected building: {:?}", building_type); // Keep for debugging if desired
                } else {
                    selected_building_res.0 = Some(building_type);
                    // println!("Selected building: {:?}", building_type); // Keep for debugging if desired
                }
            }
            Interaction::Hovered => {
                // Only apply hover effect if it's not the currently selected button
                if !is_selected {
                    *bg_color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {
                // Base color (selected, normal, or unaffordable) is already set.
            }
        }
    }
}

fn update_construction_details_panel_system(
    selected_building: Res<SelectedBuilding>,
    game_state: Res<GameState>,
    mut details_panel_query: Query<Entity, With<ConstructionItemDetailsPanel>>,
    mut commands: Commands,
) {
    if !selected_building.is_changed() && !game_state.is_changed(){ // Only update if selection or game state changed
        return;
    }

    if let Ok(details_panel_entity) = details_panel_query.get_single_mut() {
        commands.entity(details_panel_entity).despawn_descendants();
        let building_meta_map = get_building_metadata();

        if let Some(building_type) = selected_building.0 {
            if let Some(meta) = building_meta_map.get(&building_type) {
                commands.entity(details_panel_entity).with_children(|parent| {
                    parent.spawn(TextBundle::from_section(meta.name, TextStyle { font_size: 24.0, ..default() }).with_style(Style{ margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                    parent.spawn(TextBundle::from_section("A standard construction offering from Nexus Core.", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect::bottom(Val::Px(8.0)), ..default()}));
                    
                    parent.spawn(TextBundle::from_section("Costs:", TextStyle { font_size: 18.0, ..default() }).with_style(Style{ margin: UiRect::bottom(Val::Px(3.0)), ..default()}));
                    parent.spawn(TextBundle::from_section(" - Credits: Varies (see build action)", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                        if material_costs.is_empty() { 
                            parent.spawn(TextBundle::from_section(" - Materials: None", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                        }
                        for (res, amount) in material_costs { 
                            parent.spawn(TextBundle::from_section(format!(" - {:?}: {}", res, amount), TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()})); 
                        }
                    } else { 
                        parent.spawn(TextBundle::from_section(" - Materials: None specified", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    }

                    parent.spawn(TextBundle::from_section("Requirements:", TextStyle { font_size: 18.0, ..default() }).with_style(Style{ margin: UiRect { top: Val::Px(8.0), bottom: Val::Px(3.0), left: Val::Px(0.0), right: Val::Px(0.0) }, ..default()}));
                    if let Some(tech) = meta.required_tech { 
                        parent.spawn(TextBundle::from_section(format!(" - Tech: {:?}", tech), TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    } else { 
                        parent.spawn(TextBundle::from_section(" - Tech: None", TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    }
                    if let Some(dp) = meta.required_dp { 
                        parent.spawn(TextBundle::from_section(format!(" - Min. DP: {:?}", dp), TextStyle { font_size: 16.0, ..default() }).with_style(Style{ margin: UiRect { left: Val::Px(10.0), bottom: Val::Px(2.0), right: Val::Px(0.0), top: Val::Px(0.0) }, ..default()}));
                    }

                    parent.spawn((
                        ButtonBundle {
                            style: Style { width: Val::Percent(50.0), padding: UiRect::all(Val::Px(8.0)), margin: UiRect::top(Val::Px(15.0)), justify_content: JustifyContent::Center, align_self: AlignSelf::Center, ..default() },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        ConfirmBuildButton(building_type)
                    )).with_children(|p| { p.spawn(TextBundle::from_section("Initiate Construction", TextStyle { font_size: 18.0, ..default() })); });
                });
            }
        } else {
            commands.entity(details_panel_entity).with_children(|parent| {
                parent.spawn(TextBundle::from_section("Select an item from the list to see details.", TextStyle { font_size: 16.0, color: Color::GRAY, ..default() }));
            });
        }
    }
}

fn construction_interaction_system(
    mut interaction_query: Query<(&Interaction, &ConfirmBuildButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
    mut selected_building: ResMut<SelectedBuilding>, // Changed to ResMut
    time: Res<Time>,
) {
    for (interaction, confirm_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            let building_type = confirm_button.0;
            let building_meta_map = get_building_metadata();
            let meta = if let Some(m) = building_meta_map.get(&building_type) { m } else { 
                game_state::add_notification(&mut game_state, format!("Error: No metadata for {:?}", building_type), time.elapsed_seconds_f64());
                return; 
            };

            if let Some(req_tech) = meta.required_tech {
                if !game_state.unlocked_techs.contains(&req_tech) {
                    game_state::add_notification(&mut game_state, format!("Failed: {:?} requires tech {:?}", meta.name, req_tech), time.elapsed_seconds_f64());
                    return;
                }
            }
            
            let mut can_afford_materials = true;
            if let Some(material_costs) = game_state.building_costs.get(&building_type) {
                for (res, &required_amount) in material_costs {
                    if game_state.current_resources.get(res).unwrap_or(&0.0) < &required_amount {
                        can_afford_materials = false;
                        break;
                    }
                }
            }
            if !can_afford_materials {
                game_state::add_notification(&mut game_state, format!("Failed: Not enough materials for {:?}", meta.name), time.elapsed_seconds_f64());
                return;
            }

            // Deduct material costs
            let costs_to_deduct: Option<Vec<(ResourceType, f32)>> = 
                game_state.building_costs.get(&building_type)
                .map(|costs_map| {
                    costs_map.iter().map(|(res_type, val)| (*res_type, *val)).collect()
                });

            if let Some(actual_costs) = costs_to_deduct {
                for (res, cost_val) in actual_costs {
                    if let Some(current_res_val) = game_state.current_resources.get_mut(&res) {
                        *current_res_val -= cost_val;
                    }
                }
            }
            
            let mut construction_triggered = false;
            match building_type {
                GameBuildingType::Fabricator => { game_state::add_fabricator(&mut game_state, 0); construction_triggered = true; }
                GameBuildingType::ProcessingPlant => { game_state::add_processing_plant(&mut game_state, 0); construction_triggered = true; }
                GameBuildingType::BasicDwelling => { game_state::add_habitation_structure(&mut game_state, 0); construction_triggered = true; }
                GameBuildingType::Extractor => { commands.spawn(Extractor { power_consumption: 15, resource_type: ResourceType::FerrocreteOre, extraction_rate: 2.5 }); construction_triggered = true; }
                GameBuildingType::BioDome => { commands.spawn(BioDome { power_consumption: 10, production_rate: 5.0 }); construction_triggered = true; }
                GameBuildingType::PowerRelay => { commands.spawn(PowerRelay { power_output: 50 }); construction_triggered = true; }
                GameBuildingType::StorageSilo => { commands.spawn(StorageSilo { capacity: 1000 }); construction_triggered = true; }
                GameBuildingType::ResearchInstitute => { commands.spawn(ResearchInstitute { power_consumption: 5 }); construction_triggered = true; }
                GameBuildingType::WellnessPost => { commands.spawn(WellnessPost { health_capacity: 50, jobs_provided: 5 }); construction_triggered = true; }
                GameBuildingType::SecurityStation => { commands.spawn(SecurityStation { police_capacity: 50, jobs_provided: 5 }); construction_triggered = true; }
                _ => { game_state::add_notification(&mut game_state, format!("Construction for {:?} not yet implemented.", meta.name), time.elapsed_seconds_f64()); }
            }

            if construction_triggered {
                game_state::add_notification(&mut game_state, format!("Construction started: {:?}", meta.name), time.elapsed_seconds_f64());
                selected_building.0 = None; 
            }
        }
    }
}

fn app_drawer_button_system(
    mut current_app_res: ResMut<CurrentApp>,
    mut button_query: Query<(&Interaction, &AppDrawerButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, app_button, mut bg_color) in button_query.iter_mut() {
        // Determine base color based on whether this button is the current app
        if app_button.0 == current_app_res.0 {
            *bg_color = ACTIVE_BUTTON_COLOR.into();
        } else {
            *bg_color = NORMAL_BUTTON.into();
        }

        // Apply interaction effects on top of the base color
        match *interaction {
            Interaction::Pressed => {
                *bg_color = PRESSED_BUTTON.into();
                // If pressed, update the current app resource
                // This might trigger a color change in the next frame/iteration due to the logic above
                if current_app_res.0 != app_button.0 {
                    current_app_res.0 = app_button.0;
                    println!("Switched to app: {:?}", current_app_res.0);
                }
            }
            Interaction::Hovered => {
                // Only apply hover effect if it's not the currently active button
                if app_button.0 != current_app_res.0 {
                    *bg_color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {
                // The base color (active or normal) is already set, so nothing to do for None.
            }
        }
    }
}


fn admin_spire_button_system(
    mut button_queries: ParamSet<(
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<ConstructSpireButton>)>,
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<UpgradeSpireButton>)>,
    )>,
    mut game_state: ResMut<GameState>,
    mut log: ResMut<MessageLog>,
) {
    // Handle Construct Spire Button
    for (interaction, mut color) in button_queries.p0().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                // Call game_state function
                // Temporarily store original credits to check if they changed
                let credits_before = game_state.credits;
                game_state::construct_administrative_spire(&mut game_state);
                // Check if spire was constructed or if there was an issue (e.g. not enough credits)
                if game_state.administrative_spire.is_some() && game_state.credits < credits_before {
                    log.message = "Administrative Spire constructed.".to_string();
                } else if game_state.administrative_spire.is_some() {
                    // Spire exists, but credits didn't change - means it was already there
                    log.message = "Administrative Spire already exists.".to_string();
                } else {
                    // Spire not constructed, likely due to cost (original function prints to console)
                    log.message = "Failed to construct Spire. Check credits/console.".to_string();
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                if game_state.administrative_spire.is_none() {
                    // Manually use the known first tier cost for "Command Post"
                    log.message = "Command Post: Cost 1000 Credits. Upkeep not applicable for construction.".to_string();
                } else {
                    log.message = "Administrative Spire already constructed.".to_string();
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }

    // Handle Upgrade Spire Button
    for (interaction, mut color) in button_queries.p1().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if game_state.administrative_spire.is_none() {
                    log.message = "Construct Administrative Spire first.".to_string();
                    continue;
                }
                // Call game_state function
                let current_tier_before_upgrade = game_state.administrative_spire.as_ref().map(|s| s.current_tier_index);
                game_state::upgrade_administrative_spire(&mut game_state);
                let current_tier_after_upgrade = game_state.administrative_spire.as_ref().map(|s| s.current_tier_index);

                if current_tier_after_upgrade > current_tier_before_upgrade {
                    log.message = "Administrative Spire upgraded.".to_string();
                } else if current_tier_after_upgrade == current_tier_before_upgrade && current_tier_before_upgrade.is_some() {
                     log.message = "Spire upgrade failed (check console/requirements).".to_string();
                     // Try to provide more specific feedback
                    if let Some(spire) = &game_state.administrative_spire {
                        if spire.current_tier_index < spire.available_tiers.len() - 1 {
                            let next_tier_info = &spire.available_tiers[spire.current_tier_index + 1];
                            if game_state.credits < next_tier_info.upgrade_credits_cost as f64 {
                                 log.message = format!("Upgrade failed: Need {} credits.", next_tier_info.upgrade_credits_cost);
                            } else if next_tier_info.nutrient_paste_link_required && !spire.is_linked_to_hub {
                                 log.message = "Upgrade failed: Nutrient Paste link required.".to_string();
                            } else {
                                 // Power check is more complex, rely on console for now or simplify
                                 log.message = "Upgrade failed: Check power or other requirements (console).".to_string();
                            }
                        } else {
                            log.message = "Spire already at max tier.".to_string();
                        }
                    }
                } else {
                    // Should not happen if spire exists, but as a fallback
                    log.message = "Error during Spire upgrade (check console).".to_string();
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                if let Some(spire) = &game_state.administrative_spire {
                    if spire.current_tier_index < spire.available_tiers.len() - 1 {
                        let next_tier_info = &spire.available_tiers[spire.current_tier_index + 1];
                        // AdministrativeSpireTier doesn't have an explicit upkeep field in game_state.rs
                        log.message = format!("Upgrade to {}: Cost {} Credits. Upkeep not specified on tier.", next_tier_info.name, next_tier_info.upgrade_credits_cost);
                    } else {
                        log.message = "Spire at max tier.".to_string();
                    }
                } else {
                    log.message = "Construct Spire first.".to_string();
                }
            }
            Interaction::None => { *color = NORMAL_BUTTON.into(); }
        }
    }
}