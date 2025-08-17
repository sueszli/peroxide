use crate::dom;
use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;

/// Shows floating connection status notification. Doesn't disappear automatically.
pub fn show_connection_notification(status: &str) {
    let doc = dom::document();

    // create if missing
    let status_element = match doc.get_element_by_id("connection_status") {
        Some(element) => element,
        None => {
            let div = doc.create_element("div").unwrap();
            div.set_id("connection_status");

            div.set_attribute(
                "style",
                "position: fixed; \
                 top: 20px; \
                 left: 20px; \
                 height: 34px; \
                 background-color: rgba(255, 255, 255, 0.95); \
                 border: 2px solid #333; \
                 border-radius: 25px 0 0 25px; \
                 padding: 8px 16px; \
                 font-size: 14px; \
                 font-weight: bold; \
                 z-index: 9999; \
                 box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2); \
                 backdrop-filter: blur(5px); \
                 display: flex; \
                 align-items: center; \
                 text-align: left; \
                 color: #333;",
            )
            .unwrap();

            let body = doc.body().unwrap();
            body.insert_before(&div, body.first_child().as_ref()).unwrap();

            div
        }
    };

    // update content
    status_element.set_text_content(Some(status));
}

/// Shows a notification in the floating pill that automatically disappears after 5 seconds with fade-out.
pub fn show_notification(message: &str) {
    let doc = dom::document();
    let body = doc.body().unwrap();

    if doc.get_element_by_id("notification_pill").is_some() {
        return;
    }

    let div = doc.create_element("div").unwrap();
    div.set_id("notification_pill");

    // position the pill to the right of connection status, filling remaining horizontal space
    div.set_attribute(
        "style",
        "position: fixed; \
         top: 20px; \
         left: 200px; \
         right: 20px; \
         height: 34px; \
         background-color: rgba(255, 255, 255, 0.95); \
         border: 2px solid #333; \
         border-left: none; \
         border-radius: 0 25px 25px 0; \
         padding: 8px 16px; \
         font-size: 14px; \
         font-weight: bold; \
         z-index: 9999; \
         box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2); \
         backdrop-filter: blur(5px); \
         display: flex; \
         align-items: center; \
         justify-content: flex-start; \
         text-align: left; \
         color: #333; \
         opacity: 1; \
         transition: opacity 0.5s ease-in-out;",
    )
    .unwrap();

    body.insert_before(&div, body.first_child().as_ref()).unwrap();

    let doc = dom::document();
    let notification_pill = doc.get_element_by_id("notification_pill").unwrap();

    // set the notification message with consistent text color
    let current_style = notification_pill.get_attribute("style").unwrap_or_default();
    let updated_style = dom::update_style_property(&current_style, "color", "#333");
    notification_pill.set_attribute("style", &updated_style).unwrap();
    notification_pill.set_text_content(Some(message));

    // auto removal after 5 seconds
    let callback = Closure::wrap(Box::new(move || {
        let doc = dom::document();
        if let Some(pill) = doc.get_element_by_id("notification_pill") {
            pill.set_text_content(Some(""));

            // reset color to default
            let current_style = pill.get_attribute("style").unwrap_or_default();
            let updated_style = dom::update_style_property(&current_style, "color", "#333");
            pill.set_attribute("style", &updated_style).unwrap();
        }
    }) as Box<dyn FnMut()>);
    web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), 5000).unwrap();

    callback.forget();
}
