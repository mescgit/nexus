mod components;
mod game_state;
mod resources;
mod systems;
mod ui;
mod app;

fn main() {
    app::build_app().run();
}