use bevy::prelude::*;

use example_client_logic::GamePlugin;

pub fn main() {
    App::new().add_plugin(GamePlugin).run();
}
