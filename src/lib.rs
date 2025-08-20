mod p2p;
mod utils;
mod view;

use std::{cell::RefCell, rc::Rc};

use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::*;

const CHAT_HTML: &str = r#"
    <div>
        <h2>Chat</h2>

        <div id="logbox"></div>

        <div id="wrapper">
            <textarea id="message" placeholder="Enter message"></textarea>
            <button id="send">Send</button> 
        </div>
    </div>
    <style>
        p {
            margin-top: 2rem;
            margin-bottom: 2rem;
        }
        h2 {
            margin-top: 6rem;
        }
        textarea {
            width: 80%;
            height: 4rem;
        }
        button {
            cursor: pointer;
            font-family: 'Lucida Console', monospace;
            padding: 0.5rem;

            background-color: #f0f0f0;
            border: 1px solid;
            width: 20%;
        }
        #logbox {
            margin-top: 2rem;
            margin-bottom: 1rem;

            height: 400px;
            overflow-y: auto;
            border: 1px solid;
        }
        #wrapper {
            display: flex;
            justify-content: center;
        }
    </style>
"#;
pub fn render_chat(peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>>) {
    utils::document().body().unwrap().set_inner_html(CHAT_HTML);

    let get_message = || utils::document().get_element_by_id("message").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value();
    let clear_message = || utils::document().get_element_by_id("message").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().set_value("");

    let text_area = utils::document().get_element_by_id("message").unwrap();
    let con_clone = peer_connection.clone();
    utils::onkeypress(&text_area, move |event: KeyboardEvent| {
        if event.key() == "Enter" {
            if let Some(con) = &*con_clone.borrow() {
                let msg = get_message();
                let success = con.send_message(&msg);
                if success {
                    append_log(&format!("You: {}", msg));
                    clear_message();
                }
            }
        }
    });

    let btn = utils::document().get_element_by_id("send").unwrap();
    utils::onclick(&btn, move || {
        if let Some(con) = &*peer_connection.borrow() {
            let msg = get_message();
            let success = con.send_message(&msg);
            if success {
                append_log(&format!("You: {}", msg));
                clear_message();
            }
        }
    });
}

pub fn append_log(message: &str) {
    let elem = utils::document().get_element_by_id("logbox").unwrap();
    let div = utils::document().create_element("div").unwrap();
    div.set_text_content(Some(message));
    elem.append_child(&div).unwrap();
    let elem = elem.dyn_into::<HtmlElement>().unwrap();
    elem.set_scroll_top(elem.scroll_height());
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    view::init();

    let callbacks = view::ActorCallbacks {
        on_connection_established: |peer_connection| render_chat(peer_connection),
        on_message: |msg| {
            append_log(&format!("Peer: {}", msg));
        },
    };
    view::render_role_selection(move || view::render_host(callbacks), move || view::render_guest(callbacks));

    Ok(())
}
