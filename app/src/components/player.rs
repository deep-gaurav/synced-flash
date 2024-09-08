use std::future::IntoFuture;

use ev::MessageEvent;
use leptos::*;
use leptos_use::{use_event_listener, use_event_listener_with_options, UseEventListenerOptions};
use logging::warn;
use tracing::info;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    js_sys::{Array, ArrayBuffer, Uint8Array},
    Blob, DomRect, HtmlCanvasElement, HtmlElement, HtmlInputElement, MediaStreamTrack,
    RtcBundlePolicy, RtcConfiguration, RtcDataChannelInit, RtcIceConnectionState, RtcIceServer,
    RtcPeerConnection, RtcPeerConnectionState, RtcRtpTransceiverInit, RtcSessionDescription,
    RtcSessionDescriptionInit,
};

use crate::networking::room_manager::{self, RoomManager};

use super::virtual_buttons::KeyEvent;

#[component]
pub fn Player(
    swf_data: ReadSignal<Option<(String, Vec<u8>)>>,
    key_event_rx: ReadSignal<Option<KeyEvent>>,
    key_event_tx: WriteSignal<Option<KeyEvent>>,
) -> impl IntoView {
    let canvas_ref = create_node_ref::<leptos::html::Canvas>();
    let (is_web, set_is_web) = create_signal(false);

    create_effect(move |_| {
        set_is_web.set(true);
    });

    create_effect(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let el: &HtmlElement = canvas.as_ref();
            let el_html: HtmlElement = el.clone();
            let _ = use_event_listener_with_options(
                el_html.clone(),
                leptos::ev::touchstart,
                move |ev| {
                    info!("Recceived start");
                    if let Some(canvas) = canvas_ref.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                key_event_tx.set(Some(KeyEvent::MouseDown(x * dpr, y * dpr)));

                                ev.prevent_default();
                            }
                        }
                    }
                },
                UseEventListenerOptions::default().passive(false),
            );

            let _ = use_event_listener_with_options(
                el_html.clone(),
                leptos::ev::touchend,
                move |ev| {
                    info!("Recceived end");
                    if let Some(canvas) = canvas_ref.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                key_event_tx.set(Some(KeyEvent::MouseUp(x * dpr, y * dpr)));

                                ev.prevent_default();
                            }
                        }
                    }
                },
                UseEventListenerOptions::default().passive(false),
            );

            let _ = use_event_listener_with_options(
                el_html,
                leptos::ev::touchmove,
                move |ev| {
                    if let Some(canvas) = canvas_ref.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                key_event_tx.set(Some(KeyEvent::MouseMove(x * dpr, y * dpr)));

                                ev.prevent_default();
                            }
                        }
                    }
                },
                UseEventListenerOptions::default().passive(false),
            );

            let room_manager = expect_context::<RoomManager>();
            if room_manager.is_host() == Some(true) {
                leptos::spawn_local(async move {
                    setup_webrtc_sender(canvas, room_manager, key_event_tx).await;
                });
            }
        }
    });
    view! {
        <canvas ref=canvas_ref class="h-full w-full"
            class=("hidden", move || swf_data.with(|v| v.is_none()))
            on:mousemove=move|ev|{
                let dpr = window().device_pixel_ratio();
                key_event_tx.set(Some(KeyEvent::MouseMove(f64::from(ev.offset_x())*dpr, f64::from(ev.offset_y())*dpr)));
            }
            on:mousedown=move|ev|{
                let dpr = window().device_pixel_ratio();
                key_event_tx.set(Some(KeyEvent::MouseDown(f64::from(ev.offset_x())*dpr, f64::from(ev.offset_y())*dpr)));
            }
            on:mouseup=move|ev|{
                let dpr = window().device_pixel_ratio();
                key_event_tx.set(Some(KeyEvent::MouseUp(f64::from(ev.offset_x())*dpr, f64::from(ev.offset_y())*dpr)));
                key_event_tx.set(Some(KeyEvent::MouseMove(f64::from(ev.offset_x())*dpr, f64::from(ev.offset_y())*dpr)));
            }

        ></canvas>
        {
            move || {
                if is_web.get() {

                    #[cfg(all(
                        feature = "ruffle_web_common",
                        feature = "ruffle_core",
                        feature = "ruffle_render"
                    ))]
                    {
                        use crate::components::player_web::PlayerWeb;
                        return view! {
                            <PlayerWeb
                                canvas_ref=canvas_ref
                                swf_data=swf_data
                                key_event_rx
                            />
                        }.into_view()
                    }
                    view! {

                    }.into_view()
                }else{
                    view! {

                    }.into_view()
                }
            }
        }
    }
}

pub fn is_point_in_rect(point: (f64, f64), rect: DomRect) -> bool {
    point.0 > rect.left()
        && point.0 < rect.right()
        && point.1 > rect.top()
        && point.1 < rect.bottom()
}

pub async fn setup_webrtc_sender(
    canvas: leptos::HtmlElement<leptos::html::Canvas>,
    room_manager: RoomManager,
    events_tx: WriteSignal<Option<KeyEvent>>,
) {
    let owner = Owner::current();
    let pc = RtcPeerConnection::new_with_configuration(&{
        let config = RtcConfiguration::new();
        config.set_bundle_policy(RtcBundlePolicy::MaxBundle);
        config.set_ice_servers(&{
            let array = Array::new();
            array.push(&JsValue::from({
                let ice_server = RtcIceServer::new();
                ice_server.set_urls(&JsValue::from_str("stun:stun.cloudflare.com:3478"));
                ice_server
            }));
            JsValue::from(array)
        });
        config
    });

    if let Ok(pc) = pc {
        pc.create_data_channel("server-events");
        let media = canvas.capture_stream();
        match media {
            Ok(media) => {
                let mut tranceivers = vec![];
                let tracks = media.get_tracks();
                for track in tracks.iter() {
                    match track.dyn_into::<MediaStreamTrack>() {
                        Ok(track) => {
                            tranceivers.push(pc.add_transceiver_with_media_stream_track_and_init(
                                &track,
                                &{
                                    let init = RtcRtpTransceiverInit::new();
                                    init.set_direction(
                                        web_sys::RtcRtpTransceiverDirection::Sendonly,
                                    );
                                    init
                                },
                            ));
                        }
                        Err(err) => {
                            warn!("track not MediaStreamTrack {err:?}")
                        }
                    }
                }
                let offer = wasm_bindgen_futures::JsFuture::from(pc.create_offer()).await;
                match offer {
                    Ok(offer) => {
                        info!(
                            "Info created, is init {}, is desc {}",
                            offer.has_type::<RtcSessionDescriptionInit>(),
                            offer.has_type::<RtcSessionDescription>()
                        );
                        let local_offer: RtcSessionDescriptionInit = offer.unchecked_into();
                        if let Err(err) = wasm_bindgen_futures::JsFuture::from(
                            pc.set_local_description(&local_offer),
                        )
                        .await
                        {
                            warn!("Set local description failed {err:?}")
                        }
                        let send_tracks = tranceivers
                            .iter()
                            .map(|t| (t.mid(), t.sender().track().map(|t| t.id())))
                            .collect::<Vec<_>>();
                        info!(
                            "Ready to send sdp {:?} tracks: {:?}",
                            local_offer.get_sdp(),
                            send_tracks
                        );
                        room_manager.send_rtc_message(common::message::RTCMessage::AddHostSdp(
                            local_offer.get_sdp(),
                            send_tracks,
                        ));
                        let pc_st = pc.clone();
                        let rm2 = room_manager.clone();
                        let _ = use_event_listener(
                            pc.clone(),
                            ev::Custom::<ev::Event>::new("iceconnectionstatechange"),
                            move |_| {
                                info!("Connectionstate {:?}", pc_st.ice_connection_state());
                                if pc_st.ice_connection_state() == RtcIceConnectionState::Connected
                                {
                                    info!("rtc connected");
                                }
                            },
                        );
                        if let Some(rtc_signal) = room_manager.get_rtc_signal() {
                            create_effect(move |_| {
                                if let Some(msg) = rtc_signal.get() {
                                    match msg {
                                        common::message::RTCMessage::AddHostSdp(_, _) => {
                                            warn!("Shouldnt receive AddHostSdp on client")
                                        }
                                        common::message::RTCMessage::AddHostRemoteSdp(sdp) => {
                                            let description = RtcSessionDescriptionInit::new(
                                                web_sys::RtcSdpType::Answer,
                                            );
                                            description.set_sdp(&sdp);
                                            let pc2 = pc.clone();
                                            leptos::spawn_local(async move {
                                                let result = wasm_bindgen_futures::JsFuture::from(
                                                    pc2.set_remote_description(&description),
                                                )
                                                .await;
                                                if let Err(err) = result {
                                                    warn!("Set Remote description failed {err:?}")
                                                }
                                            })
                                        }
                                        common::message::RTCMessage::RequestJoinSdp
                                        | common::message::RTCMessage::JoinRemoteSdp(_)
                                        | common::message::RTCMessage::SendJoinLocalSdp(_) => {
                                            warn!("Non host rtc message received")
                                        }
                                        common::message::RTCMessage::RequestDataChannel(_) => {
                                            warn!("Not expected RequestDataChannel")
                                        }
                                        common::message::RTCMessage::DataChannelCreated((
                                            name,
                                            id,
                                        )) => {
                                            info!("RequestDataChannel received {name} {id}");
                                            let dc = pc
                                                .clone()
                                                .create_data_channel_with_data_channel_dict(
                                                    &name,
                                                    &{
                                                        let init = RtcDataChannelInit::new();
                                                        init.set_negotiated(true);
                                                        init.set_id(id as u16);
                                                        init
                                                    },
                                                );
                                            if let Some(owner) = owner {
                                                with_owner(owner, || {
                                                    let _ = use_event_listener(
                                                        dc.clone(),
                                                        ev::Custom::<MessageEvent>::new("message"),
                                                        move |ev| {
                                                            let data =
                                                                ev.data().dyn_into::<ArrayBuffer>();
                                                            match data {
                                                                Ok(data) => {
                                                                    let uint8buf =
                                                                        Uint8Array::new(&data);
                                                                    let data_vec =
                                                                        uint8buf.to_vec();
                                                                    if let Ok(data) =
                                                                        bincode::deserialize::<
                                                                            KeyEvent,
                                                                        >(
                                                                            &data_vec
                                                                        )
                                                                    {
                                                                        events_tx.set(Some(data));
                                                                    }
                                                                }
                                                                Err(er) => {
                                                                    warn!("ev data not arraybuffer {er:?}")
                                                                }
                                                            }
                                                        },
                                                    );
                                                });
                                                let dc2 = dc.clone();
                                            }
                                        }
                                        common::message::RTCMessage::MakeJoinOffer(_)
                                        | common::message::RTCMessage::JoinAnswer(_) => {
                                            warn!("Unexpected player msg")
                                        }
                                    }
                                }
                            });
                        }
                    }
                    Err(err) => {
                        warn!("Offer creation failed {err:?}")
                    }
                }
            }
            Err(err) => {
                warn!("Cannt capture canvas media {err:?}")
            }
        }
    }
}
