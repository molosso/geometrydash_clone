//! Boîte englobante alignée sur les axes (AABB), en tuiles.

/// Tolérance numérique : deux boîtes qui se touchent « par la tranche »
/// ne sont pas considérées comme se chevauchant.
pub const EPSILON: f32 = 1e-4;

/// Rectangle défini par son centre et ses demi-dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Abscisse du centre.
    pub cx: f32,
    /// Ordonnée du centre.
    pub cy: f32,
    /// Demi-largeur.
    pub hw: f32,
    /// Demi-hauteur.
    pub hh: f32,
}

impl Aabb {
    /// Crée une boîte à partir de son centre et de ses demi-dimensions.
    pub const fn new(cx: f32, cy: f32, hw: f32, hh: f32) -> Self {
        Self { cx, cy, hw, hh }
    }

    /// Bord gauche.
    pub fn min_x(&self) -> f32 {
        self.cx - self.hw
    }

    /// Bord droit.
    pub fn max_x(&self) -> f32 {
        self.cx + self.hw
    }

    /// Bord inférieur.
    pub fn min_y(&self) -> f32 {
        self.cy - self.hh
    }

    /// Bord supérieur.
    pub fn max_y(&self) -> f32 {
        self.cy + self.hh
    }

    /// Vrai si les deux boîtes se chevauchent réellement (au-delà de
    /// [`EPSILON`]). Se toucher exactement par un bord ne compte pas.
    pub fn overlaps(&self, other: &Aabb) -> bool {
        (self.cx - other.cx).abs() < self.hw + other.hw - EPSILON
            && (self.cy - other.cy).abs() < self.hh + other.hh - EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_boxes_are_detected() {
        let a = Aabb::new(0.0, 0.0, 0.5, 0.5);
        let b = Aabb::new(0.6, 0.0, 0.5, 0.5);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn touching_edges_do_not_overlap() {
        let a = Aabb::new(0.0, 0.0, 0.5, 0.5);
        let b = Aabb::new(1.0, 0.0, 0.5, 0.5);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn far_boxes_do_not_overlap() {
        let a = Aabb::new(0.0, 0.0, 0.5, 0.5);
        let b = Aabb::new(0.0, 5.0, 0.5, 0.5);
        assert!(!a.overlaps(&b));
    }
}
