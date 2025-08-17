//! # WebRTC Peer-to-Peer Connection Protocol
//!
//! This module implements a WebRTC peer-to-peer connection protocol between two peers:
//! a "host" and a "guest".
//!
//! # Protocol Overview
//!
//! Roles:
//! - **Host (Offerer)**: The peer that initiates the connection by creating an offer
//! - **Guest (Answerer)**: The peer that receives the offer and creates an answer
//!
//! Connection Flow:
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
//! - Uses STUN server (stun.l.google.com:19302) for NAT traversal
//! - Creates a single data channel labeled "app" for message exchange
//! - Implements the offer/answer model as defined in RFC 3264
//! - Automatically handles ICE candidate gathering and connection state changes
//!
//! References:
//! - [RFC 3264: An Offer/Answer Model with SDP](https://datatracker.ietf.org/doc/html/rfc3264)
//! - [WebRTC API Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API)

use js_sys;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelState, RtcIceServer, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcPeerConnectionState};

pub struct PeerConnectionCallbacks {
    pub on_sdp_ready: Box<dyn FnMut(String)>,
    pub on_connection_status_change: Box<dyn FnMut(&str)>,
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
        if let Some(dc) = self.dc.borrow().as_ref() {
            let is_valid_message = dc.ready_state() == RtcDataChannelState::Open && !message.trim().is_empty();
            if is_valid_message {
                dc.send_with_str(message).unwrap();
                console_log!("message sent: {}", message);
                true
            } else {
                false
            }
        } else {
            false
        }
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
    let pc = create_rtc_configuration();
    setup_ice_candidate_callback(&pc, callbacks.on_sdp_ready);
    setup_connection_state_callback(&pc, callbacks.on_connection_status_change);

    let dc = pc.create_data_channel("app");
    setup_data_channel_callbacks(&dc, callbacks.on_connection_established, callbacks.on_message_received);

    PeerConnection { pc, dc: Rc::new(RefCell::new(Some(dc))) }
}

pub async fn create_guest_peer_connection(offer: &str, callbacks: PeerConnectionCallbacks) -> Result<PeerConnection, JsValue> {
    let pc = create_rtc_configuration();
    setup_ice_candidate_callback(&pc, callbacks.on_sdp_ready);
    setup_connection_state_callback(&pc, callbacks.on_connection_status_change);

    let on_connection_established = Rc::new(RefCell::new(callbacks.on_connection_established));
    let on_message_received = Rc::new(RefCell::new(callbacks.on_message_received));

    let dc_storage = Rc::new(RefCell::new(None));
    let dc_storage_clone = dc_storage.clone();

    let ondatachannel_callback = Closure::wrap(Box::new(move |e: RtcDataChannelEvent| {
        let dc = e.channel();
        console_log!("data channel created: {}", dc.label());
        *dc_storage_clone.borrow_mut() = Some(dc.clone());

        let on_conn_est = on_connection_established.clone();
        let on_msg_recv = on_message_received.clone();
        setup_data_channel_callbacks(&dc, Box::new(move || on_conn_est.borrow_mut()()), Box::new(move |data| on_msg_recv.borrow_mut()(data)));
    }) as Box<dyn FnMut(RtcDataChannelEvent)>);
    pc.set_ondatachannel(Some(ondatachannel_callback.as_ref().unchecked_ref()));
    ondatachannel_callback.forget();

    let sdp = js_sys::JSON::parse(offer)?;
    JsFuture::from(pc.set_remote_description(&sdp.into())).await?;
    JsFuture::from(pc.set_local_description(&JsFuture::from(pc.create_answer()).await?.into())).await?;

    Ok(PeerConnection { pc, dc: dc_storage })
}

fn create_rtc_configuration() -> RtcPeerConnection {
    let ice_server = RtcIceServer::new();
    ice_server.set_urls(&js_sys::Array::of1(&JsValue::from_str("stun:stun.l.google.com:19302")));
    let configuration = RtcConfiguration::new();
    configuration.set_ice_servers(&js_sys::Array::of1(&ice_server));
    RtcPeerConnection::new_with_configuration(&configuration).unwrap()
}

fn setup_ice_candidate_callback(pc: &RtcPeerConnection, mut callback: Box<dyn FnMut(String)>) {
    let onicecandidate_callback = Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
        if event.candidate().is_none() {
            let pc = event.current_target().unwrap().dyn_into::<RtcPeerConnection>().unwrap();
            if let Some(desc) = pc.local_description() {
                let json_str = js_sys::JSON::stringify(&desc).unwrap().as_string().unwrap();
                console_log!("generated sdp: {}", json_str);
                callback(json_str);
            }
        }
    }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>);
    pc.set_onicecandidate(Some(onicecandidate_callback.as_ref().unchecked_ref()));
    onicecandidate_callback.forget();
}

fn setup_connection_state_callback(pc: &RtcPeerConnection, mut callback: Box<dyn FnMut(&str)>) {
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
        callback(state_str);
    }) as Box<dyn FnMut(Event)>);
    pc.set_onconnectionstatechange(Some(onconnectionstatechange_callback.as_ref().unchecked_ref()));
    onconnectionstatechange_callback.forget();
}

fn setup_data_channel_callbacks(dc: &RtcDataChannel, mut on_open: Box<dyn FnMut()>, mut on_message: Box<dyn FnMut(String)>) {
    let onopen_callback = Closure::wrap(Box::new(move || {
        console_log!("data channel opened");
        on_open();
    }) as Box<dyn FnMut()>);
    dc.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
        if let Some(data) = event.data().as_string() {
            console_log!("message received: {}", data);
            on_message(data);
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    dc.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();
}
