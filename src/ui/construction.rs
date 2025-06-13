use bevy::prelude::*;
use crate::game_state::{self, add_bio_dome, add_extractor, add_power_relay, add_research_institute, add_storage_silo, GameState, ServiceType, ZoneType, BuildingType as GameBuildingType, DevelopmentPhase};
use super::*;

#[derive(Component)]
pub(super) struct ConstructionPanel;
#[derive(Component)]
pub(super) struct ConstructionCategoryTab(pub ConstructionCategory);
#[derive(Component)]
pub(super) struct ConstructionItemListPanel;
#[derive(Component)]
pub(super) struct ConstructionItemButton(pub GameBuildingType);
#[derive(Component)]
pub(super) struct ConstructionItemDetailsPanel;
#[derive(Component)]
pub(super) struct ConfirmBuildButton(pub GameBuildingType);
#[derive(Component)]
pub(super) struct ConstructHabitationButton(pub usize);
#[derive(Component)]
pub(super) struct ConstructServiceButton(pub ServiceType, pub usize);
#[derive(Component)]
pub(super) struct ConstructZoneButton(pub ZoneType, pub usize);

#[derive(Clone, Copy, Debug)]
pub(super) struct BuildingMetadata {
    pub name: &'static str,
    pub category: ConstructionCategory,
    pub required_tech: Option<game_state::Tech>,
    pub required_dp: Option<DevelopmentPhase>,
    pub workforce_required: u32,
}

use std::collections::HashMap;

fn get_building_metadata() -> HashMap<GameBuildingType, BuildingMetadata> {
    let mut meta = HashMap::new();
    meta.insert(GameBuildingType::Extractor, BuildingMetadata { name: "Extractor", category: ConstructionCategory::Operations, required_tech: None, required_dp: None, workforce_required: 5 });
    meta.insert(GameBuildingType::BioDome, BuildingMetadata { name: "Bio-Dome", category: ConstructionCategory::Operations, required_tech: None, required_dp: None, workforce_required: 10 });
    meta.insert(GameBuildingType::PowerRelay, BuildingMetadata { name: "Power Relay", category: ConstructionCategory::Operations, required_tech: None, required_dp: None, workforce_required: 0 });
    meta.insert(GameBuildingType::StorageSilo, BuildingMetadata { name: "Storage Silo", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None, workforce_required: 0 });
    meta.insert(GameBuildingType::ResearchInstitute, BuildingMetadata { name: "Research Institute", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: None, workforce_required: 15 });
    meta.insert(GameBuildingType::Fabricator, BuildingMetadata { name: "Fabricator", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: Some(DevelopmentPhase::DP2), workforce_required: 20 });
    meta.insert(GameBuildingType::ProcessingPlant, BuildingMetadata { name: "Processing Plant", category: ConstructionCategory::Operations, required_tech: Some(Tech::BasicConstructionProtocols), required_dp: Some(DevelopmentPhase::DP2), workforce_required: 20 });
    meta
}

fn tag_for_building(bt: GameBuildingType) -> &'static str {
    match bt {
        GameBuildingType::Extractor => "build_menu.extractor",
        GameBuildingType::BioDome => "build_menu.bio_dome",
        GameBuildingType::PowerRelay => "build_menu.power_relay",
        GameBuildingType::StorageSilo => "build_menu.storage_silo",
        GameBuildingType::ResearchInstitute => "build_menu.research_institute",
        GameBuildingType::Fabricator => "build_menu.fabricator",
        GameBuildingType::ProcessingPlant => "build_menu.processing_plant",
    }
}

pub(super) fn build(viewport: &mut ChildBuilder, _assets: &Res<AssetServer>) {
                viewport.spawn((NodeBundle { style: Style {display: Display::None, width: Val::Percent(100.0), height:Val::Percent(100.0), flex_direction: FlexDirection::Column, ..default()}, ..default() }, ConstructionPanel))
                .with_children(|con| {
                    con.spawn(TextBundle::from_section("CONSTRUCTION", TextStyle{font_size: 28.0, color: BORDER_COLOR, ..default()}).with_style(Style{margin: UiRect::bottom(Val::Px(10.0)), ..default()}));
                    con.spawn(NodeBundle { style: Style { flex_direction: FlexDirection::Row, margin: UiRect::bottom(Val::Px(5.0)), ..default()}, ..default()})
                    .with_children(|tabs|{
                        let categories = [ConstructionCategory::Operations, ConstructionCategory::Habitation, ConstructionCategory::Services, ConstructionCategory::Zones];
                        for category in categories {
                            let mut e = tabs.spawn((ButtonBundle {style: Style{padding: UiRect::all(Val::Px(8.0)), margin: UiRect::horizontal(Val::Px(5.0)), ..default()}, background_color: NORMAL_BUTTON.into(), ..default()}, ConstructionCategoryTab(category)));
                            if category == ConstructionCategory::Services {
                                e.insert(UiTag("build_menu.services"));
                            }
                            e.with_children(|button| { button.spawn(TextBundle::from_section(format!("{:?}", category), TextStyle {font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default()})); });
                        }
                    });
                    con.spawn(NodeBundle { style: Style {flex_direction: FlexDirection::Row, flex_grow: 1.0, ..default()}, ..default()})
                    .with_children(|main| {
                        main.spawn((NodeBundle{style: Style{width:Val::Percent(40.0), height: Val::Percent(100.0), border: UiRect::all(Val::Px(1.0)), padding: UiRect::all(Val::Px(5.0)), flex_direction: FlexDirection::Column, ..default()}, border_color: BORDER_COLOR.into(), ..default()}, ConstructionItemListPanel));
                        main.spawn((NodeBundle{style: Style{flex_grow: 1.0, height: Val::Percent(100.0), border: UiRect::all(Val::Px(1.0)), padding: UiRect::all(Val::Px(10.0)), margin: UiRect::left(Val::Px(10.0)), flex_direction:FlexDirection::Column, ..default()}, border_color: BORDER_COLOR.into(), ..default()}, ConstructionItemDetailsPanel));
                    });
                });
}
pub(super) fn construction_interaction_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ConfirmBuildButton>)>,
    selected_building: Res<SelectedBuilding>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
     if let Some(building_type) = selected_building.0 {
        if let Ok(Interaction::Pressed) = interaction_query.get_single() {
             let costs = game_state.building_costs.get(&building_type).unwrap().clone();

             let can_afford = costs.iter().all(|(res, &req)| game_state.current_resources.get(res).unwrap_or(&0.0) >= &req);
             if !can_afford {
                 game_state::add_notification(&mut game_state.notifications, format!("Insufficient materials for {:?}.", building_type), time.elapsed_seconds_f64());
                 return;
             }

             for (res, cost) in &costs {
                 *game_state.current_resources.get_mut(res).unwrap() -= cost;
             }

             match building_type {
                GameBuildingType::Extractor => add_extractor(&mut game_state),
                GameBuildingType::BioDome => add_bio_dome(&mut game_state),
                GameBuildingType::PowerRelay => add_power_relay(&mut game_state),
                GameBuildingType::ResearchInstitute => add_research_institute(&mut game_state),
                GameBuildingType::Fabricator => game_state::add_fabricator(&mut game_state, 0),
                GameBuildingType::ProcessingPlant => game_state::add_processing_plant(&mut game_state, 0),
                GameBuildingType::StorageSilo => add_storage_silo(&mut game_state),
             }
             game_state::add_notification(&mut game_state.notifications, format!("Construction started: {:?}", building_type), time.elapsed_seconds_f64());
        }
     }
}

pub(super) fn zone_construction_system(
    mut interaction_query: Query<(&Interaction, &ConstructZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button_data) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let zone_type = button_data.0;
            let tier_index = button_data.1;
            game_state::add_zone(&mut game_state, zone_type, tier_index);
            // Notifications will be handled in game_state::add_zone
        }
    }
}

pub(super) fn service_construction_system(
    mut interaction_query: Query<(&Interaction, &ConstructServiceButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button_data) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let service_type = button_data.0;
            let tier_index = button_data.1;
            // Assuming add_service_building might take position as an Option in the future.
            // For now, it's not used as per game_state.rs, but good to keep in mind.
            game_state::add_service_building(&mut game_state, service_type, tier_index, None);
            // Consider adding a notification here if desired, similar to other construction systems
        }
    }
}


pub(super) fn construction_category_tab_system(
    mut current_category: ResMut<CurrentConstructionCategory>,
    mut button_query: Query<(&Interaction, &ConstructionCategoryTab, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, tab, mut bg_color) in button_query.iter_mut() {
        if tab.0 == current_category.0 { *bg_color = ACTIVE_BUTTON.into(); } else { *bg_color = NORMAL_BUTTON.into(); }

        if *interaction == Interaction::Pressed {
             if current_category.0 != tab.0 { current_category.0 = tab.0; }
        } else if *interaction == Interaction::Hovered && tab.0 != current_category.0 {
            *bg_color = HOVERED_BUTTON.into();
        }
    }
}

pub(super) fn update_construction_list_system(
    current_app: Res<CurrentApp>,
    current_category: Res<CurrentConstructionCategory>,
    game_state: Res<GameState>,
    mut item_list_panel_query: Query<Entity, With<ConstructionItemListPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Construction { return; }

    if current_category.is_changed() || (current_app.is_changed() && current_app.0 == AppType::Construction) || game_state.is_changed() {
        if let Ok(panel_entity) = item_list_panel_query.get_single_mut() {
            commands.entity(panel_entity).despawn_descendants();

            commands.entity(panel_entity).with_children(|parent| {
                match current_category.0 {
                    ConstructionCategory::Operations => {
                        let building_meta_map = get_building_metadata();
                        let items: Vec<_> = ALL_BUILDING_TYPES.iter()
                            .filter_map(|bt| building_meta_map.get(bt).map(|meta| (bt, meta)))
                            .filter(|(_, meta)| meta.category == current_category.0)
                            .filter(|(_, meta)| {
                                meta.required_dp.is_none() || meta.required_dp.unwrap() <= game_state.current_development_phase
                            })
                            .collect();

                        if items.is_empty() {
                            parent.spawn(TextBundle::from_section("No items in this category.", TextStyle{color: LABEL_TEXT_COLOR, ..default()}));
                            return;
                        }

                        for (building_type, meta) in items {
                            let mut btn = parent.spawn((
                                ButtonBundle { style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(8.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() }, background_color: NORMAL_BUTTON.into(), ..default()},
                                ConstructionItemButton(*building_type)
                            ));
                            btn.insert(UiTag(tag_for_building(*building_type)));
                            btn.with_children(|p| {
                                p.spawn(TextBundle::from_section(meta.name, TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }));
                            });
                        }
                    },
                    ConstructionCategory::Habitation => {
                        let habitation_tiers = game_state::get_habitation_tiers();
                        for (tier_index, tier) in habitation_tiers.iter().enumerate() {
                            let can_afford = game_state.credits >= tier.construction_credits_cost as f64;
                            let mut btn = parent.spawn((
                                ButtonBundle {
                                    style: Style { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(8.0)), margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                                    background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                    ..default()
                                },
                                ConstructHabitationButton(tier_index)
                            ));
                            if tier_index == 0 {
                                btn.insert(UiTag("build_menu.basic_dwelling"));
                            }
                            btn.with_children(|p| {
                                p.spawn(TextBundle::from_section(
                                    format!("{} ({} Cr)", tier.name, tier.construction_credits_cost),
                                    TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }
                                ));
                            });
                        }
                    },
                    ConstructionCategory::Services => {
                        let service_types = [
                            ServiceType::Wellness,
                            ServiceType::Security,
                            ServiceType::Education,
                            ServiceType::Recreation,
                            ServiceType::Spiritual,
                        ];

                        for service_type in service_types.iter() {
                            let service_tiers = game_state::get_service_building_tiers(*service_type);
                            if service_tiers.is_empty() {
                                // This case should ideally not happen if services always have tiers
                                parent.spawn(TextBundle::from_section(
                                    format!("No tiers available for {:?}.", service_type),
                                    TextStyle { color: LABEL_TEXT_COLOR, ..default() },
                                ));
                                continue;
                            }

                            for (tier_index, tier) in service_tiers.iter().enumerate() {
                                let can_afford = game_state.credits >= tier.construction_credits_cost as f64;
                                parent.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            margin: UiRect::bottom(Val::Px(4.0)),
                                            ..default()
                                        },
                                        background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                        ..default()
                                    },
                                    ConstructServiceButton(*service_type, tier_index)
                                )).with_children(|p| {
                                    p.spawn(TextBundle::from_section(
                                        format!("{:?} - {} ({} Cr)", service_type, tier.name, tier.construction_credits_cost),
                                        TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }
                                    ));
                                });
                            }
                        }
                    },
                    ConstructionCategory::Zones => {
                        let zone_types = [ZoneType::Commercial, ZoneType::LightIndustry];

                        for zone_type in zone_types.iter() {
                            let zone_tiers = game_state::get_zone_tiers(*zone_type);
                            if zone_tiers.is_empty() {
                                parent.spawn(TextBundle::from_section(
                                    format!("No tiers available for {:?}.", zone_type),
                                    TextStyle { color: LABEL_TEXT_COLOR, ..default() },
                                ));
                                continue;
                            }

                            for (tier_index, tier) in zone_tiers.iter().enumerate() {
                                let can_afford = game_state.credits >= tier.construction_credits_cost as f64;
                                parent.spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            margin: UiRect::bottom(Val::Px(4.0)),
                                            ..default()
                                        },
                                        background_color: if can_afford { NORMAL_BUTTON.into() } else { DISABLED_BUTTON.into() },
                                        ..default()
                                    },
                                    ConstructZoneButton(*zone_type, tier_index)
                                )).with_children(|p| {
                                    p.spawn(TextBundle::from_section(
                                        format!("{:?} - {} ({} Cr)", zone_type, tier.name, tier.construction_credits_cost),
                                        TextStyle { font_size: 16.0, color: PRIMARY_TEXT_COLOR, ..default() }
                                    ));
                                });
                            }
                        }
                    }
                }
            });
        }
    }
}


pub(super) fn habitation_construction_system(
    mut interaction_query: Query<(&Interaction, &ConstructHabitationButton)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let tier_index = button.0;
            game_state::add_habitation_structure(&mut game_state, tier_index, None);
        }
    }
}


pub(super) fn construction_item_interaction_system(
    mut selected_building_res: ResMut<SelectedBuilding>,
    game_state: Res<GameState>,
    mut button_query: Query<(&Interaction, &ConstructionItemButton, &mut BackgroundColor), With<Button>>,
) {
    let can_afford = |bt: GameBuildingType| -> bool {
         game_state.building_costs.get(&bt).map_or(true, |costs| {
            costs.iter().all(|(res, &req)| game_state.current_resources.get(res).unwrap_or(&0.0) >= &req)
        })
    };

    for (interaction, item_button, mut bg_color) in button_query.iter_mut() {
        let building_type = item_button.0;
        let is_selected = selected_building_res.0 == Some(building_type);

        if is_selected { *bg_color = ACTIVE_BUTTON.into(); }
        else if !can_afford(building_type) { *bg_color = DISABLED_BUTTON.into(); }
        else { *bg_color = NORMAL_BUTTON.into(); }

        if *interaction == Interaction::Pressed {
            selected_building_res.0 = if is_selected { None } else { Some(building_type) };
        } else if *interaction == Interaction::Hovered && !is_selected && can_afford(building_type) {
             *bg_color = HOVERED_BUTTON.into();
        }
    }
}

pub(super) fn update_construction_details_panel_system(
    selected_building: Res<SelectedBuilding>,
    game_state: Res<GameState>,
    mut details_panel_query: Query<Entity, With<ConstructionItemDetailsPanel>>,
    mut commands: Commands,
) {
    if !selected_building.is_changed() { return; }

    if let Ok(panel_entity) = details_panel_query.get_single_mut() {
        commands.entity(panel_entity).despawn_descendants();
        let building_meta_map = get_building_metadata();

        if let Some(building_type) = selected_building.0 {
             if let Some(meta) = building_meta_map.get(&building_type) {
                commands.entity(panel_entity).with_children(|parent| {
                    parent.spawn(TextBundle::from_section(meta.name, TextStyle{font_size: 22.0, color: PRIMARY_TEXT_COLOR, ..default()}));

                    parent.spawn(TextBundle::from_section(format!("- Workforce Required: {}", meta.workforce_required), TextStyle{ color: LABEL_TEXT_COLOR, ..default()}));

                    if let Some(costs) = game_state.building_costs.get(&building_type) {
                        for (res, amount) in costs {
                            parent.spawn(TextBundle::from_section(format!("- {:?}: {}", res, amount), TextStyle{ color: LABEL_TEXT_COLOR, ..default()}));
                        }
                    }
                     if let Some(tech) = meta.required_tech {
                         parent.spawn(TextBundle::from_section(format!("Req: {:?}", tech), TextStyle{ color: LABEL_TEXT_COLOR, ..default()}));
                     }
                    parent.spawn((ButtonBundle{style:Style{position_type: PositionType::Absolute, bottom: Val::Px(10.0), right: Val::Px(10.0), padding: UiRect::all(Val::Px(10.0)), ..default()}, background_color: NORMAL_BUTTON.into(), ..default()}, ConfirmBuildButton(building_type)))
                    .with_children(|button| { button.spawn(TextBundle::from_section("CONSTRUCT", TextStyle{color: PRIMARY_TEXT_COLOR, ..default()})); });
                });
            }
        } else {
            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn(TextBundle::from_section("Select an item to view details.", TextStyle { color: LABEL_TEXT_COLOR, ..default() }));
            });
        }
    }
}
