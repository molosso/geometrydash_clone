//! Caméra 2D du jeu.
//!
//! Pendant une partie, la caméra défile automatiquement : elle reste à
//! distance fixe devant le joueur. Le tremblement (shake) et les effets de
//! transition seront ajoutés à l'étape « effets ».

use bevy::prelude::*;

use crate::game::{AppState, GameplaySet};
use crate::player::PlayerSim;
use crate::utils::constants::CAMERA_LEAD_TILES;
use crate::utils::coords::tile_to_world_x;

/// Marqueur identifiant la caméra principale.
#[derive(Component, Debug)]
pub struct MainCamera;

/// Plugin de la caméra.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(OnEnter(AppState::MainMenu), reset_camera)
            .add_systems(Update, follow_player.in_set(GameplaySet::Present));
    }
}

/// Crée la caméra 2D. `Camera2d` ajoute automatiquement les composants requis
/// (projection orthographique, transformation, etc.).
fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

/// Ramène la caméra à l'origine (menu principal).
fn reset_camera(mut cameras: Query<&mut Transform, With<MainCamera>>) {
    for mut transform in &mut cameras {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

/// Défilement automatique : la caméra précède le joueur de quelques tuiles.
fn follow_player(sim: Res<PlayerSim>, mut cameras: Query<&mut Transform, With<MainCamera>>) {
    let target_x = tile_to_world_x(sim.state.x + CAMERA_LEAD_TILES);
    for mut transform in &mut cameras {
        transform.translation.x = target_x;
        transform.translation.y = 0.0;
    }
}
