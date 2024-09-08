use leptos::*;
use leptos_meta::Title;
use leptos_router::*;
use tracing::info;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{js_sys::Uint8Array, Blob};

use crate::{
    components::{
        chatbox::ChatBox, player::Player, room_info::RoomInfo, video_player::VideoPlayer,
        virtual_buttons::VirtualButtons,
    },
    networking::room_manager::RoomManager,
};

#[derive(Params, PartialEq, Clone)]
struct RoomParam {
    id: Option<String>,
}
#[component]
pub fn RoomPage() -> impl IntoView {
    let params = use_params::<RoomParam>();
    let (swf_data, set_swf_data) = create_signal(Option::<(String, Vec<u8>)>::None);

    let room_manager = expect_context::<RoomManager>();
    // create_effect(move |_| {
    //     if let Some(video_name) = video_name.get() {
    //         room_manager.set_selected_video(video_name);
    //     }
    // });

    let (is_csr, set_is_csr) = create_signal(false);
    create_effect(move |_| set_is_csr.set(true));

    let (keyevent_rx, keyevent_tx) = create_signal(None);
    let room_info = room_manager.get_room_info();

    view! {
        {move || {
            if let Ok(RoomParam { id: Some(room_id) }) = params.get() {
                if !room_id.is_empty() {
                    view! {
                        <Title text=format!("Room {room_id}") />
                        <Player  swf_data=swf_data key_event_rx=keyevent_rx key_event_tx=keyevent_tx/>
                        {
                            move || {
                                if is_csr.get(){
                                    view! {
                                        <RoomInfo />
                                        <ChatBox />
                                        <VirtualButtons event_rx=keyevent_rx event_sender=keyevent_tx />

                                        {
                                            move || {
                                                if let Some(room_info) = room_info.get(){
                                                    if !room_info.is_host {
                                                        view! {
                                                            <VideoPlayer
                                                                events_rx=keyevent_rx
                                                                events_tx=keyevent_tx
                                                            />
                                                        }.into_view()
                                                    }else{
                                                        view! {}.into_view()
                                                    }
                                                }else{
                                                    view! {}.into_view()
                                                }
                                            }
                                        }
                                    }.into_view()
                                }else {
                                    view! {}.into_view()
                                }
                            }
                        }

                        {
                            move || {
                                let room_info = room_info.get().map(|r|r.is_host);

                                if room_info != Some(false){
                                    view! {
                                        <div
                                            class="h-full w-full flex px-10 py-4 items-center justify-center flex-col"
                                            class=("hidden", move || swf_data.with(|v| v.is_some()))
                                        >
                                            <div class="h-4" />
                                            <h1 class="text-xl font-bold2">"Room " {room_id.to_uppercase()}</h1>

                                            <div class="h-full w-full my-8 p-4 flex flex-col items-center justify-center border-white border-dotted border-2 rounded-sm">
                                                <div class="h-4" />
                                                <label
                                                    for="video-picker"
                                                    class="flex flex-col items-center justify-center"
                                                >
                                                    <div>"Drag and Drop Video"</div>
                                                    <div>"Or"</div>
                                                    <div>"Click here to pick"</div>
                                                </label>
                                                <input
                                                    class="hidden"
                                                    type="file"
                                                    id="video-picker"
                                                    on:change=move |ev| {
                                                        let input_el = ev
                                                            .unchecked_ref::<web_sys::Event>()
                                                            .target()
                                                            .unwrap_throw()
                                                            .unchecked_into::<web_sys::HtmlInputElement>();
                                                        let files = input_el.files();
                                                        if let Some(file) = files.and_then(|f| f.item(0)) {
                                                            let name = file.name();
                                                            leptos::spawn_local(async move {
                                                                let blob: &Blob = file.as_ref();
                                                                let array_buf_fut = wasm_bindgen_futures::JsFuture::from(
                                                                        blob.array_buffer(),
                                                                    )
                                                                    .await;
                                                                if let Ok(array_buf_jsval) = array_buf_fut {
                                                                    let uint8array = Uint8Array::new(&array_buf_jsval).to_vec();
                                                                    set_swf_data.set(Some((name, uint8array)));
                                                                }
                                                            });
                                                        }
                                                    }
                                                />
                                            </div>
                                        </div>
                                    }.into_view()
                                }else{
                                    view! {}.into_view()
                                }
                            }
                        }

                    }
                        .into_view()
                } else {
                    view! { <Redirect path="/" /> }.into_view()
                }
            } else {
                view! { <Redirect path="/" /> }.into_view()
            }
        }}
    }
}
