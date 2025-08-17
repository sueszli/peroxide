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

/// Gets or sets the inner text content of an element by its ID.
///
/// For HTML: `<div id="myDiv">hello</div>`
/// ```rust
/// let text = get_elem_innertext("myDiv"); // Returns Some("hello")
/// ```
pub fn get_elem_innertext(id: &str) -> Option<String> {
    document().get_element_by_id(id)?.text_content()
}
pub fn set_elem_innertext(id: &str, text: &str) {
    if let Some(element) = document().get_element_by_id(id) {
        element.set_text_content(Some(text));
    }
}
