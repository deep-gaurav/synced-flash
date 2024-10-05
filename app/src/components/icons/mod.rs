use leptos::*;

pub enum Icons {
    Gamepad,
    Edit,
}

impl Icons {
    pub fn svg(&self) -> &'static str {
        match self {
            Icons::Gamepad => include_str!("gamepad.svg"),
            Icons::Edit => include_str!("edit.svg"),
        }
    }
}

#[component]
pub fn Icon(icon: Icons, #[prop(into, optional)] class: Option<TextProp>) -> impl IntoView {
    view! {
        <span class=class inner_html=icon.svg()>
        </span>
    }
}
