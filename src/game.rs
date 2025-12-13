use crate::{planet::Planet, player::Player, vector2::Vector2};
use std::collections::VecDeque;

// Trajectory prediction constants
const TRAJECTORY_NUM_STEPS: usize = 100000;
pub const TRAJECTORY_DT: f64 = 0.016;
const TRAJECTORY_SUBSTEPS: usize = 5;

pub struct Game {
    pub big_gravity: f64,
    pub planets: Vec<Planet>,
    pub player: Player,
    pub cached_trajectories: CachedTrajectories,
}

pub struct CachedTrajectories {
    pub player_positions: VecDeque<Vector2>,
    pub player_velocities: VecDeque<Vector2>,
    pub player_rotations: VecDeque<f64>,
    pub planet_positions: Vec<VecDeque<Vector2>>,
    pub planet_velocities: Vec<VecDeque<Vector2>>,
    pub is_valid: bool,
}

impl Game {
    pub fn new(planets: Vec<Planet>, player: Player) -> Self {
        let mut game = Self {
            big_gravity: 0.000001,
            planets,
            player,
            cached_trajectories: CachedTrajectories {
                player_positions: VecDeque::new(),
                player_velocities: VecDeque::new(),
                player_rotations: VecDeque::new(),
                planet_positions: Vec::new(),
                planet_velocities: Vec::new(),
                is_valid: false,
            },
        };
        game.recalculate_trajectories();
        game
    }

    pub fn recalculate_trajectories(&mut self) {
        let num_steps = TRAJECTORY_NUM_STEPS;
        let dt = TRAJECTORY_DT;
        let substeps = TRAJECTORY_SUBSTEPS;

        // Create a copy of the game state for prediction
        let mut predicted_game = Game {
            big_gravity: self.big_gravity,
            planets: self.planets.clone(),
            player: self.player,
            cached_trajectories: CachedTrajectories {
                player_positions: VecDeque::new(),
                player_velocities: VecDeque::new(),
                player_rotations: VecDeque::new(),
                planet_positions: Vec::new(),
                planet_velocities: Vec::new(),
                is_valid: false,
            },
        };

        let mut player_positions = VecDeque::with_capacity(num_steps);
        let mut player_velocities = VecDeque::with_capacity(num_steps);
        let mut player_rotations = VecDeque::with_capacity(num_steps);
        let mut planet_positions: Vec<VecDeque<Vector2>> = vec![VecDeque::with_capacity(num_steps); self.planets.len()];
        let mut planet_velocities: Vec<VecDeque<Vector2>> = vec![VecDeque::with_capacity(num_steps); self.planets.len()];

        // Simulate forward and collect positions
        for _ in 0..num_steps {
            player_positions.push_back(predicted_game.player.position);
            player_velocities.push_back(predicted_game.player.velocity);
            player_rotations.push_back(predicted_game.player.rotation);

            for (i, planet) in predicted_game.planets.iter().enumerate() {
                planet_positions[i].push_back(planet.position);
                planet_velocities[i].push_back(planet.velocity);
            }

            // Use substeps for accurate physics
            for _ in 0..substeps {
                predicted_game.update(dt / substeps as f64);
            }
        }

        self.cached_trajectories = CachedTrajectories {
            player_positions,
            player_velocities,
            player_rotations,
            planet_positions,
            planet_velocities,
            is_valid: true,
        };
    }

    pub fn advance_trajectory(&mut self) {
        if !self.cached_trajectories.is_valid || self.cached_trajectories.player_positions.is_empty() {
            return;
        }

        // Set player to the first cached position (index 0)
        // NOTE: We DON'T set rotation here - let the player rotate freely
        self.player.position = self.cached_trajectories.player_positions[0];
        self.player.velocity = self.cached_trajectories.player_velocities[0];
        // self.player.rotation = self.cached_trajectories.player_rotations[0]; // Don't overwrite rotation

        // Set planet positions and velocities
        for (i, planet) in self.planets.iter_mut().enumerate() {
            planet.position = self.cached_trajectories.planet_positions[i][0];
            planet.velocity = self.cached_trajectories.planet_velocities[i][0];
        }

        // Remove the positions we just used (index 0) - O(1) with VecDeque
        self.cached_trajectories.player_positions.pop_front();
        self.cached_trajectories.player_velocities.pop_front();
        self.cached_trajectories.player_rotations.pop_front();

        for i in 0..self.planets.len() {
            self.cached_trajectories.planet_positions[i].pop_front();
            self.cached_trajectories.planet_velocities[i].pop_front();
        }
    }

    pub fn extend_trajectories(&mut self, num_steps: usize) {
        // Batch extend multiple steps at once for better performance
        if num_steps == 0 {
            return;
        }

        let dt = TRAJECTORY_DT;
        let substeps = TRAJECTORY_SUBSTEPS;

        // Get the last cached state
        let last_idx = self.cached_trajectories.player_positions.len() - 1;

        let mut predicted_game = Game {
            big_gravity: self.big_gravity,
            planets: self.planets.clone(),
            player: Player {
                position: self.cached_trajectories.player_positions[last_idx],
                velocity: self.cached_trajectories.player_velocities[last_idx],
                rotation: self.cached_trajectories.player_rotations[last_idx],
                mass: self.player.mass,
            },
            cached_trajectories: CachedTrajectories {
                player_positions: VecDeque::new(),
                player_velocities: VecDeque::new(),
                player_rotations: VecDeque::new(),
                planet_positions: Vec::new(),
                planet_velocities: Vec::new(),
                is_valid: false,
            },
        };

        // Set planet states from last cached positions
        for i in 0..predicted_game.planets.len() {
            predicted_game.planets[i].position = self.cached_trajectories.planet_positions[i][last_idx];
            predicted_game.planets[i].velocity = self.cached_trajectories.planet_velocities[i][last_idx];
        }

        // Simulate multiple steps forward
        for _ in 0..num_steps {
            // Use substeps for accurate physics
            for _ in 0..substeps {
                predicted_game.update(dt / substeps as f64);
            }

            // Append the new state
            self.cached_trajectories.player_positions.push_back(predicted_game.player.position);
            self.cached_trajectories.player_velocities.push_back(predicted_game.player.velocity);
            self.cached_trajectories.player_rotations.push_back(predicted_game.player.rotation);

            for i in 0..predicted_game.planets.len() {
                self.cached_trajectories.planet_positions[i].push_back(predicted_game.planets[i].position);
                self.cached_trajectories.planet_velocities[i].push_back(predicted_game.planets[i].velocity);
            }
        }
    }


    pub fn update(&mut self, dt: f64) {
        // Calculate all accelerations for planets
        let mut planet_accelerations = vec![Vector2 { x: 0.0, y: 0.0 }; self.planets.len()];

        // Planet-to-planet forces
        for i in 0..self.planets.len() {
            for j in (i + 1)..self.planets.len() {
                let diff = self.planets[j].position.subtract(&self.planets[i].position);
                let distance = diff.magnitude();
                if distance > 0.0 {
                    let force_magnitude = self.big_gravity / (distance * distance);
                    let direction = diff.scale(1.0 / distance); // normalize

                    // Force on planet i from planet j
                    let accel_i = direction.scale(force_magnitude * self.planets[j].mass);
                    planet_accelerations[i] = planet_accelerations[i].add(&accel_i);

                    // Force on planet j from planet i (opposite direction)
                    let accel_j = direction.scale(-force_magnitude * self.planets[i].mass);
                    planet_accelerations[j] = planet_accelerations[j].add(&accel_j);
                }
            }
        }

        // Player acceleration from planets
        let mut player_acceleration = Vector2 { x: 0.0, y: 0.0 };
        for (i, planet) in self.planets.iter().enumerate() {
            let diff = planet.position.subtract(&self.player.position);
            let distance = diff.magnitude();
            if distance > 0.0 {
                // F = G * m1 * m2 / r^2
                let force_magnitude = self.big_gravity * planet.mass * self.player.mass / (distance * distance);
                let direction = diff.scale(1.0 / distance);

                // Player: a = F / m_player
                let player_accel = direction.scale(force_magnitude / self.player.mass);
                player_acceleration = player_acceleration.add(&player_accel);

                // Planet: a = F / m_planet (opposite direction, Newton's 3rd law)
                let planet_accel = direction.scale(-force_magnitude / planet.mass);
                planet_accelerations[i] = planet_accelerations[i].add(&planet_accel);
            }
        }

        // Update velocities and positions
        for (i, planet) in self.planets.iter_mut().enumerate() {
            planet.velocity = planet.velocity.add(&planet_accelerations[i].scale(dt));
            planet.position = planet.position.add(&planet.velocity.scale(dt));
        }

        self.player.velocity = self.player.velocity.add(&player_acceleration.scale(dt));
        self.player.position = self.player.position.add(&self.player.velocity.scale(dt));
    }
}