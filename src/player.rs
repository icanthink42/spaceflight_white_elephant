use crate::vector2::Vector2;

#[derive(Clone, Copy)]
pub struct Player {
    pub position: Vector2,
    pub velocity: Vector2,
    pub mass: f64,
    pub rotation: f64,
}

impl Player {
    pub fn new(position: Vector2, velocity: Vector2, mass: f64, rotation: f64) -> Self {
        Self { position, velocity, mass, rotation }
    }
}