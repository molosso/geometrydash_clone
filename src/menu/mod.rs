//! Menu principal : choix du niveau et lancement de la partie.
//!
//! Contrôles : ↑/↓ (ou ←/→) changent de niveau, `D` active/désactive le
//! double saut, `Entrée` lance la partie. Une interface complète (boutons,
//! paramètres, records) arrivera à l'étape UI.

use bevy::prelude::*;

use crate::game::AppState;
use crate::gameplay::GameplaySettings;
use crate::levels::{LevelLibrary, SelectedLevel};
use crate::ui::ScreenAnchor;
use crate::utils::constants::{PLACEHOLDER_CUBE_SIZE, Z_UI};

/// Marqueur des entités appartenant au menu : elles sont détruites en le quittant.
#[derive(Component, Debug)]
struct MenuEntity;

/// Marqueur du texte d'information (niveau choisi, aide).
#[derive(Component, Debug)]
struct MenuInfoText;

/// Plugin du menu principal.
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_menu)
            .add_systems(OnExit(AppState::MainMenu), despawn_menu)
            .add_systems(
                Update,
                (menu_input, update_menu_text)
                    .chain()
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}

/// Crée le titre, le cube décoratif et le texte d'information.
fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        MenuEntity,
        // `Sprite::from_color` dessine un rectangle uni, sans texture.
        Sprite::from_color(
            Color::srgb(0.2, 0.9, 1.0),
            Vec2::splat(PLACEHOLDER_CUBE_SIZE),
        ),
        Transform::from_xyz(0.0, 60.0, 0.0),
    ));
    commands.spawn((
        MenuEntity,
        Text2d::new("CUBE DASH"),
        TextColor(Color::WHITE),
        ScreenAnchor::new(0.0, 0.0, 0.0, 190.0),
        Transform::from_xyz(0.0, 0.0, Z_UI).with_scale(Vec3::splat(3.0)),
    ));
    commands.spawn((
        MenuEntity,
        MenuInfoText,
        Text2d::new(""),
        TextColor(Color::srgb(0.85, 0.9, 1.0)),
        ScreenAnchor::new(0.0, 0.0, 0.0, -110.0),
        Transform::from_xyz(0.0, 0.0, Z_UI).with_scale(Vec3::splat(1.2)),
    ));
}

/// Supprime toutes les entités du menu.
fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Gère le clavier dans le menu.
fn menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    library: Res<LevelLibrary>,
    mut selected: ResMut<SelectedLevel>,
    mut settings: ResMut<GameplaySettings>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let count = library.levels.len();
    if count > 0 {
        if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::ArrowRight]) {
            selected.0 = (selected.0 + 1) % count;
        }
        if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::ArrowLeft]) {
            selected.0 = (selected.0 + count - 1) % count;
        }
        // Protège contre un indice devenu invalide (niveaux rechargés).
        selected.0 = selected.0.min(count - 1);
    }

    if keys.just_pressed(KeyCode::KeyD) {
        settings.double_jump = !settings.double_jump;
    }
    if count > 0 && keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        next_state.set(AppState::Playing);
    }
}

/// Réécrit le texte d'information (uniquement quand il change).
fn update_menu_text(
    library: Res<LevelLibrary>,
    selected: Res<SelectedLevel>,
    settings: Res<GameplaySettings>,
    new_texts: Query<(), Added<MenuInfoText>>,
    mut last: Local<String>,
    mut texts: Query<&mut Text2d, With<MenuInfoText>>,
) {
    if !new_texts.is_empty() {
        last.clear();
    }

    let content = if library.levels.is_empty() {
        "Aucun niveau trouve dans assets/levels".to_string()
    } else {
        format!(
            "< {} >   ({}/{})\n\nEntree : jouer\nHaut / Bas : changer de niveau\n\
             D : double saut [{}]\n\n\
             En jeu : Espace / Haut / W / clic = sauter\nR : recommencer   Echap : pause",
            library.levels[selected.0].name,
            selected.0 + 1,
            library.levels.len(),
            if settings.double_jump { "OUI" } else { "NON" },
        )
    };

    if *last == content {
        return;
    }
    for mut text in &mut texts {
        *text = Text2d::new(content.clone());
    }
    *last = content;
}
