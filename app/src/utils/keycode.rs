use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,

    Space,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    CtrlLeft,
    CtrlRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyEvent {
    Down(Key),
    Up(Key),
    MouseMove(f64, f64),
    MouseDown(f64, f64),
    MouseUp(f64, f64),
}

impl TryFrom<web_sys::KeyboardEvent> for Key {
    type Error = ();

    fn try_from(value: web_sys::KeyboardEvent) -> Result<Self, Self::Error> {
        match value.code().as_str() {
            "KeyA" => Ok(Key::A),
            "KeyB" => Ok(Key::B),
            "KeyC" => Ok(Key::C),
            "KeyD" => Ok(Key::D),
            "KeyE" => Ok(Key::E),
            "KeyF" => Ok(Key::F),
            "KeyG" => Ok(Key::G),
            "KeyH" => Ok(Key::H),
            "KeyI" => Ok(Key::I),
            "KeyJ" => Ok(Key::J),
            "KeyK" => Ok(Key::K),
            "KeyL" => Ok(Key::L),
            "KeyM" => Ok(Key::M),
            "KeyN" => Ok(Key::N),
            "KeyO" => Ok(Key::O),
            "KeyP" => Ok(Key::P),
            "KeyQ" => Ok(Key::Q),
            "KeyR" => Ok(Key::R),
            "KeyS" => Ok(Key::S),
            "KeyT" => Ok(Key::T),
            "KeyU" => Ok(Key::U),
            "KeyV" => Ok(Key::V),
            "KeyW" => Ok(Key::W),
            "KeyX" => Ok(Key::X),
            "KeyY" => Ok(Key::Y),
            "KeyZ" => Ok(Key::Z),

            "ControlLeft" => Ok(Key::CtrlLeft),
            "ControlRight" => Ok(Key::CtrlRight),

            "SPace" => Ok(Key::Space),

            "ArrowUp" => Ok(Key::UpArrow),
            "ArrowDown" => Ok(Key::DownArrow),
            "ArrowLeft" => Ok(Key::LeftArrow),
            "ArrowRight" => Ok(Key::RightArrow),
            _ => Err(()),
        }
    }
}

impl Key {
    pub fn get_symbol(&self) -> String {
        match &self {
            Key::UpArrow => "⬆️".to_string(),
            Key::DownArrow => "⬇️".to_string(),
            Key::LeftArrow => "⬅️".to_string(),
            Key::RightArrow => "➡️".to_string(),

            Key::Space => "".to_string(),
            Key::A => "A".to_string(),
            Key::B => "B".to_string(),
            Key::C => "C".to_string(),
            Key::D => "D".to_string(),
            Key::E => "E".to_string(),
            Key::F => "F".to_string(),
            Key::G => "G".to_string(),
            Key::H => "H".to_string(),
            Key::I => "I".to_string(),
            Key::J => "J".to_string(),
            Key::K => "K".to_string(),
            Key::L => "L".to_string(),
            Key::M => "M".to_string(),
            Key::N => "N".to_string(),
            Key::O => "O".to_string(),
            Key::P => "P".to_string(),
            Key::Q => "Q".to_string(),
            Key::R => "R".to_string(),
            Key::S => "S".to_string(),
            Key::T => "T".to_string(),
            Key::U => "U".to_string(),
            Key::V => "V".to_string(),
            Key::W => "W".to_string(),
            Key::X => "X".to_string(),
            Key::Y => "Y".to_string(),
            Key::Z => "Z".to_string(),

            Key::CtrlLeft => "Ctrl".to_string(),
            Key::CtrlRight => "Ctrl".to_string(),
        }
    }
}
