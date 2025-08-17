use crate::dom;
use js_sys::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::*;
use wasm_bindgen_futures::*;
use web_sys::*;

/// Shows floating connection status notification. Doesn't disappear automatically.
pub fn update_connection_status(status: &str) {
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
                 background-color: rgba(255, 255, 255, 0.95); \
                 border: 2px solid #333; \
                 border-radius: 25px; \
                 padding: 8px 16px; \
                 font-size: 14px; \
                 font-weight: bold; \
                 z-index: 9999; \
                 box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2); \
                 backdrop-filter: blur(5px);",
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

#[derive(Clone)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    fn get_background_color(&self) -> &str {
        match self {
            ToastType::Success => "rgba(76, 175, 80, 0.95)",
            ToastType::Error => "rgba(244, 67, 54, 0.95)",
            ToastType::Warning => "rgba(255, 193, 7, 0.95)",
            ToastType::Info => "rgba(33, 150, 243, 0.95)",
        }
    }

    fn get_border_color(&self) -> &str {
        match self {
            ToastType::Success => "#4CAF50",
            ToastType::Error => "#F44336",
            ToastType::Warning => "#FFC107",
            ToastType::Info => "#2196F3",
        }
    }
}

/// Shows a toast notification that automatically disappears after a specified duration.
pub fn show_toast(message: &str, toast_type: ToastType) {
    let doc = dom::document();
    let body = doc.body().unwrap();

    let timestamp = js_sys::Date::now() as u64;
    let toast_id = format!("toast_{}", timestamp);

    let toast = doc.create_element("div").unwrap();
    toast.set_id(&toast_id);
    toast.set_text_content(Some(message));

    // calculate position based on existing toasts
    let existing_toasts = doc.query_selector_all("[id^='toast_']").unwrap();
    let vertical_offset = 20 + (existing_toasts.length() * 70); // 70px spacing between toasts

    // apply styling based on toast type
    let style = format!(
        "position: fixed; \
         top: {}px; \
         right: 20px; \
         background-color: {}; \
         border: 2px solid {}; \
         border-radius: 8px; \
         padding: 12px 16px; \
         font-size: 14px; \
         font-weight: bold; \
         color: white; \
         z-index: 10000; \
         box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); \
         backdrop-filter: blur(5px); \
         max-width: 300px; \
         word-wrap: break-word; \
         animation: slideInRight 0.3s ease-out;",
        vertical_offset,
        toast_type.get_background_color(),
        toast_type.get_border_color()
    );

    toast.set_attribute("style", &style).unwrap();
    add_toast_animations();
    body.append_child(&toast).unwrap();

    // auto removal
    let toast_id_clone = toast_id.clone();
    let callback = Closure::wrap(Box::new(move || {
        remove_toast(&toast_id_clone);
    }) as Box<dyn FnMut()>);
    web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), 4000).unwrap(); // 4s floating

    callback.forget();
}

/// Removes a specific toast notification and repositions remaining toasts to fill the gap.
fn remove_toast(toast_id: &str) {
    let doc = dom::document();

    if let Some(toast) = doc.get_element_by_id(toast_id) {
        // fade-out animation
        let current_style = toast.get_attribute("style").unwrap_or_default();
        let fade_style = format!("{}; animation: fadeOut 0.3s ease-out;", current_style);
        toast.set_attribute("style", &fade_style).unwrap();

        // remove after animation completes
        let toast_clone = toast.clone();
        let callback = Closure::wrap(Box::new(move || {
            if let Some(parent) = toast_clone.parent_node() {
                parent.remove_child(&toast_clone).unwrap();
            }
            reposition_toasts();
        }) as Box<dyn FnMut()>);

        web_sys::window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), 300).unwrap();

        callback.forget();
    }
}

/// Repositions all remaining toast notifications after one is removed to fill the gap.
fn reposition_toasts() {
    let doc = dom::document();
    let toasts = doc.query_selector_all("[id^='toast_']").unwrap();

    for i in 0..toasts.length() {
        if let Some(toast) = toasts.item(i) {
            let element = toast.dyn_into::<HtmlElement>().unwrap();
            let new_top = 20 + (i * 70);

            let current_style = element.get_attribute("style").unwrap_or_default();
            let updated_style = dom::update_style_property(&current_style, "top", &format!("{}px", new_top));
            element.set_attribute("style", &updated_style).unwrap();
        }
    }
}

/// Adds CSS animations for toast notifications if they don't already exist.
fn add_toast_animations() {
    let doc = dom::document();

    if doc.get_element_by_id("toast_animations").is_some() {
        return;
    }

    let head = doc.head().unwrap();
    let style = doc.create_element("style").unwrap();
    style.set_id("toast_animations");

    let css = r#"
        @keyframes slideInRight {
            from {
                transform: translateX(100%);
                opacity: 0;
            }
            to {
                transform: translateX(0);
                opacity: 1;
            }
        }
        
        @keyframes fadeOut {
            from {
                opacity: 1;
            }
            to {
                opacity: 0;
            }
        }
    "#;

    style.set_text_content(Some(css));
    head.append_child(&style).unwrap();
}
