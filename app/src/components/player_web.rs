use std::{
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use leptos::*;
use leptos_use::use_raf_fn;
use ruffle_core::{
    backend::{audio::NullAudioBackend, log::LogBackend, storage::MemoryStorageBackend},
    compatibility_rules::CompatibilityRules,
    events::KeyCode,
    tag_utils::SwfMovie,
    Player, PlayerBuilder, PlayerRuntime, SandboxType, StageAlign, StageScaleMode,
    ViewportDimensions,
};
use ruffle_render::{backend::RenderBackend, quality::StageQuality};
use std::error::Error;
use tracing::{info, warn};
use url::Url;
use wasm_bindgen::JsValue;
use web_sys::js_sys;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ruffle_core::backend::storage::StorageBackend;
use web_sys::Storage;

use super::virtual_buttons::{Key, KeyEvent};

#[component]
pub fn PlayerWeb(
    canvas_ref: NodeRef<leptos::html::Canvas>,
    swf_data: ReadSignal<Option<(String, Vec<u8>)>>,
    key_event_rx: ReadSignal<Option<KeyEvent>>,
) -> impl IntoView {
    let (player, set_player) = create_signal(Option::<Arc<Mutex<ruffle_core::Player>>>::None);
    let (timestamp, set_timestamp) = create_signal(None);
    let (canvas_data, set_canvas_data) = create_signal((0, 0, window().device_pixel_ratio()));

    create_effect(move |_| {
        if let Some((swf_name, swf_data)) = swf_data.get() {
            if let Some(player) = player.get() {
                match load_swf(&swf_data, &swf_name, player.clone()) {
                    Ok(_) => {
                        if let Ok(core) = &mut player.lock() {
                            core.set_is_playing(true);
                        }
                    }
                    Err(err) => {
                        warn!("Cannot load swf {err:?}");
                    }
                }
            }
        }
    });

    create_effect(move |_| {
        if let Some(player) = player.get() {
            if let Some(event) = key_event_rx.get() {
                if let Ok(player) = &mut player.lock() {
                    let ruffleevent = event.ruffle_event();
                    // info!("Sending event {ruffleevent:?}");
                    // info!("Is mouse in stage {}", player.mouse_in_stage());
                    let is_handled = player.handle_event(ruffleevent);
                    // info!("Is handled {is_handled}")
                }
            }
        }
    });
    create_effect(move |_| {
        if player.get().is_some() {
            return;
        }

        if let Some(canvas) = canvas_ref.get() {
            let canvas_ref: &web_sys::HtmlCanvasElement = canvas.as_ref();
            let canvas_element: web_sys::HtmlCanvasElement = canvas_ref.clone();
            let quality = ruffle_render::quality::StageQuality::High;
            leptos::spawn_local(async move {
                let rendere_backend = create_renderer(canvas_element, quality).await;
                if let Ok(renderer) = rendere_backend {
                    let mut player_builder = PlayerBuilder::new()
                        .with_storage(Box::new(MemoryStorageBackend::new()))
                        .with_audio(NullAudioBackend::new())
                        .with_boxed_renderer(renderer)
                        .with_log(WebLogBackend::new())
                        // .with_ui(ui::WebUiBackend::new(js_player.clone(), &canvas))
                        .with_letterbox(ruffle_core::config::Letterbox::Fullscreen)
                        .with_max_execution_duration(Duration::from_secs_f64(15.0))
                        .with_player_version(None)
                        .with_player_runtime(PlayerRuntime::FlashPlayer)
                        .with_compatibility_rules(CompatibilityRules::default())
                        .with_quality(quality)
                        .with_align(StageAlign::empty(), false)
                        .with_scale_mode(StageScaleMode::ShowAll, false)
                        .with_frame_rate(None)
                        // // FIXME - should this be configurable?
                        .with_sandbox_type(SandboxType::Remote)
                        .with_page_url(window().location().href().ok());
                    #[cfg(feature = "ruffle_video_software")]
                    {
                        use ruffle_video_software::backend::SoftwareVideoBackend;
                        player_builder = player_builder.with_video(SoftwareVideoBackend::new());
                    }

                    let player_builder = player_builder.build();
                    if let Ok(player) = &mut player_builder.lock() {
                        player.set_window_mode("window");
                    }
                    set_player.set(Some(player_builder));
                }
            });
        }
    });

    use_raf_fn(move |time| {
        if let (Some(player), Some(canvas)) = (player.get_untracked(), canvas_ref.get_untracked()) {
            let (old_width, old_height, old_ratio) = canvas_data.get_untracked();
            let mut new_dimensions = None;
            let device_pixel_ratio = window().device_pixel_ratio();
            let (canvas_width, canvas_height) = (canvas.client_width(), canvas.client_height());
            if canvas_height != old_height
                || canvas_width != old_width
                || device_pixel_ratio != old_ratio
            {
                set_canvas_data.set((canvas_width, canvas_height, device_pixel_ratio));

                new_dimensions = Some((
                    (f64::from(canvas_width) * device_pixel_ratio) as u32,
                    (f64::from(canvas_height) * device_pixel_ratio) as u32,
                    device_pixel_ratio,
                ));
            }
            let dt = timestamp
                .get_untracked()
                .map_or(0.0, |prev_timestamp| time.timestamp - prev_timestamp);
            set_timestamp.set(Some(time.timestamp));

            if let Ok(core) = &mut player.lock() {
                if let Some((viewport_width, viewport_height, device_pixel_ratio)) = new_dimensions
                {
                    info!("Set width {viewport_width} height {viewport_height}");
                    canvas.set_width(viewport_width);
                    canvas.set_height(viewport_height);

                    core.set_viewport_dimensions(ViewportDimensions {
                        width: viewport_width,
                        height: viewport_height,
                        scale_factor: device_pixel_ratio,
                    });
                }

                // info!("Tick with dt {dt}");
                core.tick(dt);

                // Render if the core signals a new frame, or if we resized.
                if core.needs_render() || new_dimensions.is_some() {
                    core.render();
                }
            }
        }
    });

    view! {}
}

pub async fn create_renderer(
    canvas: web_sys::HtmlCanvasElement,
    quality: StageQuality,
) -> Result<Box<dyn RenderBackend>, Box<dyn Error>> {
    #[cfg(not(target_family = "wasm"))]
    return Err("Only wasm is supported target".into());

    let window = web_sys::window().ok_or("Expected window")?;
    let document = window.document().ok_or("Expected document")?;
    #[cfg(all(
        target_family = "wasm",
        not(any(
            feature = "canvas",
            feature = "webgl",
            feature = "webgpu",
            feature = "wgpu-webgl"
        ))
    ))]
    std::compile_error!("You must enable one of the render backend features (e.g., webgl).");

    let _is_transparent = true;

    let renderer_list = vec![
        // "wgpu-webgl", // Disabled due to stack size error
        "webgpu", "webgl", "canvas",
    ];
    // if let Some(preferred_renderer) = &self.preferred_renderer {
    //     if let Some(pos) = renderer_list.iter().position(|&r| r == preferred_renderer) {
    //         renderer_list.remove(pos);
    //         renderer_list.insert(0, preferred_renderer.as_str());
    //     } else {
    //         tracing::error!("Unrecognized renderer name: {}", preferred_renderer);
    //     }
    // }

    // Try to create a backend, falling through to the next backend on failure.
    // We must recreate the canvas each attempt, as only a single context may be created per canvas
    // with `getContext`.
    for renderer in renderer_list {
        match renderer {
            #[cfg(all(feature = "webgpu", target_family = "wasm"))]
            "webgpu" => {
                // Check that we have access to WebGPU (navigator.gpu should exist).
                if web_sys::window()
                    .ok_or(JsValue::FALSE)
                    .and_then(|window| {
                        js_sys::Reflect::has(&window.navigator(), &JsValue::from_str("gpu"))
                    })
                    .unwrap_or_default()
                {
                    tracing::info!("Creating wgpu webgpu renderer...");

                    match ruffle_render_wgpu::backend::WgpuRenderBackend::for_canvas(
                        canvas.clone(),
                        true,
                    )
                    .await
                    {
                        Ok(renderer) => {
                            return Ok(Box::new(renderer));
                        }
                        Err(error) => {
                            tracing::error!("Error creating wgpu webgpu renderer: {}", error)
                        }
                    }
                }
            }
            #[cfg(all(feature = "wgpu-webgl", target_family = "wasm"))]
            "wgpu-webgl" => {
                tracing::info!("Creating wgpu webgl renderer...");

                match ruffle_render_wgpu::backend::WgpuRenderBackend::for_canvas(
                    canvas.clone(),
                    false,
                )
                .await
                {
                    Ok(renderer) => {
                        return Ok(Box::new(renderer));
                    }
                    Err(error) => {
                        tracing::error!("Error creating wgpu webgl renderer: {}", error)
                    }
                }
            }
            #[cfg(feature = "webgl")]
            "webgl" => {
                tracing::info!("Creating WebGL renderer...");
                match ruffle_render_webgl::WebGlRenderBackend::new(
                    &canvas,
                    _is_transparent,
                    quality,
                ) {
                    Ok(renderer) => {
                        return Ok(Box::new(renderer));
                    }
                    Err(error) => {
                        tracing::error!("Error creating WebGL renderer: {}", error)
                    }
                }
            }
            #[cfg(feature = "canvas")]
            "canvas" => {
                tracing::info!("Creating Canvas renderer...");
                match ruffle_render_canvas::WebCanvasRenderBackend::new(&canvas, _is_transparent) {
                    Ok(renderer) => {
                        return Ok(Box::new(renderer));
                    }
                    Err(error) => tracing::error!("Error creating canvas renderer: {}", error),
                }
            }
            _ => {}
        }
    }
    Err("Unable to create renderer".into())
}

pub struct WebLogBackend {}

impl WebLogBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl LogBackend for WebLogBackend {
    fn avm_trace(&self, message: &str) {
        tracing::info!(target: "avm_trace", "{}", message);
    }
}

pub fn load_swf(data: &[u8], name: &str, player: Arc<Mutex<Player>>) -> Result<(), String> {
    let window = web_sys::window().ok_or("Expected window".to_string())?;
    let mut url = Url::from_str(
        &window
            .location()
            .href()
            .map_err(|e| format!("window location href expected {e:?}"))?,
    )
    .map_err(|e| format!("cant create url {e:?}"))?;
    url.set_query(None);
    url.set_fragment(None);
    if let Ok(mut segments) = url.path_segments_mut() {
        segments.pop();
        segments.push(&name);
    }
    let mut movie = SwfMovie::from_data(data, url.to_string(), None)
        .map_err(|e| format!("Error loading movie: {e}"))?;
    // movie.append_parameters(parse_movie_parameters(&parameters));

    // self.on_metadata(movie.header());

    let mut player = player
        .lock()
        .map_err(|e| format!("cant lock player {e:?}"))?;
    player.update(|uc| {
        uc.replace_root_movie(movie);
    });
    Ok(())
}

impl Key {
    pub fn ruffle_key(&self) -> KeyCode {
        match self {
            Key::UpArrow => KeyCode::UP,
            Key::DownArrow => KeyCode::DOWN,
            Key::LeftArrow => KeyCode::LEFT,
            Key::RightArrow => KeyCode::RIGHT,
        }
    }
}

impl KeyEvent {
    pub fn ruffle_event(&self) -> ruffle_core::events::PlayerEvent {
        match self {
            // KeyEvent::Down(_) => ruffle_core::PlayerEvent::MouseDown {
            //     x: 0.0,
            //     y: 0.0,
            //     button: ruffle_core::events::MouseButton::Left,
            //     index: None,
            // },
            // KeyEvent::Up(_) => ruffle_core::PlayerEvent::MouseUp {
            //     x: 0.0,
            //     y: 0.0,
            //     button: ruffle_core::events::MouseButton::Left,
            //     // index: None,
            // },
            KeyEvent::Down(key) => ruffle_core::PlayerEvent::KeyDown {
                key_code: key.ruffle_key(),
                key_char: None,
            },
            KeyEvent::Up(key) => ruffle_core::PlayerEvent::KeyUp {
                key_code: key.ruffle_key(),
                key_char: None,
            },
            KeyEvent::MouseMove(x, y) => ruffle_core::PlayerEvent::MouseMove { x: *x, y: *y },
            KeyEvent::MouseDown(x, y) => ruffle_core::PlayerEvent::MouseDown {
                x: *x,
                y: *y,
                button: ruffle_core::events::MouseButton::Left,
                index: None,
            },
            KeyEvent::MouseUp(x, y) => ruffle_core::PlayerEvent::MouseUp {
                x: *x,
                y: *y,
                button: ruffle_core::events::MouseButton::Left,
            },
        }
    }
}

pub struct LocalStorageBackend {
    storage: Storage,
}

impl LocalStorageBackend {
    pub(crate) fn new(storage: Storage) -> Self {
        LocalStorageBackend { storage }
    }
}

impl StorageBackend for LocalStorageBackend {
    fn get(&self, name: &str) -> Option<Vec<u8>> {
        if let Ok(Some(data)) = self.storage.get(name) {
            if let Ok(data) = BASE64_STANDARD.decode(data) {
                return Some(data);
            }
        }

        None
    }

    fn put(&mut self, name: &str, value: &[u8]) -> bool {
        self.storage
            .set(name, &BASE64_STANDARD.encode(value))
            .is_ok()
    }

    fn remove_key(&mut self, name: &str) {
        let _ = self.storage.delete(name);
    }
}
