//! Test d'intégration : vérifie la séquence de démarrage sans fenêtre ni GPU.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use cube_dash::game::{AppState, GameStatePlugin};

/// Construit une application « headless » contenant uniquement la machine à états.
fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, GameStatePlugin));
    app
}

/// Au premier tick l'état est `Boot`.
#[test]
fn starts_in_boot() {
    let mut app = headless_app();
    app.update();

    let state = app.world().resource::<State<AppState>>();
    assert_eq!(*state.get(), AppState::Boot);
}

/// Après quelques ticks, `Boot` doit avoir laissé la place au menu principal.
#[test]
fn boot_leads_to_main_menu() {
    let mut app = headless_app();
    for _ in 0..3 {
        app.update();
    }

    let state = app.world().resource::<State<AppState>>();
    assert_eq!(*state.get(), AppState::MainMenu);
}
