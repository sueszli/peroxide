use wasm_bindgen::prelude::*;
use web_sys::{Element, Event};

// 
// debug
// 

pub fn mount_window_size_listener(document: &web_sys::Document) {
    let elem = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&elem).unwrap();

    fn update(elem: &Element) {
        let width = web_sys::window().unwrap().inner_width().unwrap().as_f64().unwrap();
        let height = web_sys::window().unwrap().inner_height().unwrap().as_f64().unwrap();
        elem.set_text_content(Some(&format!("window size: {:.0} x {:.0}", width, height)));
    }

    update(&elem);

    let callback = Closure::<dyn FnMut(Event)>::new(move |_| update(&elem));
    web_sys::window().unwrap().add_event_listener_with_callback("resize", callback.as_ref().unchecked_ref()).unwrap();

    callback.forget(); // don't drop
}

pub fn mount_mousemove_listener(document: &web_sys::Document) {
    let elem = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&elem).unwrap();

    fn update(elem: &Element, event: web_sys::MouseEvent) {
        let x: i32 = event.client_x();
        let y = event.client_y();
        elem.set_text_content(Some(&format!("mouse position: {:.0} x {:.0}", x, y)));
    }

    let callback = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| update(&elem, event));
    web_sys::window().unwrap().add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref()).unwrap();

    callback.forget();
}
