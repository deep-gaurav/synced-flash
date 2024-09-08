use leptos::*;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::components::portal::Portal;
use crate::MountPoints;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
    Space,

    X,
    C,
    W,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyEvent {
    Down(Key),
    Up(Key),
    MouseMove(f64, f64),
    MouseDown(f64, f64),
    MouseUp(f64, f64),
}

impl Key {
    pub fn get_symbol(&self) -> String {
        match &self {
            Key::UpArrow => "⬆️".to_string(),
            Key::DownArrow => "⬇️".to_string(),
            Key::LeftArrow => "⬅️".to_string(),
            Key::RightArrow => "➡️".to_string(),

            Key::C => "C".to_string(),
            Key::W => "W".to_string(),
            Key::X => "X".to_string(),

            Key::Space => "".to_string(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct VirtualKey {
    key: Key,
    top: f32,
    left: f32,
    width: f32,
    height: f32,
}

#[component]
pub fn VirtualButtons(
    event_rx: ReadSignal<Option<KeyEvent>>,
    event_sender: WriteSignal<Option<KeyEvent>>,
) -> impl IntoView {
    let MountPoints { speaker_point, .. } = expect_context::<MountPoints>();

    let (virtual_keys, set_virtual_keys) = create_signal(Vec::new());

    // create_effect(move |_| {
    //     info!("event rx {:?}", event_rx.get());
    // });
    view! {
        {
            move || {
                if let Some(speaker_point) = speaker_point.get() {

                    let el:&web_sys::Element = speaker_point.as_ref();
                    view! {
                        <Portal
                            mount=el.clone()
                            class="h-full w-full bg-black p-2"
                        >
                            <button class="text-sm"
                                type="button"
                                on:click=move|_|{
                                    set_virtual_keys.set(
                                        vec![
                                            VirtualKey{
                                                key: Key::UpArrow,
                                                left:42.5,
                                                top:75.0,
                                                width: 15.0,
                                                height: 5.0,
                                            },
                                            VirtualKey{
                                                key: Key::DownArrow,
                                                left:42.5,
                                                top:80.0,
                                                width: 15.0,
                                                height: 5.0
                                            },

                                            VirtualKey{
                                                key: Key::LeftArrow,
                                                left:42.5 - 15.0,
                                                top:80.0,
                                                width: 15.0,
                                                height: 5.0
                                            },


                                            VirtualKey{
                                                key: Key::RightArrow,
                                                left:42.5 + 15.0,
                                                top:80.0,
                                                width: 15.0,
                                                height: 5.0
                                            },

                                            VirtualKey{
                                                key: Key::Space,
                                                left:42.5 - 15.0,
                                                top:88.0,
                                                width: 45.0,
                                                height: 5.0
                                            }
                                        ]
                                    )
                                }
                            > "Add Gamepad" </button>

                            <button class="text-sm"
                                type="button"
                                on:click=move|_|{
                                    set_virtual_keys.set(
                                        vec![
                                            VirtualKey{
                                                key: Key::X,
                                                left:42.5 - 15.0,
                                                top:80.0,
                                                width: 15.0,
                                                height: 5.0
                                            },


                                            VirtualKey{
                                                key: Key::C,
                                                left:42.5 + 15.0,
                                                top:80.0,
                                                width: 15.0,
                                                height: 5.0
                                            },


                                            VirtualKey{
                                                key: Key::W,
                                                left:42.5 - 15.0,
                                                top:88.0,
                                                width: 45.0,
                                                height: 5.0
                                            }

                                        ]
                                    )
                                }
                            > "Add Gamepad 2" </button>

                            <Portal
                                class="absolute h-full w-full top-0 left-0 pointer-events-none z-30"
                            >
                                <For
                                    each=move||virtual_keys.get()
                                    key=|k|k.key
                                    children=move|k| {
                                        view!{
                                            <button
                                                type="button"
                                                class="border border-white absolute bg-white/50 text-lg flex items-center justify-center pointer-events-auto"
                                                style=format!("top:{}%; left:{}%; height:{}%; width:{}%;", k.top, k.left, k.height, k.width)
                                                // on:click=move|_| {
                                                //     event_sender.set(Some(KeyEvent::Down(k.key)));
                                                //     event_sender.set(Some(KeyEvent::Up(k.key)))
                                                // }

                                                on:touchstart=move|ev|{
                                                    ev.prevent_default();
                                                    event_sender.set(Some(KeyEvent::Down(k.key)));
                                                }
                                                on:touchend=move|ev|{
                                                    ev.prevent_default();
                                                    event_sender.set(Some(KeyEvent::Up(k.key)))
                                                }
                                                on:mousedown=move|_|{
                                                    event_sender.set(Some(KeyEvent::Down(k.key)));
                                                }
                                                on:mouseup=move|_|{
                                                    event_sender.set(Some(KeyEvent::Up(k.key)))
                                                }
                                            >
                                                {
                                                    k.key.get_symbol()
                                                }
                                            </button>
                                        }
                                    }
                                />
                            </Portal>

                        </Portal>
                    }
                }else{
                    view! {}.into_view()
                }
            }
        }
    }
}
