/// Texture management module
use image::{DynamicImage, GenericImageView, Rgba};

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>, // ARGB format
}

impl Texture {
    /// Load a texture from a file path
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let img = image::open(path)
            .map_err(|e| format!("Failed to load image {}: {}", path, e))?;

        Ok(Self::from_dynamic_image(img))
    }

    /// Load a texture from embedded bytes
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("Failed to load embedded image: {}", e))?;

        Ok(Self::from_dynamic_image(img))
    }

    /// Convert a DynamicImage to our Texture format
    fn from_dynamic_image(img: DynamicImage) -> Self {
        let (width, height) = img.dimensions();
        let rgba_img = img.to_rgba8();

        let mut pixels = Vec::with_capacity((width * height) as usize);

        for pixel in rgba_img.pixels() {
            let Rgba([r, g, b, a]) = *pixel;
            // Convert to ARGB u32 format
            let argb = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            pixels.push(argb);
        }

        Texture {
            width,
            height,
            pixels,
        }
    }

    /// Sample a pixel from the texture at normalized coordinates (0.0 to 1.0)
    pub fn sample(&self, u: f64, v: f64) -> u32 {
        let x = (u * self.width as f64).rem_euclid(self.width as f64) as u32;
        let y = (v * self.height as f64).rem_euclid(self.height as f64) as u32;

        let index = (y * self.width + x) as usize;
        self.pixels[index]
    }
}

