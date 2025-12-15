use crate::game::Game;

#[cfg(not(target_arch = "wasm32"))]
use winit::event::{KeyEvent, ElementState};
#[cfg(not(target_arch = "wasm32"))]
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputState {
    pub rotate_left: bool,
    pub rotate_right: bool,
    pub thrust: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            rotate_left: false,
            rotate_right: false,
            thrust: false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_key_event(&mut self, event: &KeyEvent) {
        let pressed = event.state == ElementState::Pressed;

        match event.physical_key {
            PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.rotate_left = pressed;
            }
            PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.rotate_right = pressed;
            }
            PhysicalKey::Code(KeyCode::Space) |
            PhysicalKey::Code(KeyCode::KeyW) |
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.thrust = pressed;
            }
            _ => {}
        }
    }

    pub fn apply_to_game(&self, game: &mut Game, dt: f64) {
        // Update player rotation (doesn't change trajectory)
        let rotation_speed = 3.0; // radians per second
        if self.rotate_left {
            game.player.rotation -= rotation_speed * dt;
        }
        if self.rotate_right {
            game.player.rotation += rotation_speed * dt;
        }

        // Apply thrust force if thrusting (changes trajectory)
        if self.thrust {
            let thrust_force = 25.0; // thrust force magnitude
            let thrust_x = game.player.rotation.sin() * thrust_force;
            let thrust_y = -game.player.rotation.cos() * thrust_force;

            game.player.velocity.x += thrust_x / game.player.mass * dt;
            game.player.velocity.y += thrust_y / game.player.mass * dt;

            // Only recalculate trajectory when thrust changes velocity
            game.recalculate_trajectories();
        }
    }
}

