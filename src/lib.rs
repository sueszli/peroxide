use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;
use wasm_bindgen_futures::spawn_local;
use js_sys::{Array, JSON, Reflect};
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

struct WebRtcState {
    pc: RtcPeerConnection,
    dc: RtcDataChannel,
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

    let my_id: HtmlTextAreaElement = document.get_element_by_id("myId").unwrap().dyn_into()?;
    let offer_btn: HtmlButtonElement = document.get_element_by_id("offerBtn").unwrap().dyn_into()?;
    let peer_id: HtmlTextAreaElement = document.get_element_by_id("peerId").unwrap().dyn_into()?;
    let connect_btn: HtmlButtonElement = document.get_element_by_id("connectBtn").unwrap().dyn_into()?;
    let status = document.get_element_by_id("status").unwrap();
    let chat_box = document.get_element_by_id("chatBox").unwrap();
    let ping_btn: HtmlButtonElement = document.get_element_by_id("pingBtn").unwrap().dyn_into()?;

    Ok(())
}
