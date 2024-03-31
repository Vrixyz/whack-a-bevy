use bevy::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

use example_client_logic::GamePlugin;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    App::new().add_plugins(GamePlugin).run();
}
