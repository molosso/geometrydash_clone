//! Création des entités visuelles d'un niveau : sol, blocs, pics, portails…
//!
//! Les collisions ne passent **pas** par ces entités : elles sont calculées
//! à partir des données du niveau ([`crate::collision::LevelGeometry`]).
//! Ces entités ne sont que de l'affichage, ce qui garde la simulation rapide.

use bevy::prelude::*;

use crate::gameplay::GameplayEntity;
use crate::levels::data::{LevelData, ObjectKind};
use crate::portals::portal_color;
use crate::utils::constants::{TILE_SIZE, Z_GROUND, Z_OBSTACLES, Z_PORTALS};
use crate::utils::coords::{tile_to_world_x, tile_to_world_y};

/// Crée tout le décor et les obstacles d'un niveau.
pub fn spawn_level(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    level: &LevelData,
) {
    spawn_world_bounds(commands, level);

    // Un seul maillage par orientation de pic, partagé par toutes les entités.
    let half = TILE_SIZE * 0.5;
    let spike_up = meshes.add(Triangle2d::new(
        Vec2::new(-half, -half),
        Vec2::new(half, -half),
        Vec2::new(0.0, half),
    ));
    let spike_down = meshes.add(Triangle2d::new(
        Vec2::new(-half, half),
        Vec2::new(half, half),
        Vec2::new(0.0, -half),
    ));
    let spike_material = materials.add(Color::srgb(0.95, 0.25, 0.3));

    for object in &level.objects {
        let (x, y) = (object.x as f32, object.y as f32);
        match object.kind {
            ObjectKind::Block => {
                spawn_rect(
                    commands,
                    tile_center(x + 0.5, y + 0.5),
                    Vec2::splat(TILE_SIZE),
                    Color::srgb(0.12, 0.26, 0.50),
                    Z_OBSTACLES,
                );
                spawn_rect(
                    commands,
                    tile_center(x + 0.5, y + 0.5),
                    Vec2::splat(TILE_SIZE - 8.0),
                    Color::srgb(0.20, 0.42, 0.78),
                    Z_OBSTACLES + 0.1,
                );
            }
            ObjectKind::Platform => {
                spawn_rect(
                    commands,
                    tile_center(x + 0.5, y + 0.85),
                    Vec2::new(TILE_SIZE, TILE_SIZE * 0.3),
                    Color::srgb(0.95, 0.62, 0.20),
                    Z_OBSTACLES,
                );
            }
            ObjectKind::Spike => {
                let center = tile_center(x + 0.5, y + 0.5);
                commands.spawn((
                    GameplayEntity,
                    Mesh2d(spike_up.clone()),
                    MeshMaterial2d(spike_material.clone()),
                    Transform::from_xyz(center.x, center.y, Z_OBSTACLES),
                ));
            }
            ObjectKind::SpikeDown => {
                let center = tile_center(x + 0.5, y + 0.5);
                commands.spawn((
                    GameplayEntity,
                    Mesh2d(spike_down.clone()),
                    MeshMaterial2d(spike_material.clone()),
                    Transform::from_xyz(center.x, center.y, Z_OBSTACLES),
                ));
            }
            ObjectKind::Checkpoint => {
                spawn_rect(
                    commands,
                    tile_center(x + 0.5, y + 1.5),
                    Vec2::new(TILE_SIZE * 0.12, TILE_SIZE * 3.0),
                    Color::srgb(1.0, 0.9, 0.2),
                    Z_PORTALS,
                );
            }
            ObjectKind::Portal(kind) => {
                spawn_rect(
                    commands,
                    tile_center(x + 0.5, y + 1.0),
                    Vec2::new(TILE_SIZE * 0.6, TILE_SIZE * 2.0),
                    portal_color(kind),
                    Z_PORTALS,
                );
                spawn_rect(
                    commands,
                    tile_center(x + 0.5, y + 1.0),
                    Vec2::new(TILE_SIZE * 0.25, TILE_SIZE * 1.8),
                    Color::srgba(1.0, 1.0, 1.0, 0.45),
                    Z_PORTALS + 0.1,
                );
            }
        }
    }
}

/// Sol, plafond et ligne d'arrivée.
fn spawn_world_bounds(commands: &mut Commands, level: &LevelData) {
    let width = (level.length + 80.0) * TILE_SIZE;
    let center_x = tile_to_world_x(level.length / 2.0);
    let ground_color = Color::srgb(0.07, 0.09, 0.20);
    let line_color = Color::srgb(0.35, 0.85, 1.0);

    // Sol : un grand rectangle sous la ligne y = 0.
    spawn_rect(
        commands,
        Vec2::new(center_x, tile_to_world_y(-5.0)),
        Vec2::new(width, TILE_SIZE * 10.0),
        ground_color,
        Z_GROUND,
    );
    spawn_rect(
        commands,
        Vec2::new(center_x, tile_to_world_y(0.0) - 2.0),
        Vec2::new(width, 4.0),
        line_color,
        Z_GROUND + 0.1,
    );

    // Plafond : même principe, au-dessus de `height`.
    spawn_rect(
        commands,
        Vec2::new(center_x, tile_to_world_y(level.height + 5.0)),
        Vec2::new(width, TILE_SIZE * 10.0),
        ground_color,
        Z_GROUND,
    );
    spawn_rect(
        commands,
        Vec2::new(center_x, tile_to_world_y(level.height) + 2.0),
        Vec2::new(width, 4.0),
        line_color,
        Z_GROUND + 0.1,
    );

    // Ligne d'arrivée : colonne lumineuse sur toute la hauteur.
    spawn_rect(
        commands,
        Vec2::new(
            tile_to_world_x(level.length),
            tile_to_world_y(level.height / 2.0),
        ),
        Vec2::new(TILE_SIZE * 0.5, level.height * TILE_SIZE),
        Color::srgba(1.0, 1.0, 1.0, 0.35),
        Z_PORTALS,
    );
    spawn_rect(
        commands,
        Vec2::new(
            tile_to_world_x(level.length),
            tile_to_world_y(level.height / 2.0),
        ),
        Vec2::new(6.0, level.height * TILE_SIZE),
        Color::WHITE,
        Z_PORTALS + 0.1,
    );
}

/// Centre en pixels d'un point exprimé en tuiles.
fn tile_center(tile_x: f32, tile_y: f32) -> Vec2 {
    Vec2::new(tile_to_world_x(tile_x), tile_to_world_y(tile_y))
}

/// Crée un rectangle uni qui disparaît avec le niveau.
fn spawn_rect(commands: &mut Commands, center: Vec2, size: Vec2, color: Color, z: f32) {
    commands.spawn((
        GameplayEntity,
        Sprite::from_color(color, size),
        Transform::from_xyz(center.x, center.y, z),
    ));
}
