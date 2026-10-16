//! Assemblage de l'application : plugins Bevy par défaut + plugins du jeu.

use bevy::prelude::*;

use crate::camera::CameraPlugin;
use crate::game::GameStatePlugin;
use crate::menu::MenuPlugin;
use crate::utils::constants::{BACKGROUND_RGB, WINDOW_TITLE};

/// Plugin racine regroupant tous les plugins propres au jeu.
///
/// Séparer « plugins Bevy » et « plugins du jeu » permet de tester le jeu
/// avec `MinimalPlugins` (sans fenêtre ni GPU), voir `tests/`.
pub struct CubeDashPlugin;

impl Plugin for CubeDashPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GameStatePlugin, CameraPlugin, MenuPlugin));
    }
}

/// Construit l'[`App`] complète, prête à être lancée avec `.run()`.
pub fn build_app() -> App {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            // Configuration de la fenêtre principale.
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: WINDOW_TITLE.to_string(),
                    ..default()
                }),
                ..default()
            })
            // Filtrage « nearest » : les textures restent nettes (style pixel art).
            .set(ImagePlugin::default_nearest()),
    )
    // Couleur de fond utilisée quand rien d'autre n'est dessiné.
    .insert_resource(ClearColor(Color::srgb(
        BACKGROUND_RGB[0],
        BACKGROUND_RGB[1],
        BACKGROUND_RGB[2],
    )))
    .add_plugins(CubeDashPlugin);

    app
}
