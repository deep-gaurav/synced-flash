use leptos::*;
use tracing::info;
use web_sys::{js_sys::Uint8Array, Blob, DomRect, HtmlInputElement};

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
            on:touchstart=move|ev| {

                if let Some(canvas) = canvas_ref.get_untracked(){
                    if let Some(touch) = ev.target_touches().item(0) {
                        let dpr = window().device_pixel_ratio();

                        let rect = canvas.get_bounding_client_rect();
                        let x = f64::from(touch.client_x()) - rect.left();
                        let y = f64::from(touch.client_y()) - rect.top();
                        ev.prevent_default();

                        if is_point_in_rect((x,y), rect) {
                            key_event_tx.set(Some(KeyEvent::MouseDown(x*dpr, y*dpr)));
                        }
                    }
                }
            }

            on:touchend=move|ev| {

                if let Some(canvas) = canvas_ref.get_untracked(){
                    if let Some(touch) = ev.target_touches().item(0) {
                        let dpr = window().device_pixel_ratio();

                        let rect = canvas.get_bounding_client_rect();
                        let x = f64::from(touch.client_x()) - rect.left();
                        let y = f64::from(touch.client_y()) - rect.top();
                        ev.prevent_default();

                        if is_point_in_rect((x,y), rect) {
                            key_event_tx.set(Some(KeyEvent::MouseUp(x*dpr, y*dpr)));
                        }
                    }
                }
            }

            on:touchmove=move|ev| {

                if let Some(canvas) = canvas_ref.get_untracked(){
                    if let Some(touch) = ev.target_touches().item(0) {
                        let dpr = window().device_pixel_ratio();

                        let rect = canvas.get_bounding_client_rect();
                        let x = f64::from(touch.client_x()) - rect.left();
                        let y = f64::from(touch.client_y()) - rect.top();
                        ev.prevent_default();


                        if is_point_in_rect((x,y), rect) {
                            key_event_tx.set(Some(KeyEvent::MouseMove(x*dpr, y*dpr)));
                        }
                    }
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
