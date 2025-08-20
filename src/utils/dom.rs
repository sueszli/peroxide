use wasm_bindgen::{JsCast, prelude::*};
use web_sys::*;

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
