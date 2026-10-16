//! Caméra 2D du jeu.
//!
//! Pour l'instant : une caméra fixe. Le défilement automatique, le shake et
//! le suivi du joueur seront ajoutés dans une étape dédiée.

use bevy::prelude::*;

/// Marqueur identifiant la caméra principale.
///
/// Les systèmes futurs (défilement, shake) utiliseront `With<MainCamera>`
/// pour la retrouver sans ambiguïté.
#[derive(Component, Debug)]
pub struct MainCamera;

/// Plugin créant la caméra au démarrage de l'application.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

/// Crée la caméra 2D. `Camera2d` ajoute automatiquement les composants requis
/// (projection orthographique, transformation, etc.).
fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}
