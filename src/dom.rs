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
    static DOC: Document = window().unwrap().document().unwrap();
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

/// Updates or creates a connection status overlay indicator in the DOM.
///
/// This function dynamically manages a connection status indicator that appears as a
/// fixed-position overlay in the top-left corner of the viewport. The indicator is
/// styled as a rounded box with high z-index to ensure it appears above all other content.
///
/// # Behavior
///
/// - **First call**: Creates a new `div` element with id "connection_status" and inserts
///   it at the beginning of the document body with inline styling applied.
/// - **Subsequent calls**: Finds the existing element and updates its text content.
///
/// # Styling
///
/// The status indicator has the following visual characteristics:
/// - Fixed position at top: 20px, left: 20px
/// - Semi-transparent white background with dark border
/// - Rounded corners (25px border-radius)  
/// - High z-index (9999) to appear above other elements
/// - Box shadow and backdrop blur for visual prominence
///
/// # Arguments
///
/// * `status` - The status text to display (e.g., "🟢 Connected", "🔴 Disconnected")
///
/// # Examples
///
/// ```rust
/// // Create or update the status indicator
/// update_connection_status("🟢 Connected");
/// update_connection_status("🔴 Disconnected");
/// update_connection_status("🟡 Connecting...");
/// ```
pub fn update_connection_status(status: &str) {
    let doc = document();

    // Check if connection_status div already exists
    let status_element = match doc.get_element_by_id("connection_status") {
        Some(element) => element,
        None => {
            // Create new div element
            let div = doc.create_element("div").unwrap();
            div.set_id("connection_status");

            // Add inline styling for fixed overlay positioning
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

            // Insert at the beginning of body (topmost layer)
            let body = doc.body().unwrap();
            body.insert_before(&div, body.first_child().as_ref()).unwrap();

            div
        }
    };

    // Update the text content
    status_element.set_text_content(Some(status));
}

/// Toast notification types for different styling and behavior
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
///
/// This function creates temporary notification overlays that appear in the top-right corner
/// of the viewport and stack vertically if multiple notifications are shown. Each notification
/// automatically removes itself after the specified duration.
///
/// # Behavior
///
/// - Creates a new `div` element with a unique ID based on timestamp
/// - Positions the toast in the top-right corner with automatic stacking
/// - Applies styling based on the toast type (success, error, warning, info)
/// - Automatically removes the toast after the specified duration
/// - Handles dynamic repositioning of remaining toasts when one is removed
///
/// # Styling
///
/// Each toast notification has:
/// - Fixed position in top-right corner with dynamic vertical offset
/// - Type-specific background and border colors
/// - Rounded corners, padding, and box shadow
/// - High z-index (10000) to appear above other elements
/// - Smooth transitions for appearance and removal
///
/// # Arguments
///
/// * `message` - The text message to display in the toast
/// * `toast_type` - The type of toast which determines styling (Success, Error, Warning, Info)
/// * `duration_ms` - How long the toast should remain visible in milliseconds
///
/// # Examples
///
/// ```rust
/// // Show different types of toast notifications
/// show_toast("Operation completed successfully!", ToastType::Success, 3000);
/// show_toast("An error occurred!", ToastType::Error, 5000);
/// show_toast("Warning: Please check your input", ToastType::Warning, 4000);
/// show_toast("Here's some helpful information", ToastType::Info, 3000);
/// ```
pub fn show_toast(message: &str, toast_type: ToastType, duration_ms: i32) {
    let doc = document();
    let body = doc.body().unwrap();

    // Create unique ID based on timestamp
    let timestamp = js_sys::Date::now() as u64;
    let toast_id = format!("toast_{}", timestamp);

    // Create toast element
    let toast = doc.create_element("div").unwrap();
    toast.set_id(&toast_id);
    toast.set_text_content(Some(message));

    // Calculate position based on existing toasts
    let existing_toasts = doc.query_selector_all("[id^='toast_']").unwrap();
    let vertical_offset = 20 + (existing_toasts.length() * 70); // 70px spacing between toasts

    // Apply styling based on toast type
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

    // Add CSS animation keyframes if not already present
    add_toast_animations();

    // Insert toast into body
    body.append_child(&toast).unwrap();

    // Set up auto-removal
    let toast_id_clone = toast_id.clone();
    let callback = Closure::wrap(Box::new(move || {
        remove_toast(&toast_id_clone);
    }) as Box<dyn FnMut()>);

    window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), duration_ms).unwrap();

    callback.forget();
}

/// Removes a specific toast notification and repositions remaining toasts.
///
/// This function handles the removal of a toast notification by its ID and automatically
/// repositions any remaining toast notifications to fill the gap.
///
/// # Arguments
///
/// * `toast_id` - The unique ID of the toast to remove
fn remove_toast(toast_id: &str) {
    let doc = document();

    if let Some(toast) = doc.get_element_by_id(toast_id) {
        // Add fade-out animation
        let current_style = toast.get_attribute("style").unwrap_or_default();
        let fade_style = format!("{}; animation: fadeOut 0.3s ease-out;", current_style);
        toast.set_attribute("style", &fade_style).unwrap();

        // Remove after animation completes
        let toast_clone = toast.clone();
        let callback = Closure::wrap(Box::new(move || {
            if let Some(parent) = toast_clone.parent_node() {
                parent.remove_child(&toast_clone).unwrap();
            }
            reposition_toasts();
        }) as Box<dyn FnMut()>);

        window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), 300).unwrap();

        callback.forget();
    }
}

/// Repositions all remaining toast notifications after one is removed.
///
/// This function queries all existing toast notifications and updates their vertical
/// positions to maintain proper spacing and eliminate gaps.
fn reposition_toasts() {
    let doc = document();
    let toasts = doc.query_selector_all("[id^='toast_']").unwrap();

    for i in 0..toasts.length() {
        if let Some(toast) = toasts.item(i) {
            let element = toast.dyn_into::<HtmlElement>().unwrap();
            let new_top = 20 + (i * 70);

            let current_style = element.get_attribute("style").unwrap_or_default();
            let updated_style = update_style_property(&current_style, "top", &format!("{}px", new_top));
            element.set_attribute("style", &updated_style).unwrap();
        }
    }
}

/// Updates a specific CSS property in a style string.
///
/// # Arguments
///
/// * `style` - The current style string
/// * `property` - The CSS property to update
/// * `value` - The new value for the property
///
/// # Returns
///
/// Updated style string with the new property value
fn update_style_property(style: &str, property: &str, value: &str) -> String {
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

/// Adds CSS animations for toast notifications if they don't already exist.
///
/// This function injects CSS keyframe animations into the document head for smooth
/// toast appearance and disappearance effects.
fn add_toast_animations() {
    let doc = document();

    // Check if animations are already added
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
