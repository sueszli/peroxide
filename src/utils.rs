use wasm_bindgen::prelude::*;
use web_sys::{Element, Event};

pub fn insert_style(string: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let head = document.head().unwrap();

    let style = document.create_element("style").unwrap();
    style.set_text_content(Some(string));
    head.append_child(&style).unwrap();
}

pub fn insert_html(string: &str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    body.set_inner_html(string);
}

// 
// debug
// 

pub fn display_resize() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let elem: Element = document.create_element("div").unwrap();
    body.append_child(&elem).unwrap();

    fn update(elem: &Element) {
        let width = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap();
        let height = web_sys::window().unwrap().inner_height().unwrap().as_f64().unwrap();
        elem.set_text_content(Some(&format!("window size: {:.0} x {:.0}", width, height)));
    }

    update(&elem);

    let callback = Closure::<dyn FnMut(Event)>::new(move |_| update(&elem));
    window.add_event_listener_with_callback("resize", callback.as_ref().unchecked_ref()).unwrap();

    callback.forget(); // don't drop
}

pub fn display_mousemove() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    let elem = document.create_element("div").unwrap();
    body.append_child(&elem).unwrap();

    fn update(elem: &Element, event: web_sys::MouseEvent) {
        let x: i32 = event.client_x();
        let y = event.client_y();
        elem.set_text_content(Some(&format!("mouse position: [{}, {}]", x, y)));
    }

    let callback = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| update(&elem, event));
    window.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref()).unwrap();

    callback.forget();
}
