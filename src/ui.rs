use crate::dom;

/// Independently created, inserted and styled floating DOM node.
/// Shows connection status notifications.
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

/// Independently created, inserted and styled floating DOM node.
/// Shows general user notifications.
pub fn show_notification(message: &str) {
    let doc = dom::document();
    let body = doc.body().unwrap();

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
             color: #333;",
        )
        .unwrap();
        body.insert_before(&div, body.first_child().as_ref()).unwrap();
        div
    };

    // update content
    div.set_text_content(Some(message));
}
