//! Views work similar to a SPA where you dynamically re-render each page instead of fetching new HTML.
//! They replace the content of the `<body>` element.

use crate::dom;
use crate::p2p;
use crate::utils;

use std::{cell::RefCell, rc::Rc};

use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::*;

const GLOBAL_STYLING: &str = r#"
    * { margin: 0; padding: 0; }
    *::-webkit-scrollbar { display: none !important; }
    body {
        max-width: 800px; margin: 0 auto; padding: 0 1rem;
        font-family: 'Lucida Console', monospace;
    }
"#;
pub fn init() {
    console_error_panic_hook::set_once(); // map panics to console.error

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

/// Not influenced by view changes.
pub fn update_connection_notification(status: &str) {
    let doc = dom::document();

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
                 width: 150px; \
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
    status_element.set_text_content(Some(status));
}

/// Not influenced by view changes.
pub fn update_notification(message: &str) {
    let doc = dom::document();

    let div = if let Some(existing) = doc.get_element_by_id("notification_pill_right") {
        existing
    } else {
        let div = doc.create_element("div").unwrap();
        div.set_id("notification_pill_right");
        div.set_attribute(
            "style",
            "position: fixed; \
             top: 20px; \
             left: 200px; \
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
    div.set_text_content(Some(message));
}

//
// connection setup
//

const ROLE_SELECTION_HTML: &str = r#"
    </div>
        <h2>Choose your role</h2>

        <p>Connect directly with another user without any servers! One user chooses "Host" to get an invite code, the other chooses "Guest" to reply with a response code. Just copy and paste the two codes between each other to link up.</p>

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
        margin-top: 7rem;
    }
    button {
        cursor: pointer;
        font-family: 'Lucida Console', monospace;
        margin-top: 1rem;
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

#[derive(Clone)]
pub struct ActorCallbacks {
    pub on_connection_established: Rc<dyn Fn(Rc<RefCell<Option<p2p::PeerConnection>>>)>,
    pub on_message: Rc<dyn Fn(String)>,
}

const HOST_HTML: &str = r#"
    <div>
        <h2>Host</h2>

        <p>Send this invite code to your guest:</p>

        <textarea id="my_sdp" readonly></textarea>

        <p>Enter your guest's response code:</p>

        <textarea id="peer_sdp" placeholder="Enter here"></textarea>
        <button id="connect">Connect</button>
    </div>
    <style>
        p {
            margin-top: 2rem;
            margin-bottom: 2rem;
        }
        h2 {
            margin-top: 7rem;
        }
        textarea {
            width: 100%;
            height: 4rem;
        }
        button {
            cursor: pointer;
            font-family: 'Lucida Console', monospace;
            margin-top: 1rem;
            padding: 0.5rem;

            background-color: #f0f0f0;
            border: 1px solid;
            width: 20%;
        }
    </style>
"#;
pub fn render_host(callbacks: ActorCallbacks) {
    dom::document().body().unwrap().set_inner_html(HOST_HTML);

    let peer_connection = Rc::new(RefCell::new(None));

    let update_my_sdp = |json: String| {
        let elem = dom::document().get_element_by_id("my_sdp").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
        let sdp_str = utils::compress(&json).unwrap();
        elem.set_inner_html(&sdp_str);
    };
    let on_established = {
        let pc_ref = peer_connection.clone();
        let on_connection_established = callbacks.on_connection_established.clone();
        move || {
            update_notification("Connection established successfully!");
            (on_connection_established)(pc_ref.clone());
        }
    };
    let p2p_callbacks = p2p::PeerConnectionCallbacks {
        on_sdp_ready: Box::new(update_my_sdp),
        on_connection_status_change: Box::new(|state_str| update_connection_notification(&state_str)),
        on_connection_established: Box::new(on_established),
        on_message_received: Box::new(move |msg| (callbacks.on_message)(msg)),
    };
    let pc = p2p::create_host_peer_connection(p2p_callbacks);
    let pc_ref = peer_connection.clone();
    wasm_bindgen_futures::spawn_local(async move {
        pc.create_offer().await.unwrap();
        *pc_ref.borrow_mut() = Some(pc);
        update_notification("Created invite code! Share it with your Guest.");
    });

    let connect_handler = {
        let pc_ref = peer_connection.clone();
        move || {
            let content = dom::document().get_element_by_id("peer_sdp").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value();
            let sdp_str = utils::decompress(&content).unwrap();
            let pc_ref = pc_ref.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(con) = pc_ref.borrow().as_ref() {
                    con.set_remote_description(&sdp_str).await.unwrap();
                    update_notification("Attempting to establish connection...");
                }
            });
        }
    };
    let connect_btn = dom::document().get_element_by_id("connect").unwrap();
    dom::onclick(&connect_btn, connect_handler);
}

const GUEST_HTML: &str = r#"
    <div>
        <h2>Guest</h2>

        <p>Enter your host's invite code:</p>

        <textarea id="peer_sdp" placeholder="Enter here"></textarea>
        <button id="connect">Connect</button>
        
        <p>Send this response code to your host:</p>

        <textarea id="my_sdp" readonly></textarea>
    </div>
    <style>
        p {
            margin-top: 2rem;
            margin-bottom: 2rem;
        }
        h2 {
            margin-top: 7rem;
        }
        textarea {
            width: 100%;
            height: 4rem;
        }
        button {
            cursor: pointer;
            font-family: 'Lucida Console', monospace;
            margin-top: 1rem;
            padding: 0.5rem;

            background-color: #f0f0f0;
            border: 1px solid;
            width: 20%;
        }
    </style>
"#;
pub fn render_guest(callbacks: ActorCallbacks) {
    dom::document().body().unwrap().set_inner_html(GUEST_HTML);

    let peer_connection = Rc::new(RefCell::new(None));

    let update_my_sdp = |json: String| {
        let elem = dom::document().get_element_by_id("my_sdp").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
        let sdp_str = utils::compress(&json).unwrap();
        elem.set_inner_html(&sdp_str);
    };

    let connect_handler = {
        let pc_ref = peer_connection.clone();
        let on_connection_established = callbacks.on_connection_established.clone();
        let on_message = callbacks.on_message.clone();
        move || {
            let content = dom::document().get_element_by_id("peer_sdp").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value();
            let sdp_str = utils::decompress(&content).unwrap();
            let pc_ref = pc_ref.clone();
            let on_connection_established = on_connection_established.clone();
            let on_message = on_message.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let pc_ref_for_callback = pc_ref.clone();
                let on_established = move || {
                    update_notification("Connection established successfully!");
                    (on_connection_established)(pc_ref_for_callback.clone());
                };
                let p2p_callbacks = p2p::PeerConnectionCallbacks {
                    on_sdp_ready: Box::new(update_my_sdp),
                    on_connection_status_change: Box::new(|state| update_connection_notification(&state)),
                    on_connection_established: Box::new(on_established),
                    on_message_received: Box::new(move |msg| (on_message)(msg)),
                };
                *pc_ref.borrow_mut() = Some(p2p::create_guest_peer_connection(&sdp_str, p2p_callbacks).await.unwrap());
                update_notification("Created response code! Share it with your host.");
            });
        }
    };
    let connect_btn = dom::document().get_element_by_id("connect").unwrap();
    dom::onclick(&connect_btn, connect_handler);
}
