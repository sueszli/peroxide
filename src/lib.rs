use wasm_bindgen::prelude::*;
use web_sys::{ Event, Element };

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
    pc: web_sys::RtcPeerConnection,
    dc: web_sys::RtcDataChannel,
    status_elem: Element,
    chat_box_elem: Element,
    ping_btn_elem: Element,
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
    body.set_inner_html(HTML);

    // let callback = Closure::wrap(Box::new(move |_: Event| {
    //     // 
    // }) as Box<dyn Fn(Event)>);

    // create_offer_button.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
    // callback.forget();

    Ok(())
}
