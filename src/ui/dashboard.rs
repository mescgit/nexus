use bevy::prelude::*;
use crate::game_state::{
    GameState, ColonyStats, GraphData, LoadGameEvent, SaveGameEvent, DevelopmentPhase,
    Tech, ServiceType, ZoneType, // Explicitly keeping these for get_building_display_info
    get_legacy_structure_tiers, construct_legacy_structure, upgrade_legacy_structure,
    construct_administrative_spire, upgrade_administrative_spire,
    // Add generic building interaction functions explicitly
    assign_workforce_to_building_by_id, unassign_workforce_from_building_by_id,
    upgrade_building_by_id, remove_building_by_id, toggle_building_active_state_by_id
};
// Import Building and BuildingVariant directly from components::building
use crate::components::building::{Building, BuildingVariant};
// Import new SelectedPlacedBuildingId and other common UI elements
use super::{
    CurrentApp, AppType, SelectedPlacedBuildingId, // Replaces SelectedZone
    NORMAL_BUTTON, HOVERED_BUTTON, ACTIVE_BUTTON, DISABLED_BUTTON,
    PRIMARY_TEXT_COLOR, LABEL_TEXT_COLOR, BORDER_COLOR, UiTag, ResourceType // ResourceType is from ui/mod.rs
};


#[derive(Component)]
pub(super) struct DashboardPanel;
#[derive(Component)]
pub(super) struct NotificationsPanel;
#[derive(Component)]
pub(super) struct AnalyticsGraphPanel;
#[derive(Component)]
pub(super) struct GraphArea;
#[derive(Component)]
pub(super) struct AdminSpireInfoPanel;
#[derive(Component)]
pub(super) struct LegacyStructurePanel;
#[derive(Component)]
pub(super) struct ConstructLegacyStructureButton;
#[derive(Component)]
pub(super) struct UpgradeLegacyStructureButton;
#[derive(Component)]
pub(super) struct AdminSpireTierText;
#[derive(Component)]
pub(super) struct ConstructSpireButton;
#[derive(Component)]
pub(super) struct UpgradeSpireButton;
#[derive(Component)]
pub(super) struct ManagedStructuresPanel;

// Renamed and new components
#[derive(Component)]
pub(super) struct ManagedBuildingButton(pub String); // Replaces ZoneListButton
#[derive(Component)]
pub(super) struct BuildingDetailsPanel; // Replaces ZoneDetailsPanel
#[derive(Component)]
pub(super) struct UpgradeBuildingButton(pub String); // Replaces UpgradeZoneButton
#[derive(Component)]
pub(super) struct RemoveBuildingButton(pub String); // Replaces RemoveZoneButton
#[derive(Component)]
pub(super) struct AssignWorkforceButton(pub String); // Replaces AssignSpecialistToZoneButton
#[derive(Component)]
pub(super) struct UnassignWorkforceButton(pub String); // Replaces UnassignSpecialistFromZoneButton
#[derive(Component)]
pub(super) struct ToggleActiveButton(pub String); // New component

#[derive(Component)]
pub(super) struct SaveGameButton;
#[derive(Component)]
pub(super) struct LoadGameButton;

// Helper to get building display info (name, workforce, etc.)
// This is a simplified version. A more robust solution might involve methods on Building/BuildingVariant
// or more detailed data structures passed from game_state.
fn get_building_display_info(building: &Building) -> (String, u32, u32, String) { // Changed to use direct import
    let name: String;
    let current_workforce = building.assigned_workforce;
    let max_workforce: u32;
    let mut type_specific_info = "".to_string();

    match &building.variant {
        BuildingVariant::Habitation { available_tiers, current_inhabitants } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.specialist_slots; // Habitation uses specialist_slots for its "jobs"
            type_specific_info = format!("Inhabitants: {}/{}", current_inhabitants, tier.housing_capacity);
        }
        BuildingVariant::Service { service_type, available_tiers } => {
            let tier = &available_tiers[building.current_tier_index];
            name = format!("{:?} {}", service_type, tier.name);
            max_workforce = tier.workforce_requirement;
            type_specific_info = format!("Coverage: {} units, Radius: {:.0}", tier.service_capacity, tier.service_radius);
        }
        BuildingVariant::Zone { zone_type, available_tiers } => {
            let tier = &available_tiers[building.current_tier_index];
            name = format!("{:?} {}", zone_type, tier.name);
            max_workforce = tier.workforce_requirement;
            if *zone_type == ZoneType::Commercial {
                type_specific_info = format!("Income: {}", tier.income_generation);
            }
        }
        BuildingVariant::Extractor { available_tiers } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.workforce_requirement;
            type_specific_info = format!("Output: {:.1}/s {:?}", tier.extraction_rate_per_sec, tier.resource_type);
        }
        BuildingVariant::BioDome { available_tiers } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.workforce_requirement;
            type_specific_info = format!("Output: {:.1}/s Nutrient Paste", tier.nutrient_paste_output_per_sec);
        }
        BuildingVariant::PowerRelay { available_tiers } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.workforce_requirement; // Should be 0
            type_specific_info = format!("Generation: {} MW", tier.power_generation);
        }
        BuildingVariant::ResearchInstitute { available_tiers } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.workforce_requirement;
            type_specific_info = format!("Research: {:.1}/s", tier.research_points_per_sec);
        }
        BuildingVariant::StorageSilo { available_tiers } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.workforce_requirement; // Should be 0
            type_specific_info = format!("Capacity: +{}", tier.storage_capacity_increase);
        }
        BuildingVariant::Fabricator { available_tiers, .. } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.workforce_requirement;
            type_specific_info = format!("Produces: {:?} ({}/{}s)", tier.output_product, tier.output_quantity, tier.production_time_secs);
        }
        BuildingVariant::ProcessingPlant { available_tiers, .. } => {
            let tier = &available_tiers[building.current_tier_index];
            name = tier.name.clone();
            max_workforce = tier.workforce_requirement;
            if let Some(output_res) = tier.output_resource {
                 type_specific_info = format!("Output: {:?} ({:?}/s)", output_res.0, tier.processing_rate_per_sec.unwrap_or(0.0));
            } else if let Some(unlock_res) = tier.unlocks_resource {
                type_specific_info = format!("Unlocks: {:?}", unlock_res);
            }
        }
        // These should ideally not be in the main `buildings` vec if handled completely separately
        BuildingVariant::AdministrativeSpire { .. } => { name = "Admin Spire".to_string(); max_workforce = 0; } // Placeholder
        BuildingVariant::LegacyStructure { .. } => { name = "Legacy Structure".to_string(); max_workforce = 0; } // Placeholder
    }
    (name, current_workforce, max_workforce, type_specific_info)
}


pub(super) fn update_managed_structures_panel_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    selected_building_id: Res<SelectedPlacedBuildingId>, // Changed from SelectedZone
    panel_query: Query<Entity, With<ManagedStructuresPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Dashboard {
        return;
    }

    if game_state.is_changed()
        || (current_app.is_changed() && current_app.0 == AppType::Dashboard)
        || selected_building_id.is_changed() // Changed from selected_zone
    {
        if let Ok(panel_entity) = panel_query.get_single() {
            commands.entity(panel_entity).despawn_descendants();

            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(
                        "Managed Structures", // Changed title
                        TextStyle {
                            font_size: 16.0,
                            color: LABEL_TEXT_COLOR,
                            ..default()
                        },
                    )
                    .with_style(Style { margin: UiRect::bottom(Val::Px(5.0)), ..default() }),
                );

                if game_state.buildings.is_empty() { // Changed from game_state.zones
                    parent.spawn(TextBundle::from_section(
                        "No structures built.", // Changed text
                        TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                    ));
                } else {
                    for building in game_state.buildings.iter() {
                        // Skip Admin Spire and Legacy Structure if they are handled by separate UI panels
                        // and not intended to be in this generic list.
                        // This logic might need refinement based on how Admin/Legacy are stored.
                        // For now, we assume they are not in the `buildings` Vec or get filtered by get_building_display_info.
                        // if matches!(building.variant, BuildingVariant::AdministrativeSpire{..} | BuildingVariant::LegacyStructure{..}) {
                        //    continue;
                        // }

                        let (name, current_workforce, max_workforce, _type_info) = get_building_display_info(building);

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        margin: UiRect::bottom(Val::Px(2.0)),
                                        justify_content: JustifyContent::FlexStart, // Align text to left
                                        ..default()
                                    },
                                    background_color: if selected_building_id.0.as_ref() == Some(&building.id) { ACTIVE_BUTTON.into() } else { NORMAL_BUTTON.into() },
                                    ..default()
                                },
                                ManagedBuildingButton(building.id.clone()), // Changed component
                            ))
                            .with_children(|button_parent| {
                                button_parent.spawn(TextBundle::from_section(
                                    format!("{} (Workers: {}/{})", name, current_workforce, max_workforce),
                                    TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                                ));
                            });
                    }
                }

                let mut details_panel = parent.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::top(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            min_height: Val::Px(200.0), // Increased min_height for more details
                            ..default()
                        },
                        border_color: Color::rgba(0.5, 0.5, 0.5, 0.5).into(),
                        background_color: Color::rgba(0.1, 0.1, 0.1, 0.3).into(),
                        ..default()
                    },
                    BuildingDetailsPanel, // Changed component
                ));

                if let Some(selected_id) = &selected_building_id.0 {
                    if let Some(building) = game_state.buildings.iter().find(|b| b.id == *selected_id) {
                        let (name, current_workforce, max_workforce, type_specific_info) = get_building_display_info(building);
                        let mut upkeep_cost: u32 = 0;
                        let mut power_req: u32 = 0;
                        let mut civic_contribution: u32 = 0;
                        let mut can_upgrade = false;
                        let mut next_tier_name = "".to_string();
                        let mut next_tier_cost = 0;
                        let mut tech_req_for_upgrade: Option<Tech> = None;

                        // Extract tier-specific details
                        match &building.variant {
                            BuildingVariant::Habitation { available_tiers, .. } => {
                                let tier = &available_tiers[building.current_tier_index];
                                upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement;
                                if building.current_tier_index < available_tiers.len() - 1 {
                                    let next_tier = &available_tiers[building.current_tier_index + 1];
                                    can_upgrade = true; next_tier_name = next_tier.name.clone();
                                    next_tier_cost = next_tier.construction_credits_cost; tech_req_for_upgrade = next_tier.required_tech;
                                }
                            }
                            BuildingVariant::Service { available_tiers, .. } => {
                                let tier = &available_tiers[building.current_tier_index];
                                upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement; civic_contribution = tier.civic_index_contribution;
                                if building.current_tier_index < available_tiers.len() - 1 {
                                   let next_tier = &available_tiers[building.current_tier_index + 1];
                                    can_upgrade = true; next_tier_name = next_tier.name.clone();
                                    next_tier_cost = next_tier.construction_credits_cost; tech_req_for_upgrade = next_tier.required_tech;
                                }
                            }
                            BuildingVariant::Zone { available_tiers, .. } => {
                                let tier = &available_tiers[building.current_tier_index];
                                upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement; civic_contribution = tier.civic_index_contribution;
                                 if building.current_tier_index < available_tiers.len() - 1 {
                                    let next_tier = &available_tiers[building.current_tier_index + 1];
                                    can_upgrade = true; next_tier_name = next_tier.name.clone();
                                    next_tier_cost = next_tier.construction_credits_cost; tech_req_for_upgrade = next_tier.required_tech;
                                }
                            }
                            BuildingVariant::Extractor { available_tiers, ..} => { let tier = &available_tiers[building.current_tier_index]; upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement; /* no upgrade for default extractor */ }
                            BuildingVariant::BioDome { available_tiers, ..} => { let tier = &available_tiers[building.current_tier_index]; upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement; }
                            BuildingVariant::PowerRelay { available_tiers, ..} => { let tier = &available_tiers[building.current_tier_index]; upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement;}
                            BuildingVariant::ResearchInstitute { available_tiers, ..} => { let tier = &available_tiers[building.current_tier_index]; upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement;}
                            BuildingVariant::StorageSilo { available_tiers, ..} => { let tier = &available_tiers[building.current_tier_index]; upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement;}
                            BuildingVariant::Fabricator { available_tiers, .. } => {
                                let tier = &available_tiers[building.current_tier_index];
                                upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement;
                                if building.current_tier_index < available_tiers.len() - 1 {
                                    let next_tier = &available_tiers[building.current_tier_index + 1];
                                    can_upgrade = true; next_tier_name = next_tier.name.clone();
                                    next_tier_cost = next_tier.construction_credits_cost; tech_req_for_upgrade = next_tier.required_tech;
                                }
                            }
                             BuildingVariant::ProcessingPlant { available_tiers, .. } => {
                                let tier = &available_tiers[building.current_tier_index];
                                upkeep_cost = tier.upkeep_cost; power_req = tier.power_requirement;
                                if building.current_tier_index < available_tiers.len() - 1 {
                                    let next_tier = &available_tiers[building.current_tier_index + 1];
                                    can_upgrade = true; next_tier_name = next_tier.name.clone();
                                    next_tier_cost = next_tier.construction_credits_cost; tech_req_for_upgrade = next_tier.required_tech;
                                }
                            }
                            _ => {} // Admin/Legacy are not in this list or have no generic tier data here.
                        }

                        details_panel.with_children(|dp| {
                            dp.spawn( TextBundle::from_section( name.clone(), TextStyle { font_size: 15.0, color: PRIMARY_TEXT_COLOR, ..default() },).with_style(Style { margin: UiRect::bottom(Val::Px(4.0)), ..default() }));
                            dp.spawn( TextBundle::from_section( format!("Workers: {}/{}", current_workforce, max_workforce), TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },).with_style(Style { margin: UiRect::bottom(Val::Px(2.0)), ..default() }));
                            dp.spawn( TextBundle::from_section( format!("Upkeep: {} Cr, Power: {} MW", upkeep_cost, power_req), TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },).with_style(Style { margin: UiRect::bottom(Val::Px(2.0)), ..default() }));
                            if civic_contribution > 0 {
                                dp.spawn( TextBundle::from_section( format!("Civic Index: {}", civic_contribution), TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },).with_style(Style { margin: UiRect::bottom(Val::Px(2.0)), ..default() }));
                            }
                            if !type_specific_info.is_empty() {
                                dp.spawn( TextBundle::from_section( type_specific_info, TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },).with_style(Style { margin: UiRect::bottom(Val::Px(8.0)), ..default() }));
                            }

                            // Upgrade Button
                            if can_upgrade {
                                let tech_met = tech_req_for_upgrade.map_or(true, |tech| game_state.unlocked_techs.contains(&tech));
                                let can_afford_upgrade = game_state.credits >= next_tier_cost as f64 && tech_met;
                                dp.spawn(( ButtonBundle { style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() }, background_color: if can_afford_upgrade { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() }, ..default() }, UpgradeBuildingButton(selected_id.clone()),))
                                    .with_children(|btn| {
                                        let req_text = if !tech_met { format!(" (Req: {:?})", tech_req_for_upgrade.unwrap()) } else { "".to_string() };
                                        btn.spawn(TextBundle::from_section( format!("Upgrade to {} ({} Cr){}", next_tier_name, next_tier_cost, req_text), TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },));
                                    });
                            } else if max_workforce > 0 { // Only show "max tier" for buildings that are expected to upgrade
                                dp.spawn(TextBundle::from_section( "Max tier reached", TextStyle { font_size: 14.0, color: Color::CYAN, ..default() },));
                            }

                            // Remove Button
                            dp.spawn(( ButtonBundle { style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(4.0)), ..default() }, background_color: NORMAL_BUTTON.into(), ..default() }, RemoveBuildingButton(selected_id.clone()),))
                                .with_children(|btn| { btn.spawn(TextBundle::from_section( "Remove Structure", TextStyle { font_size: 14.0, color: Color::TOMATO, ..default() }, )); });

                            // Workforce assignment buttons (if applicable)
                            if max_workforce > 0 { // Only show if building uses workforce
                                // Toggle Active Button (before workforce assignment)
                                dp.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::all(Val::Px(5.0)),
                                            margin: UiRect::bottom(Val::Px(4.0)),
                                            ..default()
                                        },
                                        background_color: NORMAL_BUTTON.into(), // Color could change based on building.is_active
                                        ..default()
                                    },
                                    ToggleActiveButton(selected_id.clone()),
                                )).with_children(|btn| {
                                    btn.spawn(TextBundle::from_section(
                                        if building.is_active { "Deactivate Building" } else { "Activate Building" },
                                        TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                                    ));
                                });

                                dp.spawn(NodeBundle { style: Style { height: Val::Px(10.0), ..default() }, ..default() });
                                let can_assign_more = current_workforce < max_workforce;
                                let available_general_inhabitants = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
                                let can_assign = can_assign_more && available_general_inhabitants > 0;
                                dp.spawn(( ButtonBundle { style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() }, background_color: if can_assign { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() }, ..default() }, AssignWorkforceButton(selected_id.clone()),))
                                    .with_children(|btn| { btn.spawn(TextBundle::from_section( "Assign Workforce (+1)", TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },)); });

                                let can_unassign = current_workforce > 0;
                                dp.spawn(( ButtonBundle { style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), ..default() }, background_color: if can_unassign { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() }, ..default() }, UnassignWorkforceButton(selected_id.clone()),))
                                    .with_children(|btn| { btn.spawn(TextBundle::from_section( "Unassign Workforce (-1)", TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },)); });
                            }
                        });
                    }
                } else {
                     details_panel.with_children(|dp| {
                        dp.spawn(TextBundle::from_section("Select a structure to view details.", TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() }));
                    });
                }
            });
        }
    }
}

pub(super) fn assign_workforce_button_interaction_system( // Renamed
    mut interaction_query: Query<(&Interaction, &AssignWorkforceButton), (Changed<Interaction>, With<Button>)>, // Changed component
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let building_id = &button.0;
            // Call new generic game_state function.
            // The function itself will check workforce capacity and available inhabitants.
            assign_workforce_to_building_by_id(&mut game_state, building_id, 1);
        }
    }
}

pub(super) fn unassign_workforce_button_interaction_system( // Renamed
    mut interaction_query: Query<(&Interaction, &UnassignWorkforceButton), (Changed<Interaction>, With<Button>)>, // Changed component
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let building_id = &button.0;
            // Call new generic game_state function.
            unassign_workforce_from_building_by_id(&mut game_state, building_id, 1);
        }
    }
}

pub(super) fn toggle_building_active_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &ToggleActiveButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            toggle_building_active_state_by_id(&mut game_state, &button.0);
        }
    }
}

pub(super) fn build(viewport: &mut ChildBuilder, _assets: &Res<AssetServer>) {
                viewport.spawn((NodeBundle { style: Style {display: Display::Flex, width: Val::Percent(100.0), height:Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default()}, ..default() }, DashboardPanel))
                .with_children(|dash| {
                    dash.spawn(TextBundle::from_section("NEXUS DASHBOARD", TextStyle{font_size: 28.0, color: BORDER_COLOR, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(10.0)), ..default()}));

                    dash.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ..default()
                    }).with_children(|main_dash_area| {
                        // Left side of dashboard
                        main_dash_area.spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            ..default()
                        }).with_children(|left_col| {
                            left_col.spawn((NodeBundle { style: Style {width: Val::Percent(100.0), height: Val::Percent(30.0), flex_direction: FlexDirection::Column, padding: UiRect::all(Val::Px(5.0)), border: UiRect::all(Val::Px(1.0)), ..default()}, background_color: Color::rgba(0.0,0.0,0.0,0.3).into(), border_color: BORDER_COLOR.into(), ..default()}, NotificationsPanel));
                            left_col.spawn((NodeBundle { style: Style {width: Val::Percent(100.0), flex_grow: 1.0, margin: UiRect::top(Val::Px(10.0)), border: UiRect::all(Val::Px(1.0)), ..default()}, background_color: Color::rgba(0.0,0.0,0.0,0.3).into(), border_color: BORDER_COLOR.into(), ..default()}, AnalyticsGraphPanel))
                            .with_children(|graph_panel| {
                                graph_panel.spawn((NodeBundle{style: Style{width: Val::Percent(100.0), height:Val::Percent(100.0), ..default()}, ..default()}, GraphArea));
                            });
                        });
                        // Right side of dashboard
                        main_dash_area.spawn(NodeBundle {
                             style: Style {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(300.0),
                                height: Val::Percent(100.0),
                                margin: UiRect::left(Val::Px(10.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                             },
                             background_color: Color::rgba(0.0,0.0,0.0,0.3).into(),
                             border_color: BORDER_COLOR.into(),
                             ..default()
                        }).with_children(|right_col| {
                            right_col.spawn(TextBundle::from_section("SYSTEM CONTROL", TextStyle{font_size: 18.0, color: LABEL_TEXT_COLOR, ..default()}));
                            right_col.spawn((NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    margin: UiRect::top(Val::Px(10.0)),
                                    padding: UiRect::all(Val::Px(5.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                border_color: BORDER_COLOR.into(),
                                ..default()
                            }, AdminSpireInfoPanel));

                            // New panel for Legacy Structure
                            right_col.spawn((NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    margin: UiRect::top(Val::Px(10.0)),
                                    padding: UiRect::all(Val::Px(5.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                border_color: BORDER_COLOR.into(),
                                ..default()
                            }, LegacyStructurePanel));

                            // Add ManagedStructuresPanel here
                            right_col.spawn((
                                NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Column,
                                        margin: UiRect::top(Val::Px(10.0)),
                                        padding: UiRect::all(Val::Px(5.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        flex_grow: 1.0, // Allow it to take available space
                                        ..default()
                                    },
                                    background_color: Color::rgba(0.0,0.0,0.0,0.2).into(), // Slightly different bg for distinction
                                    border_color: BORDER_COLOR.into(),
                                    ..default()
                                },
                                ManagedStructuresPanel,
                            ));
                        });
                    });
                });
}
pub(super) fn update_dashboard_notifications_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    notifications_panel_query: Query<Entity, With<NotificationsPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Dashboard { return; }

    if game_state.is_changed() || current_app.is_changed() {
        if let Ok(panel_entity) = notifications_panel_query.get_single() {
            commands.entity(panel_entity).despawn_descendants();
            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn(TextBundle::from_section("EVENT LOG", TextStyle{font_size: 18.0, color: LABEL_TEXT_COLOR, ..default()}));
                for event in game_state.notifications.iter().take(5) {
                    parent.spawn(TextBundle::from_section(
                        format!("[{:.1}] {}", event.timestamp, event.message),
                        TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() }
                    ).with_style(Style{ margin: UiRect::top(Val::Px(4.0)), ..default()}));
                }
            });
        }
    }
}

pub(super) fn update_admin_spire_panel_system(
    game_state: Res<GameState>,
    panel_query: Query<Entity, With<AdminSpireInfoPanel>>,
    mut commands: Commands,
) {
    if !game_state.is_changed() {
        return;
    }

    if let Ok(panel_entity) = panel_query.get_single() {
        commands.entity(panel_entity).despawn_descendants();

        commands.entity(panel_entity).with_children(|parent| {
            parent.spawn(TextBundle::from_section("Administrative Spire", TextStyle{font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default()}));

            if let Some(spire) = &game_state.administrative_spire {
                let current_tier = &spire.available_tiers[spire.current_tier_index];
                parent.spawn((
                    TextBundle::from_section(format!("Tier: {}", current_tier.name), TextStyle{font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}),
                    AdminSpireTierText,
                ));
                 parent.spawn(TextBundle::from_section(format!("Phase: {:?}", game_state.current_development_phase), TextStyle{font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}));

                if spire.current_tier_index < spire.available_tiers.len() - 1 {
                    let next_tier = &spire.available_tiers[spire.current_tier_index + 1];
                    let can_afford = game_state.credits >= next_tier.upgrade_credits_cost as f64;
                    parent.spawn((
                        ButtonBundle {
                            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(10.0)), ..default()},
                            background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                            ..default()
                        },
                        UpgradeSpireButton
                    )).with_children(|btn| {
                        btn.spawn(TextBundle::from_section(
                            format!("Upgrade to {}\n({} Cr)", next_tier.name, next_tier.upgrade_credits_cost),
                            TextStyle{font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}
                        ));
                    });
                } else {
                    parent.spawn(TextBundle::from_section("Max Tier Reached", TextStyle{font_size: 14.0, color: Color::CYAN, ..default()}));
                }
            } else {
                 parent.spawn((
                    TextBundle::from_section("Status: Not Constructed", TextStyle{font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}),
                    AdminSpireTierText
                ));
                let can_afford = game_state.credits >= 1000.0; // Hardcoded from game_state
                parent.spawn((
                    ButtonBundle {
                        style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(10.0)), ..default()},
                        background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                        ..default()
                    },
                    ConstructSpireButton,
                    UiTag("dashboard.construct_spire"),
                )).with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        "Construct (1000 Cr)",
                        TextStyle{font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}
                    ));
                });
            }

            // Save and Load Buttons
            parent.spawn(NodeBundle { style: Style {flex_direction: FlexDirection::Row, margin: UiRect::top(Val::Px(20.0)), ..default()}, ..default()})
                .with_children(|buttons| {
                    buttons.spawn((
                        ButtonBundle {
                            style: Style { flex_grow: 1.0, padding: UiRect::all(Val::Px(5.0)), margin: UiRect::right(Val::Px(2.0)), ..default() },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        SaveGameButton,
                    )).with_children(|btn| {
                        btn.spawn(TextBundle::from_section("Save", TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}));
                    });
                    buttons.spawn((
                        ButtonBundle {
                            style: Style { flex_grow: 1.0, padding: UiRect::all(Val::Px(5.0)), margin: UiRect::left(Val::Px(2.0)), ..default() },
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                        LoadGameButton,
                    )).with_children(|btn| {
                        btn.spawn(TextBundle::from_section("Load", TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}));
                    });
                });
        });
    }
}

pub(super) fn managed_building_button_interaction_system( // Renamed
    mut selected_building_id_res: ResMut<SelectedPlacedBuildingId>, // Changed
    mut button_query: Query<(&Interaction, &ManagedBuildingButton, &mut BackgroundColor), With<Button>>, // Changed
) {
    for (interaction, building_button_data, mut bg_color) in button_query.iter_mut() { // Changed
        let is_currently_selected_in_res = selected_building_id_res.0.as_ref() == Some(&building_button_data.0);

        if *interaction == Interaction::Pressed {
            if is_currently_selected_in_res {
                selected_building_id_res.0 = None;
            } else {
                selected_building_id_res.0 = Some(building_button_data.0.clone());
            }
        }

        // Update visual state based on interaction and selection
        if is_currently_selected_in_res { // If it is the selected one, color it active
            *bg_color = ACTIVE_BUTTON.into();
        } else if *interaction == Interaction::Hovered { // If hovered and not selected, color it hovered
            *bg_color = HOVERED_BUTTON.into();
        } else { // Otherwise, normal color
            *bg_color = NORMAL_BUTTON.into();
        }
    }
}

pub(super) fn upgrade_building_button_interaction_system( // Renamed
    mut interaction_query: Query<(&Interaction, &UpgradeBuildingButton), (Changed<Interaction>, With<Button>)>, // Changed component
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
             // Affordability is checked visually by button color.
             // game_state::upgrade_building_by_id will handle actual cost deduction and validation.
            upgrade_building_by_id(&mut game_state, &button.0);
        }
    }
}

pub(super) fn remove_building_button_interaction_system( // Renamed
    mut interaction_query: Query<(&Interaction, &RemoveBuildingButton), (Changed<Interaction>, With<Button>)>, // Changed component
    mut game_state: ResMut<GameState>,
    mut selected_building_id: ResMut<SelectedPlacedBuildingId>, // Changed resource
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            remove_building_by_id(&mut game_state, &button.0);
            selected_building_id.0 = None; // Clear selection after removal
        }
    }
}

// This system is defined after assign_workforce_button_interaction_system in the original file
// It will be modified in the next step.
// pub(super) fn unassign_specialist_from_zone_button_interaction_system(...)

pub(super) fn admin_spire_button_interaction_system(
    mut game_state: ResMut<GameState>,
    mut interaction_query: ParamSet<(
        Query<&Interaction, (Changed<Interaction>, With<ConstructSpireButton>)>,
        Query<&Interaction, (Changed<Interaction>, With<UpgradeSpireButton>)>,
    )>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.p0().get_single() {
        construct_administrative_spire(&mut game_state);
    }

    if let Ok(Interaction::Pressed) = interaction_query.p1().get_single() {
        upgrade_administrative_spire(&mut game_state);
    }
}

pub(super) fn save_load_button_system(
    mut save_ew: EventWriter<SaveGameEvent>,
    mut load_ew: EventWriter<LoadGameEvent>,
    mut interaction_query: Query<(&Interaction, Option<&SaveGameButton>, Option<&LoadGameButton>), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, save_btn, load_btn) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if save_btn.is_some() {
                save_ew.send(SaveGameEvent);
            }
            if load_btn.is_some() {
                load_ew.send(LoadGameEvent);
            }
        }
    }
}

trait GraphableFn: Fn(&ColonyStats) -> f32 + Send + Sync {}
impl<F: Fn(&ColonyStats) -> f32 + Send + Sync> GraphableFn for F {}

pub(super) fn draw_graph_gizmos(
    mut gizmos: Gizmos,
    graph_data: Res<GraphData>,
    graph_area_query: Query<(&Node, &GlobalTransform), With<GraphArea>>,
) {
     if graph_data.history.is_empty() { return; }

    let (graph_node, transform) = if let Ok(result) = graph_area_query.get_single() { result } else { return; };
    let graph_area = graph_node.size();
    if graph_area.x <= 0.0 || graph_area.y <= 0.0 { return; }

    let bottom_left = transform.translation().truncate() - graph_area / 2.0;
    let max_val = graph_data.history.iter().fold(1.0f32, |max, stats| max.max(stats.credits as f32).max(stats.happiness).max(stats.net_power.abs()));

    let graph_lines: [(Color, Box<dyn GraphableFn>); 3] = [
        (Color::GOLD, Box::new(|stats| stats.credits as f32)),
        (Color::LIME_GREEN, Box::new(|stats| stats.happiness)),
        (Color::CYAN, Box::new(|stats| stats.net_power)),
    ];

    for (color, get_value) in graph_lines.iter() {
        let points: Vec<Vec2> = graph_data.history.iter().enumerate().map(|(i, stats)| {
            let x = bottom_left.x + (i as f32 / (graph_data.history.len() - 1).max(1) as f32) * graph_area.x;
            let y_val = get_value(stats);
            let y_normalized = y_val / max_val;
            let y = bottom_left.y + (y_normalized * 0.5 + 0.5) * graph_area.y;

            Vec2::new(x, y.clamp(bottom_left.y, bottom_left.y + graph_area.y))
        }).collect();

        if points.len() > 1 {
            gizmos.linestrip_2d(points, *color);
        }
    }
    gizmos.line_2d(
        Vec2::new(bottom_left.x, bottom_left.y + graph_area.y / 2.0),
        Vec2::new(bottom_left.x + graph_area.x, bottom_left.y + graph_area.y / 2.0),
        Color::rgba(1.0, 1.0, 1.0, 0.2),
    );
}
pub(super) fn update_legacy_structure_panel_system(
    game_state: Res<GameState>,
    panel_query: Query<Entity, With<LegacyStructurePanel>>,
    mut commands: Commands,
) {
    if !game_state.is_changed() { return; }

    if let Ok(panel_entity) = panel_query.get_single() {
        commands.entity(panel_entity).despawn_descendants();

        commands.entity(panel_entity).with_children(|parent| {
            parent.spawn(TextBundle::from_section("Legacy Structure", TextStyle{font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default()}));

            if game_state.current_development_phase < DevelopmentPhase::DP3 {
                parent.spawn(TextBundle::from_section(
                    "Available in Development Phase 3",
                    TextStyle { color: LABEL_TEXT_COLOR, ..default() },
                ));
            } else if let Some(structure) = &game_state.legacy_structure {
                let current_tier = &structure.available_tiers[structure.current_tier_index];
                parent.spawn(TextBundle::from_section(format!("Tier: {}", current_tier.name), TextStyle{font_size: 14.0, color: LABEL_TEXT_COLOR, ..default()}));
                parent.spawn(TextBundle::from_section(format!("Happiness: +{}", current_tier.happiness_bonus), TextStyle{font_size: 12.0, color: Color::LIME_GREEN, ..default()}));
                parent.spawn(TextBundle::from_section(format!("Income: +{}/cycle", current_tier.income_bonus), TextStyle{font_size: 12.0, color: Color::GOLD, ..default()}));

                if structure.current_tier_index < structure.available_tiers.len() - 1 {
                    let next_tier = &structure.available_tiers[structure.current_tier_index + 1];
                    let can_afford = game_state.credits >= next_tier.construction_credits_cost as f64;
                    parent.spawn((
                        ButtonBundle {
                            style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(10.0)), ..default()},
                            background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                            ..default()
                        },
                        UpgradeLegacyStructureButton
                    )).with_children(|btn| {
                        btn.spawn(TextBundle::from_section(
                            format!("Upgrade to {}\n({} Cr)", next_tier.name, next_tier.construction_credits_cost),
                            TextStyle{font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}
                        ));
                    });
                } else {
                    parent.spawn(TextBundle::from_section("Max Tier Reached", TextStyle{font_size: 14.0, color: Color::CYAN, ..default()}));
                }
            } else {
                let tiers = get_legacy_structure_tiers();
                let initial_tier = &tiers[0];
                let can_afford = game_state.credits >= initial_tier.construction_credits_cost as f64;
                parent.spawn((
                    ButtonBundle {
                        style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(5.0)), margin: UiRect::top(Val::Px(10.0)), ..default()},
                        background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                        ..default()
                    },
                    ConstructLegacyStructureButton
                )).with_children(|btn| {
                    btn.spawn(TextBundle::from_section(
                        format!("Construct {}\n({} Cr)", initial_tier.name, initial_tier.construction_credits_cost),
                        TextStyle{font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default()}
                    ));
                });
            }
        });
    }
}

pub(super) fn legacy_structure_button_system(
    mut game_state: ResMut<GameState>,
    mut interaction_query: ParamSet<(
        Query<&Interaction, (Changed<Interaction>, With<ConstructLegacyStructureButton>)>,
        Query<&Interaction, (Changed<Interaction>, With<UpgradeLegacyStructureButton>)>,
    )>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.p0().get_single() {
        construct_legacy_structure(&mut game_state);
    }

    if let Ok(Interaction::Pressed) = interaction_query.p1().get_single() {
        upgrade_legacy_structure(&mut game_state);
    }
}