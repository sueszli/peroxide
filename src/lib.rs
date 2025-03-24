use wasm_bindgen::prelude::*;
use web_sys::*;

const STYLING: &str = r#"
    body { max-width: 600px; margin: 0 auto; }
    textarea { width: 100%; height: 3rem; }
    #chatBox { height: 200px; overflow-y: auto; border: 1px solid; }
"#;

struct WebRtcState {
    pc: web_sys::RtcPeerConnection,
    dc: web_sys::RtcDataChannel,
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once(); // panics to console.error

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let head = document.head().unwrap();

    // styling
    let style = document.create_element("style")?;
    style.set_text_content(Some(STYLING));
    head.append_child(&style)?;

    // html
    // <textarea id="myId" readonly></textarea>
    // <button id="offerBtn">Create Offer</button>
    // <textarea id="peerId" placeholder="Enter peer ID here"></textarea>
    // <button id="connectBtn">Connect</button>
    // <div id="status">Ready</div>
    // <div id="chatBox"></div>
    // <button id="pingBtn" disabled>Send Ping</button> 

    let my_id: web_sys::HtmlTextAreaElement = document.create_element("textarea")?.dyn_into()?;
    my_id.set_id("myId");
    my_id.set_read_only(true);
    body.append_child(&my_id)?;

    let offer_btn = document.create_element("button")?;
    offer_btn.set_id("offerBtn");
    offer_btn.set_text_content(Some("Create Offer"));
    body.append_child(&offer_btn)?;

    let peer_id: web_sys::HtmlTextAreaElement = document.create_element("textarea")?.dyn_into()?;
    peer_id.set_id("peerId");
    peer_id.set_placeholder("Enter peer ID here");
    body.append_child(&peer_id)?;

    let connect_btn = document.create_element("button")?;
    connect_btn.set_id("connectBtn");
    connect_btn.set_text_content(Some("Connect"));
    body.append_child(&connect_btn)?;

    let status = document.create_element("div")?;
    status.set_id("status");
    status.set_text_content(Some("Ready"));
    body.append_child(&status)?;

    let chat_box = document.create_element("div")?;
    chat_box.set_id("chatBox");
    body.append_child(&chat_box)?;

    let ping_btn: web_sys::HtmlButtonElement = document.create_element("button")?.dyn_into()?;
    ping_btn.set_id("pingBtn");
    ping_btn.set_text_content(Some("Send Ping"));
    ping_btn.set_disabled(true);
    body.append_child(&ping_btn)?;

    // logic

    Ok(())
}
