//! # WebRTC Peer-to-Peer Connection Protocol
//!
//! This module implements a WebRTC peer-to-peer connection protocol between two peers:
//!
//! - Host (Offerer): The peer that initiates the connection by creating an offer
//! - Guest (Answerer): The peer that receives the offer and creates an answer
//!
//! Connection Flow:
//!
//! 1. Host creates a `PeerConnection` and generates an offer (SDP)
//! 2. Host shares the offer with the guest (out-of-band, e.g., copy/paste)
//! 3. Guest receives the offer and creates their own `PeerConnection`
//! 4. Guest processes the offer and generates an answer (SDP)
//! 5. Guest shares the answer with the host (out-of-band)
//! 6. Host receives and processes the answer
//! 7. ICE candidates are exchanged and the connection is established
//! 8. Data channel opens and peers can exchange messages
//!
//! Technical Details:
//!
//! - Uses STUN server (stun.l.google.com:19302) for NAT traversal
//! - Creates a single data channel labeled "app" for message exchange
//! - Implements the offer/answer model as defined in RFC 3264
//! - Automatically handles ICE candidate gathering and connection state changes
//!
//! References:
//!
//! - [RFC 3264: An Offer/Answer Model with SDP](https://datatracker.ietf.org/doc/html/rfc3264)
//! - [WebRTC API Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API)

use crate::utils::{Kestrel, Thrush};
use js_sys;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelState, RtcIceServer, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcPeerConnectionState};

pub struct PeerConnectionCallbacks {
    pub on_sdp_ready: Box<dyn FnMut(String)>,
    pub on_connection_status_change: Box<dyn FnMut(&'static str)>,
    pub on_connection_established: Box<dyn FnMut()>,
    pub on_message_received: Box<dyn FnMut(String)>,
}

#[derive(Debug)]
pub struct PeerConnection {
    pc: RtcPeerConnection,
    dc: Rc<RefCell<Option<RtcDataChannel>>>,
}

impl PeerConnection {
    pub fn send_message(&self, message: &str) -> bool {
        console_log!("sent message: {}", message);
        self.dc
            .borrow()
            .as_ref()
            .filter(|dc: &&RtcDataChannel| dc.ready_state() == RtcDataChannelState::Open && !message.trim().is_empty())
            .map(|dc| dc.send_with_str(message).ok().pipe(|_| true))
            .unwrap_or(false)
    }

    pub async fn create_offer(&self) -> Result<(), JsValue> {
        let offer = JsFuture::from(self.pc.create_offer()).await?;
        JsFuture::from(self.pc.set_local_description(&offer.into())).await?;
        Ok(())
    }

    pub async fn set_remote_description(&self, sdp: &str) -> Result<(), JsValue> {
        let sdp = js_sys::JSON::parse(sdp)?;
        JsFuture::from(self.pc.set_remote_description(&sdp.into())).await?;
        Ok(())
    }
}

pub fn create_host_peer_connection(callbacks: PeerConnectionCallbacks) -> PeerConnection {
    let pc = create_rtc_peer_connection();
    let dc = pc.create_data_channel("app");
    let dc_ref = Rc::new(RefCell::new(Some(dc.clone())));

    setup_ice_callback(&pc, callbacks.on_sdp_ready);
    setup_connection_state_callback(&pc, callbacks.on_connection_status_change);
    setup_data_channel_callbacks(&dc, callbacks.on_connection_established, callbacks.on_message_received);

    PeerConnection { pc, dc: dc_ref }
}

pub async fn create_guest_peer_connection(offer: &str, callbacks: PeerConnectionCallbacks) -> Result<PeerConnection, JsValue> {
    let dc_ref = Rc::new(RefCell::new(None));
    let pc = create_rtc_peer_connection();

    setup_ice_callback(&pc, callbacks.on_sdp_ready);
    setup_connection_state_callback(&pc, callbacks.on_connection_status_change);
    setup_guest_data_channel_callbacks(&pc, dc_ref.clone(), callbacks.on_connection_established, callbacks.on_message_received)?;

    let sdp = js_sys::JSON::parse(offer)?;
    JsFuture::from(pc.set_remote_description(&sdp.into())).await?;

    let answer = JsFuture::from(pc.create_answer()).await?;
    JsFuture::from(pc.set_local_description(&answer.into())).await?;

    Ok(PeerConnection { pc, dc: dc_ref })
}

fn create_rtc_peer_connection() -> RtcPeerConnection {
    let ice_server = RtcIceServer::new();
    ice_server.set_urls(&js_sys::Array::of1(&JsValue::from_str("stun:stun.l.google.com:19302")));
    let config = RtcConfiguration::new();
    config.set_ice_servers(&js_sys::Array::of1(&ice_server));
    RtcPeerConnection::new_with_configuration(&config).unwrap()
}

fn setup_ice_callback(pc: &RtcPeerConnection, mut callback: Box<dyn FnMut(String)>) {
    Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
        event.candidate().is_none().then(|| {
            event
                .current_target()
                .and_then(|t| t.dyn_into::<RtcPeerConnection>().ok())
                .and_then(|pc| pc.local_description())
                .and_then(|desc| js_sys::JSON::stringify(&desc).ok())
                .and_then(|s| s.as_string())
                .tap(|sdp| console_log!("generated sdp: {:?}", sdp))
                .tap(|sdp| {
                    if let Some(sdp_str) = sdp {
                        callback(sdp_str.clone());
                    }
                })
        });
    }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>)
    .tap(|closure| pc.set_onicecandidate(Some(closure.as_ref().unchecked_ref())))
    .forget();
}

fn setup_connection_state_callback(pc: &RtcPeerConnection, mut callback: Box<dyn FnMut(&'static str)>) {
    let state_mapper = |pc: RtcPeerConnection| match pc.connection_state() {
        RtcPeerConnectionState::New => "🟡 New",
        RtcPeerConnectionState::Connecting => "🟡 Connecting",
        RtcPeerConnectionState::Connected => "🟢 Connected",
        RtcPeerConnectionState::Disconnected => "🔴 Disconnected",
        RtcPeerConnectionState::Failed => "🔴 Failed",
        RtcPeerConnectionState::Closed => "🔴 Closed",
        _ => "🔴 Unknown error",
    };

    let closure = Closure::wrap(Box::new(move |event: Event| {
        if let Some(target) = event.current_target() {
            if let Ok(pc) = target.dyn_into::<RtcPeerConnection>() {
                let state = state_mapper(pc);
                console_log!("connection status changed: {}", state);
                callback(state);
            }
        }
    }) as Box<dyn FnMut(Event)>);

    pc.set_onconnectionstatechange(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
}

fn setup_data_channel_callbacks(dc: &RtcDataChannel, mut on_open: Box<dyn FnMut()>, mut on_message: Box<dyn FnMut(String)>) {
    let open_closure = Closure::wrap(Box::new(move || {
        console_log!("data channel opened");
        on_open();
    }) as Box<dyn FnMut()>);
    dc.set_onopen(Some(open_closure.as_ref().unchecked_ref()));
    open_closure.forget();

    let message_closure = Closure::wrap(Box::new(move |event: MessageEvent| {
        let data = event.data().as_string();
        console_log!("message received: {:?}", data);
        if let Some(data_str) = data {
            on_message(data_str);
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    dc.set_onmessage(Some(message_closure.as_ref().unchecked_ref()));
    message_closure.forget();
}

fn setup_guest_data_channel_callbacks(pc: &RtcPeerConnection, dc_storage: Rc<RefCell<Option<RtcDataChannel>>>, on_open: Box<dyn FnMut()>, on_message: Box<dyn FnMut(String)>) -> Result<(), JsValue> {
    let callbacks = (Rc::new(RefCell::new(Some(on_open))), Rc::new(RefCell::new(Some(on_message))));

    let closure = Closure::wrap({
        let dc_storage = dc_storage.clone();
        let (on_open, on_message) = callbacks;

        Box::new(move |e: RtcDataChannelEvent| {
            let dc = e.channel();
            console_log!("data channel created: {}", dc.label());

            let open_cb = on_open.borrow_mut().take();
            let msg_cb = on_message.borrow_mut().take();

            if let (Some(open), Some(msg)) = (open_cb, msg_cb) {
                setup_data_channel_callbacks(&dc, open, msg);
            }

            *dc_storage.borrow_mut() = Some(dc);
        }) as Box<dyn FnMut(RtcDataChannelEvent)>
    });

    pc.set_ondatachannel(Some(closure.as_ref().unchecked_ref()));
    closure.forget();

    Ok(())
}
