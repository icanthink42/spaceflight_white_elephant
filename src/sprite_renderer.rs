/// Sprite rendering module for drawing textured circles
use crate::texture::Texture;

/// Draw a circular sprite (texture mapped onto a circle)
pub fn draw_circular_sprite(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: i32,
    cy: i32,
    radius: i32,
    texture: &Texture,
) {
    let r_sq = radius * radius;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= r_sq {
                let x = cx + dx;
                let y = cy + dy;

                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    // Calculate UV coordinates (spherical mapping)
                    let u = 0.5 + (dx as f64 / (2.0 * radius as f64));
                    let v = 0.5 + (dy as f64 / (2.0 * radius as f64));

                    let color = texture.sample(u, v);

                    // Blend with alpha
                    let alpha = (color >> 24) & 0xFF;
                    if alpha > 0 {
                        let idx = (y as usize) * width + (x as usize);
                        if alpha == 255 {
                            buffer[idx] = color & 0xFFFFFF; // Remove alpha channel
                        } else {
                            // Alpha blending
                            let src_r = ((color >> 16) & 0xFF) as u32;
                            let src_g = ((color >> 8) & 0xFF) as u32;
                            let src_b = (color & 0xFF) as u32;

                            let dst = buffer[idx];
                            let dst_r = ((dst >> 16) & 0xFF) as u32;
                            let dst_g = ((dst >> 8) & 0xFF) as u32;
                            let dst_b = (dst & 0xFF) as u32;

                            let alpha = alpha as u32;
                            let inv_alpha = 255 - alpha;

                            let r = (src_r * alpha + dst_r * inv_alpha) / 255;
                            let g = (src_g * alpha + dst_g * inv_alpha) / 255;
                            let b = (src_b * alpha + dst_b * inv_alpha) / 255;

                            buffer[idx] = (r << 16) | (g << 8) | b;
                        }
                    }
                }
            }
        }
    }
}

