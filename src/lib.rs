mod utils;

use wasm_bindgen::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use js_sys::*;
use web_sys::*;
use std::rc::Rc;
use std::cell::RefCell;

// 
// rendering
// 

const STYLING: &str = r#"
    * { margin: 0; padding: 0; }

    *::-webkit-scrollbar { display: none !important; } /* hide scrollbar */
    
    body {
        max-width: 800px; margin: 0 auto; padding: 0 1rem;
        font-family: 'Lucida Console', monospace;
    }

    section { margin-bottom: 2rem; margin-top: 1rem; }

    button {
        cursor: pointer;
        font-family: 'Lucida Console', monospace;
        padding: 0.5rem;
        border: 1px solid;
    }

    textarea { width: 100%; height: 3rem; }
    #chat_box { height: 200px; overflow-y: auto; border: 1px solid; }
"#;

const HTML: &str = r#"
    <section>
        <span>Status: <span id="status">Ready</span></span>
    </section>

    <section id="decision">
        <h2>Decision</h2>

        <div>
            <button id="host_selection">Host</button>
            <button id="guest_selection">Guest</button>
        </div>
    </section>

    <section id="host">
        <h2>Host</h2>

        <div>Send this invite code to your guest:</div>

        <textarea class="my_id" readonly></textarea>

        <div>Enter your guest's response code:</div>

        <textarea class="peer_id" placeholder="Enter here"></textarea>
        <button class="connect">Connect</button>
    </section>

    <section id="guest">
        <h2>Guest</h2>

        <div>Enter your host's invite code:</div>

        <textarea class="peer_id" placeholder="Enter here"></textarea>
        <button class="connect">Connect</button>
        
        <div>Send this response code to host:</div>

        <textarea class="my_id" readonly></textarea>
    </section>

    <section id="chat">
        <h2>Chat</h2>

        <div id="chat_box"></div>
        <button id="ping">Send Ping</button> 
    </section>
"#;

fn init_ui() {
    let document = web_sys::window().unwrap().document().unwrap();
    let head = document.head().unwrap();
    let body = document.body().unwrap();

    let style = document.create_element("style").unwrap();
    style.set_text_content(Some(STYLING));
    head.append_child(&style).unwrap();

    body.set_inner_html(HTML);
}

fn set_my_id(id: &str) {
    let document = window().unwrap().document().unwrap();
    let elements = document.get_elements_by_class_name("my_id").dyn_into::<HtmlCollection>().unwrap();
    for i in 0..elements.length() {
        let host_id = elements.item(i).unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();

        let compressed = utils::compress_string(id);
        host_id.set_value(&compressed);
    }
}

fn clear_my_id() {
    let document = window().unwrap().document().unwrap();
    let elems = document.get_elements_by_class_name("my_id").dyn_into::<HtmlCollection>().unwrap();
    for i in 0..elems.length() {
        let host_id = elems.item(i).unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
        host_id.set_value("");
    }
}

fn get_peer_id() -> String {
    let document = window().unwrap().document().unwrap();
    let elems = document.get_elements_by_class_name("peer_id").dyn_into::<HtmlCollection>().unwrap();
    let ids = (0..elems.length()).map(|i| {
        elems.item(i).unwrap().dyn_into::<HtmlTextAreaElement>().unwrap().value()
    }).collect::<Vec<String>>();
    let largest = ids.iter().max_by_key(|id| id.len()).unwrap().to_string();
    return utils::decompress_string(&largest);
}

fn set_status(message: &str) {
    let document = window().unwrap().document().unwrap();
    let status = document.get_element_by_id("status").unwrap();

    status.set_text_content(Some(message));
}

fn append_chat_box(message: &str) {
    let document = window().unwrap().document().unwrap();
    let chatbox = document.get_element_by_id("chat_box").unwrap();

    let div = document.create_element("div").unwrap();
    div.set_text_content(Some(message));
    chatbox.append_child(&div).unwrap();
    let chatbox: HtmlElement = chatbox.dyn_into().unwrap();
    chatbox.set_scroll_top(chatbox.scroll_height());
}

fn enable_section(section: &str) {
    let document = window().unwrap().document().unwrap();
    let section = document.get_element_by_id(section).unwrap();
    section.set_attribute("style", "").unwrap();
}

fn disable_section(section: &str) {
    let document = window().unwrap().document().unwrap();
    let section = document.get_element_by_id(section).unwrap();
    section.set_attribute("style", "display: none;").unwrap();
}

// 
// logic
// 

fn setup_data_channel(dc: &RtcDataChannel) {
    let onopen_callback = Closure::wrap(Box::new(move || {
        disable_section("host");
        disable_section("guest");
        enable_section("chat");
    }) as Box<dyn FnMut()>);
    dc.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    
    let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        if let Some(data) = event.data().as_string() {
            append_chat_box(&format!("Peer: {}", data));
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    dc.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();
}

fn create_peer_connection() -> RtcPeerConnection {
    let ice_server: RtcIceServer = RtcIceServer::new();
    ice_server.set_urls(&js_sys::Array::of1(&JsValue::from_str("stun:stun.l.google.com:19302")));
    let configuration = RtcConfiguration::new();
    configuration.set_ice_servers(&js_sys::Array::of1(&ice_server));
    let pc = RtcPeerConnection::new_with_configuration(&configuration).unwrap();

    let pc_clone = pc.clone();
    let onicecandidate_callback = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
        if event.candidate().is_none() {
            if let Some(desc) = pc_clone.local_description() {
                let json_str = js_sys::JSON::stringify(&desc).unwrap().as_string().unwrap();
                set_my_id(&json_str);
            }
        }
    }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);
    pc.set_onicecandidate(Some(onicecandidate_callback.as_ref().unchecked_ref()));
    onicecandidate_callback.forget();
    
    let pc_clone = pc.clone();
    let onconnectionstatechange_callback = Closure::wrap(Box::new(move || {
        let state_str = match pc_clone.connection_state() {
            RtcPeerConnectionState::New => "new",
            RtcPeerConnectionState::Connecting => "Connecting",
            RtcPeerConnectionState::Connected => "Connected",
            RtcPeerConnectionState::Disconnected => "Disconnected",
            RtcPeerConnectionState::Failed => "Failed",
            RtcPeerConnectionState::Closed => "Closed",
            _ => "Unknown error",
        };
        set_status(&format!("{}", state_str));
    }) as Box<dyn FnMut()>);
    pc.set_onconnectionstatechange(Some(onconnectionstatechange_callback.as_ref().unchecked_ref()));
    onconnectionstatechange_callback.forget();

    return pc;
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // panics to console.error
    
    init_ui();

    let sections = vec!["host", "guest", "chat"];
    for section in sections {
        disable_section(section);
    }

    {
        let guest_selector = web_sys::window().unwrap().document().unwrap().get_element_by_id("guest_selection").unwrap().dyn_into::<HtmlButtonElement>().unwrap();
        let guest_selector_callback = Closure::wrap(Box::new(move || {
            disable_section("decision");
            enable_section("guest");
            clear_my_id();
        }) as Box<dyn FnMut()>);
        guest_selector.set_onclick(Some(guest_selector_callback.as_ref().unchecked_ref()));
        guest_selector_callback.forget();
    }

    let peer_connection: Rc<RefCell<Option<RtcPeerConnection>>> = Rc::new(RefCell::new(None));
    let data_channel: Rc<RefCell<Option<RtcDataChannel>>> = Rc::new(RefCell::new(None));

    {
        let btn = web_sys::window().unwrap().document().unwrap().get_element_by_id("host_selection").unwrap().dyn_into::<HtmlButtonElement>().unwrap();

        let pc = peer_connection.clone();
        let dc = data_channel.clone();
        let btn_callback = Closure::wrap(Box::new(move || {
            let pc_clone = pc.clone();
            let dc_clone = dc.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                disable_section("decision");
                enable_section("host");

                let pc = create_peer_connection();
                let dc = pc.create_data_channel("chat");
                
                setup_data_channel(&dc);
                *pc_clone.borrow_mut() = Some(pc.clone());
                *dc_clone.borrow_mut() = Some(dc.clone());

                let offer = JsFuture::from(pc.create_offer()).await.unwrap();
                JsFuture::from(pc.set_local_description(&offer.into())).await.unwrap();
                set_status("Offer created! Share your ID.");
            });
        }) as Box<dyn FnMut()>);
        btn.set_onclick(Some(btn_callback.as_ref().unchecked_ref()));
        btn_callback.forget();
    }

    {
        let btns = web_sys::window().unwrap().document().unwrap().get_elements_by_class_name("connect").dyn_into::<HtmlCollection>().unwrap();
        for i in 0..btns.length() {
            let btn = btns.item(i).unwrap().dyn_into::<HtmlButtonElement>().unwrap();

            let pc = peer_connection.clone();
            let dc = data_channel.clone();
            let btn_callback = Closure::wrap(Box::new(move || {
                let pc_clone = pc.clone();
                let dc_clone = dc.clone();
    
                wasm_bindgen_futures::spawn_local(async move {
                    let sdp = js_sys::JSON::parse(&get_peer_id()).unwrap();
                    let sdp_type = Reflect::get(&sdp, &"type".into()).unwrap().as_string().unwrap();
    
                    if sdp_type == "offer" {
                        let pc = create_peer_connection();
                        let dc_inner = dc_clone.clone();
                        
                        let ondatachannel = Closure::wrap(Box::new(move |e: RtcDataChannelEvent| {
                            let dc = e.channel();
                            setup_data_channel(&dc);
                            *dc_inner.borrow_mut() = Some(dc);
                        }) as Box<dyn FnMut(RtcDataChannelEvent)>);
                        pc.set_ondatachannel(Some(ondatachannel.as_ref().unchecked_ref()));
                        ondatachannel.forget();
    
                        JsFuture::from(pc.set_remote_description(&sdp.into())).await.unwrap();
                        JsFuture::from(pc.set_local_description(&JsFuture::from(pc.create_answer()).await.unwrap().into())).await.unwrap();
                        
                        *pc_clone.borrow_mut() = Some(pc);
                        set_status("Answered offer! Share your ID back.");
                    
                    } else if sdp_type == "answer" {
                        let promise = pc_clone.borrow().as_ref().unwrap().set_remote_description(&sdp.into());
                        JsFuture::from(promise).await.unwrap();
                        set_status("Connecting...");
                    }
                });
            }) as Box<dyn FnMut()>);
            btn.set_onclick(Some(btn_callback.as_ref().unchecked_ref()));
            btn_callback.forget();
        }
    }

    {
        let btn = web_sys::window().unwrap().document().unwrap().get_element_by_id("ping").unwrap().dyn_into::<HtmlButtonElement>().unwrap();
        
        let dc = data_channel.clone();
        let btn_callback = Closure::wrap(Box::new(move || {
            if let Some(dc) = &*dc.borrow() {
                if dc.ready_state() == RtcDataChannelState::Open {
                    dc.send_with_str("ping").unwrap();
                    append_chat_box("You: ping");
                }
            }
        }) as Box<dyn FnMut()>);
        btn.set_onclick(Some(btn_callback.as_ref().unchecked_ref()));
        btn_callback.forget();
    }

    Ok(())
}
