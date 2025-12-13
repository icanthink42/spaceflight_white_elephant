use crate::game::Game;
use crate::planet::Planet;
use crate::player::Player;
use crate::vector2::Vector2;

pub fn create_universe() -> Game {
    // Initialize game - scaled down for gameplay
    let planets = vec![
        Planet::new(
            "Earth".to_string(),
            150.0,         // radius
            1e12,         // mass (scaled for gameplay)
            Vector2 { x: 0.0, y: 0.0 },
            Vector2 { x: 0.0, y: 0.0 },
            0x4040FF      // blue
        ),
        Planet::new(
            "Moon".to_string(),
            20.0,         // radius
            1e11,         // mass (scaled for gameplay)
            Vector2 { x: 5000.0, y: 0.0 },  // Earth-Moon distance
            Vector2 { x: 0.0, y: 80.0 },     // orbital velocity
            0xAAAAAA      // gray
        ),
    ];

    let player = Player::new(
        Vector2 { x: 550.0, y: 0.0 },    // Just above Earth surface
        Vector2 { x: 0.0, y: 50.0 },       // Starting from rest
        1.0,                            // mass
        0.0
    );

    Game::new(planets, player)
}

