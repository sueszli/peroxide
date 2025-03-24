use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ Event, Element };

const STYLING: &str = r#"
body { max-width: 600px; margin: 0 auto; }
textarea { width: 100%; height: 3rem; }
#chatBox { height: 200px; overflow-y: auto; border: 1px solid; }
"#;

const ICE_SERVERS: [&str; 1] = [ "stun:stun.l.google.com:19302", ];

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

    // insert styling
    let style = document.create_element("style")?;
    style.set_text_content(Some(STYLING));
    head.append_child(&style)?;

    // offer generation
    let create_offer_button = document.create_element("button")?;
    create_offer_button.set_inner_html("Create Offer");
    body.append_child(&create_offer_button)?;
    
    let callback = Closure::wrap(Box::new(move |_: Event| {
        // ...
    }) as Box<dyn Fn(Event)>);

    create_offer_button.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
    callback.forget();

    Ok(())
}
