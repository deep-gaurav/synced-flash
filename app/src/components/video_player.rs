use ev::MessageEvent;
use leptos::*;
use leptos_use::{use_event_listener, use_event_listener_with_options, UseEventListenerOptions};
use logging::warn;
use tracing::info;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    js_sys::Array, MediaStream, RtcBundlePolicy, RtcConfiguration, RtcDataChannelInit,
    RtcIceConnectionState, RtcIceServer, RtcPeerConnection, RtcPeerConnectionState,
    RtcSessionDescriptionInit, RtcTrackEvent,
};

use crate::{
    components::player::is_point_in_rect, networking::room_manager::RoomManager,
    utils::keycode::KeyEvent,
};

#[component]
pub fn VideoPlayer(
    events_rx: ReadSignal<Option<KeyEvent>>,
    events_tx: WriteSignal<Option<KeyEvent>>,
) -> impl IntoView {
    let video_node = create_node_ref::<leptos::html::Video>();

    let (media_stream, set_media_stream) = create_signal(Option::<MediaStream>::None);

    create_effect(move |_| {
        if let (Some(media_stream), Some(video)) = (media_stream.get(), video_node.get()) {
            video.set_src_object(Some(&media_stream));
        }
    });

    create_effect(move |_| {
        let room_manager = expect_context::<RoomManager>();
        leptos::spawn_local(
            async move { join_pc(room_manager, set_media_stream, events_rx).await },
        );
    });

    create_effect(move |_| {
        if let Some(canvas) = video_node.get() {
            let el: &web_sys::HtmlElement = canvas.as_ref();
            let el_html: web_sys::HtmlElement = el.clone();
            let _ = use_event_listener_with_options(
                el_html.clone(),
                leptos::ev::touchstart,
                move |ev| {
                    // info!("Recceived start");
                    if let Some(canvas) = video_node.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                events_tx.set(Some(KeyEvent::MouseDown(
                                    x * dpr / rect.width(),
                                    y * dpr / rect.height(),
                                )));

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
                    // info!("Recceived end");
                    if let Some(canvas) = video_node.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                events_tx.set(Some(KeyEvent::MouseUp(
                                    x * dpr / rect.width(),
                                    y * dpr / rect.height(),
                                )));

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
                    if let Some(canvas) = video_node.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                events_tx.set(Some(KeyEvent::MouseMove(
                                    x * dpr / rect.width(),
                                    y * dpr / rect.height(),
                                )));

                                ev.prevent_default();
                            }
                        }
                    }
                },
                UseEventListenerOptions::default().passive(false),
            );
        }
    });

    view! {
        <div  class="h-full w-full flex flex-col">
            <div class="h-full w-full absolute flex items-center justify-center">
                <div class="text-lg"> "Please Wait.." </div>
            </div>
            <div class="flex-1 overflow-auto w-full relative" >
                <video
                    ref=video_node
                    class="h-full w-full"
                    autoplay
                    muted
                    playsinline

                    on:mousemove=move|ev|{
                        if let Some(canvas) = video_node.get_untracked(){
                            let rect = canvas.get_bounding_client_rect();
                            let dpr = window().device_pixel_ratio();
                            events_tx.set(Some(KeyEvent::MouseMove(f64::from(ev.offset_x())*dpr/rect.width(), f64::from(ev.offset_y())*dpr/rect.height())));
                        }
                    }
                    on:mousedown=move|ev|{
                        if let Some(canvas) = video_node.get_untracked(){
                            let rect = canvas.get_bounding_client_rect();
                            let dpr = window().device_pixel_ratio();
                            events_tx.set(Some(KeyEvent::MouseDown(f64::from(ev.offset_x())*dpr/rect.width(), f64::from(ev.offset_y())*dpr/rect.height())));
                        }
                    }
                    on:mouseup=move|ev|{
                        if let Some(canvas) = video_node.get_untracked(){
                            let rect = canvas.get_bounding_client_rect();
                            let dpr = window().device_pixel_ratio();
                            events_tx.set(Some(KeyEvent::MouseUp(f64::from(ev.offset_x())*dpr/rect.width(), f64::from(ev.offset_y())*dpr/rect.height())));
                        }
                    }
                />
            </div>
        </div>
    }
}

pub async fn join_pc(
    room_manager: RoomManager,
    media_setter: WriteSignal<Option<MediaStream>>,
    events_rx: ReadSignal<Option<KeyEvent>>,
) {
    let owner = Owner::current();
    let pc = RtcPeerConnection::new_with_configuration(&{
        let config = RtcConfiguration::new();
        config.set_bundle_policy(RtcBundlePolicy::MaxBundle);
        config.set_ice_servers(&{
            let array = Array::new();
            array.push(&JsValue::from({
                let ice_server = RtcIceServer::new();
                let from_str = JsValue::from_str("stun:stun.cloudflare.com:3478");
                ice_server.set_urls(&from_str);
                ice_server
            }));
            JsValue::from(array)
        });
        config
    });
    if let Ok(pc) = pc {
        pc.create_data_channel("server-events");

        let pc2 = pc.clone();
        let rm2 = room_manager.clone();
        leptos::spawn_local(async move {
            let offer = wasm_bindgen_futures::JsFuture::from(pc2.create_offer()).await;

            if let Ok(offer) = offer {
                let local_offer: RtcSessionDescriptionInit = offer.unchecked_into();
                if let Err(err) =
                    wasm_bindgen_futures::JsFuture::from(pc2.set_local_description(&local_offer))
                        .await
                {
                    warn!("Set local description failed {err:?}")
                }
                if let Some(sdp) = local_offer.get_sdp() {
                    info!("Make session offer");
                    rm2.send_rtc_message(common::message::RTCMessage::MakeJoinOffer(sdp));
                }
            }
        });

        let _ = use_event_listener(
            pc.clone(),
            ev::Custom::<RtcTrackEvent>::new("track"),
            move |ev| {
                info!("Received track");
                let track = ev.track();
                let mc = MediaStream::new();
                if let Ok(mc) = mc {
                    mc.add_track(&track);
                    media_setter.set(Some(mc));
                }
            },
        );

        let pc_ev = pc.clone();
        let rm2 = room_manager.clone();
        let _ = use_event_listener(
            pc.clone(),
            ev::Custom::<ev::Event>::new("iceconnectionstatechange"),
            move |_| {
                info!("Connectionstate {:?}", pc_ev.ice_connection_state());
                if pc_ev.ice_connection_state() == RtcIceConnectionState::Connected {
                    info!("Connected, request dc");
                    rm2.send_rtc_message(common::message::RTCMessage::RequestDataChannel(
                        "events".to_string(),
                    ));
                }
            },
        );
        let rtc_signal = room_manager.get_rtc_signal();
        if let Some(rtc_signal) = rtc_signal {
            create_effect(move |_| {
                if let Some(msg) = rtc_signal.get() {
                    match msg {
                        common::message::RTCMessage::JoinRemoteSdp(sdp) => {
                            info!("Received join remote sdp");
                            let description =
                                RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer);
                            description.set_sdp(&sdp);
                            let pc2 = pc.clone();
                            leptos::spawn_local(async move {
                                let result = wasm_bindgen_futures::JsFuture::from(
                                    pc2.set_remote_description(&description),
                                )
                                .await;
                                if let Err(err) = result {
                                    warn!("Set Remote description failed {err:?}")
                                } else {
                                    let answer =
                                        wasm_bindgen_futures::JsFuture::from(pc2.create_answer())
                                            .await;
                                    match answer {
                                        Ok(answer) => {
                                            let answer: RtcSessionDescriptionInit =
                                                answer.unchecked_into();
                                            if let Err(err) = wasm_bindgen_futures::JsFuture::from(
                                                pc2.set_local_description(&answer),
                                            )
                                            .await
                                            {
                                                warn!("Local description set failed {err:?}")
                                            }
                                            if let Some(sdp) = answer.get_sdp() {
                                                let rm = expect_context::<RoomManager>();
                                                rm.send_rtc_message(
                                                    common::message::RTCMessage::SendJoinLocalSdp(
                                                        sdp,
                                                    ),
                                                );
                                                info!("Send answer");
                                            }
                                        }
                                        Err(err) => {
                                            warn!("Answer creation failed {err:?}")
                                        }
                                    }
                                }
                            })
                        }

                        common::message::RTCMessage::AddHostSdp(_, _)
                        | common::message::RTCMessage::AddHostRemoteSdp(_)
                        | common::message::RTCMessage::RequestJoinSdp
                        | common::message::RTCMessage::SendJoinLocalSdp(_) => {
                            warn!("Received non join rtc signal")
                        }

                        common::message::RTCMessage::RequestDataChannel(_) => {
                            warn!("Not expected RequestDataChannel")
                        }
                        common::message::RTCMessage::DataChannelCreated((name, id)) => {
                            info!("dc received");
                            let dc = pc.clone().create_data_channel_with_data_channel_dict(
                                &format!("{name}-sub"),
                                &{
                                    let init = RtcDataChannelInit::new();
                                    init.set_negotiated(true);
                                    init.set_id(id as u16);
                                    init
                                },
                            );
                            info!("DataChannel created");
                            let dc2 = dc.clone();
                            if let Some(owner) = owner {
                                with_owner(owner, || {
                                    create_effect(move |_| {
                                        let event = events_rx.get();
                                        if let Some(event) = event {
                                            let data = bincode::serialize(&event);
                                            if let Ok(data) = data {
                                                if let Err(err) = dc2.send_with_u8_array(&data) {
                                                    warn!("Failed to send to data_channel {err:?}")
                                                }
                                            }
                                        }
                                    });
                                });
                            }
                            info!("Sending join sdp");
                            room_manager
                                .send_rtc_message(common::message::RTCMessage::RequestJoinSdp);
                        }
                        common::message::RTCMessage::MakeJoinOffer(_) => {
                            warn!("Unexpected make offer");
                        }
                        common::message::RTCMessage::JoinAnswer(sdp) => {
                            info!("Received join answer");
                            let description =
                                RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
                            description.set_sdp(&sdp);
                            let pc2 = pc.clone();
                            leptos::spawn_local(async move {
                                let result = wasm_bindgen_futures::JsFuture::from(
                                    pc2.set_remote_description(&description),
                                )
                                .await;
                                match result {
                                    Ok(_) => info!("remote desc set"),
                                    Err(er) => info!("remote desc set failed {er:?}"),
                                }
                                // info!("Set result {")
                            });
                        }
                    }
                }
            });
        }
    }
}
