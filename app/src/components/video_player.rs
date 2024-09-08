use leptos::*;
use leptos_use::use_event_listener;
use logging::warn;
use tracing::info;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    js_sys::Array, MediaStream, RtcBundlePolicy, RtcConfiguration, RtcIceServer, RtcPeerConnection,
    RtcSessionDescriptionInit, RtcTrackEvent,
};

use crate::networking::room_manager::RoomManager;

#[component]
pub fn VideoPlayer() -> impl IntoView {
    let video_node = create_node_ref::<leptos::html::Video>();

    let (media_stream, set_media_stream) = create_signal(Option::<MediaStream>::None);

    create_effect(move |_| {
        if let (Some(media_stream), Some(video)) = (media_stream.get(), video_node.get()) {
            video.set_src_object(Some(&media_stream));
        }
    });

    create_effect(move |_| {
        let room_manager = expect_context::<RoomManager>();
        leptos::spawn_local(async move { join_pc(room_manager, set_media_stream).await });
    });

    view! {
        <div  class="h-full w-full flex flex-col">
            <div class="flex-1 overflow-auto w-full relative" >
                <video
                    ref=video_node
                    class="h-full w-full"
                    autoplay
                    muted
                    playsinline
                />
            </div>
        </div>
    }
}

pub async fn join_pc(room_manager: RoomManager, media_setter: WriteSignal<Option<MediaStream>>) {
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

        room_manager.send_rtc_message(common::message::RTCMessage::RequestJoinSdp);
        let rtc_signal = room_manager.get_rtc_signal();
        if let Some(rtc_signal) = rtc_signal {
            create_effect(move |_| {
                if let Some(msg) = rtc_signal.get() {
                    match msg {
                        common::message::RTCMessage::JoinRemoteSdp(sdp) => {
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
                    }
                }
            });
        }
    }
}
