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
        box-sizing: border-box;
    }
    button { font-family: 'Lucida Console', monospace; padding: 0.5rem; }

    textarea { width: 100%; height: 3rem; }
    #chatBox { height: 200px; overflow-y: auto; border: 1px solid; }
"#;

const HTML: &str = r#"
    <span>Status: <span id="status">Ready</span></span>

    </br></br></br>

    <h2>Read Code</h2>
    <textarea id="myId" readonly></textarea>
    <button id="offerBtn">Get invite code</button>
    
    </br></br></br>

    <h2>Write Code</h2>
    <textarea id="peerId" placeholder="Enter peer ID here"></textarea>
    <button id="connectBtn">Connect</button>

    </br></br></br>

    <h2>Chat</h2>
    <div id="chatBox"></div>
    <button id="pingBtn" disabled>Send Ping</button> 
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
    let my_id = document.get_element_by_id("myId").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();

    my_id.set_value(id);
}

fn get_peer_id() -> String {
    let document = window().unwrap().document().unwrap();
    let peer_id = document.get_element_by_id("peerId").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();

    return peer_id.value();
}

fn set_status(message: &str) {
    let document = window().unwrap().document().unwrap();
    let status = document.get_element_by_id("status").unwrap();

    status.set_text_content(Some(message));
}

fn enable_chat_box() {
    let document = window().unwrap().document().unwrap();
    let ping_btn = document.get_element_by_id("pingBtn").unwrap().dyn_into::<HtmlButtonElement>().unwrap();
    ping_btn.set_disabled(false);
}

// 
// stateless logic
// 

fn append_chat_box(message: &str) {
    let document = window().unwrap().document().unwrap();
    let chatbox = document.get_element_by_id("chatBox").unwrap();

    let div = document.create_element("div").unwrap();
    div.set_text_content(Some(message));
    chatbox.append_child(&div).unwrap();
    let chatbox: HtmlElement = chatbox.dyn_into().unwrap();
    chatbox.set_scroll_top(chatbox.scroll_height());
}

pub fn setup_data_channel(dc: &RtcDataChannel) {
    let onopen_callback = Closure::wrap(Box::new(move || {
        enable_chat_box();
        append_chat_box("Connected!");
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

pub fn create_peer_connection() -> RtcPeerConnection {
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
            RtcPeerConnectionState::Connecting => "connecting",
            RtcPeerConnectionState::Connected => "connected",
            RtcPeerConnectionState::Disconnected => "disconnected",
            RtcPeerConnectionState::Failed => "failed",
            RtcPeerConnectionState::Closed => "closed",
            _ => "unknown",
        };
        set_status(&format!("Connection: {}", state_str));
    }) as Box<dyn FnMut()>);
    pc.set_onconnectionstatechange(Some(onconnectionstatechange_callback.as_ref().unchecked_ref()));
    onconnectionstatechange_callback.forget();

    return pc;
}

//
// stateful logic
//

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // panics to console.error
    
    init_ui();

    let peer_connection: Rc<RefCell<Option<RtcPeerConnection>>> = Rc::new(RefCell::new(None));
    let data_channel: Rc<RefCell<Option<RtcDataChannel>>> = Rc::new(RefCell::new(None));

    {
        let btn = web_sys::window().unwrap().document().unwrap().get_element_by_id("offerBtn").unwrap().dyn_into::<HtmlButtonElement>().unwrap();

        let pc = peer_connection.clone();
        let dc = data_channel.clone();
        let btn_callback = Closure::wrap(Box::new(move || {
            let pc_clone = pc.clone();
            let dc_clone = dc.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
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
        let btn = web_sys::window().unwrap().document().unwrap().get_element_by_id("connectBtn").unwrap().dyn_into::<HtmlButtonElement>().unwrap();
        
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

    {
        let btn = web_sys::window().unwrap().document().unwrap().get_element_by_id("pingBtn").unwrap().dyn_into::<HtmlButtonElement>().unwrap();
        
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
