//! Menu principal.
//!
//! Pour l'instant : un simple cube affiché en placeholder, afin de vérifier
//! que le rendu et les transitions d'états fonctionnent. L'interface réelle
//! (boutons, texte) arrive à l'étape UI.

use bevy::prelude::*;

use crate::game::AppState;
use crate::utils::constants::PLACEHOLDER_CUBE_SIZE;

/// Marqueur des entités appartenant au menu : elles sont détruites en le quittant.
#[derive(Component, Debug)]
struct MenuEntity;

/// Plugin du menu principal.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_menu)
            .add_systems(OnExit(AppState::MainMenu), despawn_menu);
    }
}

/// Affiche un cube cyan au centre de l'écran.
fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        MenuEntity,
        // `Sprite::from_color` dessine un rectangle uni, sans texture.
        Sprite::from_color(Color::srgb(0.2, 0.9, 1.0), Vec2::splat(PLACEHOLDER_CUBE_SIZE)),
    ));
}

/// Supprime toutes les entités du menu.
fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
