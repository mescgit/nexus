use bevy::prelude::*;

use crate::game_state::GameLogicPlugin;
use crate::systems::TutorialPlugin;
use crate::ui::UiPlugin;
use crate::alerts::AlertPlugin;

pub fn build_app() -> App {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::rgb(0.1, 0.05, 0.15)))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Nexus Core: Colony Manager".into(),
                    ..default()
                }),
                ..default()
            }),
            GameLogicPlugin,
            TutorialPlugin,
            AlertPlugin,
            UiPlugin,
        ))
        .insert_resource(Time::<Fixed>::from_seconds(1.0));
    app
}