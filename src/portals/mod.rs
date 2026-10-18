//! Apparence des portails.
//!
//! L'*effet* d'un portail est géré par la physique
//! ([`crate::physics::apply_portal`]) ; ce module ne s'occupe que du visuel.

use bevy::prelude::*;

use crate::levels::data::PortalKind;

/// Couleur d'affichage d'un portail.
pub fn portal_color(kind: PortalKind) -> Color {
    match kind {
        PortalKind::GravityFlip => Color::srgb(0.9, 0.3, 0.95),
        PortalKind::GravityNormal => Color::srgb(0.3, 0.6, 1.0),
        PortalKind::GravityInverted => Color::srgb(1.0, 0.6, 0.1),
        PortalKind::Speed(multiplier) if multiplier < 0.99 => Color::srgb(0.4, 0.5, 1.0),
        PortalKind::Speed(multiplier) if multiplier < 1.01 => Color::srgb(0.3, 1.0, 0.4),
        PortalKind::Speed(multiplier) if multiplier < 1.5 => Color::srgb(1.0, 0.95, 0.2),
        PortalKind::Speed(_) => Color::srgb(1.0, 0.25, 0.25),
        PortalKind::Mini => Color::srgb(1.0, 0.45, 0.7),
        PortalKind::Normal => Color::srgb(0.4, 1.0, 0.7),
    }
}

/// Nom lisible d'un portail (éditeur, débogage).
pub fn portal_label(kind: PortalKind) -> String {
    match kind {
        PortalKind::GravityFlip => "Gravite (inversion)".to_string(),
        PortalKind::GravityNormal => "Gravite normale".to_string(),
        PortalKind::GravityInverted => "Gravite inversee".to_string(),
        PortalKind::Speed(multiplier) => format!("Vitesse x{multiplier:.1}"),
        PortalKind::Mini => "Mini".to_string(),
        PortalKind::Normal => "Taille normale".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_label_shows_multiplier() {
        assert_eq!(portal_label(PortalKind::Speed(1.5)), "Vitesse x1.5");
    }

    #[test]
    fn faster_portals_have_different_colors() {
        assert_ne!(
            portal_color(PortalKind::Speed(1.0)),
            portal_color(PortalKind::Speed(2.0))
        );
    }
}
