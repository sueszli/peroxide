use js_sys;
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelState, RtcIceServer, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcPeerConnectionState};

/// This function is responsible for the first step in the protocol.
/// Create a new `RtcPeerConnection` and set up event handlers.
///
/// We have two peers:
///
/// - The "host" (offerer) is the peer that initiates the connection by creating an offer.
/// - The "guest" (answerer) is the peer that receives the offer and creates an answer.
///
/// See: https://datatracker.ietf.org/doc/html/rfc3264
///
/// # Arguments
///
/// - `on_offer_generation` - A callback that is called when the ICE candidate gathering is complete and the SDP is ready to be sent to the other peer.
/// - `on_connection_status_change` - A callback that is called when the connection state of the peer connection changes.
pub fn create_host_peer_connection<F1, F2>(mut on_offer_generation: F1, mut on_connection_status_change: F2) -> RtcPeerConnection
where
    F1: 'static + FnMut(String),
    F2: 'static + FnMut(&str),
{
    let ice_server = RtcIceServer::new();
    ice_server.set_urls(&js_sys::Array::of1(&JsValue::from_str("stun:stun.l.google.com:19302")));
    let configuration = RtcConfiguration::new();
    configuration.set_ice_servers(&js_sys::Array::of1(&ice_server));
    let pc = RtcPeerConnection::new_with_configuration(&configuration).unwrap();

    // also called by the guest, because both the offer and the answer use the SDP data format
    let onicecandidate_callback = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
        if event.candidate().is_none() {
            let pc = event.current_target().unwrap().dyn_into::<RtcPeerConnection>().unwrap();
            if let Some(desc) = pc.local_description() {
                let json_str = js_sys::JSON::stringify(&desc).unwrap().as_string().unwrap();
                console_log!("generated sdp: {}", json_str);
                on_offer_generation(json_str);
            }
        }
    }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);
    pc.set_onicecandidate(Some(onicecandidate_callback.as_ref().unchecked_ref()));
    onicecandidate_callback.forget();

    // change in connection state
    let onconnectionstatechange_callback = Closure::wrap(Box::new(move |event: Event| {
        let pc = event.current_target().unwrap().dyn_into::<RtcPeerConnection>().unwrap();
        let state_str = match pc.connection_state() {
            RtcPeerConnectionState::New => "🟡 New",
            RtcPeerConnectionState::Connecting => "🟡 Connecting",
            RtcPeerConnectionState::Connected => "🟢 Connected",
            RtcPeerConnectionState::Disconnected => "🔴 Disconnected",
            RtcPeerConnectionState::Failed => "🔴 Failed",
            RtcPeerConnectionState::Closed => "🔴 Closed",
            _ => "🔴 Unknown error",
        };
        console_log!("connection status changed: {}", state_str);
        on_connection_status_change(state_str);
    }) as Box<dyn FnMut(Event)>);
    pc.set_onconnectionstatechange(Some(onconnectionstatechange_callback.as_ref().unchecked_ref()));
    onconnectionstatechange_callback.forget();

    pc
}

/// Use the hosts offer, to create an answer.
///
/// # Arguments
///
/// - `offer` - The SDP offer from the host
/// - `on_answer_generation` - Callback when the SDP answer is generated
/// - `on_connection_status_change` - Callback when connection state changes
/// - `on_datachannel_created` - Callback when data channel is created
pub async fn create_guest_peer_connection<F1, F2, F3>(offer: &str, on_answer_generation: F1, on_connection_status_change: F2, mut on_datachannel_created: F3) -> RtcPeerConnection
where
    F1: 'static + FnMut(String),
    F2: 'static + FnMut(&str),
    F3: 'static + FnMut(RtcDataChannel),
{
    let sdp = js_sys::JSON::parse(offer).unwrap();
    let pc = create_host_peer_connection(on_answer_generation, on_connection_status_change);

    let ondatachannel_callback = Closure::wrap(Box::new(move |e: RtcDataChannelEvent| {
        let dc = e.channel();
        console_log!("data channel created: {}", dc.label());
        on_datachannel_created(dc);
    }) as Box<dyn FnMut(RtcDataChannelEvent)>);
    pc.set_ondatachannel(Some(ondatachannel_callback.as_ref().unchecked_ref()));
    ondatachannel_callback.forget();

    JsFuture::from(pc.set_remote_description(&sdp.into())).await.unwrap();
    JsFuture::from(pc.set_local_description(&JsFuture::from(pc.create_answer()).await.unwrap().into())).await.unwrap();

    pc
}

/// Update the data channel with event handlers.
///
/// # Arguments
///
/// - `dc` - The RtcDataChannel to set up
/// - `on_connection_established` - Callback when the data channel opens
/// - `on_message_received` - Callback when a message is received
pub fn config_data_channel<F1, F2>(dc: &RtcDataChannel, mut on_connection_established: F1, mut on_message_received: F2)
where
    F1: 'static + FnMut(),
    F2: 'static + FnMut(String),
{
    let onopen_callback = Closure::wrap(Box::new(move || {
        console_log!("data channel opened");
        on_connection_established();
    }) as Box<dyn FnMut()>);
    dc.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        if let Some(data) = event.data().as_string() {
            console_log!("message received: {}", data);
            on_message_received(data);
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    dc.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();
}

/// Send a message through the data channel if it's open.
/// Returns true if message was sent, false if channel is not ready
pub fn send_message(dc: &RtcDataChannel, message: &str) -> bool {
    if dc.ready_state() == RtcDataChannelState::Open && !message.trim().is_empty() {
        dc.send_with_str(message).unwrap();
        console_log!("message sent: {}", message);
        true
    } else {
        false
    }
}
