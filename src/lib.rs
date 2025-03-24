use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

fn bare_bones() {
    log("Hello from Rust!");

    let js: JsValue = 4.into();
    console::log_2(&"Logging arbitrary values looks like".into(), &js);

    console_log!("Hello {}!", "world");
}

fn using_a_macro() {
    console_log!("Let's print some numbers...");
    console_log!("1 + 3 = {}", 1 + 3);
}

#[wasm_bindgen]
pub fn run() {
    bare_bones();
    using_a_macro();
}
