use bevy::prelude::*;

use crate::resources::tutorial::TutorialState;
use crate::ui::UiTag;

#[derive(Component)]
struct HighlightOriginalColor(Color);

#[derive(Resource)]
pub struct TutorialUi {
    pub container: Entity,
    pub text: Entity,
    pub button: Entity,
}

#[derive(Component)]
struct TutorialOkButton;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .add_systems(Startup, setup_tutorial_ui)
            .add_systems(Update, (tutorial_system, tutorial_ok_button_system));
    }
}

fn setup_tutorial_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let container = commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                left: Val::Px(20.0),
                bottom: Val::Px(20.0),
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.7).into(),
            visibility: Visibility::Hidden,
            ..default()
        })
        .id();

    let text_entity = commands
        .spawn(TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 20.0,
                color: Color::WHITE,
            },
        ))
        .id();

    let button_entity = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    padding: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                ..default()
            },
            TutorialOkButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "OK",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ));
        })
        .id();

    commands
        .entity(container)
        .push_children(&[text_entity, button_entity]);
    commands.insert_resource(TutorialUi {
        container,
        text: text_entity,
        button: button_entity,
    });
}

pub fn tutorial_system(world: &mut World) {
    let (trigger, title, content, action, highlight_tag, step_index, total_steps) = {
        let state = world.resource::<TutorialState>();
        if state.current_step >= state.steps.len() {
            return;
        }
        let step = &state.steps[state.current_step];
        (
            step.trigger,
            step.title,
            step.content,
            step.required_action,
            step.ui_highlight,
            state.current_step,
            state.steps.len(),
        )
    };

    if trigger(world) {
        info!(
            "[Tutorial {}/{}] {} - {}",
            step_index + 1,
            total_steps,
            title,
            content
        );

        // Update on-screen tutorial text
        let (container, text_entity) = {
            let ui = world.resource::<TutorialUi>();
            (ui.container, ui.text)
        };

        if let Some(mut text) = world.entity_mut(text_entity).get_mut::<Text>() {
            text.sections[0].value = format!("{}\n{}", title, content);
        }
        world.entity_mut(container).insert(Visibility::Visible);

        // clear previous highlight
        if let Some(prev) = world.resource::<TutorialState>().highlighted {
            if let Some(mut entity) = world.get_entity_mut(prev) {
                let orig_color = entity
                    .get::<HighlightOriginalColor>()
                    .map(|c| c.0);
                if let (Some(mut bg), Some(orig)) = (entity.get_mut::<BackgroundColor>(), orig_color) {
                    bg.0 = orig;
                }
                entity.remove::<HighlightOriginalColor>();
            }
            world.resource_mut::<TutorialState>().highlighted = None;
        }

        // apply new highlight if defined
        if let Some(tag) = highlight_tag {
            let mut q = world.query::<(Entity, &UiTag)>();
            if let Some((entity, _)) = q.iter(world).find(|(_, t)| t.0 == tag) {
                if let Some(mut entity_mut) = world.get_entity_mut(entity) {
                    if let Some(mut bg) = entity_mut.get_mut::<BackgroundColor>() {
                        let old = bg.0;
                        bg.0 = Color::YELLOW;
                        entity_mut.insert(HighlightOriginalColor(old));
                        world.resource_mut::<TutorialState>().highlighted = Some(entity);
                    }
                }
            }
        }

        if let Some(action_fn) = action {
            action_fn(world);
        }
        world.resource_mut::<TutorialState>().current_step += 1;
    }
}

fn tutorial_ok_button_system(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<TutorialOkButton>)>,
    tutorial_ui: Res<TutorialUi>,
    mut vis_query: Query<&mut Visibility>,
    mut state: ResMut<TutorialState>,
    mut bg_query: Query<(Entity, &mut BackgroundColor, Option<&HighlightOriginalColor>)>,
    mut commands: Commands,
) {
    if let Ok(Interaction::Pressed) = interaction_query.get_single() {
        if let Ok(mut vis) = vis_query.get_mut(tutorial_ui.container) {
            *vis = Visibility::Hidden;
        }
        if let Some(entity) = state.highlighted.take() {
            if let Ok((_, mut color, orig)) = bg_query.get_mut(entity) {
                if let Some(orig) = orig {
                    color.0 = orig.0;
                }
            }
            commands.entity(entity).remove::<HighlightOriginalColor>();
        }
    }
}