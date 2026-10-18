//! Interface : ancrage à l'écran et HUD (tentatives, progression, temps, score).
//!
//! Le texte utilise la police intégrée à Bevy, qui ne contient que les
//! caractères ASCII : les messages à l'écran n'ont donc pas d'accents.
//! Une vraie police (dossier `assets/fonts`) viendra avec l'étape « menus ».
//!
//! ## Ancrage
//!
//! Les éléments d'interface sont de simples entités du monde 2D qui suivent la
//! caméra. [`ScreenAnchor`] indique où les placer par rapport à l'écran, ce
//! qui évite d'utiliser le système d'UI de Bevy pour ces besoins simples.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::MainCamera;
use crate::game::GameplaySet;
use crate::gameplay::{GameplayEntity, Session};
use crate::levels::ActiveLevel;
use crate::utils::constants::Z_UI;

/// Largeur de la barre de progression (pixels).
const BAR_WIDTH: f32 = 420.0;
/// Hauteur de la barre de progression (pixels).
const BAR_HEIGHT: f32 = 12.0;

/// Position d'une entité par rapport à l'écran.
///
/// Position finale = centre de la caméra + `(fx × demi-largeur, fy × demi-hauteur)`
/// + `(ox, oy)` pixels. Exemples : `(-1, 1)` = coin haut-gauche, `(0, 0)` = centre.
#[derive(Component, Debug, Clone, Copy)]
pub struct ScreenAnchor {
    /// Fraction horizontale de la demi-largeur de l'écran (-1 gauche, 1 droite).
    pub fx: f32,
    /// Fraction verticale de la demi-hauteur de l'écran (-1 bas, 1 haut).
    pub fy: f32,
    /// Décalage horizontal en pixels.
    pub ox: f32,
    /// Décalage vertical en pixels.
    pub oy: f32,
}

impl ScreenAnchor {
    /// Centre de l'écran.
    pub const CENTER: ScreenAnchor = ScreenAnchor {
        fx: 0.0,
        fy: 0.0,
        ox: 0.0,
        oy: 0.0,
    };

    /// Crée un ancrage.
    pub const fn new(fx: f32, fy: f32, ox: f32, oy: f32) -> Self {
        Self { fx, fy, ox, oy }
    }
}

/// Marqueur du bloc de texte du HUD.
#[derive(Component, Debug)]
pub struct HudText;

/// Marqueur de la partie remplie de la barre de progression.
#[derive(Component, Debug)]
pub struct ProgressBarFill;

/// Plugin de l'interface.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_hud, update_progress_bar).in_set(GameplaySet::Present),
        )
        // Toujours actif (jeu, pause, menu) ; après la caméra, pour éviter tout décalage.
        .add_systems(Update, apply_screen_anchors.after(GameplaySet::Present));
    }
}

/// Crée le HUD d'une partie.
pub fn spawn_hud(commands: &mut Commands) {
    // Bloc de texte en haut à gauche.
    commands.spawn((
        GameplayEntity,
        HudText,
        Text2d::new(""),
        TextColor(Color::WHITE),
        ScreenAnchor::new(-1.0, 1.0, 150.0, -70.0),
        Transform::from_xyz(0.0, 0.0, Z_UI).with_scale(Vec3::splat(1.1)),
    ));

    // Fond de la barre de progression, en haut au centre.
    commands.spawn((
        GameplayEntity,
        Sprite::from_color(
            Color::srgba(1.0, 1.0, 1.0, 0.18),
            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
        ),
        ScreenAnchor::new(0.0, 1.0, 0.0, -28.0),
        Transform::from_xyz(0.0, 0.0, Z_UI),
    ));

    // Partie remplie : sa largeur est pilotée par `Transform::scale.x`.
    commands.spawn((
        GameplayEntity,
        ProgressBarFill,
        Sprite::from_color(
            Color::srgb(0.3, 0.95, 0.6),
            Vec2::new(BAR_WIDTH, BAR_HEIGHT),
        ),
        ScreenAnchor::new(0.0, 1.0, -BAR_WIDTH / 2.0, -28.0),
        Transform::from_xyz(0.0, 0.0, Z_UI + 0.1).with_scale(Vec3::new(0.001, 1.0, 1.0)),
    ));
}

/// Place les éléments ancrés par rapport à la caméra et à la taille de la fenêtre.
fn apply_screen_anchors(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<&Transform, (With<MainCamera>, Without<ScreenAnchor>)>,
    mut anchored: Query<(&ScreenAnchor, &mut Transform), Without<MainCamera>>,
) {
    let (Some(window), Some(camera)) = (windows.iter().next(), cameras.iter().next()) else {
        return;
    };
    let (half_w, half_h) = (window.width() / 2.0, window.height() / 2.0);

    for (anchor, mut transform) in &mut anchored {
        transform.translation.x = camera.translation.x + anchor.fx * half_w + anchor.ox;
        transform.translation.y = camera.translation.y + anchor.fy * half_h + anchor.oy;
    }
}

/// Met à jour le texte du HUD (uniquement quand il change).
fn update_hud(
    session: Res<Session>,
    active: Res<ActiveLevel>,
    new_huds: Query<(), Added<HudText>>,
    mut last: Local<String>,
    mut texts: Query<&mut Text2d, With<HudText>>,
) {
    // Un HUD vient d'apparaître : on oublie le cache pour forcer l'écriture.
    if !new_huds.is_empty() {
        last.clear();
    }

    let content = format!(
        "{}\nTentative {}\nProgres {}%\nTemps {:.1} s\nScore {}",
        active.data.name,
        session.attempts,
        session.percent(),
        session.attempt_time,
        session.score,
    );
    if *last == content {
        return;
    }
    for mut text in &mut texts {
        *text = Text2d::new(content.clone());
    }
    *last = content;
}

/// Ajuste la barre de progression.
fn update_progress_bar(
    session: Res<Session>,
    mut bars: Query<(&mut Transform, &mut ScreenAnchor), With<ProgressBarFill>>,
) {
    let progress = session.progress.clamp(0.0, 1.0);
    for (mut transform, mut anchor) in &mut bars {
        transform.scale.x = progress.max(0.001);
        // Garde le bord gauche fixe pendant que la barre s'élargit.
        anchor.ox = -BAR_WIDTH / 2.0 + BAR_WIDTH * progress / 2.0;
    }
}
