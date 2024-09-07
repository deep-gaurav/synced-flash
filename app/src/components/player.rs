use leptos::*;
use leptos_use::{use_event_listener_with_options, UseEventListenerOptions};
use tracing::info;
use web_sys::{
    js_sys::Uint8Array, Blob, DomRect, HtmlCanvasElement, HtmlElement, HtmlInputElement,
};

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
