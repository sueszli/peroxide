use wasm_bindgen::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use js_sys::*;
use web_sys::*;

// 
// rendering
// 

const STYLING: &str = r#"
    body { max-width: 600px; margin: 0 auto; }
    textarea { width: 100%; height: 3rem; }
    #chatBox { height: 200px; overflow-y: auto; border: 1px solid; }
"#;

const HTML: &str = r#"
    <textarea id="myId" readonly></textarea>
    <button id="offerBtn">Create Offer</button>
    <textarea id="peerId" placeholder="Enter peer ID here"></textarea>
    <button id="connectBtn">Connect</button>
    <div id="status">Ready</div>
    <div id="chatBox"></div>
    <button id="pingBtn" disabled>Send Ping</button> 
"#;

fn set_my_id(id: &str) {
    let document = window().unwrap().document().unwrap();
    let my_id = document.get_element_by_id("myId").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();

    my_id.set_value(id);
}

fn set_status(message: &str) {
    let document = window().unwrap().document().unwrap();
    let status = document.get_element_by_id("status").unwrap();

    status.set_text_content(Some(message));
}

fn append_chat_box(message: &str) {
    let document = window().unwrap().document().unwrap();
    let chatbox = document.get_element_by_id("chatBox").unwrap();

    let div = document.create_element("div").unwrap();
    div.set_text_content(Some(message));
    chatbox.append_child(&div).unwrap();
    let chatbox: HtmlElement = chatbox.dyn_into().unwrap();
    chatbox.set_scroll_top(chatbox.scroll_height());
}

//
// logic
//

fn setup_pc(pc: &RtcPeerConnection) {
    // on ice candidate: display my id
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
    
    // on connection state change: display new status
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

    // on data channel: display new messages
    let ondatachannel_callback = Closure::wrap(Box::new(move |event: RtcDataChannelEvent| {
        let channel = event.channel();
        
        // on open
        let onopen_callback = Closure::wrap(Box::new(move || {
            append_chat_box("Connected!");
        }) as Box<dyn FnMut()>);
        channel.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        // on message
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(data) = event.data().as_string() {
                append_chat_box(&format!("Peer: {}", data));
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

    }) as Box<dyn FnMut(RtcDataChannelEvent)>);
    pc.set_ondatachannel(Some(ondatachannel_callback.as_ref().unchecked_ref()));
    ondatachannel_callback.forget();
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // redirect panics to console.error

    let document = web_sys::window().unwrap().document().unwrap();
    let style = document.create_element("style")?;
    style.set_text_content(Some(STYLING));
    document.head().unwrap().append_child(&style)?;
    document.body().unwrap().set_inner_html(HTML);

    let offer_btn = document.get_element_by_id("offerBtn").unwrap().dyn_into::<HtmlButtonElement>().unwrap();
    let offer_btn_callback = Closure::wrap(Box::new(move || {
        wasm_bindgen_futures::spawn_local(async move {
            let ice_server = RtcIceServer::new();
            ice_server.set_urls(&js_sys::Array::of1(&JsValue::from_str("stun:stun.l.google.com:19302")));
            let configuration = RtcConfiguration::new();
            configuration.set_ice_servers(&js_sys::Array::of1(&ice_server));
            let pc = RtcPeerConnection::new_with_configuration(&configuration).unwrap();
            setup_pc(&pc);
            let channel = pc.create_data_channel("chat");            

            // on open
            let on_open = Closure::wrap(Box::new(move || {
                append_chat_box("Connected!");
            }) as Box<dyn FnMut()>);
            channel.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            on_open.forget();
            
            // on message
            let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(data) = e.data().as_string() {
                    append_chat_box(&format!("Peer: {}", data));
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
            
            match JsFuture::from(pc.create_offer()).await {
                Ok(js_offer) => {
                    let offer: RtcSessionDescriptionInit = js_offer.into();
                    if let Err(e) = JsFuture::from(pc.set_local_description(&offer)).await {
                        console::error_1(&e);
                    }
                }
                Err(e) => console::error_1(&e),
            }

            set_status("Offer created! Share your ID.");
        });
    }) as Box<dyn FnMut()>);
    offer_btn.set_onclick(Some(offer_btn_callback.as_ref().unchecked_ref()));
    offer_btn_callback.forget();

    Ok(())
}
