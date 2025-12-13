#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod planet;
mod vector2;
mod game;
mod player;
mod render;
mod initial_universe;
mod keyboard_input;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
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
            }
            WindowEvent::RedrawRequested => {
                // Update game state
                if let (Some(game), Some(last_update)) = (&mut self.game, &mut self.last_update) {
                    let now = Instant::now();
                    let dt = now.duration_since(*last_update).as_secs_f64();
                    *last_update = now;

                    // Apply input to game (this will recalculate trajectory if input changed)
                    self.input_state.apply_to_game(game, dt);

                    // Accumulate time and advance trajectory steps based on real elapsed time
                    self.time_accumulator += dt;

                    while self.time_accumulator >= TRAJECTORY_DT {
                        game.advance_trajectory();
                        self.time_accumulator -= TRAJECTORY_DT;
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

                    render_game(&mut buffer, width, height, game, self.input_state.thrust);

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
    };

    event_loop.run_app(&mut app).unwrap();
}
