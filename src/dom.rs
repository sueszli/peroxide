use wasm_bindgen::{JsCast, prelude::*};
use web_sys::*;

thread_local! {
    static DOC: Document = window().unwrap().document().unwrap();
}
pub fn document() -> Document {
    DOC.with(|d| d.clone())
}

/// Binds a lambda to be executed when a key is pressed on the specified element.
pub fn onkeypress<F: 'static + FnMut(KeyboardEvent)>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut(KeyboardEvent)>);
    element.add_event_listener_with_callback("keypress", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

/// Binds a lambda to be executed when the specified element is clicked.
pub fn onclick<F: 'static + FnMut()>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut()>);
    element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}
