use crate::game::Game;

pub fn render_game(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    game: &Game,
    is_thrusting: bool,
    zoom_level: f64,
) {
    // Clear to black (space)
    buffer.fill(0x000000);

    let center_x = width / 2;
    let center_y = height / 2;

    // Camera follows player
    let camera_x = game.player.position.x;
    let camera_y = game.player.position.y;

    // Scale: 1 pixel = 1 unit, multiplied by zoom level
    let scale = 1.0 * zoom_level;

    // Draw orbital predictions
    draw_orbital_predictions(buffer, width, height, game, camera_x, camera_y, scale, center_x, center_y);

    // Draw planets
    for planet in &game.planets {
        let screen_x = ((planet.position.x - camera_x) * scale) as i32 + center_x as i32;
        let screen_y = ((planet.position.y - camera_y) * scale) as i32 + center_y as i32;
        let radius = (planet.radius * scale).max(5.0) as i32;

        draw_circle(buffer, width, height, screen_x, screen_y, radius, planet.color);
    }

    // Draw player as rotated rectangle
    draw_rotated_rect(
        buffer,
        width,
        height,
        center_x as i32,
        center_y as i32,
        8,
        4,
        game.player.rotation,
        0xFF0000
    );

    // Draw thrust flame if thrusting
    if is_thrusting {
        draw_thrust_flame(
            buffer,
            width,
            height,
            center_x as i32,
            center_y as i32,
            game.player.rotation,
            12
        );
    }
}

fn draw_orbital_predictions(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    game: &Game,
    camera_x: f64,
    camera_y: f64,
    scale: f64,
    center_x: usize,
    center_y: usize,
) {
    if !game.cached_trajectories.is_valid {
        return;
    }

    // Draw player trajectory in absolute coordinates (relative to origin 0,0)
    let dim_player_color = 0x800000; // Dim red
    let mut last_pos: Option<(i32, i32)> = None;

    for position in &game.cached_trajectories.player_positions {
        let screen_x = ((position.x - camera_x) * scale) as i32 + center_x as i32;
        let screen_y = ((position.y - camera_y) * scale) as i32 + center_y as i32;

        if let Some((last_x, last_y)) = last_pos {
            draw_line(buffer, width, height, last_x, last_y, screen_x, screen_y, dim_player_color);
        }

        last_pos = Some((screen_x, screen_y));
    }

    // Draw planet trajectories in absolute coordinates (relative to origin 0,0)
    for i in 0..game.planets.len() {
        let color = game.planets[i].color;
        let dim_color = ((color >> 16) / 2) << 16 | (((color >> 8) & 0xFF) / 2) << 8 | ((color & 0xFF) / 2);
        let mut last_pos: Option<(i32, i32)> = None;

        for position in &game.cached_trajectories.planet_positions[i] {
            let screen_x = ((position.x - camera_x) * scale) as i32 + center_x as i32;
            let screen_y = ((position.y - camera_y) * scale) as i32 + center_y as i32;

            if let Some((last_x, last_y)) = last_pos {
                draw_line(buffer, width, height, last_x, last_y, screen_x, screen_y, dim_color);
            }

            last_pos = Some((screen_x, screen_y));
        }
    }
}

fn draw_line(buffer: &mut [u32], width: usize, height: usize, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    // Bresenham's line algorithm
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            buffer[y as usize * width + x as usize] = color;
        }

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_circle(buffer: &mut [u32], width: usize, height: usize, cx: i32, cy: i32, radius: i32, color: u32) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    buffer[py as usize * width + px as usize] = color;
                }
            }
        }
    }
}

fn draw_rotated_rect(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: i32,
    cy: i32,
    rect_width: i32,
    rect_height: i32,
    rotation: f64,
    color: u32
) {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();

    // Draw filled rotated rectangle
    for local_y in -rect_height..=rect_height {
        for local_x in -rect_width..=rect_width {
            // Rotate the point
            let rotated_x = (local_x as f64 * cos_r - local_y as f64 * sin_r) as i32;
            let rotated_y = (local_x as f64 * sin_r + local_y as f64 * cos_r) as i32;

            let px = cx + rotated_x;
            let py = cy + rotated_y;

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                buffer[py as usize * width + px as usize] = color;
            }
        }
    }
}

fn draw_thrust_flame(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: i32,
    cy: i32,
    rotation: f64,
    length: i32
) {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();

    // Draw flame coming from the back of the ship (opposite direction)
    for i in 0..length {
        let local_y = 5 + i; // Start from back of ship

        for local_x in -2..=2 {
            // Taper the flame
            let width_factor = 1.0 - (i as f64 / length as f64);
            if (local_x as f64).abs() > 2.0 * width_factor {
                continue;
            }

            // Rotate the point
            let rotated_x = (local_x as f64 * cos_r - local_y as f64 * sin_r) as i32;
            let rotated_y = (local_x as f64 * sin_r + local_y as f64 * cos_r) as i32;

            let px = cx + rotated_x;
            let py = cy + rotated_y;

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                // Gradient from yellow to orange
                let color = if i < length / 2 { 0xFFFF00 } else { 0xFF8800 };
                buffer[py as usize * width + px as usize] = color;
            }
        }
    }
}

