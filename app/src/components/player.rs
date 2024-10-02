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

use crate::{
    networking::{
        room_manager::{self, RoomManager},
        rtc_connect::receive_peer_connections,
    },
    utils::keycode::{Key, KeyEvent},
};

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
                    // info!("Recceived start");
                    if let Some(canvas) = canvas_ref.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                key_event_tx.set(Some(KeyEvent::MouseDown(
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
                    if let Some(canvas) = canvas_ref.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                key_event_tx.set(Some(KeyEvent::MouseUp(
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
                    if let Some(canvas) = canvas_ref.get_untracked() {
                        if let Some(touch) = ev.changed_touches().item(0) {
                            let dpr = window().device_pixel_ratio();

                            let rect = canvas.get_bounding_client_rect();
                            let (x, y) = (f64::from(touch.client_x()), f64::from(touch.client_y()));

                            if is_point_in_rect((x, y), rect.clone()) {
                                let x = x - rect.left();
                                let y = y - rect.top();
                                key_event_tx.set(Some(KeyEvent::MouseMove(
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

    let owner = Owner::current();
    create_effect(move |_| {
        let room_manager = expect_context::<RoomManager>();
        let rtc_message_receiver = room_manager.get_rtc_signal();
        let rtc_config = room_manager.get_rtc_config();
        if let (Some(owner), Some(rtc_message_receiver), Some(rtc_config)) =
            (owner, rtc_message_receiver, rtc_config)
        {
            with_owner(owner, || {
                let (rtc_rx, rtc_tx) = create_signal(None);
                create_effect(move |_| {
                    if let Some(msg) = rtc_rx.get() {
                        info!("Sending {msg:?}");
                        room_manager.send_rtc_message(msg);
                    }
                });
                receive_peer_connections(
                    canvas_ref,
                    rtc_config,
                    rtc_message_receiver,
                    rtc_tx,
                    key_event_tx,
                );
            });
        }
    });
    view! {
        <canvas ref=canvas_ref class="h-full w-full"
            tabindex="1"
            class=("hidden", move || swf_data.with(|v| v.is_none()))
            on:mousemove=move|ev|{
                if let Some(canvas) = canvas_ref.get_untracked(){
                    let rect = canvas.get_bounding_client_rect();
                    let dpr = window().device_pixel_ratio();
                    key_event_tx.set(Some(KeyEvent::MouseMove(f64::from(ev.offset_x())*dpr/rect.width(), f64::from(ev.offset_y())*dpr/rect.height())));
                }
            }
            on:mousedown=move|ev|{
                if let Some(canvas) = canvas_ref.get_untracked(){
                    let rect = canvas.get_bounding_client_rect();
                    let dpr = window().device_pixel_ratio();
                    key_event_tx.set(Some(KeyEvent::MouseDown(f64::from(ev.offset_x())*dpr/rect.width(), f64::from(ev.offset_y())*dpr/rect.height())));
                }
            }
            on:mouseup=move|ev|{
                if let Some(canvas) = canvas_ref.get_untracked(){
                    let rect = canvas.get_bounding_client_rect();
                    let dpr = window().device_pixel_ratio();
                    key_event_tx.set(Some(KeyEvent::MouseUp(f64::from(ev.offset_x())*dpr/rect.width(), f64::from(ev.offset_y())*dpr/rect.height())));
                }
            }

            on:keydown=move|ev|{
                if let Ok(key) = Key::try_from(ev){
                    key_event_tx.set(Some(KeyEvent::Down(key)));
                }else {
                    warn!("Cant convert event to KeyEvent")
                }
            }

            on:keyup=move|ev|{
                if let Ok(key) = Key::try_from(ev){
                    key_event_tx.set(Some(KeyEvent::Up(key)));
                }else{
                    warn!("Cant convert event to KeyEvent")
                }
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
