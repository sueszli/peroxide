//! Views work similar to a SPA where you dynamically re-render each page instead of fetching new HTML.
//! They work by replacing the entire `body` element with a new one.

use crate::dom;

const GLOBAL_STYLING: &str = r#"
    * { margin: 0; padding: 0; }
    *::-webkit-scrollbar { display: none !important; }
    body {
        max-width: 800px; margin: 0 auto; padding: 0 1rem;
        font-family: 'Lucida Console', monospace;
    }
"#;
pub fn init() {
    // placed in head
    let doc = dom::document();
    let head = doc.head().unwrap();
    let style = doc.create_element("style").unwrap();
    style.set_text_content(Some(GLOBAL_STYLING));
    head.append_child(&style).unwrap();

    update_connection_notification("🔴 Disconnected");
    update_notification("");
}

//
// notifications
//

/// Shows connection status notifications.
/// This element is placed outside of the `body`, so it can't be accidentally removed.
pub fn update_connection_notification(status: &str) {
    let doc = dom::document();

    // create if missing
    let status_element = match doc.get_element_by_id("notification_pill_left") {
        Some(element) => element,
        // inline styling so it doesn't affect any other element
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
pub fn update_notification(message: &str) {
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

//
// connection setup
//

const ROLE_SELECTION_HTML: &str = r#"
    </div>
        <h2>Choose your role</h2>

        <p>This application establishes a peer-to-peer connection between two users. You can choose to be the host or the guest.</p>

        <div>
            <button id="host_selection">Host</button>
            <button id="guest_selection">Guest</button>
        </div>
    </div>
    <style>
    div {
        display: flex;
        justify-content: center;
    }
    p {
        margin-top: 2rem;
        margin-bottom: 2rem;
    }
    h2 {
        margin-top: 6rem;
    }
    button {
        cursor: pointer;
        font-family: 'Lucida Console', monospace;
        padding: 0.5rem;
        margin: 1rem 1rem;
        
        background-color: #f0f0f0;
        border: 1px solid;
        width: 30%;
    }
    </style>
"#;
pub fn render_role_selection(on_host_selection: impl Fn() + 'static, on_guest_selection: impl Fn() + 'static) {
    dom::document().body().unwrap().set_inner_html(ROLE_SELECTION_HTML);

    let host_btn = dom::document().get_element_by_id("host_selection").unwrap();
    dom::onclick(&host_btn, move || on_host_selection());

    let guest_btn = dom::document().get_element_by_id("guest_selection").unwrap();
    dom::onclick(&guest_btn, move || on_guest_selection());
}
