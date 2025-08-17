#[macro_use]
mod dom;
mod p2p;
mod ui;
mod utils;

use js_sys::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use web_sys::*;

const STYLING: &str = r#"
    * { margin: 0; padding: 0; }
    *::-webkit-scrollbar { display: none !important; }
    body {
        max-width: 800px; margin: 0 auto; padding: 0 1rem;
        font-family: 'Lucida Console', monospace;
    }

    #decision div {
        display: flex;
        justify-content: center;
    }
    #decision button {
        margin: 1rem 1rem;
        width: 30%;
    }

    h2 {
        margin-top: 6rem;
    }
    p {
        margin-top: 2rem;
        margin-bottom: 2rem;
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

    #log #wrapper {
        display: flex;
        justify-content: center;
    }
    #log textarea {
        width: 80%;
    }
    #log button {
        width: 20%;
    }
    #logbox {
        margin-top: 2rem;
        margin-bottom: 1rem;

        height: 400px;
        overflow-y: auto;
        border: 1px solid;
    }
"#;

const HTML: &str = r#"
    <section id="decision">
        <h2>Choose your role</h2>

        <p>This application establishes a peer-to-peer connection between two users. You can choose to be the host or the guest.</p>

        <div>
            <button id="host_selection">Host</button>
            <button id="guest_selection">Guest</button>
        </div>
    </section>

    <section id="host">
        <h2>Host</h2>

        <p>Send this invite code to your guest:</p>

        <textarea class="my_sdp" readonly></textarea>

        <p>Enter your guest's response code:</p>

        <textarea class="peer_sdp" placeholder="Enter here"></textarea>
        <button class="connect">Connect</button>
    </section>

    <section id="guest">
        <h2>Guest</h2>

        <p>Enter your host's invite code:</p>

        <textarea class="peer_sdp" placeholder="Enter here"></textarea>
        <button class="connect">Connect</button>
        
        <p>Send this response code to your host:</p>

        <textarea class="my_sdp" readonly></textarea>
    </section>

    <section id="log">
        <h2>Chat</h2>

        <div id="logbox"></div>

        <div id="wrapper">
            <textarea id="message" placeholder="Enter message"></textarea>
            <button id="send">Send</button> 
        </div>
    </section>
"#;

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

    // init dom
    let head = dom::document().head().unwrap();
    let body = dom::document().body().unwrap();
    let style = dom::document().create_element("style").unwrap();
    style.set_text_content(Some(STYLING));
    head.append_child(&style).unwrap();
    body.set_inner_html(HTML);

    ui::show_connection_notification("🔴 Disconnected");
    ui::show_notification("");

    vec!["host", "guest", "log"].iter().for_each(|&section| disable_section(section));

    // USER CHOSE GUEST MODE
    {
        // hide stuff, wait for offer from host
        let btn = dom::document().get_element_by_id("guest_selection").unwrap();
        dom::onclick(&btn, move || {
            disable_section("decision");
            enable_section("guest");
            clear_my_sdp_str();
        });
    }

    // USER CHOSE HOST MODE
    let peer_connection: Rc<RefCell<Option<RtcPeerConnection>>> = Rc::new(RefCell::new(None));
    let data_channel: Rc<RefCell<Option<RtcDataChannel>>> = Rc::new(RefCell::new(None));

    {
        let btn: Element = dom::document().get_element_by_id("host_selection").unwrap();

        let pc = peer_connection.clone();
        let dc = data_channel.clone();
        dom::onclick(&btn, move || {
            let pc_clone = pc.clone();
            let dc_clone = dc.clone();

            wasm_bindgen_futures::spawn_local(async move {
                disable_section("decision");
                enable_section("host");

                let on_connection_established = || {
                    disable_section("host");
                    disable_section("guest");
                    enable_section("log");
                    ui::show_notification("Connection established successfully!");
                };
                let on_message_received = |msg| {
                    append_log(&format!("Peer: {}", msg));
                };
                let (pc, dc) = p2p::create_host_peer_connection(
                    |json| set_my_sdp_str(&json),                             // on_offer_generation
                    |state_str| ui::show_connection_notification(&state_str), // on_connection_status_change
                    on_connection_established,                                // on_connection_established
                    on_message_received,                                      // on_message_received
                );
                *pc_clone.borrow_mut() = Some(pc.clone());
                *dc_clone.borrow_mut() = Some(dc.clone());

                // create offer
                let offer = JsFuture::from(pc.create_offer()).await.unwrap();
                JsFuture::from(pc.set_local_description(&offer.into())).await.unwrap();
                ui::show_notification("Created invite code! Share it with your Guest.");
            });
        });
    }

    // CONNECT BUTTON PRESSED IN EITHER OF THOSE ROLES
    {
        let btns = dom::document().get_elements_by_class_name("connect");
        for i in 0..btns.length() {
            let btn: Element = btns.item(i).unwrap();

            let pc = peer_connection.clone();
            let dc = data_channel.clone();
            dom::onclick(&btn, move || {
                let pc_clone = pc.clone();
                let dc_clone = dc.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let sdp_str = get_peer_sdp_str();
                    let sdp = js_sys::JSON::parse(&sdp_str).unwrap();
                    let sdp_type = js_sys::Reflect::get(&sdp, &"type".into()).unwrap().as_string().unwrap();

                    if sdp_type == "offer" {
                        // guest: receive offer, create answer
                        let dc_clone_inner = dc_clone.clone();

                        let on_connection_established = || {
                            disable_section("host");
                            disable_section("guest");
                            enable_section("log");
                            ui::show_notification("Connection established successfully!");
                        };
                        let on_message_received = |msg| {
                            append_log(&format!("Peer: {}", msg));
                        };
                        let pc = p2p::create_guest_peer_connection(
                            &sdp_str,                                         // offer
                            |json| set_my_sdp_str(&json),                     // on_answer_generation
                            |state| ui::show_connection_notification(&state), // on_connection_state_change
                            on_connection_established,                        // on_connection_established
                            on_message_received,                              // on_message_received
                            move |dc| {
                                // on_datachannel_created (already configured)
                                *dc_clone_inner.borrow_mut() = Some(dc);
                            },
                        )
                        .await;
                        *pc_clone.borrow_mut() = Some(pc);
                        ui::show_notification("Created response code! Share it with your host.");
                    } else if sdp_type == "answer" {
                        // host: receive answer, establish connection
                        let pc_ref = pc_clone.borrow();
                        if let Some(pc) = pc_ref.as_ref() {
                            let sdp = js_sys::JSON::parse(&sdp_str).unwrap(); // answer
                            let promise = pc.set_remote_description(&sdp.into());
                            JsFuture::from(promise).await.unwrap();
                            ui::show_notification("Attempting to establish connection...");
                        }
                    }
                });
            });
        }
    }

    // CONNECTION SUCCESSFUL, SHOWING CHAT UI
    {
        let text_area = dom::document().get_element_by_id("message").unwrap();
        let dc_clone = data_channel.clone();
        dom::onkeypress(&text_area, move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                if let Some(dc) = &*dc_clone.borrow() {
                    let msg = get_message();
                    let success = p2p::send_message(&dc, &msg);
                    if success {
                        append_log(&format!("You: {}", msg));
                        clear_message();
                    }
                }
            }
        });

        let btn = dom::document().get_element_by_id("send").unwrap();

        let dc = data_channel;
        dom::onclick(&btn, move || {
            if let Some(dc) = &*dc.borrow() {
                let msg = get_message();
                let success = p2p::send_message(&dc, &msg);
                if success {
                    append_log(&format!("You: {}", msg));
                    clear_message();
                }
            }
        });
    }

    Ok(())
}
