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

        <textarea class="my_sdp" readonly></textarea>

        <p>Enter your guest's response code:</p>

        <textarea class="peer_sdp" placeholder="Enter here"></textarea>
        <button class="connect">Connect</button>
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
pub fn render_host() {
    dom::document().body().unwrap().set_inner_html(HOST_HTML);

    let peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>> = Rc::new(RefCell::new(None));
    let peer_connection_clone = peer_connection.clone();

    let callbacks = p2p::PeerConnectionCallbacks {
        on_sdp_ready: Box::new(|json| set_my_sdp_str(&json)),
        on_connection_status_change: Box::new(|state_str| view::update_connection_notification(&state_str)),
        on_connection_established: Box::new(|| view::update_notification("Connection established successfully!")),
        on_message_received: Box::new(|msg| {
            // append_log(&format!("Peer: {}", msg));
        }),
    };
    wasm_bindgen_futures::spawn_local(async move {
        let pc = p2p::create_host_peer_connection(callbacks);
        pc.create_offer().await.unwrap();
        view::update_notification("Created invite code! Share it with your Guest.");
        *peer_connection_clone.borrow_mut() = Some(pc);
    });
}

const GUEST_HTML: &str = r#"
    <div>
        <h2>Guest</h2>

        <p>Enter your host's invite code:</p>

        <textarea class="peer_sdp" placeholder="Enter here"></textarea>
        <button class="connect">Connect</button>
        
        <p>Send this response code to your host:</p>

        <textarea class="my_sdp" readonly></textarea>
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
pub fn render_guest() {
    dom::document().body().unwrap().set_inner_html(GUEST_HTML);
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
pub fn render_chat() {
    dom::document().body().unwrap().set_inner_html(CHAT_HTML);
}

fn set_my_sdp_str(id: &str) {
    let elements = dom::document().get_elements_by_class_name("my_sdp").dyn_into::<HtmlCollection>().unwrap();
    for i in 0..elements.length() {
        let my_sdp_str = elements.item(i).unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();

        let compressed = utils::compress_string(id);
        my_sdp_str.set_value(&compressed);
    }
}

fn clear_my_sdp_str() {
    let elems = dom::document().get_elements_by_class_name("my_sdp").dyn_into::<HtmlCollection>().unwrap();
    for i in 0..elems.length() {
        let my_sdp_str = elems.item(i).unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
        my_sdp_str.set_value("");
    }
}

fn get_peer_sdp_str() -> String {
    let elems = dom::document().get_elements_by_class_name("peer_sdp").dyn_into::<HtmlCollection>().unwrap();
    let ids = (0..elems.length()).map(|i| elems.item(i).unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value()).collect::<Vec<String>>();
    let largest = ids.iter().max_by_key(|id| id.len()).unwrap();
    return utils::decompress_string(&largest);
}

fn get_message() -> String {
    let message = dom::document().get_element_by_id("message").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
    return message.value();
}
fn clear_message() {
    let message = dom::document().get_element_by_id("message").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
    message.set_value("");
}

fn append_log(message: &str) {
    let elem = dom::document().get_element_by_id("logbox").unwrap();
    let div = dom::document().create_element("div").unwrap();
    div.set_text_content(Some(message));
    elem.append_child(&div).unwrap();
    let elem = elem.dyn_into::<HtmlElement>().unwrap();
    elem.set_scroll_top(elem.scroll_height());
}

fn enable_section(section: &str) {
    let section = dom::document().get_element_by_id(section).unwrap();
    section.set_attribute("style", "").unwrap();
}
fn disable_section(section: &str) {
    let section = dom::document().get_element_by_id(section).unwrap();
    section.set_attribute("style", "display: none;").unwrap();
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // map panics to console.error

    view::init();
    view::render_role_selection(|| render_host(), || render_guest());

    // //
    // // host view
    // //
    // let peer_connection: Rc<RefCell<Option<p2p::PeerConnection>>> = Rc::new(RefCell::new(None));

    // {
    //     let btn: Element = dom::document().get_element_by_id("host_selection").unwrap();

    //     let con = peer_connection.clone();
    //     dom::onclick(&btn, move || {
    //         let con_clone = con.clone();

    //         wasm_bindgen_futures::spawn_local(async move {
    //             disable_section("decision");
    //             enable_section("host");

    //             let callbacks = p2p::PeerConnectionCallbacks {
    //                 on_sdp_ready: Box::new(|json| set_my_sdp_str(&json)),
    //                 on_connection_status_change: Box::new(|state_str| view::update_connection_notification(&state_str)),
    //                 on_connection_established: Box::new(|| {
    //                     disable_section("host");
    //                     disable_section("guest");
    //                     enable_section("log");
    //                     view::update_notification("Connection established successfully!");
    //                 }),
    //                 on_message_received: Box::new(|msg| {
    //                     append_log(&format!("Peer: {}", msg));
    //                 }),
    //             };

    //             let peer_conn = p2p::create_host_peer_connection(callbacks);
    //             peer_conn.create_offer().await.unwrap();
    //             *con_clone.borrow_mut() = Some(peer_conn);

    //             view::update_notification("Created invite code! Share it with your Guest.");
    //         });
    //     });
    // }

    // // logic for both
    // {
    //     let btns = dom::document().get_elements_by_class_name("connect");
    //     for i in 0..btns.length() {
    //         let btn: Element = btns.item(i).unwrap();

    //         let con = peer_connection.clone();
    //         dom::onclick(&btn, move || {
    //             let con_clone = con.clone();

    //             wasm_bindgen_futures::spawn_local(async move {
    //                 let sdp_str = get_peer_sdp_str();
    //                 let sdp = js_sys::JSON::parse(&sdp_str).unwrap();
    //                 let sdp_type = js_sys::Reflect::get(&sdp, &"type".into()).unwrap().as_string().unwrap();

    //                 if sdp_type == "offer" {
    //                     // guest: receive offer, create answer
    //                     let callbacks = p2p::PeerConnectionCallbacks {
    //                         on_sdp_ready: Box::new(|json| set_my_sdp_str(&json)),
    //                         on_connection_status_change: Box::new(|state| view::update_connection_notification(&state)),
    //                         on_connection_established: Box::new(|| {
    //                             disable_section("host");
    //                             disable_section("guest");
    //                             enable_section("log");
    //                             view::update_notification("Connection established successfully!");
    //                         }),
    //                         on_message_received: Box::new(|msg| {
    //                             append_log(&format!("Peer: {}", msg));
    //                         }),
    //                     };

    //                     let peer_conn = p2p::create_guest_peer_connection(&sdp_str, callbacks).await.unwrap();
    //                     *con_clone.borrow_mut() = Some(peer_conn);
    //                     view::update_notification("Created response code! Share it with your host.");
    //                 } else if sdp_type == "answer" {
    //                     // host: receive answer, establish connection
    //                     let con_ref = con_clone.borrow();
    //                     if let Some(con) = con_ref.as_ref() {
    //                         con.set_remote_description(&sdp_str).await.unwrap();
    //                         view::update_notification("Attempting to establish connection...");
    //                     }
    //                 }
    //             });
    //         });
    //     }
    // }

    // //
    // // CHAT VIEW
    // //
    // {
    //     let text_area = dom::document().get_element_by_id("message").unwrap();
    //     let con_clone = peer_connection.clone();
    //     dom::onkeypress(&text_area, move |event: KeyboardEvent| {
    //         if event.key() == "Enter" {
    //             if let Some(con) = &*con_clone.borrow() {
    //                 let msg = get_message();
    //                 let success = con.send_message(&msg);
    //                 if success {
    //                     append_log(&format!("You: {}", msg));
    //                     clear_message();
    //                 }
    //             }
    //         }
    //     });

    //     let btn = dom::document().get_element_by_id("send").unwrap();
    //     dom::onclick(&btn, move || {
    //         if let Some(con) = &*peer_connection.borrow() {
    //             let msg = get_message();
    //             let success = con.send_message(&msg);
    //             if success {
    //                 append_log(&format!("You: {}", msg));
    //                 clear_message();
    //             }
    //         }
    //     });
    // }

    Ok(())
}
