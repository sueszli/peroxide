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
        self.dc
            .borrow()
            .as_ref()
            .filter(|dc| dc.ready_state() == RtcDataChannelState::Open && !message.trim().is_empty())
            .map(|dc| {
                dc.send_with_str(message).ok();
                console_log!("message sent: {}", message);
                true
            })
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
    create_rtc_peer_connection().pipe(|pc| (pc.create_data_channel("app"), pc)).pipe(|(dc, pc)| {
        let dc_ref = Rc::new(RefCell::new(Some(dc.clone())));
        setup_ice_callback(&pc, callbacks.on_sdp_ready);
        setup_connection_state_callback(&pc, callbacks.on_connection_status_change);
        setup_data_channel_callbacks(&dc, callbacks.on_connection_established, callbacks.on_message_received);
        PeerConnection { pc, dc: dc_ref }
    })
}

pub async fn create_guest_peer_connection(offer: &str, callbacks: PeerConnectionCallbacks) -> Result<PeerConnection, JsValue> {
    let dc_ref = Rc::new(RefCell::new(None));

    create_rtc_peer_connection()
        .tap(|pc| setup_ice_callback(pc, callbacks.on_sdp_ready))
        .tap(|pc| setup_connection_state_callback(pc, callbacks.on_connection_status_change))
        .tap(|pc| setup_guest_data_channel(pc, dc_ref.clone(), callbacks.on_connection_established, callbacks.on_message_received).unwrap())
        .pipe(|pc| async move {
            js_sys::JSON::parse(offer)?.pipe(|sdp| JsFuture::from(pc.set_remote_description(&sdp.into()))).await?;

            JsFuture::from(pc.create_answer()).await?.pipe(|answer| JsFuture::from(pc.set_local_description(&answer.into()))).await?;

            Ok(PeerConnection { pc, dc: dc_ref })
        })
        .await
}

fn create_rtc_peer_connection() -> RtcPeerConnection {
    RtcIceServer::new().tap(|s| s.set_urls(&js_sys::Array::of1(&JsValue::from_str("stun:stun.l.google.com:19302")))).pipe(|ice_server| RtcConfiguration::new().tap(|c| c.set_ice_servers(&js_sys::Array::of1(&ice_server)))).pipe(|config| RtcPeerConnection::new_with_configuration(&config).unwrap())
}

fn setup_ice_callback(pc: &RtcPeerConnection, mut callback: Box<dyn FnMut(String)>) {
    Closure::wrap(Box::new(move |event: RtcPeerConnectionIceEvent| {
        event.candidate().is_none().then(|| {
            event.current_target().and_then(|t| t.dyn_into::<RtcPeerConnection>().ok()).and_then(|pc| pc.local_description()).and_then(|desc| js_sys::JSON::stringify(&desc).ok()).and_then(|s| s.as_string()).map(|sdp| {
                console_log!("generated sdp: {}", sdp);
                callback(sdp);
            })
        });
    }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>)
    .tap(|closure| pc.set_onicecandidate(Some(closure.as_ref().unchecked_ref())))
    .forget();
}

fn setup_connection_state_callback(pc: &RtcPeerConnection, mut callback: Box<dyn FnMut(&'static str)>) {
    Closure::wrap(Box::new(move |event: Event| {
        event
            .current_target()
            .and_then(|t| t.dyn_into::<RtcPeerConnection>().ok())
            .map(|pc| match pc.connection_state() {
                RtcPeerConnectionState::New => "🟡 New",
                RtcPeerConnectionState::Connecting => "🟡 Connecting",
                RtcPeerConnectionState::Connected => "🟢 Connected",
                RtcPeerConnectionState::Disconnected => "🔴 Disconnected",
                RtcPeerConnectionState::Failed => "🔴 Failed",
                RtcPeerConnectionState::Closed => "🔴 Closed",
                _ => "🔴 Unknown error",
            })
            .map(|state| {
                console_log!("connection status changed: {}", state);
                callback(state);
            });
    }) as Box<dyn FnMut(Event)>)
    .tap(|closure| pc.set_onconnectionstatechange(Some(closure.as_ref().unchecked_ref())))
    .forget();
}

fn setup_data_channel_callbacks(dc: &RtcDataChannel, mut on_open: Box<dyn FnMut()>, mut on_message: Box<dyn FnMut(String)>) {
    Closure::wrap(Box::new(move || {
        console_log!("data channel opened");
        on_open();
    }) as Box<dyn FnMut()>)
    .tap(|closure| dc.set_onopen(Some(closure.as_ref().unchecked_ref())))
    .forget();

    Closure::wrap(Box::new(move |event: MessageEvent| {
        event.data().as_string().map(|data| {
            console_log!("message received: {}", data);
            on_message(data);
        });
    }) as Box<dyn FnMut(MessageEvent)>)
    .tap(|closure| dc.set_onmessage(Some(closure.as_ref().unchecked_ref())))
    .forget();
}

fn setup_guest_data_channel(pc: &RtcPeerConnection, dc_storage: Rc<RefCell<Option<RtcDataChannel>>>, on_open: Box<dyn FnMut()>, on_message: Box<dyn FnMut(String)>) -> Result<(), JsValue> {
    let callbacks = (Rc::new(RefCell::new(Some(on_open))), Rc::new(RefCell::new(Some(on_message))));

    Closure::wrap({
        let dc_storage = dc_storage.clone();
        let (on_open, on_message) = callbacks;

        Box::new(move |e: RtcDataChannelEvent| {
            e.channel()
                .tap(|dc| console_log!("data channel created: {}", dc.label()))
                .tap(|dc| {
                    (on_open.borrow_mut().take(), on_message.borrow_mut().take()).pipe(|(open_cb, msg_cb)| match (open_cb, msg_cb) {
                        (Some(open), Some(msg)) => setup_data_channel_callbacks(dc, open, msg),
                        _ => {}
                    })
                })
                .pipe(|dc| *dc_storage.borrow_mut() = Some(dc));
        }) as Box<dyn FnMut(RtcDataChannelEvent)>
    })
    .tap(|closure| pc.set_ondatachannel(Some(closure.as_ref().unchecked_ref())))
    .forget();

    Ok(())
}
