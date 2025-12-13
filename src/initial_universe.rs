use crate::game::Game;
use crate::planet::Planet;
use crate::player::Player;
use crate::vector2::Vector2;
use crate::texture::Texture;

// Embed planet textures at compile time
const BEN_TEXTURE_BYTES: &[u8] = include_bytes!("../resources/ben.png");
const EARTH_TEXTURE_BYTES: &[u8] = include_bytes!("../resources/earth.png");
const MARTY_TEXTURE_BYTES: &[u8] = include_bytes!("../resources/marty.png");
const SHIRLEY_TEXTURE_BYTES: &[u8] = include_bytes!("../resources/shirley.png");

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

    // Ben planet orbiting Sun (further out and larger)
    let ben_orbit_radius = 25000.0;
    let ben_mass = 8e12;
    let (ben_position, ben_velocity) = calculate_stable_orbit(
        sun_position,
        sun_velocity,
        sun_mass,
        ben_orbit_radius,
        0.1, // circular orbit
        big_gravity,
    );

    // Marty planet orbiting Sun (even further out and larger)
    let marty_orbit_radius = 38000.0;
    let marty_mass = 1e13;
    let (marty_position, marty_velocity) = calculate_stable_orbit(
        sun_position,
        sun_velocity,
        sun_mass,
        marty_orbit_radius,
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

    // Shirley orbiting Marty (small and light moon)
    let shirley_orbit_radius = 800.0;
    let shirley_mass = 5e10;
    let (shirley_position, shirley_velocity) = calculate_stable_orbit(
        marty_position,
        marty_velocity,
        marty_mass,
        shirley_orbit_radius,
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

    // Load Earth texture from embedded bytes
    let mut earth = Planet::new(
        "Earth".to_string(),
        150.0,
        earth_mass,
        earth_position,
        earth_velocity,
        0x4040FF // blue fallback color
    )
    .with_description("A small blue planet with an atmosphere primarily composed of nitrogen and oxygen. It is the only known planet to support life.".to_string());

    if let Ok(texture) = Texture::load_from_bytes(EARTH_TEXTURE_BYTES) {
        earth = earth.with_texture(texture);
    }

    // Load Ben planet texture from embedded bytes
    let mut ben_planet = Planet::new(
        "Ben".to_string(),
        250.0, // Larger radius than Earth (150)
        ben_mass,
        ben_position,
        ben_velocity,
        0xFF8040 // orange fallback color
    )
    .with_description("A slightly eccentric planet that originated from outside the solar system but was captured by the Sun's gravity.".to_string());

    if let Ok(texture) = Texture::load_from_bytes(BEN_TEXTURE_BYTES) {
        ben_planet = ben_planet.with_texture(texture);
    }

    // Load Marty planet texture from embedded bytes
    let mut marty_planet = Planet::new(
        "Marty".to_string(),
        350.0, // Even larger radius than Ben (250)
        marty_mass,
        marty_position,
        marty_velocity,
        0xFF40FF // magenta fallback color
    )
    .with_description("The largest planet in the outer system with a moon. Marty was one of the first planets formed in the solar system.".to_string());

    if let Ok(texture) = Texture::load_from_bytes(MARTY_TEXTURE_BYTES) {
        marty_planet = marty_planet.with_texture(texture);
    }

    // Load Shirley moon texture from embedded bytes
    let mut shirley_moon = Planet::new(
        "Shirley".to_string(),
        40.0, // Small radius (between Moon at 20 and player orbit)
        shirley_mass,
        shirley_position,
        shirley_velocity,
        0xFFFFAA // light yellow fallback color
    )
    .with_description("One of the two moons of Marty. Shirley was discovered before Marty and was the main focus of astronomy until Marty was discovered. Nowdays many people don't even know about Shirley.".to_string());

    if let Ok(texture) = Texture::load_from_bytes(SHIRLEY_TEXTURE_BYTES) {
        shirley_moon = shirley_moon.with_texture(texture);
    }

    let planets = vec![
        Planet::new(
            "Sun".to_string(),
            300.0,
            sun_mass,
            sun_position,
            sun_velocity,
            0xFFFF00 // yellow
        )
        .with_description("The star at the center of our solar system. The Sun is the primary source of energy for the solar system and is responsible for the planets' orbits.".to_string()),
        earth,
        ben_planet,
        marty_planet,
        shirley_moon,
        Planet::new(
            "Moon".to_string(),
            20.0,
            1e11,
            moon_position,
            moon_velocity,
            0xAAAAAA // gray
        )
        .with_description("Earths natural satellite, but not for long. The earth-moon system is very unstable.".to_string()),
    ];

    let player = Player::new(
        player_position,
        player_velocity,
        1.0,
        0.0
    );

    Game::new(planets, player)
}

