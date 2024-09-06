use leptos::*;
use tracing::info;
use web_sys::{js_sys::Uint8Array, Blob, HtmlInputElement};

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
