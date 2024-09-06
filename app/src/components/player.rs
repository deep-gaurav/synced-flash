use leptos::*;
use web_sys::{js_sys::Uint8Array, Blob, HtmlInputElement};

#[component]
pub fn Player(swf_data: ReadSignal<Option<(String, Vec<u8>)>>) -> impl IntoView {
    let canvas_ref = create_node_ref::<leptos::html::Canvas>();
    let (is_web, set_is_web) = create_signal(false);

    create_effect(move |_| {
        set_is_web.set(true);
    });
    view! {
        <canvas ref=canvas_ref class="h-full w-full"
            class=("hidden", move || swf_data.with(|v| v.is_none()))
        ></canvas>
        {
            move || {
                if is_web.get() {

                    #[cfg(target_family = "wasm")]
                    {
                        use crate::components::player_web::PlayerWeb;
                        return view! {
                            <PlayerWeb
                                canvas_ref=canvas_ref
                                swf_data=swf_data
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
