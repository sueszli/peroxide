mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // panics to console.error

    utils::show_mousemove();
    utils::show_resize();

    Ok(())
}
