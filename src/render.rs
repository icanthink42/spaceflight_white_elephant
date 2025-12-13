use crate::game::Game;
use crate::vector2::Vector2;

pub fn render_game(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    game: &Game,
    is_thrusting: bool,
    zoom_level: f64,
    time_warp: f64,
    show_absolute_trajectories: bool,
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
    draw_orbital_predictions(buffer, width, height, game, camera_x, camera_y, scale, center_x, center_y, show_absolute_trajectories);

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

    // Draw time warp indicator in top right
    draw_text(buffer, width, height, &format!("Time Warp: {:.1}x", time_warp), width - 200, 10, 0xFFFFFF);

    // Draw trajectory mode in top left
    let mode_text = if show_absolute_trajectories {
        "Absolute Trajectories"
    } else {
        "Planet-Relative Trajectories"
    };
    draw_text(buffer, width, height, mode_text, 10, 10, 0xFFFFFF);
}

fn draw_text(buffer: &mut [u32], width: usize, height: usize, text: &str, x: usize, y: usize, color: u32) {
    // Simple 5x7 pixel font for basic text rendering
    let mut current_x = x;

    for ch in text.chars() {
        draw_char(buffer, width, height, ch, current_x, y, color);
        current_x += 6; // 5 pixels + 1 spacing
    }
}

fn draw_char(buffer: &mut [u32], width: usize, height: usize, ch: char, x: usize, y: usize, color: u32) {
    let glyph = get_glyph(ch);

    for (row, &line) in glyph.iter().enumerate() {
        for col in 0..5 {
            if line & (1 << (4 - col)) != 0 {
                let px = x + col;
                let py = y + row;
                if px < width && py < height {
                    buffer[py * width + px] = color;
                }
            }
        }
    }
}

fn get_glyph(ch: char) -> &'static [u8; 7] {
    match ch {
        '0' => &[0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => &[0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => &[0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => &[0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => &[0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => &[0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => &[0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => &[0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => &[0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        'A' => &[0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'P' => &[0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'R' => &[0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'T' => &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'W' => &[0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'a' => &[0b00000, 0b00000, 0b01110, 0b00001, 0b01111, 0b10001, 0b01111],
        'b' => &[0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'c' => &[0b00000, 0b00000, 0b01110, 0b10000, 0b10000, 0b10000, 0b01110],
        'e' => &[0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110],
        'i' => &[0b00100, 0b00000, 0b01100, 0b00100, 0b00100, 0b00100, 0b01110],
        'j' => &[0b00000, 0b00000, 0b00110, 0b00010, 0b00010, 0b10010, 0b01100],
        'l' => &[0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'm' => &[0b00000, 0b00000, 0b11010, 0b10101, 0b10101, 0b10001, 0b10001],
        'n' => &[0b00000, 0b00000, 0b10110, 0b11001, 0b10001, 0b10001, 0b10001],
        'o' => &[0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        'p' => &[0b00000, 0b00000, 0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
        'r' => &[0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000],
        's' => &[0b00000, 0b00000, 0b01110, 0b10000, 0b01110, 0b00001, 0b11110],
        't' => &[0b00100, 0b00100, 0b11110, 0b00100, 0b00100, 0b00100, 0b00010],
        'u' => &[0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b10011, 0b01101],
        'v' => &[0b00000, 0b00000, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'x' => &[0b00000, 0b00000, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001],
        'y' => &[0b00000, 0b00000, 0b10001, 0b10001, 0b01111, 0b00001, 0b01110],
        '-' => &[0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '.' => &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100],
        ':' => &[0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000],
        ' ' => &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        _ => &[0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000], // Unknown char = space
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
    show_absolute: bool,
) {
    if !game.cached_trajectories.is_valid {
        return;
    }

    if show_absolute {
        // Draw in absolute coordinates
        draw_absolute_trajectories(buffer, width, height, game, camera_x, camera_y, scale, center_x, center_y);
    } else {
        // Draw relative to dominant planets
        draw_relative_trajectories(buffer, width, height, game, camera_x, camera_y, scale, center_x, center_y);
    }
}

fn draw_absolute_trajectories(
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
    // Draw player trajectory in absolute coordinates
    let dim_player_color = 0x800000;
    let mut last_pos: Option<(i32, i32)> = None;

    for position in &game.cached_trajectories.player_positions {
        let screen_x = ((position.x - camera_x) * scale) as i32 + center_x as i32;
        let screen_y = ((position.y - camera_y) * scale) as i32 + center_y as i32;

        if let Some((last_x, last_y)) = last_pos {
            draw_line(buffer, width, height, last_x, last_y, screen_x, screen_y, dim_player_color);
        }

        last_pos = Some((screen_x, screen_y));
    }

    // Draw planet trajectories in absolute coordinates (skip Sun at index 0)
    for i in 1..game.planets.len() {
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

fn draw_relative_trajectories(
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
    // Find dominant planet for player at current position
    let player_dominant = find_dominant_planet(game, &game.player.position);

    // Draw player trajectory relative to dominant planet
    let dim_player_color = 0x800000;
    let mut last_pos: Option<(i32, i32)> = None;

    for (idx, position) in game.cached_trajectories.player_positions.iter().enumerate() {
        let ref_pos = &game.cached_trajectories.planet_positions[player_dominant][idx];
        let rel_x = position.x - ref_pos.x;
        let rel_y = position.y - ref_pos.y;
        let ref_now = &game.planets[player_dominant].position;

        let screen_x = ((rel_x - (camera_x - ref_now.x)) * scale) as i32 + center_x as i32;
        let screen_y = ((rel_y - (camera_y - ref_now.y)) * scale) as i32 + center_y as i32;

        if let Some((last_x, last_y)) = last_pos {
            draw_line(buffer, width, height, last_x, last_y, screen_x, screen_y, dim_player_color);
        }

        last_pos = Some((screen_x, screen_y));
    }

    // Draw planet trajectories relative to their dominant planets (skip Sun at index 0)
    for i in 1..game.planets.len() {
        let color = game.planets[i].color;
        let dim_color = ((color >> 16) / 2) << 16 | (((color >> 8) & 0xFF) / 2) << 8 | ((color & 0xFF) / 2);
        let mut last_pos: Option<(i32, i32)> = None;
        let planet_dominant = find_dominant_planet(game, &game.planets[i].position);

        for (idx, position) in game.cached_trajectories.planet_positions[i].iter().enumerate() {
            let ref_pos = &game.cached_trajectories.planet_positions[planet_dominant][idx];
            let rel_x = position.x - ref_pos.x;
            let rel_y = position.y - ref_pos.y;
            let ref_now = &game.planets[planet_dominant].position;

            let screen_x = ((rel_x - (camera_x - ref_now.x)) * scale) as i32 + center_x as i32;
            let screen_y = ((rel_y - (camera_y - ref_now.y)) * scale) as i32 + center_y as i32;

            if let Some((last_x, last_y)) = last_pos {
                draw_line(buffer, width, height, last_x, last_y, screen_x, screen_y, dim_color);
            }

            last_pos = Some((screen_x, screen_y));
        }
    }
}

fn find_dominant_planet(game: &Game, position: &Vector2) -> usize {
    let mut max_accel = 0.0;
    let mut dominant_idx = 0;

    for (i, planet) in game.planets.iter().enumerate() {
        let diff = planet.position.subtract(position);
        let distance = diff.magnitude();
        if distance > 0.0 {
            // Gravitational acceleration: a = G * M / r^2
            let accel = game.big_gravity * planet.mass / (distance * distance);
            if accel > max_accel {
                max_accel = accel;
                dominant_idx = i;
            }
        }
    }

    dominant_idx
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

