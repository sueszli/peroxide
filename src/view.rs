//! Views work similar to a SPA where you dynamically re-render each page instead of fetching new HTML.
//! They work by replacing the entire `body` element with a new one.

use crate::dom;

/// Placed in the `head` node, so it can't be accidentally removed.
const GLOBAL_STYLING: &str = r#"
    * { margin: 0; padding: 0; }
    *::-webkit-scrollbar { display: none !important; }
    body {
        max-width: 800px; margin: 0 auto; padding: 0 1rem;
        font-family: 'Lucida Console', monospace;
    }
"#;
pub fn init_ui() {
    let doc = dom::document();
    let head = doc.head().unwrap();
    let style = doc.create_element("style").unwrap();
    style.set_text_content(Some(GLOBAL_STYLING));
    head.append_child(&style).unwrap();

    show_connection_notification("🔴 Disconnected");
    show_notification("");
}

/// Shows connection status notifications.
/// This element is placed outside of the `body`, so it can't be accidentally removed.
pub fn show_connection_notification(status: &str) {
    let doc = dom::document();

    // create if missing
    let status_element = match doc.get_element_by_id("notification_pill_left") {
        Some(element) => element,
        None => {
            let div = doc.create_element("div").unwrap();
            div.set_id("notification_pill_left");
            div.set_attribute(
                "style",
                "position: fixed; \
                 top: 20px; \
                 left: 20px; \
                 height: 24px; \
                 width: 130px; \
                 background-color: rgba(255, 255, 255, 0.95); \
                 border: 1.5px solid #333; \
                 border-radius: 20px; \
                 padding: 6px 12px; \
                 font-size: 14px; \
                 font-weight: bold; \
                 z-index: 9999; \
                 box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15); \
                 backdrop-filter: blur(3px); \
                 display: flex; \
                 align-items: center; \
                 text-align: left; \
                 color: #333; \
                 font-family: 'Lucida Console', monospace;",
            )
            .unwrap();
            let document_element = doc.document_element().unwrap();
            document_element.append_child(&div).unwrap();
            div
        }
    };

    // update content
    status_element.set_text_content(Some(status));
}

/// Shows general user notifications.
/// This element is placed outside of the `body`, so it can't be accidentally removed.
pub fn show_notification(message: &str) {
    let doc = dom::document();

    // create if missing
    let div = if let Some(existing) = doc.get_element_by_id("notification_pill_right") {
        existing
    } else {
        let div = doc.create_element("div").unwrap();
        div.set_id("notification_pill_right");
        div.set_attribute(
            "style",
            "position: fixed; \
             top: 20px; \
             left: 180px; \
             right: 20px; \
             height: 24px; \
             background-color: rgba(255, 255, 255, 0.95); \
             border: 1.5px solid #333; \
             border-radius: 20px; \
             padding: 6px 12px 6px 20px; \
             font-size: 14px; \
             font-weight: bold; \
             z-index: 9999; \
             box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15); \
             backdrop-filter: blur(3px); \
             display: flex; \
             align-items: center; \
             justify-content: flex-start; \
             text-align: left; \
             color: #333; \
             font-family: 'Lucida Console', monospace;",
        )
        .unwrap();
        let document_element = doc.document_element().unwrap();
        document_element.append_child(&div).unwrap();
        div
    };

    // update content
    div.set_text_content(Some(message));
}
