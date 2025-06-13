use bevy::prelude::*;
use crate::game_state::{self, construct_legacy_structure, get_legacy_structure_tiers, upgrade_legacy_structure, ColonyStats, GameState, GraphData, LoadGameEvent, SaveGameEvent, ZoneType};
use super::*;

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
#[derive(Component)]
pub(super) struct ZoneListButton(pub String);
#[derive(Component)]
pub(super) struct ZoneDetailsPanel;
#[derive(Component)]
pub(super) struct UpgradeZoneButton(pub String);
#[derive(Component)]
pub(super) struct RemoveZoneButton(pub String);
#[derive(Component)]
pub(super) struct AssignSpecialistToZoneButton(pub String);
#[derive(Component)]
pub(super) struct UnassignSpecialistFromZoneButton(pub String);
#[derive(Component)]
pub(super) struct SaveGameButton;
#[derive(Component)]
pub(super) struct LoadGameButton;

pub(super) fn update_managed_structures_panel_system(
    current_app: Res<CurrentApp>,
    game_state: Res<GameState>,
    selected_zone: Res<SelectedZone>,
    panel_query: Query<Entity, With<ManagedStructuresPanel>>,
    mut commands: Commands,
) {
    if current_app.0 != AppType::Dashboard {
        return;
    }

    if game_state.is_changed()
        || (current_app.is_changed() && current_app.0 == AppType::Dashboard)
        || selected_zone.is_changed()
    {
        if let Ok(panel_entity) = panel_query.get_single() {
            commands.entity(panel_entity).despawn_descendants();

            commands.entity(panel_entity).with_children(|parent| {
                parent.spawn(
                    TextBundle::from_section(
                        "Managed Zones List",
                        TextStyle {
                            font_size: 16.0,
                            color: LABEL_TEXT_COLOR,
                            ..default()
                        },
                    )
                    .with_style(Style { margin: UiRect::bottom(Val::Px(5.0)), ..default() }),
                );

                if game_state.zones.is_empty() {
                    parent.spawn(TextBundle::from_section(
                        "No zones established.",
                        TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                    ));
                } else {
                    for zone in game_state.zones.iter() {
                        if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            padding: UiRect::all(Val::Px(5.0)),
                                            margin: UiRect::bottom(Val::Px(2.0)),
                                            justify_content: JustifyContent::Center,
                                            ..default()
                                        },
                                        background_color: NORMAL_BUTTON.into(),
                                        ..default()
                                    },
                                    ZoneListButton(zone.id.clone()),
                                ))
                                .with_children(|button_parent| {
                                    button_parent.spawn(TextBundle::from_section(
                                        format!(
                                            "{:?} - {} (Workers: {}/{})",
                                            zone.zone_type,
                                            tier.name,
                                            zone.assigned_specialists,
                                            tier.specialist_jobs_provided
                                        ),
                                        TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                                    ));
                                });
                        }
                    }
                }

                let mut details_panel = parent.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::top(Val::Px(10.0)),
                            padding: UiRect::all(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            min_height: Val::Px(150.0),
                            ..default()
                        },
                        border_color: Color::rgba(0.5, 0.5, 0.5, 0.5).into(),
                        background_color: Color::rgba(0.1, 0.1, 0.1, 0.3).into(),
                        ..default()
                    },
                    ZoneDetailsPanel,
                ));

                if let Some(selected_zone_id) = &selected_zone.0 {
                    if let Some(zone) = game_state.zones.iter().find(|z| z.id == *selected_zone_id) {
                        if let Some(current_tier) = zone.available_tiers.get(zone.current_tier_index) {
                            details_panel.with_children(|details_parent| {
                                details_parent.spawn(
                                    TextBundle::from_section(
                                        format!("Zone Details: {:?} - {}", zone.zone_type, current_tier.name),
                                        TextStyle { font_size: 15.0, color: PRIMARY_TEXT_COLOR, ..default() },
                                    )
                                    .with_style(Style { margin: UiRect::bottom(Val::Px(4.0)), ..default() }),
                                );

                                details_parent.spawn(
                                    TextBundle::from_section(
                                        format!("Workers: {}/{}", zone.assigned_specialists, current_tier.specialist_jobs_provided),
                                        TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },
                                    )
                                    .with_style(Style { margin: UiRect::bottom(Val::Px(2.0)), ..default() }),
                                );

                                details_parent.spawn(
                                    TextBundle::from_section(
                                        format!("Civic Index: {}", current_tier.civic_index_contribution),
                                        TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },
                                    )
                                    .with_style(Style { margin: UiRect::bottom(Val::Px(2.0)), ..default() }),
                                );

                                if zone.zone_type == ZoneType::Commercial {
                                    details_parent.spawn(
                                        TextBundle::from_section(
                                            format!("Income: {} Cr/cycle", current_tier.income_generation),
                                            TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },
                                        )
                                        .with_style(Style { margin: UiRect::bottom(Val::Px(2.0)), ..default() }),
                                    );
                                }

                                details_parent.spawn(
                                    TextBundle::from_section(
                                        format!("Upkeep: {} Cr/cycle", current_tier.upkeep_cost),
                                        TextStyle { font_size: 14.0, color: LABEL_TEXT_COLOR, ..default() },
                                    )
                                    .with_style(Style { margin: UiRect::bottom(Val::Px(8.0)), ..default() }),
                                );

                                if zone.current_tier_index < zone.available_tiers.len() - 1 {
                                    let next_tier = &zone.available_tiers[zone.current_tier_index + 1];
                                    let can_afford_upgrade =
                                        game_state.credits >= next_tier.construction_credits_cost as f64;
                                    details_parent
                                        .spawn((
                                            ButtonBundle {
                                                style: Style {
                                                    width: Val::Percent(100.0),
                                                    padding: UiRect::all(Val::Px(5.0)),
                                                    margin: UiRect::bottom(Val::Px(4.0)),
                                                    ..default()
                                                },
                                                background_color: if can_afford_upgrade {
                                                    NORMAL_BUTTON.into()
                                                } else {
                                                    DISABLED_BUTTON.into()
                                                },
                                                ..default()
                                            },
                                            UpgradeZoneButton(selected_zone_id.clone()),
                                        ))
                                        .with_children(|btn| {
                                            btn.spawn(TextBundle::from_section(
                                                format!("Upgrade to {} ({} Cr)", next_tier.name, next_tier.construction_credits_cost),
                                                TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                                            ));
                                        });
                                } else {
                                    details_parent.spawn(TextBundle::from_section(
                                        "Max tier reached",
                                        TextStyle { font_size: 14.0, color: Color::CYAN, ..default() },
                                    ));
                                }

                                details_parent
                                    .spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Percent(100.0),
                                                padding: UiRect::all(Val::Px(5.0)),
                                                margin: UiRect::top(Val::Px(4.0)),
                                                ..default()
                                            },
                                            background_color: NORMAL_BUTTON.into(),
                                            ..default()
                                        },
                                        RemoveZoneButton(selected_zone_id.clone()),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            "Remove Zone",
                                            TextStyle { font_size: 14.0, color: Color::TOMATO, ..default() },
                                        ));
                                    });

                                details_parent
                                    .spawn(NodeBundle { style: Style { height: Val::Px(10.0), ..default() }, ..default() });

                                let can_assign_more_specialists_to_zone =
                                    zone.assigned_specialists < current_tier.specialist_jobs_provided;
                                let available_general_inhabitants =
                                    game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
                                let can_assign =
                                    can_assign_more_specialists_to_zone && available_general_inhabitants > 0;

                                details_parent
                                    .spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Percent(100.0),
                                                padding: UiRect::all(Val::Px(5.0)),
                                                margin: UiRect::bottom(Val::Px(4.0)),
                                                ..default()
                                            },
                                            background_color: if can_assign {
                                                NORMAL_BUTTON.into()
                                            } else {
                                                DISABLED_BUTTON.into()
                                            },
                                            ..default()
                                        },
                                        AssignSpecialistToZoneButton(selected_zone_id.clone()),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            "Assign Specialist (+1)",
                                            TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                                        ));
                                    });

                                let can_unassign = zone.assigned_specialists > 0;
                                details_parent
                                    .spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: Val::Percent(100.0),
                                                padding: UiRect::all(Val::Px(5.0)),
                                                ..default()
                                            },
                                            background_color: if can_unassign {
                                                NORMAL_BUTTON.into()
                                            } else {
                                                DISABLED_BUTTON.into()
                                            },
                                            ..default()
                                        },
                                        UnassignSpecialistFromZoneButton(selected_zone_id.clone()),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn(TextBundle::from_section(
                                            "Unassign Specialist (-1)",
                                            TextStyle { font_size: 14.0, color: PRIMARY_TEXT_COLOR, ..default() },
                                        ));
                                    });
                            });
                        }
                    }
                }
            });
        }
    }
}

pub(super) fn assign_specialist_to_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &AssignSpecialistToZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let zone_id = &button.0;
            if let Some(zone) = game_state.zones.iter().find(|z| z.id == *zone_id) {
                if let Some(tier) = zone.available_tiers.get(zone.current_tier_index) {
                    let can_assign_more = zone.assigned_specialists < tier.specialist_jobs_provided;
                    let available_general = game_state.total_inhabitants.saturating_sub(game_state.assigned_specialists_total);
                    if can_assign_more && available_general > 0 {
                        game_state::assign_specialists_to_zone(&mut game_state, zone_id, 1);
                    }
                }
            }
        }
    }
}

pub(super) fn unassign_specialist_from_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &UnassignSpecialistFromZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let zone_id = &button.0;
            if let Some(zone) = game_state.zones.iter().find(|z| z.id == *zone_id) {
                if zone.assigned_specialists > 0 {
                    game_state::unassign_specialists_from_zone(&mut game_state, zone_id, 1);
                }
            }
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

pub(super) fn zone_list_button_interaction_system(
    mut selected_zone_res: ResMut<SelectedZone>,
    mut button_query: Query<(&Interaction, &ZoneListButton, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, zone_button_data, mut bg_color) in button_query.iter_mut() {
        let is_currently_selected_in_res = selected_zone_res.0.as_ref() == Some(&zone_button_data.0);

        if *interaction == Interaction::Pressed {
            if is_currently_selected_in_res {
                selected_zone_res.0 = None;
            } else {
                selected_zone_res.0 = Some(zone_button_data.0.clone());
            }
        }

        if selected_zone_res.is_changed() || *interaction != Interaction::None { // Update if selection changed or interaction happened
            if selected_zone_res.0.as_ref() == Some(&zone_button_data.0) {
                *bg_color = ACTIVE_BUTTON.into();
            } else if *interaction == Interaction::Hovered {
                *bg_color = HOVERED_BUTTON.into();
            } else {
                *bg_color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub(super) fn upgrade_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &UpgradeZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            // Affordability is checked visually by button color in update_managed_structures_panel_system.
            // Direct click interaction here assumes if it's clickable (not explicitly blocked by Bevy's interaction system itself),
            // the action should be attempted. game_state::upgrade_zone should handle the actual logic including costs.
            game_state::upgrade_zone(&mut game_state, &button.0);
        }
    }
}

pub(super) fn remove_zone_button_interaction_system(
    mut interaction_query: Query<(&Interaction, &RemoveZoneButton), (Changed<Interaction>, With<Button>)>,
    mut game_state: ResMut<GameState>,
    mut selected_zone: ResMut<SelectedZone>,
) {
    for (interaction, button) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            game_state::remove_zone(&mut game_state, &button.0);
            selected_zone.0 = None;
        }
    }
}

pub(super) fn admin_spire_button_interaction_system(
    mut game_state: ResMut<GameState>,
    mut interaction_query: ParamSet<(
        Query<&Interaction, (Changed<Interaction>, With<ConstructSpireButton>)>,
        Query<&Interaction, (Changed<Interaction>, With<UpgradeSpireButton>)>,
    )>,
) {
    if let Ok(Interaction::Pressed) = interaction_query.p0().get_single() {
        game_state::construct_administrative_spire(&mut game_state);
    }

    if let Ok(Interaction::Pressed) = interaction_query.p1().get_single() {
        game_state::upgrade_administrative_spire(&mut game_state);
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
