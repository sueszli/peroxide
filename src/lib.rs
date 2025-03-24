use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Event, HtmlElement, MouseEvent};

fn mount_window_size_listener(document: &web_sys::Document) -> Result<(), JsValue> {
    let elem = document.create_element("div")?;
    document.body().unwrap().append_child(&elem)?;

    fn update_dimensions(elem: &Element) -> Result<(), JsValue> {
        let width = web_sys::window().unwrap().inner_width()?.as_f64().unwrap();
        let height = web_sys::window().unwrap().inner_height()?.as_f64().unwrap();
        elem.set_text_content(Some(&format!("window size: {:.0} x {:.0}", width, height)));
        Ok(())
    }

    update_dimensions(&elem)?; // initial update

    let winsize_listener = Closure::<dyn FnMut(Event)>::new(move |_| {
        update_dimensions(&elem).unwrap_throw();
    });
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("resize", winsize_listener.as_ref().unchecked_ref())?;

    winsize_listener.forget(); // don't drop

    Ok(())
}

fn mount_mousemove_listener(document: &web_sys::Document) -> Result<(), JsValue> {
    let elem = document.create_element("div")?;
    document.body().unwrap().append_child(&elem)?;

    // fn update_mouse_position(elem: &Element, event: &web_sys::MouseEvent) -> Result<(), JsValue> {
    //     let x = event.client_x();
    //     let y = event.client_y();
    //     elem.set_text_content(Some(&format!("mouse position: {:.0} x {:.0}", x, y)));
    //     Ok(())
    // }

    let mousemove_listener = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event| {
        let x = event.client_x();
        let y = event.client_y();
        elem.set_text_content(Some(&format!("mouse position: {:.0} x {:.0}", x, y)));
    });
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback(
            "mousemove",
            mousemove_listener.as_ref().unchecked_ref(),
        )?;

    mousemove_listener.forget();

    Ok(())
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // panics to console.error

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();

    mount_window_size_listener(&document)?;
    mount_mousemove_listener(&document)?;

    Ok(())
}
