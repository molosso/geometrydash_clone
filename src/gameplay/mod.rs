//! Orchestration d'une partie : mise en place, pause, victoire, nettoyage.
//!
//! Ce module relie les autres : il crée le niveau et le joueur quand on
//! entre dans [`AppState::Playing`], et détruit tout en retournant au menu.
//!
//! ## Cycle de vie
//!
//! * `OnEnter(Playing)` : [`setup_gameplay`] (sans effet si une partie existe
//!   déjà, ce qui permet de reprendre après une pause) ;
//! * `OnEnter(MainMenu)` : [`teardown_gameplay`] ;
//! * pause et victoire : simples états, la partie reste en mémoire.

pub mod session;

use bevy::prelude::*;

use crate::game::{AppState, GameplaySet};
use crate::levels::data::PortalKind;
use crate::levels::{ActiveLevel, LevelLibrary, SelectedLevel};
use crate::obstacles;
use crate::physics::{DeathCause, PlayerState};
use crate::player::{self, JumpInput, PlayerSim, RunPhase};
use crate::ui::{self, ScreenAnchor};
use crate::utils::constants::{BACKGROUND_RGB, Z_UI};
pub use session::Session;

/// Marqueur de toute entité qui appartient à la partie en cours.
#[derive(Component, Debug)]
pub struct GameplayEntity;

/// Marqueur des textes superposés (pause, victoire), détruits en quittant l'état.
#[derive(Component, Debug)]
pub struct OverlayEntity;

/// Réglages de gameplay (modifiables depuis le menu).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameplaySettings {
    /// Autorise un saut supplémentaire en l'air.
    pub double_jump: bool,
    /// Réapparition aux checkpoints.
    pub checkpoints: bool,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            double_jump: false,
            checkpoints: true,
        }
    }
}

/// Événements de gameplay, pour l'audio, les particules, etc.
///
/// Ils sont vidés au début de chaque image de simulation : les systèmes
/// qui les consomment doivent tourner dans [`GameplaySet::Present`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameEvent {
    /// Le cube a sauté.
    Jumped,
    /// Le cube s'est posé.
    Landed,
    /// Le cube est mort.
    Died(DeathCause),
    /// Un checkpoint a été atteint.
    CheckpointReached,
    /// Un portail a été traversé.
    PortalEntered(PortalKind),
    /// Le niveau est terminé.
    LevelCompleted,
}

/// File des [`GameEvent`] de l'image courante.
#[derive(Resource, Debug, Default)]
pub struct GameEvents(pub Vec<GameEvent>);

/// Durée (secondes) entre la mort et la réapparition.
pub const DEATH_DELAY: f32 = 0.6;

/// Plugin de la boucle de jeu.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplaySettings>()
            .init_resource::<GameEvents>()
            .add_systems(OnEnter(AppState::Playing), setup_gameplay)
            .add_systems(OnEnter(AppState::MainMenu), teardown_gameplay)
            .add_systems(OnEnter(AppState::Paused), spawn_pause_overlay)
            .add_systems(OnExit(AppState::Paused), despawn_overlays)
            .add_systems(OnEnter(AppState::Victory), spawn_victory_overlay)
            .add_systems(OnExit(AppState::Victory), despawn_overlays)
            .add_systems(
                Update,
                (pause_input, restart_input).in_set(GameplaySet::Input),
            )
            .add_systems(Update, session::update_session.in_set(GameplaySet::Present))
            .add_systems(Update, paused_input.run_if(in_state(AppState::Paused)))
            .add_systems(Update, victory_input.run_if(in_state(AppState::Victory)));
    }
}

/// Crée le niveau, le joueur et l'interface au lancement d'une partie.
fn setup_gameplay(
    mut commands: Commands,
    library: Res<LevelLibrary>,
    selected: Res<SelectedLevel>,
    active: Option<Res<ActiveLevel>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut input: ResMut<JumpInput>,
    mut events: ResMut<GameEvents>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Reprise après une pause : la partie existe déjà, on ne recrée rien.
    if active.is_some() {
        return;
    }
    if library.levels.is_empty() {
        error!("aucun niveau disponible");
        next_state.set(AppState::MainMenu);
        return;
    }

    let index = selected.0.min(library.levels.len() - 1);
    let data = library.levels[index].clone();
    let geometry = crate::collision::LevelGeometry::from_level(&data);

    obstacles::spawn_level(&mut commands, &mut meshes, &mut materials, &data);
    player::spawn_player(&mut commands);
    ui::spawn_hud(&mut commands);

    commands.insert_resource(ClearColor(Color::srgb(
        data.background[0],
        data.background[1],
        data.background[2],
    )));
    commands.insert_resource(PlayerSim::new(PlayerState::new(data.start_speed)));
    commands.insert_resource(Session::new(index));
    commands.insert_resource(ActiveLevel { data, geometry });

    *input = JumpInput::default();
    events.0.clear();
}

/// Détruit toutes les entités de la partie.
fn despawn_gameplay(commands: &mut Commands, entities: &Query<Entity, With<GameplayEntity>>) {
    for entity in entities {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<ActiveLevel>();
    commands.remove_resource::<PlayerSim>();
    commands.remove_resource::<Session>();
}

/// Nettoyage au retour au menu principal.
fn teardown_gameplay(mut commands: Commands, entities: Query<Entity, With<GameplayEntity>>) {
    despawn_gameplay(&mut commands, &entities);
    commands.insert_resource(ClearColor(Color::srgb(
        BACKGROUND_RGB[0],
        BACKGROUND_RGB[1],
        BACKGROUND_RGB[2],
    )));
}

/// Échap pendant la partie : pause.
fn pause_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Paused);
    }
}

/// R pendant la partie : recommencer depuis le début (checkpoint oublié).
fn restart_input(keys: Res<ButtonInput<KeyCode>>, mut sim: ResMut<PlayerSim>) {
    if keys.just_pressed(KeyCode::KeyR) {
        sim.checkpoint = None;
        sim.phase = RunPhase::Dead { remaining: 0.0 };
    }
}

/// Contrôles de l'écran de pause.
fn paused_input(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Playing);
    } else if keys.just_pressed(KeyCode::KeyQ) {
        next_state.set(AppState::MainMenu);
    }
}

/// Contrôles de l'écran de victoire.
fn victory_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    entities: Query<Entity, With<GameplayEntity>>,
    library: Res<LevelLibrary>,
    mut selected: ResMut<SelectedLevel>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        // Niveau suivant (on reboucle sur le premier après le dernier).
        if !library.levels.is_empty() {
            selected.0 = (selected.0 + 1) % library.levels.len();
        }
        despawn_gameplay(&mut commands, &entities);
        next_state.set(AppState::Playing);
    } else if keys.just_pressed(KeyCode::KeyR) {
        despawn_gameplay(&mut commands, &entities);
        next_state.set(AppState::Playing);
    } else if keys.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::MainMenu);
    }
}

/// Texte « PAUSE » affiché au centre de l'écran.
fn spawn_pause_overlay(mut commands: Commands) {
    spawn_overlay(
        &mut commands,
        "PAUSE\n\nEchap : reprendre\nQ : menu principal",
    );
}

/// Écran de victoire avec les statistiques de la partie.
fn spawn_victory_overlay(mut commands: Commands, session: Res<Session>) {
    let text = format!(
        "NIVEAU TERMINE !\n\nTentatives : {}\nTemps total : {:.1} s\nScore : {}\n\n\
         Entree : niveau suivant\nR : rejouer\nEchap : menu principal",
        session.attempts, session.total_time, session.score
    );
    spawn_overlay(&mut commands, &text);
}

/// Crée un texte centré, au-dessus de tout.
fn spawn_overlay(commands: &mut Commands, text: &str) {
    commands.spawn((
        OverlayEntity,
        Text2d::new(text),
        TextColor(Color::WHITE),
        ScreenAnchor::CENTER,
        Transform::from_xyz(0.0, 0.0, Z_UI + 1.0).with_scale(Vec3::splat(1.5)),
    ));
}

/// Détruit les textes superposés.
fn despawn_overlays(mut commands: Commands, overlays: Query<Entity, With<OverlayEntity>>) {
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
}
