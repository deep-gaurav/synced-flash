use std::collections::HashMap;

use common::message::{RTCMessage, RTCSessionDesc, RtcConfig};
use leptos::{
    create_effect, create_signal, ev, store_value, with_owner, NodeRef, Owner, ReadSignal,
    SignalGet, SignalGetUntracked, SignalSet, WriteSignal,
};
use leptos_use::use_event_listener;
use tracing::{info, warn};
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    js_sys::{Array, ArrayBuffer, Uint8Array, JSON},
    Blob, MediaStream, MessageEvent, RtcConfiguration, RtcDataChannelEvent, RtcDataChannelInit,
    RtcIceCandidate, RtcIceCandidateInit, RtcIceServer, RtcOfferOptions, RtcPeerConnection,
    RtcPeerConnectionIceEvent, RtcSdpType, RtcSessionDescriptionInit, RtcTrackEvent,
};

use crate::utils::keycode::KeyEvent;

pub fn connect_rtc(rtc_config: &RtcConfig) -> Result<RtcPeerConnection, JsValue> {
    warn!("CREATING PC");
    RtcPeerConnection::new_with_configuration(&{
        let config = RtcConfiguration::new();
        config.set_ice_servers(&{
            let array = Array::new();
            array.push(&JsValue::from({
                let ice_server = RtcIceServer::new();
                ice_server.set_urls(&JsValue::from_str(&rtc_config.stun));
                ice_server
            }));
            array.push(&JsValue::from({
                let ice_server = RtcIceServer::new();
                ice_server.set_urls(&JsValue::from_str(&rtc_config.turn));
                ice_server.set_username(&rtc_config.turn_user);
                ice_server.set_credential(&rtc_config.turn_creds);
                ice_server
            }));
            JsValue::from(array)
        });
        config
    })
}

pub async fn connect_to_host(
    host_user: Uuid,
    rtc_config: &RtcConfig,
    media_setter: WriteSignal<Option<MediaStream>>,
    rtc_message_receiver: ReadSignal<Option<RTCMessage>>,
    rtc_message_sender: WriteSignal<Option<RTCMessage>>,
    events_rx: ReadSignal<Option<KeyEvent>>,
    owner: Owner,
) -> Result<(), JsValue> {
    let pc = connect_rtc(rtc_config)?;

    with_owner(owner, || {
        let _ = use_event_listener(
            pc.clone(),
            ev::Custom::<RtcTrackEvent>::new("track"),
            move |ev| {
                info!(
                    "Received track from rtc streams len {}",
                    ev.streams().length()
                );
                if let Some(stream) = ev.streams().get(0).dyn_ref::<MediaStream>() {
                    media_setter.set(Some(stream.clone()));
                }
            },
        );
    });

    let dc = pc.create_data_channel_with_data_channel_dict("events", &{
        let dict = RtcDataChannelInit::new();
        dict.set_ordered(false);
        dict.set_max_retransmits(0);
        dict
    });

    with_owner(owner, || {
        create_effect(move |_| {
            if let Some(ev) = events_rx.get() {
                let data = bincode::serialize(&ev);
                if let Ok(data) = data {
                    if let Err(err) = dc.send_with_u8_array(&data) {
                        warn!("Failed to send to data_channel {err:?}")
                    }
                }
            }
        });
    });

    with_owner(owner, || {
        let _ = use_event_listener(
            pc.clone(),
            leptos::ev::Custom::<RtcPeerConnectionIceEvent>::new("icecandidate"),
            move |ev| {
                if let Some(candidate) = ev.candidate() {
                    if let Ok(candidate) = serialize_candidate(candidate) {
                        info!("Sending ice");
                        rtc_message_sender
                            .set(Some(RTCMessage::ExchangeCandidate(host_user, candidate)));
                    } else {
                        warn!("Cant serialize candidate")
                    }
                }
            },
        );
    });

    let offer = wasm_bindgen_futures::JsFuture::from(pc.create_offer_with_rtc_offer_options(&{
        let options = RtcOfferOptions::new();
        options.set_offer_to_receive_video(true);
        options
    }))
    .await?;
    let offer = offer.unchecked_into::<RtcSessionDescriptionInit>();
    wasm_bindgen_futures::JsFuture::from(pc.set_local_description(&offer)).await?;
    rtc_message_sender.set(Some(RTCMessage::ExchangeSessionDesc(
        host_user,
        RTCSessionDesc {
            typ: JsValue::from(offer.get_type())
                .as_string()
                .expect("sdp type not string"),
            sdp: offer.get_sdp().expect("No sdp"),
        },
    )));

    with_owner(owner, || {
        create_effect({
            let pc = pc.clone();
            move |_| {
                if let Some(rtc_message) = rtc_message_receiver.get() {
                    match rtc_message {
                        RTCMessage::ExchangeSessionDesc(_, rtcsession_desc) => {
                            info!("Received sdp {rtcsession_desc:?}");
                            if let Some(rtc_sdp) =
                                RtcSdpType::from_js_value(&JsValue::from_str(&rtcsession_desc.typ))
                            {
                                let rtc_sdp = RtcSessionDescriptionInit::new(rtc_sdp);
                                rtc_sdp.set_sdp(&rtcsession_desc.sdp);
                                leptos::spawn_local({
                                    let pc = pc.clone();
                                    async move {
                                        let _ = wasm_bindgen_futures::JsFuture::from(
                                            pc.set_remote_description(&rtc_sdp),
                                        )
                                        .await;
                                    }
                                });
                            }
                        }
                        RTCMessage::ExchangeCandidate(_, candidate) => {
                            info!("Received ice");
                            if let Ok(candidate) = deserialize_candidate(&candidate) {
                                let _ = pc.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(
                                    &candidate,
                                ));
                            } else {
                                warn!("Cant deserialize candidate")
                            }
                        }
                    }
                }
            }
        });
    });

    Ok(())
}

pub fn receive_peer_connections(
    canvas: NodeRef<leptos::html::Canvas>,
    rtc_config: RtcConfig,
    rtc_message_receiver: ReadSignal<Option<RTCMessage>>,
    rtc_message_sender: WriteSignal<Option<RTCMessage>>,
    events_tx: WriteSignal<Option<KeyEvent>>,
) {
    let peers = store_value(HashMap::new());
    let pending_candidates = store_value(HashMap::<Uuid, Vec<RtcIceCandidateInit>>::new());
    create_effect(move |_| {
        if let Some(msg) = rtc_message_receiver.get() {
            match msg {
                RTCMessage::ExchangeSessionDesc(from_user, rtcsession_desc) => {
                    info!("Received sdp {rtcsession_desc:?}");

                    let rtc_config = rtc_config.clone();
                    leptos::spawn_local(async move {
                        match accept_peer_connection(&rtc_config, rtcsession_desc, canvas).await {
                            Ok((pc, answer)) => {
                                let _ = use_event_listener(
                                    pc.clone(),
                                    leptos::ev::Custom::<RtcDataChannelEvent>::new("datachannel"),
                                    move |ev| {
                                        let dc = ev.channel();
                                        if dc.label() == "events" {
                                            let _ = use_event_listener(
                                                dc,
                                                leptos::ev::Custom::<MessageEvent>::new("message"),
                                                move |ev| {
                                                    leptos::spawn_local(async move {
                                                        let data =
                                                            ev.data().dyn_into::<ArrayBuffer>();

                                                        let data = match data {
                                                            Ok(data) => {
                                                                let uint8buf =
                                                                    Uint8Array::new(&data);
                                                                Some(uint8buf)
                                                            }
                                                            Err(er) => {
                                                                warn!("ev data not arraybuffer {er:?}");
                                                                if let Ok(blob) =
                                                                    ev.data().dyn_into::<Blob>()
                                                                {
                                                                    let buf = wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await;
                                                                    if let Ok(buf) = buf {
                                                                        let arb = buf.unchecked_into::<ArrayBuffer>();
                                                                        Some(Uint8Array::new(&arb))
                                                                    } else {
                                                                        warn!("Cant get arraybuf from blob");
                                                                        None
                                                                    }
                                                                } else {
                                                                    warn!("data not blob");
                                                                    None
                                                                }
                                                            }
                                                        };

                                                        if let Some(buf) = data {
                                                            let data_vec = buf.to_vec();
                                                            if let Ok(data) =
                                                                bincode::deserialize::<KeyEvent>(
                                                                    &data_vec,
                                                                )
                                                            {
                                                                events_tx.set(Some(data));
                                                            } else {
                                                                warn!("ev not keyevent")
                                                            }
                                                        }
                                                    });
                                                },
                                            );
                                        }
                                    },
                                );
                                if let Some(candidates) =
                                    pending_candidates.with_value(|pc| pc.get(&from_user).cloned())
                                {
                                    pending_candidates.update_value(|pc| {
                                        pc.remove(&from_user);
                                    });
                                    for candidate in candidates {
                                        let _ = pc
                                            .add_ice_candidate_with_opt_rtc_ice_candidate_init(
                                                Some(&candidate),
                                            );
                                    }
                                }

                                let _ = use_event_listener(
                                    pc.clone(),
                                    leptos::ev::Custom::<RtcPeerConnectionIceEvent>::new(
                                        "icecandidate",
                                    ),
                                    move |ev| {
                                        if let Some(candidate) = ev.candidate() {
                                            if let Ok(candidate) = serialize_candidate(candidate) {
                                                info!("Sending ice");

                                                rtc_message_sender.set(Some(
                                                    RTCMessage::ExchangeCandidate(
                                                        from_user, candidate,
                                                    ),
                                                ));
                                            } else {
                                                warn!("Cant serialize candidate")
                                            }
                                        }
                                    },
                                );

                                peers.update_value(|p| {
                                    p.insert(from_user, pc);
                                });
                                rtc_message_sender
                                    .set(Some(RTCMessage::ExchangeSessionDesc(from_user, answer)));
                            }

                            Err(err) => {
                                warn!("Cant receive connection {err:?}");
                            }
                        }
                    });
                }
                RTCMessage::ExchangeCandidate(from_user, candidate) => {
                    info!("Received ice");
                    if let Ok(candidate) = deserialize_candidate(&candidate) {
                        if let Some(pc) = peers.with_value(|peers| peers.get(&from_user).cloned()) {
                            let _ = pc.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(
                                &candidate,
                            ));
                        } else {
                            pending_candidates.update_value(|ice| {
                                let ice_queue = ice.entry(from_user).or_default();
                                ice_queue.push(candidate);
                            });
                        }
                    } else {
                        warn!("Cant deserialize candidate")
                    }
                }
            }
        }
    });
}

async fn accept_peer_connection(
    rtc_config: &RtcConfig,
    rtc_session_desc: RTCSessionDesc,
    canvas: NodeRef<leptos::html::Canvas>,
) -> Result<(RtcPeerConnection, RTCSessionDesc), JsValue> {
    let pc = connect_rtc(rtc_config)?;
    let canvas = canvas
        .get_untracked()
        .ok_or(JsValue::from_str("canvas not connected"))?;
    let media_stream = canvas.capture_stream_with_frame_request_rate(30.0)?;
    for track in media_stream.get_video_tracks() {
        pc.add_track(&track.dyn_into()?, &media_stream, &Array::new());
    }

    let offer_type = RtcSdpType::from_js_value(&JsValue::from_str(&rtc_session_desc.typ))
        .ok_or(JsValue::from_str("cannot convert sdp type"))?;
    let rtc_sdp = RtcSessionDescriptionInit::new(offer_type);
    rtc_sdp.set_sdp(&rtc_session_desc.sdp);
    wasm_bindgen_futures::JsFuture::from(pc.set_remote_description(&rtc_sdp)).await?;

    let answer = wasm_bindgen_futures::JsFuture::from(pc.create_answer()).await?;
    let answer = answer.unchecked_into::<RtcSessionDescriptionInit>();

    wasm_bindgen_futures::JsFuture::from(pc.set_local_description(&answer)).await?;

    Ok((
        pc,
        RTCSessionDesc {
            typ: JsValue::from(answer.get_type())
                .as_string()
                .expect("sdp type not string"),
            sdp: answer.get_sdp().expect("No sdp"),
        },
    ))
}

pub fn serialize_candidate(candidate: RtcIceCandidate) -> Result<String, JsValue> {
    JSON::stringify(&candidate.to_json()).map(|s| s.into())
}

pub fn deserialize_candidate(candidate: &str) -> Result<RtcIceCandidateInit, JsValue> {
    let obj = JSON::parse(candidate)?;
    Ok(obj.unchecked_into())
}
