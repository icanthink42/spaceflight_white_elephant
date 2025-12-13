#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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

use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, MouseScrollDelta, ElementState, MouseButton};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::sync::Arc;
use std::time::Instant;
use softbuffer::{Context, Surface};
use crate::game::{Game, TRAJECTORY_DT};
use crate::render::render_game;
use crate::initial_universe::create_universe;
use crate::keyboard_input::InputState;

struct App {
    window: Option<Arc<Window>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    game: Option<Game>,
    last_update: Option<Instant>,
    input_state: InputState,
    time_accumulator: f64,
    zoom_level: f64,
    time_warp: f64,
    show_absolute_trajectories: bool,
    selected_planet: Option<usize>,
    mouse_pos: (f64, f64),
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Spaceflight Elephant")
                .with_inner_size(winit::dpi::LogicalSize::new(1200, 800));

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            let context = Context::new(window.clone()).unwrap();
            let surface = Surface::new(&context, window.clone()).unwrap();

            // Initialize game universe
            let game = create_universe();

            self.surface = Some(surface);
            self.window = Some(window);
            self.game = Some(game);
            self.last_update = Some(Instant::now());

            // Trigger initial render
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_state.handle_key_event(&event);

                // Handle zoom, time warp, and display mode keys
                if event.state == ElementState::Pressed {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Equal) | PhysicalKey::Code(KeyCode::NumpadAdd) => {
                            self.zoom_level *= 1.2;
                        }
                        PhysicalKey::Code(KeyCode::Minus) | PhysicalKey::Code(KeyCode::NumpadSubtract) => {
                            self.zoom_level /= 1.2;
                        }
                        PhysicalKey::Code(KeyCode::Period) => {
                            self.time_warp *= 2.0;
                            self.time_warp = self.time_warp.min(256.0);
                        }
                        PhysicalKey::Code(KeyCode::Comma) => {
                            self.time_warp /= 2.0;
                            self.time_warp = self.time_warp.max(1.0);
                        }
                        PhysicalKey::Code(KeyCode::Tab) => {
                            self.show_absolute_trajectories = !self.show_absolute_trajectories;
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let zoom_factor = match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        if y > 0.0 {
                            1.1_f64.powf(y as f64)
                        } else {
                            0.9_f64.powf(-y as f64)
                        }
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        let y = pos.y;
                        if y > 0.0 {
                            1.0 + y / 100.0
                        } else {
                            1.0 / (1.0 - y / 100.0)
                        }
                    }
                };
                self.zoom_level *= zoom_factor;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = (position.x, position.y);
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                if let (Some(window), Some(game)) = (&self.window, &self.game) {
                    let size = window.inner_size();
                    let width = size.width as f64;
                    let height = size.height as f64;

                    // Check if clicking close button on info window
                    if let Some(_planet_idx) = self.selected_planet {
                        let info_x = 50.0;
                        let info_y = 50.0;
                        let close_x = info_x + 280.0;
                        let close_y = info_y + 5.0;

                        if self.mouse_pos.0 >= close_x && self.mouse_pos.0 <= close_x + 15.0 &&
                           self.mouse_pos.1 >= close_y && self.mouse_pos.1 <= close_y + 15.0 {
                            self.selected_planet = None;
                            return;
                        }
                    }

                    // Check if clicking on a planet
                    let center_x = width / 2.0;
                    let center_y = height / 2.0;
                    let camera_x = game.player.position.x;
                    let camera_y = game.player.position.y;
                    let scale = self.zoom_level;

                    for (i, planet) in game.planets.iter().enumerate() {
                        let screen_x = ((planet.position.x - camera_x) * scale) + center_x;
                        let screen_y = ((planet.position.y - camera_y) * scale) + center_y;
                        let radius = (planet.radius * scale).max(5.0);

                        let dx = self.mouse_pos.0 - screen_x;
                        let dy = self.mouse_pos.1 - screen_y;
                        let dist_sq = dx * dx + dy * dy;

                        if dist_sq <= radius * radius {
                            self.selected_planet = Some(i);
                            return;
                        }
                    }

                    // Click elsewhere closes info window
                    self.selected_planet = None;
                }
            }
            WindowEvent::RedrawRequested => {
                // Update game state
                if let (Some(game), Some(last_update)) = (&mut self.game, &mut self.last_update) {
                    let now = Instant::now();
                    let dt = now.duration_since(*last_update).as_secs_f64();
                    *last_update = now;

                    // Apply input to game (this will recalculate trajectory if input changed)
                    self.input_state.apply_to_game(game, dt);

                    // Accumulate time with time warp multiplier and advance trajectory steps
                    self.time_accumulator += dt * self.time_warp;

                    let steps_to_advance = (self.time_accumulator / TRAJECTORY_DT) as usize;
                    if steps_to_advance > 0 {
                        // Advance multiple steps at once
                        for _ in 0..steps_to_advance {
                            game.advance_trajectory();
                        }

                        // Batch extend trajectories to maintain look-ahead
                        game.extend_trajectories(steps_to_advance);

                        self.time_accumulator -= steps_to_advance as f64 * TRAJECTORY_DT;
                    }
                }

                // Render
                if let (Some(window), Some(surface), Some(game)) =
                    (&self.window, &mut self.surface, &self.game) {

                    let (width, height) = {
                        let size = window.inner_size();
                        (size.width as usize, size.height as usize)
                    };

                    if width == 0 || height == 0 {
                        return;
                    }

                    surface.resize(
                        std::num::NonZero::new(width as u32).unwrap(),
                        std::num::NonZero::new(height as u32).unwrap()
                    ).unwrap();

                    let mut buffer = surface.buffer_mut().unwrap();

                    render_game(&mut buffer, width, height, game, self.input_state.thrust, self.zoom_level, self.time_warp, self.show_absolute_trajectories, self.selected_planet);

                    buffer.present().unwrap();
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        surface: None,
        game: None,
        last_update: None,
        input_state: InputState::new(),
        time_accumulator: 0.0,
        zoom_level: 1.0,
        time_warp: 1.0,
        show_absolute_trajectories: false, // Start with planet-relative mode
        selected_planet: None,
        mouse_pos: (0.0, 0.0),
    };

    event_loop.run_app(&mut app).unwrap();
}
