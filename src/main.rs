mod components;
mod game_state;
mod resources;
mod systems;
mod ui;
mod alerts;
mod app;

fn main() {
    app::build_app().run();
}