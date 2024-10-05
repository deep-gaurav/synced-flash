use ::svg::node::{element::path::Data, Value};
use leptos::*;
use strum::IntoEnumIterator;
use tracing::info;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::Element;

use crate::{
    components::{icons::Icon, touchmanager::TouchManager},
    utils::keycode::{Key, KeyEvent},
};

#[derive(Clone)]
struct GamepadButton {
    id: Uuid,
    position: RwSignal<(f32, f32)>,
    translation: RwSignal<(f32, f32)>,
    width: RwSignal<f32>,
    scale: RwSignal<f32>,
    button: GamepadButtonType,
}

#[derive(Clone)]
enum GamepadButtonType {
    Dpad(RwSignal<(Key, Key, Key, Key)>),
    Button(RwSignal<Key>),
}

#[component]
pub fn Gamepad(keys_sender: WriteSignal<Option<KeyEvent>>) -> impl IntoView {
    let (key_rx, key_tx) = create_signal(None);

    let (selected_button, set_selected_button) = create_signal(None);
    let (gamebuttons, set_game_buttons) = create_signal(vec![
        GamepadButton {
            id: uuid::Uuid::new_v4(),
            position: RwSignal::new((10.0, 60.0)),
            translation: RwSignal::new((0.0, 0.0)),
            width: RwSignal::new(20.0),
            scale: RwSignal::new(1.0),
            button: GamepadButtonType::Dpad(RwSignal::new((Key::A, Key::D, Key::W, Key::S))),
        },
        GamepadButton {
            id: uuid::Uuid::new_v4(),
            position: RwSignal::new((80.0, 70.0)),
            translation: RwSignal::new((0.0, 0.0)),
            width: RwSignal::new(5.0),
            scale: RwSignal::new(1.0),
            button: GamepadButtonType::Button(RwSignal::new(Key::Z)),
        },
        GamepadButton {
            id: uuid::Uuid::new_v4(),
            position: RwSignal::new((90.0, 70.0)),
            translation: RwSignal::new((0.0, 0.0)),
            width: RwSignal::new(5.0),
            scale: RwSignal::new(1.0),
            button: GamepadButtonType::Button(RwSignal::new(Key::X)),
        },
        GamepadButton {
            id: uuid::Uuid::new_v4(),
            position: RwSignal::new((85.0, 60.0)),
            translation: RwSignal::new((0.0, 0.0)),
            width: RwSignal::new(5.0),
            scale: RwSignal::new(1.0),
            button: GamepadButtonType::Button(RwSignal::new(Key::C)),
        },
        GamepadButton {
            id: uuid::Uuid::new_v4(),
            position: RwSignal::new((85.0, 80.0)),
            translation: RwSignal::new((0.0, 0.0)),
            width: RwSignal::new(5.0),
            scale: RwSignal::new(1.0),
            button: GamepadButtonType::Button(RwSignal::new(Key::V)),
        },
        GamepadButton {
            id: uuid::Uuid::new_v4(),
            position: RwSignal::new((75.0, 82.5)),
            translation: RwSignal::new((0.0, 0.0)),
            width: RwSignal::new(5.0),
            scale: RwSignal::new(1.0),
            button: GamepadButtonType::Button(RwSignal::new(Key::Space)),
        },
        GamepadButton {
            id: uuid::Uuid::new_v4(),
            position: RwSignal::new((92.5, 50.0)),
            translation: RwSignal::new((0.0, 0.0)),
            width: RwSignal::new(5.0),
            scale: RwSignal::new(1.0),
            button: GamepadButtonType::Button(RwSignal::new(Key::Enter)),
        },
    ]);

    let (is_editing_mode, set_is_editing_mode) = create_signal(false);
    let (is_enabled, set_is_enabled) = create_signal(false);
    create_effect(move |_| {
        if let Some(key) = key_rx.get() {
            info!("Ev {key:?}");

            if !is_editing_mode.get_untracked() {
                info!("Sending");
                keys_sender.set(Some(key));
            } else {
                info!("Not ending")
            }
        }
    });
    let (edit_position, set_editpostion) = create_signal((0_f32, 0_f32));

    let (is_edit_down, set_is_edit_down) = create_signal(false);
    let (edit_previous_touch, set_edit_previous_touch) = create_signal(None);

    view! {
        <div class="fixed z-50 top-0 left-0 w-full h-full flex items-center justify-center pointer-events-none select-none">

            <button
                type="button"
                class="absolute right-2 top-2 h-10 w-10 bg-blue-400 active:bg-blue-800 p-2 rounded-md pointer-events-auto"
                title="Toggle Gamepad"
                on:click=move|_|{
                    set_is_enabled.set(!is_enabled.get_untracked());
                }
            >
                <Icon icon=crate::components::icons::Icons::Gamepad />
            </button>

            <Show when={move || is_enabled.get()}>

                <button
                    type="button"
                    class="absolute right-14 top-2 h-10 w-10 bg-blue-400 active:bg-blue-800 p-2 rounded-md pointer-events-auto"
                    title="Toggle Edit Mode"
                    on:click=move|_|{
                        set_is_editing_mode.set(!is_editing_mode.get_untracked());
                        set_selected_button.set(None);
                    }
                >
                    <Icon icon=crate::components::icons::Icons::Edit />
                </button>

                <For each=move||gamebuttons.get()
                    key=move|button|button.id
                    let:button
                >
                    {
                        let (is_down, set_is_down) = create_signal(false);

                        let (previous_touch, set_previous_touch) = create_signal(None);

                        let is_selected = create_memo(move |_| selected_button.get() == Some(button.id));

                        view! {
                            <div
                                style=move||format!(
                                    "width: {}%; left: calc({}% + {}px); top: calc({}% + {}px); scale: {};",
                                    button.width.get(),
                                    button.position.get().0, button.translation.get().0,
                                    button.position.get().1,button.translation.get().1,
                                    button.scale.get(),
                                )
                                class="absolute pointer-events-auto"
                                on:click=move|ev|{
                                    if is_editing_mode.get_untracked(){
                                        set_selected_button.set(Some(button.id));
                                        ev.prevent_default();
                                    }
                                }
                                on:mousedown=move|_|{
                                    if is_editing_mode.get_untracked(){
                                        set_is_down.set(true);
                                    }
                                }
                                on:mouseup=move|_|{
                                    if is_editing_mode.get_untracked(){
                                        set_is_down.set(false);
                                    }
                                }
                                on:touchstart=move|ev|{
                                    if is_editing_mode.get_untracked(){
                                        let changed_touches = ev.changed_touches();
                                        if let Some(touch) = changed_touches.get(0) {
                                            set_previous_touch.set(Some((touch.page_x(), touch.page_y())));
                                        }
                                    }
                                }
                                on:touchend=move|_|{
                                    if is_editing_mode.get_untracked(){
                                        set_previous_touch.set(None);
                                    }
                                }
                                on:touchmove=move|ev|{
                                    info!("touch move");
                                    if is_editing_mode.get_untracked(){
                                        if let Some(previous_touch) = previous_touch.get_untracked() {
                                            let changed_touches = ev.changed_touches();
                                            if let Some(touch) = changed_touches.get(0) {
                                                let (x,y) = button.translation.get_untracked();
                                                button.translation.set(
                                                    (x+touch.page_x() as f32 - previous_touch.0 as f32, y+touch.page_y() as f32 - previous_touch.1 as f32)
                                                );
                                                set_previous_touch.set(Some((touch.page_x(),touch.page_y())))
                                            }
                                        }
                                    }
                                }
                                on:mousemove=move|ev|{
                                    if is_editing_mode.get_untracked() && is_selected.get_untracked() &&  is_down.get_untracked() {
                                        let (x,y) = button.translation.get_untracked();
                                        button.translation.set(
                                            (x+ev.movement_x() as f32, y+ev.movement_y() as f32)
                                        );
                                    }
                                }
                            >

                            {
                                match button.button {
                                    GamepadButtonType::Dpad(keys) => {
                                        view! {

                                            <GamepadDPad
                                                key_tx=key_tx
                                                keys=keys
                                                is_selected={is_selected}
                                            />
                                        }.into_view()
                                    },
                                    GamepadButtonType::Button(key) => {
                                        view! {
                                            <SingleButton key=key key_tx=key_tx
                                                is_selected=is_selected
                                            />
                                        }.into_view()
                                    },
                                }
                            }
                            </div>
                        }

                    }
                </For>
            </Show>

            <Show when={move|| is_editing_mode.get()}>
                <div class="absolute p-2 bg-blue-300 rounded-md top flex flex-col gap-2 text-black border-2 pointer-events-auto"
                    style=move||format!(
                        "left: {}px; top: {}px;",
                        edit_position.get().0,
                        edit_position.get().1,
                    )

                    on:mousedown=move|_|{
                        set_is_edit_down.set(true);
                    }
                    on:mouseup=move|_|{
                        set_is_edit_down.set(false);
                    }
                    on:mouseleave=move|_|{
                        set_is_edit_down.set(false);
                    }
                    on:touchstart=move|ev|{
                        let changed_touches = ev.changed_touches();
                        if let Some(touch) = changed_touches.get(0) {
                            set_edit_previous_touch.set(Some((touch.page_x(), touch.page_y())));
                        }
                    }
                    on:touchend=move|_|{
                        set_edit_previous_touch.set(None);
                    }
                    on:touchmove=move|ev|{
                        if let Some(previous_touch) = edit_previous_touch.get_untracked() {
                            let changed_touches = ev.changed_touches();
                            if let Some(touch) = changed_touches.get(0) {
                                let (x,y) = edit_position.get_untracked();
                                set_editpostion.set(
                                    (x+touch.page_x() as f32 - previous_touch.0 as f32, y+touch.page_y() as f32 - previous_touch.1 as f32)
                                );
                                set_edit_previous_touch.set(Some((touch.page_x(),touch.page_y())))
                            }
                        }
                    }
                    on:mousemove=move|ev|{
                        if is_edit_down.get_untracked() {
                            let (x,y) = edit_position.get_untracked();
                            set_editpostion.set(
                                (x+ev.movement_x() as f32, y+ev.movement_y() as f32)
                            );
                        }
                    }
                >
                    <div class="text-center text-sm"> "Edit Controls" </div>
                    {
                        move || if let Some(selected_button) = selected_button.get().and_then(|button_id|
                            gamebuttons.get_untracked().into_iter().find(|b|b.id == button_id)
                        ){
                            view! {
                                <div class="flex gap-2 text-xs items-center">
                                    <button type="button" class="bg-blue-500 px-2 py-1 active:bg-blue-900"
                                        on:click=move|_|{
                                            selected_button.scale.update(|scale|*scale -= 0.05);
                                        }
                                    > "-" </button>
                                    <div class="flex-grow" />
                                    <span> "Scale" </span>
                                    <div class="flex-grow" />
                                    <button type="button" class="bg-blue-500 px-2 py-1 active:bg-blue-900"
                                        on:click=move|_|{
                                            selected_button.scale.update(|scale|*scale += 0.05);
                                        }
                                    > "+" </button>
                                </div>

                                {
                                    match selected_button.button {
                                        GamepadButtonType::Dpad(keys) => {

                                            let keys_info = vec![
                                                ("Left Key", Signal::derive(move||keys.get().0), SignalSetter::map(move|k:Key|{
                                                    let keys_old = keys.get();
                                                    keys.set((k,keys_old.1,keys_old.2,keys_old.3));
                                                })),
                                                ("Right Key", Signal::derive(move||keys.get().1), SignalSetter::map(move|k:Key|{
                                                    let keys_old = keys.get();
                                                    keys.set((keys_old.0,k,keys_old.2,keys_old.3));
                                                })),

                                                ("Up Key", Signal::derive(move||keys.get().2), SignalSetter::map(move|k:Key|{
                                                    let keys_old = keys.get();
                                                    keys.set((keys_old.0,keys_old.1,k,keys_old.3));
                                                })),

                                                ("Down Key", Signal::derive(move||keys.get().3), SignalSetter::map(move|k:Key|{
                                                    let keys_old = keys.get();
                                                    keys.set((keys_old.0,keys_old.1,keys_old.2,k));
                                                })),
                                            ];
                                            view! {
                                                {
                                                    keys_info.into_iter().map(|(title, key_get,key_set)|{
                                                        view! {
                                                            <div class="flex gap-2 text-xs items-center">
                                                                <span> {title} </span>
                                                                <div class="flex-grow" />
                                                                <select
                                                                    class="bg-blue-500 px-2 py-1 active:bg-blue-900"
                                                                    prop:value=move||key_get.get().get_symbol()
                                                                    on:change=move |ev| {
                                                                        let new_value = event_target_value(&ev);
                                                                        let k = Key::iter().find(|k|k.get_symbol()== new_value);
                                                                        if let Some(k) = k {
                                                                            key_set.set(k)
                                                                        }
                                                                    }
                                                                >
                                                                    {
                                                                        Key::iter().map(|k|
                                                                            view! {
                                                                                <option selected={move||k.get_symbol() == key_get.get().get_symbol() } value={k.get_symbol()}> {k.get_symbol()} </option>
                                                                            }
                                                                        ).collect_view()
                                                                    }
                                                                </select>
                                                            </div>
                                                        }
                                                    }).collect_view()
                                                }
                                            }.into_view()
                                        },
                                        GamepadButtonType::Button(key) => {
                                            view! {
                                                <div class="flex gap-2 text-xs items-center">

                                                    <span> "Assign Key" </span>
                                                    <div class="flex-grow" />
                                                    <select
                                                        class="bg-blue-500 px-2 py-1 active:bg-blue-900"
                                                        prop:value=move||key.get().get_symbol()
                                                        on:change=move |ev| {
                                                            let new_value = event_target_value(&ev);
                                                            let k = Key::iter().find(|k|k.get_symbol() == new_value);
                                                            if let Some(k) = k {
                                                                key.set(k);
                                                            }
                                                        }
                                                    >
                                                        {
                                                            Key::iter().map(|k|
                                                                view! {
                                                                    <option selected={move||k.get_symbol() == key.get().get_symbol() } value={k.get_symbol()}> {k.get_symbol()} </option>
                                                                }
                                                            ).collect_view()
                                                        }
                                                    </select>
                                                </div>
                                            }.into_view()
                                        },
                                    }
                                }
                            }.into_view()
                        }else{
                            view! {}.into_view()
                        }
                    }
                </div>
            </Show>
        </div>
    }
}

#[component]
fn GamepadDPad(
    key_tx: WriteSignal<Option<KeyEvent>>,
    keys: RwSignal<(Key, Key, Key, Key)>,

    is_selected: Memo<bool>,
) -> impl IntoView {
    let (svg_width, svg_height) = (125.0, 100.0);

    let center_height = 40.0 / 100.0 * svg_height;

    let center_angle = 5_f64.to_radians();
    let verticle_angle = 50_f64.to_radians();

    let e_y = center_angle.sin() * svg_width;
    let e_x = center_angle.cos() * svg_width;

    let extra =
        ((svg_width.powi(2) - (center_height / 2_f64 + e_y).powi(2)).sqrt() - svg_width).abs();

    let calculated_angle = (-e_y / ((-e_x + (extra * 2.0)) / 2.0)).tanh();

    // Left Fan
    let left_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0))
            .line_by((0, -center_height / 2.0))
            .line_by(((-e_x + (extra * 2.0)) / 2.0, -e_y))
            .elliptical_arc_by((svg_width, svg_width, 0, 0, 0, 0, center_height + 2.0 * e_y))
            // .line_by(())
            .line_by(((e_x - (extra * 2.0)) / 2.0, -e_y))
            .line_by((0, -center_height / 2.0))
            .close(),
    )
    .to_string();

    let right_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0))
            .line_by((0, -center_height / 2.0))
            .line_by(((e_x - (extra * 2.0)) / 2.0, -e_y))
            .elliptical_arc_by((svg_width, svg_width, 0, 0, 1, 0, center_height + 2.0 * e_y))
            // .line_by(())
            .line_by(((-e_x + (extra * 2.0)) / 2.0, -e_y))
            .line_by((0, -center_height / 2.0))
            .close(),
    )
    .to_string();

    // Top Fan

    let veritcle_fan_height = (svg_height - center_height) / 2.0;

    let (tx, ty) = (
        verticle_angle.sin() * veritcle_fan_height,
        verticle_angle.cos() * veritcle_fan_height,
    );

    let top_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0 - center_height / 2.0))
            .line_by((tx, -ty))
            .elliptical_arc_by((
                veritcle_fan_height,
                veritcle_fan_height,
                0,
                0,
                0,
                -2.0 * tx,
                0,
            ))
            .close(),
    )
    .to_string();

    let bottom_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0 + center_height / 2.0))
            .line_by((tx, ty))
            .elliptical_arc_by((
                veritcle_fan_height,
                veritcle_fan_height,
                0,
                0,
                1,
                -2.0 * tx,
                0,
            ))
            .close(),
    )
    .to_string();

    // Top Left Fan

    let corner_fan_radius = veritcle_fan_height * 0.8;

    let (tx1, ty1) = (
        calculated_angle.cos() * corner_fan_radius,
        calculated_angle.sin() * corner_fan_radius,
    );
    let extra_angle = 90_f64.to_radians() - calculated_angle - verticle_angle;
    let (tx2, ty2) = (
        (extra_angle + calculated_angle).cos() * corner_fan_radius,
        (extra_angle + calculated_angle).sin() * corner_fan_radius,
    );

    let top_left_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0 - center_height / 2.0))
            .line_by((-tx1, -ty1))
            .elliptical_arc_to((
                corner_fan_radius,
                corner_fan_radius,
                0,
                0,
                1,
                svg_width / 2.0 - tx2,
                svg_height / 2.0 - center_height / 2.0 - ty2,
            ))
            .close(),
    )
    .to_string();

    let top_right_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0 - center_height / 2.0))
            .line_by((tx1, -ty1))
            .elliptical_arc_to((
                corner_fan_radius,
                corner_fan_radius,
                0,
                0,
                0,
                svg_width / 2.0 + tx2,
                svg_height / 2.0 - center_height / 2.0 - ty2,
            ))
            .close(),
    )
    .to_string();

    let bottom_left_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0 + center_height / 2.0))
            .line_by((-tx1, ty1))
            .elliptical_arc_to((
                corner_fan_radius,
                corner_fan_radius,
                0,
                0,
                0,
                svg_width / 2.0 - tx2,
                svg_height / 2.0 + center_height / 2.0 + ty2,
            ))
            .close(),
    )
    .to_string();

    let bottom_right_fan = Value::from(
        Data::new()
            .move_to((svg_width / 2.0, svg_height / 2.0 + center_height / 2.0))
            .line_by((tx1, ty1))
            .elliptical_arc_to((
                corner_fan_radius,
                corner_fan_radius,
                0,
                0,
                1,
                svg_width / 2.0 + tx2,
                svg_height / 2.0 + center_height / 2.0 + ty2,
            ))
            .close(),
    )
    .to_string();

    let path_class = move || {
        format!(
        "fill-blue-800/75 duration-200 transition-all hover:fill-blue-900 active:fill-blue-950 {}",
        if is_selected.get() {
            "stroke-2 stroke-yellow-500"
        } else {
            ""
        }
    )
    };

    let l_ref = create_node_ref::<leptos::svg::Path>();
    let r_ref = create_node_ref::<leptos::svg::Path>();
    let t_ref = create_node_ref::<leptos::svg::Path>();
    let b_ref = create_node_ref::<leptos::svg::Path>();

    create_effect(move |_| {
        if let (Some(l), Some(r), Some(t), Some(b)) =
            (l_ref.get(), r_ref.get(), t_ref.get(), b_ref.get())
        {
            let touch_manager = expect_context::<TouchManager>();
            let evs = vec![
                (l, keys.get_untracked().0),
                (r, keys.get_untracked().1),
                (t, keys.get_untracked().2),
                (b, keys.get_untracked().3),
            ];

            for (el, key) in evs {
                touch_manager.register_listener(
                    {
                        let el: &Element = el.as_ref();
                        el.clone()
                    },
                    SignalSetter::map(move |ev| match ev {
                        crate::components::touchmanager::TouchEvent::TouchEnter => {
                            key_tx.set(Some(KeyEvent::Down(key)));
                        }
                        crate::components::touchmanager::TouchEvent::TouchLeave => {
                            key_tx.set(Some(KeyEvent::Up(key)));
                        }
                    }),
                );
            }
        }
    });

    view! {

        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox=format!("0 0 {svg_width} {svg_height}")
        >

            {

                let (left_fan, right_fan, top_fan, bottom_fan) = (left_fan.clone(), right_fan.clone(), top_fan.clone(), bottom_fan.clone());
                move || if !is_selected.get() {
                    view! {
                        <defs>
                            <mask id="mask1">
                                <path
                                    fill="#fff"
                                    stroke="black"
                                    stroke-width=3
                                    d={left_fan.clone()}
                                />
                                <path
                                    fill="#fff"
                                    stroke="black"
                                    stroke-width=3
                                    d={right_fan.clone()}
                                />
                                <path
                                    fill="#fff"
                                    stroke="black"
                                    stroke-width=3
                                    d={top_fan.clone()}
                                />
                                <path
                                    fill="#fff"
                                    stroke="black"
                                    stroke-width=3
                                    d={bottom_fan.clone()}
                                />
                            </mask>
                        </defs>
                    }.into_view()
                }else{
                    view! {}.into_view()
                }
            }
            //Left Fan
            <path
                ref=l_ref
                on:mousedown=move|_|{
                    key_tx.set(Some(KeyEvent::Down(keys.get().0)));
                }
                on:mouseup=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().0)));
                }
                on:mouseleave=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().0)));
                }
                class=path_class
                d={left_fan}
                mask="url(#mask1)"
            />

            //Right Fan
            <path
                ref=r_ref
                on:mousedown=move|_|{
                    key_tx.set(Some(KeyEvent::Down(keys.get().1)));
                }
                on:mouseup=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().1)));
                }
                on:mouseleave=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().1)));
                }
                class=path_class
                d={right_fan}
                mask="url(#mask1)"
            />

            //Top Fan
            <path
                ref=t_ref
                on:mousedown=move|_|{
                    key_tx.set(Some(KeyEvent::Down(keys.get().2)));
                }
                on:mouseup=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().2)));
                }
                on:mouseleave=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().2)));
                }

                class=path_class
                d={top_fan}
                mask="url(#mask1)"
            />

            //Bottom Fan
            <path
                ref=b_ref
                on:mousedown=move|_|{
                    key_tx.set(Some(KeyEvent::Down(keys.get().3)));
                }
                on:mouseup=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().3)));
                }
                on:mouseleave=move|_|{
                    key_tx.set(Some(KeyEvent::Up(keys.get().3)));
                }
                class=path_class
                d={bottom_fan}
                mask="url(#mask1)"
            />

            //Top Left Fan
            <path
                class=path_class
                d={top_left_fan}
            />

            //Top Right Fan
            <path
                class=path_class
                d={top_right_fan}
            />

            //Bottom Left Fan
            <path
                class=path_class
                d={bottom_left_fan}
            />

            //Bottom Right Fan
            <path
                class=path_class
                d={bottom_right_fan}
            />
        </svg>

    }
}

#[component]
fn SingleButton(
    key: RwSignal<Key>,
    key_tx: WriteSignal<Option<KeyEvent>>,
    is_selected: Memo<bool>,
) -> impl IntoView {
    view! {

        <div class="w-full aspect-[cos(30deg)]">


            <div
                on:mousedown=move|_|{
                    key_tx.set(Some(KeyEvent::Down(key.get())));
                }
                on:mouseup=move|_|{
                    key_tx.set(Some(KeyEvent::Up(key.get())));
                }
                on:mouseleave=move|_|{
                    key_tx.set(Some(KeyEvent::Up(key.get())));
                }
                class="absolute hexagon-filled bg-blue-400/50 active:bg-blue-900 left-0 top-0 w-full h-full flex items-center justify-center">
                {
                    move || key.get().get_symbol()
                }
            </div>

            <div class="absolute w-full hexagon bg-blue-800 transition-all duration-200"
                class=(["bg-yellow-500", "hexagon-selected"], is_selected)
            />
            <div class="w-[90%] h-[90%] hexagon bg-blue-800 absolute left-[5%] top-[5%]"
                class=("hidden", move|| !is_selected.get())
             />
        </div>
    }
}
