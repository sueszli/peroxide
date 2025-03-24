mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // panics to console.error

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    utils::mount_mousemove_listener(&document);
    utils::mount_window_size_listener(&document);

    Ok(())
}
