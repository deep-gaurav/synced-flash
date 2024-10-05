use leptos::*;
use leptos_use::use_event_listener;
use logging::warn;
use tracing::info;

use crate::components::gamepad::Gamepad;
use crate::components::portal::Portal;
use crate::utils::keycode::KeyEvent;
use crate::MountPoints;

#[component]
pub fn VirtualButtons(
    event_rx: ReadSignal<Option<KeyEvent>>,
    event_sender: WriteSignal<Option<KeyEvent>>,
) -> impl IntoView {
    let MountPoints {
        speaker_point,
        main_screen,
        ..
    } = expect_context::<MountPoints>();

    let (is_fullscreen, set_is_fullscreen) = create_signal(false);

    create_effect(move |_| {
        info!("Register fullscreenchange");
        let _ = use_event_listener(document(), leptos::ev::fullscreenchange, move |_| {
            set_is_fullscreen.set(document().fullscreen_element().is_some());
        });
    });
    create_effect(move |_| {
        if !is_fullscreen.get() {
            if let Ok(screen) = window().screen() {
                if let Err(err) = screen.orientation().unlock() {
                    warn!("Cant unlock orientation {err:?}")
                }
            }
        }
    });
    view! {
        <Gamepad keys_sender=event_sender />

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
                                    if let Some(div)= main_screen.get_untracked(){
                                        if let Err(err) = div.request_fullscreen() {
                                            warn!("Cannot enter full screen {err:?}")
                                        } else if let Ok(screen) = window().screen() {
                                            if let Err(err) = screen
                                                .orientation()
                                                .lock(web_sys::OrientationLockType::Landscape)
                                            {
                                                warn!("Cant lock orientation {err:?}")
                                            }
                                        }
                                    }
                                }
                            >
                                "Full Screen"
                            </button>
                        </Portal>
                    }
                }else{
                    view! {}.into_view()
                }
            }
        }
    }
}
