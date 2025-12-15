// WASM entry point
#![cfg(target_arch = "wasm32")]

mod planet;
mod vector2;
mod game;
mod player;
mod render;
mod initial_universe;
mod keyboard_input;
mod texture;
mod sprite_renderer;
mod font;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, ImageData};
use std::cell::RefCell;
use crate::game::{Game, TRAJECTORY_DT};
use crate::initial_universe::create_universe;
use crate::keyboard_input::InputState;

thread_local! {
    static APP_STATE: RefCell<Option<AppState>> = RefCell::new(None);
}

struct AppState {
    game: Game,
    input_state: InputState,
    time_accumulator: f64,
    last_time: f64,
    zoom_level: f64,
    time_warp: f64,
    show_absolute_trajectories: bool,
    selected_planet: Option<usize>,
    mouse_pos: (f64, f64),
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"WASM module initialized".into());
}

#[wasm_bindgen]
pub fn init_game() {
    let game = create_universe();

    let state = AppState {
        game,
        input_state: InputState::new(),
        time_accumulator: 0.0,
        last_time: js_sys::Date::now(),
        zoom_level: 1.0,
        time_warp: 1.0,
        show_absolute_trajectories: false,
        selected_planet: None,
        mouse_pos: (0.0, 0.0),
    };

    APP_STATE.with(|app| {
        *app.borrow_mut() = Some(state);
    });

    web_sys::console::log_1(&"Game initialized".into());
}

#[wasm_bindgen]
pub fn update_and_render(canvas_id: &str) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id(canvas_id)
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    let width = canvas.width() as usize;
    let height = canvas.height() as usize;

    APP_STATE.with(|app| -> Result<(), JsValue> {
        let mut app_state_ref = app.borrow_mut();
        if let Some(state) = app_state_ref.as_mut() {
            // Update game
            let now = js_sys::Date::now();
            let dt = (now - state.last_time) / 1000.0; // Convert to seconds
            state.last_time = now;

            // Apply input
            state.input_state.apply_to_game(&mut state.game, dt);

            // Accumulate time with time warp multiplier
            state.time_accumulator += dt * state.time_warp;

            let steps_to_advance = (state.time_accumulator / TRAJECTORY_DT) as usize;
            if steps_to_advance > 0 {
                for _ in 0..steps_to_advance {
                    state.game.advance_trajectory();
                }
                state.game.extend_trajectories(steps_to_advance);
                state.time_accumulator -= steps_to_advance as f64 * TRAJECTORY_DT;
            }

            // Render to buffer
            let mut buffer = vec![0u32; width * height];
            crate::render::render_game(
                &mut buffer,
                width,
                height,
                &state.game,
                state.input_state.thrust,
                state.zoom_level,
                state.time_warp,
                state.show_absolute_trajectories,
                state.selected_planet,
            );

            // Convert buffer to ImageData and draw to canvas
            let context = canvas
                .get_context("2d")?
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()?;

            // Convert u32 ARGB to u8 RGBA for ImageData
            let mut rgba_data = vec![0u8; width * height * 4];
            for (i, pixel) in buffer.iter().enumerate() {
                let idx = i * 4;
                rgba_data[idx] = ((pixel >> 16) & 0xFF) as u8;     // R
                rgba_data[idx + 1] = ((pixel >> 8) & 0xFF) as u8;  // G
                rgba_data[idx + 2] = (pixel & 0xFF) as u8;         // B
                rgba_data[idx + 3] = 255;                          // A
            }

            let image_data = ImageData::new_with_u8_clamped_array_and_sh(
                wasm_bindgen::Clamped(&rgba_data),
                width as u32,
                height as u32,
            )?;

            context.put_image_data(&image_data, 0.0, 0.0)?;
        }
        Ok(())
    })?;

    Ok(())
}

#[wasm_bindgen]
pub fn handle_key_down(key_code: &str) {
    APP_STATE.with(|app| {
        if let Some(state) = app.borrow_mut().as_mut() {
            match key_code {
                "KeyA" | "ArrowLeft" => state.input_state.rotate_left = true,
                "KeyD" | "ArrowRight" => state.input_state.rotate_right = true,
                "Space" | "KeyW" | "ArrowUp" => state.input_state.thrust = true,
                "Equal" | "NumpadAdd" => state.zoom_level *= 1.2,
                "Minus" | "NumpadSubtract" => state.zoom_level /= 1.2,
                "Period" => {
                    state.time_warp *= 2.0;
                    state.time_warp = state.time_warp.min(256.0);
                }
                "Comma" => {
                    state.time_warp /= 2.0;
                    state.time_warp = state.time_warp.max(1.0);
                }
                "Tab" => state.show_absolute_trajectories = !state.show_absolute_trajectories,
                _ => {}
            }
        }
    });
}

#[wasm_bindgen]
pub fn handle_key_up(key_code: &str) {
    APP_STATE.with(|app| {
        if let Some(state) = app.borrow_mut().as_mut() {
            match key_code {
                "KeyA" | "ArrowLeft" => state.input_state.rotate_left = false,
                "KeyD" | "ArrowRight" => state.input_state.rotate_right = false,
                "Space" | "KeyW" | "ArrowUp" => state.input_state.thrust = false,
                _ => {}
            }
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_wheel(delta_y: f64) {
    APP_STATE.with(|app| {
        if let Some(state) = app.borrow_mut().as_mut() {
            let zoom_factor = if delta_y < 0.0 {
                1.1
            } else {
                0.9
            };
            state.zoom_level *= zoom_factor;
        }
    });
}

#[wasm_bindgen]
pub fn handle_mouse_click(x: f64, y: f64, canvas_width: f64, canvas_height: f64) {
    APP_STATE.with(|app| {
        if let Some(state) = app.borrow_mut().as_mut() {
            state.mouse_pos = (x, y);

            // Check if clicking close button on info window
            if let Some(_) = state.selected_planet {
                let info_x = 50.0;
                let info_y = 50.0;
                let close_x = info_x + 280.0;
                let close_y = info_y + 5.0;

                if x >= close_x && x <= close_x + 15.0 && y >= close_y && y <= close_y + 15.0 {
                    state.selected_planet = None;
                    return;
                }
            }

            // Check if clicking on a planet
            let center_x = canvas_width / 2.0;
            let center_y = canvas_height / 2.0;
            let camera_x = state.game.player.position.x;
            let camera_y = state.game.player.position.y;
            let scale = state.zoom_level;

            for (i, planet) in state.game.planets.iter().enumerate() {
                let screen_x = ((planet.position.x - camera_x) * scale) + center_x;
                let screen_y = ((planet.position.y - camera_y) * scale) + center_y;
                let radius = (planet.radius * scale).max(5.0);

                let dx = x - screen_x;
                let dy = y - screen_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= radius * radius {
                    state.selected_planet = Some(i);
                    return;
                }
            }

            // Click elsewhere closes info window
            state.selected_planet = None;
        }
    });
}

