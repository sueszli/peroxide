use wasm_bindgen::{JsCast, prelude::*};
use web_sys::*;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => (web_sys::console::error_1(&format_args!($($t)*).to_string().into()))
}

thread_local! {
    static DOC: Document = web_sys::window().unwrap().document().unwrap();
}
pub fn document() -> Document {
    DOC.with(|d| d.clone())
}

pub fn onkeypress<F: 'static + FnMut(KeyboardEvent)>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut(KeyboardEvent)>);
    element.add_event_listener_with_callback("keypress", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

pub fn onclick<F: 'static + FnMut()>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut()>);
    element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

/// Updates a specific CSS property in a style string.
///
/// ```rust
/// update_style_property("color: red; font-size: 16px;", "color", "blue"); // "color: blue; font-size: 16px;"
/// ```
pub fn update_style_property(style: &str, property: &str, value: &str) -> String {
    let properties: Vec<&str> = style.split(';').collect();
    let mut updated_properties = Vec::new();
    let mut found = false;

    for prop in properties {
        let prop = prop.trim();
        if prop.is_empty() {
            continue;
        }

        if prop.starts_with(&format!("{}:", property)) {
            updated_properties.push(format!("{}: {}", property, value));
            found = true;
        } else {
            updated_properties.push(prop.to_string());
        }
    }

    if !found {
        updated_properties.push(format!("{}: {}", property, value));
    }

    updated_properties.join("; ")
}
