use leptos::*;

use crate::components::portal::Portal;
use crate::MountPoints;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
}

#[derive(Debug, Clone)]
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
pub fn VirtualButtons(event_sender: WriteSignal<Option<KeyEvent>>) -> impl IntoView {
    let MountPoints { speaker_point, .. } = expect_context::<MountPoints>();

    let (virtual_keys, set_virtual_keys) = create_signal(Vec::new());

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
                                            }
                                        ]
                                    )
                                }
                            > "Add Gamepad" </button>

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
