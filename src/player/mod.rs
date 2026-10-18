//! Le cube : entrées, simulation par sous-pas et rendu.
//!
//! La logique de mouvement est dans [`crate::physics`] (pure). Ici, on :
//! 1. lit le clavier/la souris ([`read_input`]) ;
//! 2. appelle la physique plusieurs fois par image ([`simulate`]) ;
//! 3. gère mort, réapparition, checkpoints et victoire ;
//! 4. synchronise le sprite avec l'état physique ([`sync_player`]).

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;

use crate::game::{AppState, GameplaySet};
use crate::gameplay::{
    GameEvent, GameEvents, GameplayEntity, GameplaySettings, Session, DEATH_DELAY,
};
use crate::levels::ActiveLevel;
use crate::physics::{self, PlayerInput, PlayerState, Rules, MAX_FRAME_DT, SUBSTEP};
use crate::utils::constants::{TILE_SIZE, Z_PLAYER};
use crate::utils::coords::{tile_to_world_x, tile_to_world_y};

/// Vitesse de rotation du cube en l'air (radians par seconde).
const ROTATION_SPEED: f32 = 7.5;

/// Composant du cube affiché.
#[derive(Component, Debug, Default)]
pub struct Player {
    /// Angle visuel actuel (radians).
    pub angle: f32,
}

/// Entrées de saut de l'image courante.
#[derive(Resource, Debug, Default)]
pub struct JumpInput {
    /// Une touche de saut est maintenue.
    pub held: bool,
    /// Une touche vient d'être pressée (consommé par la simulation).
    pub pressed: bool,
}

/// Le cube court ou attend de réapparaître.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunPhase {
    /// Partie en cours.
    Running,
    /// Mort ; réapparition dans `remaining` secondes.
    Dead {
        /// Temps restant avant la réapparition.
        remaining: f32,
    },
}

/// État de simulation du joueur pendant une partie.
#[derive(Resource, Debug, Clone)]
pub struct PlayerSim {
    /// État physique courant.
    pub state: PlayerState,
    /// État de départ du niveau.
    pub initial: PlayerState,
    /// Dernier checkpoint atteint.
    pub checkpoint: Option<PlayerState>,
    /// Phase de jeu.
    pub phase: RunPhase,
}

impl PlayerSim {
    /// Crée la simulation à partir de l'état initial du niveau.
    pub fn new(initial: PlayerState) -> Self {
        Self {
            state: initial.clone(),
            initial,
            checkpoint: None,
            phase: RunPhase::Running,
        }
    }
}

/// Plugin du joueur.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JumpInput>()
            .add_systems(Update, read_input.in_set(GameplaySet::Input))
            .add_systems(Update, simulate.in_set(GameplaySet::Simulate))
            .add_systems(Update, sync_player.in_set(GameplaySet::Present));
    }
}

/// Crée le cube (un carré cyan avec un « visage » pour voir sa rotation).
pub fn spawn_player(commands: &mut Commands) {
    commands
        .spawn((
            GameplayEntity,
            Player::default(),
            Sprite::from_color(Color::srgb(0.2, 0.9, 1.0), Vec2::splat(TILE_SIZE)),
            Transform::from_xyz(tile_to_world_x(0.0), tile_to_world_y(0.5), Z_PLAYER),
        ))
        .with_children(|parent| {
            // Cadre sombre intérieur.
            parent.spawn((
                Sprite::from_color(Color::srgb(0.03, 0.12, 0.22), Vec2::splat(TILE_SIZE * 0.55)),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
            // « Œil » : permet de voir tourner le cube.
            parent.spawn((
                Sprite::from_color(Color::WHITE, Vec2::splat(TILE_SIZE * 0.16)),
                Transform::from_xyz(TILE_SIZE * 0.10, TILE_SIZE * 0.08, 0.2),
            ));
        });
}

/// Lit les touches de saut : Espace, Flèche haut, W ou clic gauche.
fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut input: ResMut<JumpInput>,
) {
    const JUMP_KEYS: [KeyCode; 3] = [KeyCode::Space, KeyCode::ArrowUp, KeyCode::KeyW];
    input.held = keys.any_pressed(JUMP_KEYS) || mouse.pressed(MouseButton::Left);
    // `|=` : un appui n'est jamais perdu, même si la simulation ne tourne pas cette image.
    input.pressed |= keys.any_just_pressed(JUMP_KEYS) || mouse.just_pressed(MouseButton::Left);
}

/// Fait avancer la simulation de l'image courante.
fn simulate(
    time: Res<Time>,
    active: Res<ActiveLevel>,
    settings: Res<GameplaySettings>,
    mut input: ResMut<JumpInput>,
    mut sim: ResMut<PlayerSim>,
    mut session: ResMut<Session>,
    mut events: ResMut<GameEvents>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    events.0.clear();
    let sim: &mut PlayerSim = &mut sim;
    let frame_dt = time.delta_secs().min(MAX_FRAME_DT);

    // --- Mort : on attend, puis on réapparaît ------------------------------
    if let RunPhase::Dead { remaining } = sim.phase {
        input.pressed = false;
        let remaining = remaining - frame_dt;
        if remaining > 0.0 {
            sim.phase = RunPhase::Dead { remaining };
        } else {
            respawn(sim, &mut session, &settings);
        }
        return;
    }

    // --- Sous-pas : la frame est découpée en pas ≤ 1/240 s ----------------
    let steps = ((frame_dt / SUBSTEP).ceil() as u32).max(1);
    let dt = frame_dt / steps as f32;
    let rules = Rules {
        double_jump: settings.double_jump,
    };
    let mut pressed = std::mem::take(&mut input.pressed);

    for _ in 0..steps {
        let step_input = PlayerInput {
            jump_held: input.held,
            jump_pressed: pressed,
        };
        pressed = false; // l'appui n'est utilisé que par le premier sous-pas

        let outcome = physics::step(&mut sim.state, &step_input, &active.geometry, &rules, dt);

        if outcome.jumped {
            events.0.push(GameEvent::Jumped);
        }
        if outcome.landed {
            events.0.push(GameEvent::Landed);
        }
        for kind in &outcome.portals {
            events.0.push(GameEvent::PortalEntered(*kind));
        }
        if outcome.checkpoint.is_some() && settings.checkpoints {
            sim.checkpoint = Some(sim.state.clone());
            events.0.push(GameEvent::CheckpointReached);
        }
        if let Some(cause) = outcome.death {
            sim.phase = RunPhase::Dead {
                remaining: DEATH_DELAY,
            };
            events.0.push(GameEvent::Died(cause));
            return;
        }
        if outcome.finished {
            session.completed = true;
            events.0.push(GameEvent::LevelCompleted);
            next_state.set(AppState::Victory);
            return;
        }
    }
}

/// Replace le cube au dernier checkpoint (ou au départ) et compte une tentative.
fn respawn(sim: &mut PlayerSim, session: &mut Session, settings: &GameplaySettings) {
    let restored = if settings.checkpoints {
        sim.checkpoint.clone()
    } else {
        None
    };
    sim.state = restored.unwrap_or_else(|| sim.initial.clone());
    sim.phase = RunPhase::Running;
    session.attempts += 1;
    session.attempt_time = 0.0;
}

/// Synchronise le sprite du cube avec l'état physique.
fn sync_player(
    time: Res<Time>,
    sim: Res<PlayerSim>,
    mut query: Query<(&mut Player, &mut Transform, &mut Visibility)>,
) {
    let dt = time.delta_secs();
    let alive = sim.phase == RunPhase::Running;

    for (mut player, mut transform, mut visibility) in &mut query {
        // Rotation : tourne en l'air, se cale sur un quart de tour au sol.
        if alive {
            if sim.state.grounded {
                let target = (player.angle / FRAC_PI_2).round() * FRAC_PI_2;
                player.angle += (target - player.angle) * (20.0 * dt).min(1.0);
            } else {
                player.angle += sim.state.gravity.sign() * ROTATION_SPEED * dt;
            }
            player.angle = player.angle.rem_euclid(TAU);
        }

        transform.translation.x = tile_to_world_x(sim.state.x);
        transform.translation.y = tile_to_world_y(sim.state.y);
        transform.translation.z = Z_PLAYER;
        transform.rotation = Quat::from_rotation_z(player.angle);
        // Le sprite fait 1 tuile ; le demi-côté physique vaut 0,5 (normal) ou 0,3 (mini).
        transform.scale = Vec3::splat(sim.state.size.half_extent() * 2.0);

        let target = if alive {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != target {
            *visibility = target;
        }
    }
}
