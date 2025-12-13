use crate::game::Game;
use crate::planet::Planet;
use crate::player::Player;
use crate::vector2::Vector2;

/// Calculate stable orbital position and velocity around a center body
/// Returns (position, velocity) relative to the center body
fn calculate_stable_orbit(
    center_position: Vector2,
    center_velocity: Vector2,
    center_mass: f64,
    radius: f64,
    eccentricity: f64,
    big_gravity: f64,
) -> (Vector2, Vector2) {
    // For circular orbit (e=0): v = sqrt(G * M / r)
    // For elliptical orbit at periapsis: v = sqrt(G * M * (1+e) / (r * (1-e)))

    let orbital_speed = if eccentricity == 0.0 {
        (big_gravity * center_mass / radius).sqrt()
    } else {
        (big_gravity * center_mass * (1.0 + eccentricity) / (radius * (1.0 - eccentricity))).sqrt()
    };

    // Position: start at the specified radius (periapsis for elliptical orbits)
    let position = Vector2 {
        x: center_position.x + radius,
        y: center_position.y,
    };

    // Velocity: perpendicular to radius vector, plus center body's velocity
    let velocity = Vector2 {
        x: center_velocity.x,
        y: center_velocity.y + orbital_speed,
    };

    (position, velocity)
}

pub fn create_universe() -> Game {
    let big_gravity = 0.000001;

    // Sun at origin
    let sun_position = Vector2 { x: 0.0, y: 0.0 };
    let sun_velocity = Vector2 { x: 0.0, y: 0.0 };
    let sun_mass = 1e15;

    // Earth orbiting Sun
    let earth_orbit_radius = 15000.0;
    let earth_mass = 6e12;
    let (earth_position, earth_velocity) = calculate_stable_orbit(
        sun_position,
        sun_velocity,
        sun_mass,
        earth_orbit_radius,
        0.0, // circular orbit
        big_gravity,
    );

    // Moon orbiting Earth
    let moon_orbit_radius = 1000.0;
    let (moon_position, moon_velocity) = calculate_stable_orbit(
        earth_position,
        earth_velocity,
        earth_mass,
        moon_orbit_radius,
        0.0, // circular orbit
        big_gravity,
    );

    // Player orbiting Earth
    let player_orbit_radius = 300.0;
    let (player_position, player_velocity) = calculate_stable_orbit(
        earth_position,
        earth_velocity,
        earth_mass,
        player_orbit_radius,
        0.0, // circular orbit
        big_gravity,
    );

    let planets = vec![
        Planet::new(
            "Sun".to_string(),
            300.0,
            sun_mass,
            sun_position,
            sun_velocity,
            0xFFFF00 // yellow
        ),
        Planet::new(
            "Earth".to_string(),
            150.0,
            earth_mass,
            earth_position,
            earth_velocity,
            0x4040FF // blue
        ),
        Planet::new(
            "Moon".to_string(),
            20.0,
            1e11,
            moon_position,
            moon_velocity,
            0xAAAAAA // gray
        ),
    ];

    let player = Player::new(
        player_position,
        player_velocity,
        1.0,
        0.0
    );

    Game::new(planets, player)
}

