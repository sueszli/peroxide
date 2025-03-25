use wasm_bindgen::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use js_sys::*;
use web_sys::*;
use std::cell::RefCell;
use std::rc::Rc;

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

struct AppState {
    peer_connection: Option<RtcPeerConnection>,
    data_channel: Option<RtcDataChannel>,
}

thread_local! {
    static APP_STATE: Rc<RefCell<AppState>> = Rc::new(RefCell::new(AppState {
        peer_connection: None,
        data_channel: None,
    }));
}

fn append_chatbox(message: &str) {
    let document = window().unwrap().document().unwrap();
    let chatbox = document.get_element_by_id("chatBox").unwrap();

    let div = document.create_element("div").unwrap();
    div.set_text_content(Some(message));
    chatbox.append_child(&div).unwrap();
    let chatbox: HtmlElement = chatbox.dyn_into().unwrap();
    chatbox.set_scroll_top(chatbox.scroll_height());
}

fn setup(pc: &RtcPeerConnection) {
    let document = web_sys::window().unwrap().document().unwrap();
    let my_id = document.get_element_by_id("myId").unwrap().dyn_into::<HtmlTextAreaElement>().unwrap();
    let status = document.get_element_by_id("status").unwrap().dyn_into::<HtmlElement>().unwrap();
    let ping_btn = document.get_element_by_id("pingBtn").unwrap().dyn_into::<HtmlButtonElement>().unwrap();

    // onicecandidate
    let pc_clone = pc.clone();
    let my_id_clone = my_id.clone();
    let onicecandidate_callback = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
        if event.candidate().is_none() {
            if let Some(desc) = pc_clone.local_description() {
                let json = js_sys::JSON::stringify(&desc).unwrap();
                my_id_clone.set_value(&json.as_string().unwrap());
            }
        }
    }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);
    pc.set_onicecandidate(Some(onicecandidate_callback.as_ref().unchecked_ref()));
    onicecandidate_callback.forget();

    // oniceconnectionstatechange
    let pc_clone = pc.clone();
    let status_clone = status.clone();
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
        status_clone.set_text_content(Some(&format!("Connection: {state_str}")));
    }) as Box<dyn FnMut()>);
    pc.set_onconnectionstatechange(Some(onconnectionstatechange_callback.as_ref().unchecked_ref()));
    onconnectionstatechange_callback.forget();

    // ondatachannel
    let ping_btn_clone = ping_btn.clone();
    let ondatachannel_callback = Closure::wrap(Box::new(move |event: RtcDataChannelEvent| {
        let channel = event.channel();
        
        let ping_btn_inner = ping_btn_clone.clone();
        let onopen_callback = Closure::wrap(Box::new(move || {
            ping_btn_inner.set_disabled(false);
            append_chatbox("Connected!");
        }) as Box<dyn FnMut()>);
        channel.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(data) = event.data().as_string() {
                append_chatbox(&format!("Peer: {}", data));
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        channel.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        APP_STATE.with(|state| {
            state.borrow_mut().data_channel = Some(channel);
        });
    }) as Box<dyn FnMut(RtcDataChannelEvent)>);
    pc.set_ondatachannel(Some(ondatachannel_callback.as_ref().unchecked_ref()));
    ondatachannel_callback.forget();
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // panics to console.error

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let head = document.head().unwrap();

    let style = document.create_element("style")?;
    style.set_text_content(Some(STYLING));
    head.append_child(&style)?;
    
    body.set_inner_html(HTML);
    // Get the offer button and set up click handler
    let offer_btn: HtmlButtonElement = document
        .get_element_by_id("offerBtn")
        .ok_or_else(|| JsValue::from_str("Offer button not found"))?
        .dyn_into()?;

    let on_offer_click = Closure::wrap(Box::new(move || {
        wasm_bindgen_futures::spawn_local(async move {
            // Fix 2: Use proper ICE server configuration
            let mut ice_server = RtcIceServer::new();
            ice_server.set_urls(&js_sys::Array::of1(&JsValue::from_str("stun:stun.l.google.com:19302")));
            
            let configuration = RtcConfiguration::new();
            configuration.set_ice_servers(&js_sys::Array::of1(&ice_server));
            
            // Fix 3: Use correct constructor with configuration
            let pc = RtcPeerConnection::new_with_configuration(&configuration)
                .map_err(|e| JsValue::from_str(&format!("Failed to create peer connection: {:?}", e))).unwrap();
            
            setup(&pc);
            
            // Fix 4: Proper error handling for data channel creation
            let channel = pc.create_data_channel("chat");
            
            let document = web_sys::window().unwrap().document().unwrap();
            let ping_btn: HtmlButtonElement = document.get_element_by_id("pingBtn").unwrap().dyn_into().unwrap();
            
            // Data channel open handler
            let ping_clone = ping_btn.clone();
            let on_open = Closure::wrap(Box::new(move || {
                ping_clone.set_disabled(false);
                append_chatbox("Connected!");
            }) as Box<dyn FnMut()>);
            channel.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            on_open.forget();
            
            // Data channel message handler
            let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(data) = e.data().as_string() {
                    append_chatbox(&format!("Peer: {}", data));
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
            
            // Create and set offer
            match JsFuture::from(pc.create_offer()).await {
                Ok(js_offer) => {
                    let offer: RtcSessionDescriptionInit = js_offer.into();
                    if let Err(e) = JsFuture::from(pc.set_local_description(&offer)).await {
                        console::error_1(&e);
                    }
                }
                Err(e) => console::error_1(&e),
            }
            
            // Update status
            let status: HtmlElement = document.get_element_by_id("status").unwrap().dyn_into().unwrap();
            status.set_text_content(Some("Offer created! Share your ID."));
        });
    }) as Box<dyn FnMut()>);

    offer_btn.set_onclick(Some(on_offer_click.as_ref().unchecked_ref()));
    on_offer_click.forget();


    Ok(())
}
