#[macro_use]
mod dom;
mod p2p;
mod utils;
mod view;

use std::{cell::RefCell, rc::Rc};

use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::*;

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
            margin-top: 6rem;
        }
        textarea {
            width: 100%;
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
    </style>
"#;
pub fn render_host(on_connection_established: Box<dyn FnMut(Rc<RefCell<Option<p2p::PeerConnection>>>)>, on_message: Box<dyn FnMut(String)>) {
    dom::document().body().unwrap().set_inner_html(HOST_HTML);

    let peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>> = Rc::new(RefCell::new(None));
    let peer_connection_clone = peer_connection.clone();

    // stage 1: create offer
    let peer_connection_for_callback = peer_connection.clone();
    let mut on_connection_established = on_connection_established;
    let callbacks = p2p::PeerConnectionCallbacks {
        on_sdp_ready: Box::new(|json| {
            let elem = dom::document().get_element_by_id("my_sdp").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
            let sdp_str = utils::compress_string(&json);
            elem.set_inner_html(&sdp_str);
        }),
        on_connection_status_change: Box::new(|state_str| view::update_connection_notification(&state_str)),
        on_connection_established: Box::new(move || {
            on_connection_established(peer_connection_for_callback.clone());
        }),
        on_message_received: on_message,
    };
    let pc = p2p::create_host_peer_connection(callbacks);
    wasm_bindgen_futures::spawn_local(async move {
        pc.create_offer().await.unwrap();
        view::update_notification("Created invite code! Share it with your Guest.");
        *peer_connection_clone.borrow_mut() = Some(pc);
    });

    // stage 3: receive answer
    let btn = dom::document().get_element_by_id("connect").unwrap();
    dom::onclick(&btn, move || {
        let elem = dom::document().get_element_by_id("peer_sdp").unwrap();
        let content = elem.dyn_into::<HtmlTextAreaElement>().unwrap().value();
        let sdp_str = utils::decompress_string(&content);

        let con = peer_connection.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(con) = con.borrow().as_ref() {
                con.set_remote_description(&sdp_str).await.unwrap();
                view::update_notification("Attempting to establish connection...");
            }
        });
    });
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
            margin-top: 6rem;
        }
        textarea {
            width: 100%;
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
    </style>
"#;
pub fn render_guest(on_connection_established: Box<dyn FnMut(Rc<RefCell<Option<p2p::PeerConnection>>>)>, on_message: Box<dyn FnMut(String)>) {
    dom::document().body().unwrap().set_inner_html(GUEST_HTML);

    // stage 2: create answer
    let peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>> = Rc::new(RefCell::new(None));
    let peer_connection_clone = peer_connection.clone();

    let on_connection_established = Rc::new(RefCell::new(on_connection_established));
    let on_message = Rc::new(RefCell::new(on_message));

    let btn = dom::document().get_element_by_id("connect").unwrap();
    dom::onclick(&btn, move || {
        let elem = dom::document().get_element_by_id("peer_sdp").unwrap();
        let content = elem.dyn_into::<HtmlTextAreaElement>().unwrap().value();
        let sdp_str = utils::decompress_string(&content);

        let peer_connection_clone = peer_connection_clone.clone();
        let peer_connection_for_callback = peer_connection.clone();
        let on_connection_established = on_connection_established.clone();
        let on_message = on_message.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let callbacks = p2p::PeerConnectionCallbacks {
                on_sdp_ready: Box::new(|json| {
                    let elem = dom::document().get_element_by_id("my_sdp").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
                    let sdp_str = utils::compress_string(&json);
                    elem.set_inner_html(&sdp_str);
                }),
                on_connection_status_change: Box::new(|state| view::update_connection_notification(&state)),
                on_connection_established: Box::new(move || {
                    on_connection_established.borrow_mut()(peer_connection_for_callback.clone());
                }),
                on_message_received: Box::new(move |msg| {
                    on_message.borrow_mut()(msg);
                }),
            };
            let peer_conn = p2p::create_guest_peer_connection(&sdp_str, callbacks).await.unwrap();
            *peer_connection_clone.borrow_mut() = Some(peer_conn);
            view::update_notification("Created response code! Share it with your host.");
        });
    });
}

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
    dom::document().body().unwrap().set_inner_html(CHAT_HTML);

    fn get_message() -> String {
        dom::document().get_element_by_id("message").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value()
    }

    fn clear_message() {
        dom::document().get_element_by_id("message").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().set_value("")
    }

    let text_area = dom::document().get_element_by_id("message").unwrap();
    let con_clone = peer_connection.clone();
    dom::onkeypress(&text_area, move |event: KeyboardEvent| {
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

    let btn = dom::document().get_element_by_id("send").unwrap();
    dom::onclick(&btn, move || {
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
    let elem = dom::document().get_element_by_id("logbox").unwrap();
    let div = dom::document().create_element("div").unwrap();
    div.set_text_content(Some(message));
    elem.append_child(&div).unwrap();
    let elem = elem.dyn_into::<HtmlElement>().unwrap();
    elem.set_scroll_top(elem.scroll_height());
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // map panics to console.error

    // TODO: the two arguments passed to render_host and render_guest should be a struct called `P2PConnectionCallbacks` and take `on_connection_established` and `on_message`
    // TODO: there are lots of confusing and unnecessary clones, boxes etc. everywhere

    view::init();
    view::render_role_selection(
        || {
            render_host(
                Box::new(|peer_connection| {
                    view::update_notification("Connection established successfully!");
                    render_chat(peer_connection);
                }),
                Box::new(|msg| {
                    append_log(&format!("Peer: {}", msg));
                }),
            );
        },
        || {
            render_guest(
                Box::new(|peer_connection| {
                    view::update_notification("Connection established successfully!");
                    render_chat(peer_connection);
                }),
                Box::new(|msg| {
                    append_log(&format!("Peer: {}", msg));
                }),
            );
        },
    );

    Ok(())
}
