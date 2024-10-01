use leptos::*;
use leptos_use::{use_event_listener_with_options, UseEventListenerOptions};
use tracing::{info, warn};
use web_sys::MediaStream;

use crate::{
    components::player::is_point_in_rect,
    networking::{room_manager::RoomManager, rtc_connect::connect_to_host},
    utils::keycode::{Key, KeyEvent},
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
            if let Err(err) = video.play() {
                warn!("Cant play vdo {err:?}")
            }
        }
    });

    let owner = Owner::current();
    create_effect(move |_| {
        let room_manager = expect_context::<RoomManager>();
        let rtc_message_receiver = room_manager.get_rtc_signal();
        let rtc_config = room_manager.get_rtc_config();
        if let Some(room_info) = room_manager.get_room_info().get_untracked() {
            let host_user = room_info.users.first().cloned();
            if let (Some(host_user), Some(owner), Some(rtc_message_receiver), Some(rtc_config)) =
                (host_user, owner, rtc_message_receiver, rtc_config)
            {
                with_owner(owner, || {
                    let (rtc_rx, rtc_tx) = create_signal(None);
                    create_effect(move |_| {
                        if let Some(msg) = rtc_rx.get() {
                            info!("Sending {msg:?}");

                            room_manager.send_rtc_message(msg);
                        }
                    });
                    leptos::spawn_local(async move {
                        info!("Connect to host ");
                        if let Err(err) = connect_to_host(
                            host_user.id,
                            &rtc_config,
                            set_media_stream,
                            rtc_message_receiver,
                            rtc_tx,
                            events_rx,
                            owner,
                        )
                        .await
                        {
                            warn!("Cannot connect to host {err:?}");
                        }
                    });
                });
            }
        }
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

                    tabindex="1"

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

                    on:keydown=move|ev|{
                        if let Ok(key) = Key::try_from(ev){
                            events_tx.set(Some(KeyEvent::Down(key)));
                        }
                    }

                    on:keyup=move|ev|{
                        if let Ok(key) = Key::try_from(ev){
                            events_tx.set(Some(KeyEvent::Up(key)));
                        }
                    }
                />
            </div>
        </div>
    }
}
