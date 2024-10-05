use std::collections::HashMap;

use leptos::{create_effect, document, store_value, with_owner, Owner, SignalSetter, StoredValue};
use leptos_use::use_event_listener;
use web_sys::Element;

#[derive(Debug)]
pub enum TouchEvent {
    TouchEnter,
    TouchLeave,
}

#[derive(Clone)]
pub struct TouchManager {
    listeners: StoredValue<Vec<(Element, SignalSetter<TouchEvent>)>>,
}

impl TouchManager {
    pub fn new() -> Self {
        let touches = store_value(HashMap::<i32, Option<Element>>::new());
        let listeners = store_value(vec![]);

        let handle_touch_start_move = move |ev: leptos::ev::TouchEvent| {
            for i in 0..ev.changed_touches().length() {
                if let Some(touch) = ev.changed_touches().get(i) {
                    let el =
                        document().element_from_point(touch.page_x() as f32, touch.page_y() as f32);
                    if let Some(element) = el {
                        let previous_element = touches
                            .with_value(|t| t.get(&touch.identifier()).map(|p| p.to_owned()))
                            .flatten();
                        if let Some(prv_el) = previous_element {
                            if let Some(notif) = listeners.with_value(|l| {
                                l.iter()
                                    .find(|l: &&(Element, SignalSetter<TouchEvent>)| l.0 == prv_el)
                                    .cloned()
                            }) {
                                if notif.0 != element {
                                    notif.1.set(TouchEvent::TouchLeave);
                                }
                            }
                        } else if let Some(notif) = listeners.with_value(|l| {
                            l.iter()
                                .find(|l: &&(Element, SignalSetter<TouchEvent>)| l.0 == element)
                                .cloned()
                        }) {
                            notif.1.set(TouchEvent::TouchEnter)
                        }

                        touches.update_value(|touches| {
                            touches.insert(touch.identifier(), Some(element));
                        });
                    } else {
                        let previous_element = touches
                            .with_value(|t| t.get(&touch.identifier()).map(|p| p.to_owned()))
                            .flatten();
                        if let Some(prv_el) = previous_element {
                            if let Some(notif) = listeners.with_value(|l| {
                                l.iter()
                                    .find(|l: &&(Element, SignalSetter<TouchEvent>)| l.0 == prv_el)
                                    .cloned()
                            }) {
                                notif.1.set(TouchEvent::TouchLeave);
                            }
                        }

                        touches.update_value(|touches| {
                            touches.insert(touch.identifier(), None);
                        });
                    }
                }
            }
        };

        let handle_touch_end_cancel = move |ev: leptos::ev::TouchEvent| {
            for i in 0..ev.changed_touches().length() {
                if let Some(touch) = ev.changed_touches().get(i) {
                    let previous_element = touches
                        .with_value(|t| t.get(&touch.identifier()).map(|p| p.to_owned()))
                        .flatten();
                    if let Some(prv_el) = previous_element {
                        if let Some(notif) = listeners.with_value(|l| {
                            l.iter()
                                .find(|l: &&(Element, SignalSetter<TouchEvent>)| l.0 == prv_el)
                                .cloned()
                        }) {
                            notif.1.set(TouchEvent::TouchLeave);
                        }
                    }

                    touches.update_value(|touches| {
                        touches.remove(&touch.identifier());
                    });
                }
            }
        };

        let owner = Owner::current();
        if let Some(owner) = owner {
            create_effect(move |_| {
                with_owner(owner, || {
                    let _ = use_event_listener(
                        document(),
                        leptos::ev::touchstart,
                        handle_touch_start_move,
                    );
                    let _ = use_event_listener(
                        document(),
                        leptos::ev::touchmove,
                        handle_touch_start_move,
                    );
                    let _ = use_event_listener(
                        document(),
                        leptos::ev::touchcancel,
                        handle_touch_end_cancel,
                    );

                    let _ = use_event_listener(
                        document(),
                        leptos::ev::touchend,
                        handle_touch_end_cancel,
                    );
                })
            });
        }
        Self { listeners }
    }

    pub fn register_listener(&self, element: Element, callback: SignalSetter<TouchEvent>) {
        self.listeners.update_value(|map| {
            if let Some(el) = map.iter_mut().find(|m| m.0 == element) {
                el.1 = callback;
            } else {
                map.push((element, callback));
            }
        });
    }
}

impl Default for TouchManager {
    fn default() -> Self {
        Self::new()
    }
}
