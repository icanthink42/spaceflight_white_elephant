use crate::game::Game;
use crate::vector2::Vector2;
use crate::sprite_renderer::draw_circular_sprite;
use crate::font::draw_text;

pub fn render_game(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    game: &Game,
    is_thrusting: bool,
    zoom_level: f64,
    time_warp: f64,
    show_absolute_trajectories: bool,
    selected_planet: Option<usize>,
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

        // Draw textured planet if texture available, otherwise solid color
        if let Some(texture) = &planet.texture {
            draw_circular_sprite(buffer, width, height, screen_x, screen_y, radius, texture);
        } else {
            draw_circle(buffer, width, height, screen_x, screen_y, radius, planet.color);
        }
    }

    // Draw player as rotated rectangle
    draw_rotated_triangle(
        buffer,
        width,
        height,
        center_x as i32,
        center_y as i32,
        8,
        6,
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

    // Draw planet info window if a planet is selected
    if let Some(planet_idx) = selected_planet {
        if planet_idx < game.planets.len() {
            draw_planet_info(buffer, width, height, &game.planets[planet_idx]);
        }
    }
}

fn draw_planet_info(buffer: &mut [u32], width: usize, height: usize, planet: &crate::planet::Planet) {
    let info_x = 50;
    let info_y = 50;
    let info_width = 300;

    // Calculate height based on content
    let has_texture = planet.texture.is_some();
    let texture_size = 120; // Size of the displayed texture
    let has_description = !planet.description.is_empty();

    // Calculate description line count (assuming ~40 chars per line at 6 pixels per char)
    let chars_per_line = (info_width - 20) / 6;
    let description_lines = if has_description {
        (planet.description.len() + chars_per_line - 1) / chars_per_line
    } else {
        0
    };

    let mut info_height = 150; // Base height
    if has_texture {
        info_height += texture_size + 15;
    }
    if has_description {
        info_height += description_lines * 10 + 15;
    }

    // Draw background box
    for y in info_y..info_y + info_height {
        for x in info_x..info_x + info_width {
            if x < width && y < height {
                buffer[y * width + x] = 0x222222;
            }
        }
    }

    // Draw border
    for x in info_x..info_x + info_width {
        if x < width {
            if info_y < height {
                buffer[info_y * width + x] = 0xFFFFFF;
            }
            if info_y + info_height - 1 < height {
                buffer[(info_y + info_height - 1) * width + x] = 0xFFFFFF;
            }
        }
    }
    for y in info_y..info_y + info_height {
        if y < height {
            if info_x < width {
                buffer[y * width + info_x] = 0xFFFFFF;
            }
            if info_x + info_width - 1 < width {
                buffer[y * width + info_x + info_width - 1] = 0xFFFFFF;
            }
        }
    }

    // Draw close button (X)
    let close_x = info_x + info_width - 20;
    let close_y = info_y + 5;
    draw_text(buffer, width, height, "X", close_x, close_y, 0xFF0000);

    // Draw planet info
    let mut y_offset = info_y + 20;

    // Planet name
    draw_text(buffer, width, height, &planet.name, info_x + 10, y_offset, 0xFFFFFF);
    y_offset += 20;

    // Divider
    for x in info_x + 10..info_x + info_width - 10 {
        if x < width && y_offset < height {
            buffer[y_offset * width + x] = 0x888888;
        }
    }
    y_offset += 15;

    // Draw planet texture if available
    if let Some(texture) = &planet.texture {
        let texture_center_x = (info_x + info_width / 2) as i32;
        let texture_center_y = (y_offset + texture_size / 2) as i32;
        draw_circular_sprite(buffer, width, height, texture_center_x, texture_center_y, texture_size as i32 / 2, texture);
        y_offset += texture_size + 15;

        // Another divider after the texture
        for x in info_x + 10..info_x + info_width - 10 {
            if x < width && y_offset < height {
                buffer[y_offset * width + x] = 0x888888;
            }
        }
        y_offset += 15;
    }

    // Draw description if available
    if has_description {
        draw_wrapped_text(buffer, width, height, &planet.description, info_x + 10, y_offset, info_width - 20, 0xAADDFF);
        y_offset += description_lines * 10 + 15;

        // Divider after description
        for x in info_x + 10..info_x + info_width - 10 {
            if x < width && y_offset < height {
                buffer[y_offset * width + x] = 0x888888;
            }
        }
        y_offset += 10;
    }

    // Mass
    draw_text(buffer, width, height, &format!("Mass: {:.2e} kg", planet.mass), info_x + 10, y_offset, 0xCCCCCC);
    y_offset += 15;

    // Radius
    draw_text(buffer, width, height, &format!("Radius: {:.0} units", planet.radius), info_x + 10, y_offset, 0xCCCCCC);
    y_offset += 15;

    // Position
    draw_text(buffer, width, height, &format!("Position: ({:.0}, {:.0})", planet.position.x, planet.position.y), info_x + 10, y_offset, 0xCCCCCC);
    y_offset += 15;

    // Velocity
    let speed = (planet.velocity.x * planet.velocity.x + planet.velocity.y * planet.velocity.y).sqrt();
    draw_text(buffer, width, height, &format!("Velocity: {:.2} units/s", speed), info_x + 10, y_offset, 0xCCCCCC);
}

fn draw_wrapped_text(buffer: &mut [u32], width: usize, height: usize, text: &str, x: usize, y: usize, max_width: usize, color: u32) {
    let chars_per_line = max_width / 6; // 6 pixels per character
    let mut current_line = 0;
    let mut chars_in_line = 0;
    let mut line_start = 0;

    for (i, ch) in text.chars().enumerate() {
        chars_in_line += 1;

        // Check if we need to wrap
        if chars_in_line >= chars_per_line || ch == '\n' {
            let line = &text[line_start..i + ch.len_utf8()];
            draw_text(buffer, width, height, line, x, y + current_line * 10, color);
            current_line += 1;
            chars_in_line = 0;
            line_start = i + ch.len_utf8();
        }
    }

    // Draw remaining text
    if line_start < text.len() {
        let line = &text[line_start..];
        draw_text(buffer, width, height, line, x, y + current_line * 10, color);
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

fn draw_rotated_triangle(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: i32,
    cy: i32,
    length: i32,     // distance from center to tip
    half_base: i32,  // half the width of the base
    rotation: f64,
    color: u32,
) {
    let adj_rot:f64 = rotation + 1.5707;
    let cos_r = adj_rot.cos();
    let sin_r = adj_rot.sin();

    // Triangle in local space:
    // Tip at ( length, 0 )
    // Base left  at ( -length, -half_base )
    // Base right at ( -length,  half_base )

    // Bounding box in local space
    for ly in -half_base..=half_base {
        for lx in -length..=length {
            // Half-space test for triangle pointing +X
            // Line 1: left edge
            let inside_left = (lx + length) * half_base >= ly * (2 * length);
            // Line 2: right edge
            let inside_right = (lx + length) * half_base >= -ly * (2 * length);
            // Line 3: behind the tip
            let inside_back = lx <= length;

            if inside_left && inside_right && inside_back {
                // Rotate
                let rx = (lx as f64 * cos_r - ly as f64 * sin_r) as i32;
                let ry = (lx as f64 * sin_r + ly as f64 * cos_r) as i32;

                let px = cx + rx;
                let py = cy + ry;

                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    buffer[py as usize * width + px as usize] = color;
                }
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

