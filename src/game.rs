//! Machine à états globale du jeu.
//!
//! Bevy gère les « écrans » du jeu avec des *états* (`States`). Chaque système
//! peut être limité à un état (`run_if(in_state(..))`) ou exécuté une seule
//! fois à l'entrée/sortie d'un état (`OnEnter` / `OnExit`).

use bevy::prelude::*;

/// Tous les écrans/modes de l'application.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    /// Démarrage : initialisation, puis (plus tard) chargement des assets.
    #[default]
    Boot,
    /// Menu principal.
    MainMenu,
    /// Sélection du niveau.
    LevelSelect,
    /// Écran des paramètres (volume, etc.).
    Settings,
    /// Meilleurs scores.
    Records,
    /// Partie en cours.
    Playing,
    /// Partie en pause.
    Paused,
    /// Éditeur de niveaux intégré.
    Editor,
    /// Niveau terminé avec succès.
    Victory,
    /// Niveau terminé par un échec définitif.
    Defeat,
}

/// Ordre d'exécution des systèmes de jeu, image par image.
///
/// 1. [`GameplaySet::Input`] : lecture du clavier/souris ;
/// 2. [`GameplaySet::Simulate`] : physique, mort, checkpoints, victoire ;
/// 3. [`GameplaySet::Present`] : affichage (cube, caméra, interface).
///
/// Les trois ensembles ne tournent que dans l'état [`AppState::Playing`] :
/// mettre le jeu en pause suffit donc à tout geler.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    /// Lecture des entrées.
    Input,
    /// Simulation du monde.
    Simulate,
    /// Mise à jour de l'affichage.
    Present,
}

/// Plugin enregistrant la machine à états et la séquence de démarrage.
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (GameplaySet::Input, GameplaySet::Simulate, GameplaySet::Present)
                .chain()
                .run_if(in_state(AppState::Playing)),
        )
        .init_state::<AppState>()
            // Tant que l'on est dans `Boot`, on tente de passer au menu.
            .add_systems(Update, finish_boot.run_if(in_state(AppState::Boot)))
            .add_systems(OnEnter(AppState::MainMenu), log_main_menu_entered);
    }
}

/// Termine la phase de démarrage et bascule vers le menu principal.
///
/// Pour l'instant il n'y a rien à charger. À l'étape « assets », ce système
/// attendra que les ressources soient prêtes avant de changer d'état.
fn finish_boot(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::MainMenu);
}

/// Trace l'entrée dans le menu principal (utile pour le débogage).
fn log_main_menu_entered() {
    info!("Entrée dans le menu principal");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'application doit toujours démarrer dans l'état `Boot`.
    #[test]
    fn default_state_is_boot() {
        assert_eq!(AppState::default(), AppState::Boot);
    }
}
