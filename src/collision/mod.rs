//! Index spatial des obstacles d'un niveau.
//!
//! [`LevelGeometry`] transforme la liste d'objets d'un [`LevelData`] en boîtes
//! de collision triées par abscisse. Pendant la simulation, on ne teste que
//! les quelques objets proches du joueur (recherche binaire) au lieu de tout
//! le niveau : le coût par pas reste constant, même pour un niveau géant.

use crate::levels::data::{LevelData, ObjectKind, PortalKind};
use crate::physics::{Aabb, START_X};

/// Distance (en tuiles) de part et d'autre du joueur dans laquelle on cherche
/// des obstacles. Les objets font au plus 1 tuile de large, le joueur 1 aussi.
const SEARCH_REACH: f32 = 2.0;

/// Zone de déclenchement d'un portail.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalZone {
    /// Indice de l'objet dans le niveau (identifiant stable).
    pub index: usize,
    /// Effet du portail.
    pub kind: PortalKind,
    /// Zone de contact.
    pub aabb: Aabb,
}

/// Zone de déclenchement d'un checkpoint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckpointZone {
    /// Indice de l'objet dans le niveau.
    pub index: usize,
    /// Zone de contact.
    pub aabb: Aabb,
}

/// Boîte de collision d'un type d'objet placé en `(x, y)`.
///
/// Les pics ont une boîte volontairement plus étroite que leur dessin :
/// c'est ce qui rend les collisions précises et justes.
pub fn object_aabb(kind: ObjectKind, x: i32, y: i32) -> Aabb {
    let (fx, fy) = (x as f32, y as f32);
    match kind {
        ObjectKind::Block => Aabb::new(fx + 0.5, fy + 0.5, 0.5, 0.5),
        ObjectKind::Platform => Aabb::new(fx + 0.5, fy + 0.85, 0.5, 0.15),
        ObjectKind::Spike => Aabb::new(fx + 0.5, fy + 0.3, 0.15, 0.3),
        ObjectKind::SpikeDown => Aabb::new(fx + 0.5, fy + 0.7, 0.15, 0.3),
        ObjectKind::Checkpoint => Aabb::new(fx + 0.5, fy + 1.0, 0.3, 2.0),
        ObjectKind::Portal(_) => Aabb::new(fx + 0.5, fy + 1.0, 0.3, 1.0),
    }
}

/// Toutes les boîtes de collision d'un niveau, prêtes à être interrogées.
#[derive(Debug, Clone, Default)]
pub struct LevelGeometry {
    solids: Vec<Aabb>,
    hazards: Vec<Aabb>,
    portals: Vec<PortalZone>,
    checkpoints: Vec<CheckpointZone>,
    /// Hauteur du plafond (tuiles).
    pub height: f32,
    /// Abscisse de la ligne d'arrivée (tuiles).
    pub finish_x: f32,
}

/// Renvoie la tranche d'éléments (triés par `min_x`) proches de `x`.
fn window<T>(items: &[T], x: f32, min_x: impl Fn(&T) -> f32) -> &[T] {
    let start = items.partition_point(|item| min_x(item) < x - SEARCH_REACH);
    let end = items.partition_point(|item| min_x(item) <= x + SEARCH_REACH);
    &items[start..end]
}

impl LevelGeometry {
    /// Construit l'index à partir des données du niveau.
    pub fn from_level(level: &LevelData) -> Self {
        let mut geometry = LevelGeometry {
            height: level.height,
            finish_x: level.length,
            ..Default::default()
        };

        for (index, object) in level.objects.iter().enumerate() {
            let aabb = object_aabb(object.kind, object.x, object.y);
            match object.kind {
                ObjectKind::Block | ObjectKind::Platform => geometry.solids.push(aabb),
                ObjectKind::Spike | ObjectKind::SpikeDown => geometry.hazards.push(aabb),
                ObjectKind::Checkpoint => {
                    geometry.checkpoints.push(CheckpointZone { index, aabb });
                }
                ObjectKind::Portal(kind) => {
                    geometry.portals.push(PortalZone { index, kind, aabb });
                }
            }
        }

        // Tri par bord gauche : indispensable pour la recherche binaire.
        let by_min_x = |a: f32, b: f32| a.total_cmp(&b);
        geometry
            .solids
            .sort_by(|a, b| by_min_x(a.min_x(), b.min_x()));
        geometry
            .hazards
            .sort_by(|a, b| by_min_x(a.min_x(), b.min_x()));
        geometry
            .portals
            .sort_by(|a, b| by_min_x(a.aabb.min_x(), b.aabb.min_x()));
        geometry
            .checkpoints
            .sort_by(|a, b| by_min_x(a.aabb.min_x(), b.aabb.min_x()));
        geometry
    }

    /// Solides (blocs, dalles) proches de l'abscisse `x`.
    pub fn solids_near(&self, x: f32) -> &[Aabb] {
        window(&self.solids, x, |a| a.min_x())
    }

    /// Pics proches de l'abscisse `x`.
    pub fn hazards_near(&self, x: f32) -> &[Aabb] {
        window(&self.hazards, x, |a| a.min_x())
    }

    /// Portails proches de l'abscisse `x`.
    pub fn portals_near(&self, x: f32) -> &[PortalZone] {
        window(&self.portals, x, |p| p.aabb.min_x())
    }

    /// Checkpoints proches de l'abscisse `x`.
    pub fn checkpoints_near(&self, x: f32) -> &[CheckpointZone] {
        window(&self.checkpoints, x, |c| c.aabb.min_x())
    }

    /// Progression (0.0 à 1.0) pour une abscisse donnée.
    pub fn progress(&self, x: f32) -> f32 {
        let span = (self.finish_x - START_X).max(f32::EPSILON);
        ((x - START_X) / span).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::data::LevelObject;

    fn level(objects: Vec<LevelObject>) -> LevelData {
        LevelData {
            name: "t".into(),
            length: 100.0,
            height: 10.0,
            start_speed: 1.0,
            background: [0.0; 3],
            objects,
        }
    }

    fn obj(kind: ObjectKind, x: i32, y: i32) -> LevelObject {
        LevelObject { kind, x, y }
    }

    #[test]
    fn objects_are_classified() {
        let geo = LevelGeometry::from_level(&level(vec![
            obj(ObjectKind::Block, 10, 0),
            obj(ObjectKind::Spike, 10, 0),
            obj(ObjectKind::Portal(PortalKind::Mini), 10, 0),
            obj(ObjectKind::Checkpoint, 10, 0),
        ]));
        assert_eq!(geo.solids_near(10.0).len(), 1);
        assert_eq!(geo.hazards_near(10.0).len(), 1);
        assert_eq!(geo.portals_near(10.0).len(), 1);
        assert_eq!(geo.checkpoints_near(10.0).len(), 1);
    }

    #[test]
    fn far_objects_are_not_returned() {
        let geo = LevelGeometry::from_level(&level(vec![obj(ObjectKind::Block, 50, 0)]));
        assert!(geo.solids_near(10.0).is_empty());
        assert_eq!(geo.solids_near(50.0).len(), 1);
    }

    #[test]
    fn portal_keeps_object_index() {
        let geo = LevelGeometry::from_level(&level(vec![
            obj(ObjectKind::Block, 5, 0),
            obj(ObjectKind::Portal(PortalKind::Normal), 20, 0),
        ]));
        assert_eq!(geo.portals_near(20.0)[0].index, 1);
    }

    #[test]
    fn progress_is_clamped() {
        let geo = LevelGeometry::from_level(&level(vec![]));
        assert_eq!(geo.progress(-5.0), 0.0);
        assert_eq!(geo.progress(50.0), 0.5);
        assert_eq!(geo.progress(500.0), 1.0);
    }
}
