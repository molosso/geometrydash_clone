//! Constantes globales du jeu.
//!
//! Centraliser les « nombres magiques » ici évite de les disperser et facilite
//! l'équilibrage du gameplay.

/// Titre de la fenêtre.
pub const WINDOW_TITLE: &str = "Cube Dash";

/// Couleur de fond (sRGB, composantes de 0.0 à 1.0).
pub const BACKGROUND_RGB: [f32; 3] = [0.04, 0.04, 0.10];

/// Taille (en pixels) du cube de démonstration affiché dans le menu.
pub const PLACEHOLDER_CUBE_SIZE: f32 = 96.0;
