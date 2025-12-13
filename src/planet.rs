use crate::vector2::Vector2;
use crate::texture::Texture;

#[derive(Clone)]
pub struct Planet {
    pub name: String,
    pub radius: f64,
    pub mass: f64,
    pub position: Vector2,
    pub velocity: Vector2,
    pub color: u32, // RGB color (0xRRGGBB)
    pub texture: Option<Texture>,
    pub description: String,
}

impl Planet {
    pub fn new(name: String, radius: f64, mass: f64, position: Vector2, velocity: Vector2, color: u32) -> Self {
        Self { name, radius, mass, position, velocity, color, texture: None, description: String::new() }
    }

    pub fn with_texture(mut self, texture: Texture) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
}