//! Constantes globales du jeu.
//!
//! Centraliser les « nombres magiques » ici évite de les disperser et facilite
//! l'équilibrage du gameplay. Les constantes de *physique* (gravité, vitesse,
//! saut) vivent dans [`crate::physics`], car elles sont exprimées en tuiles.

/// Titre de la fenêtre.
pub const WINDOW_TITLE: &str = "Cube Dash";

/// Couleur de fond du menu (sRGB, composantes de 0.0 à 1.0).
pub const BACKGROUND_RGB: [f32; 3] = [0.04, 0.04, 0.10];

/// Taille (en pixels) du cube de démonstration affiché dans le menu.
pub const PLACEHOLDER_CUBE_SIZE: f32 = 96.0;

/// Taille d'une tuile en pixels (1 unité de niveau = 1 tuile).
pub const TILE_SIZE: f32 = 48.0;

/// Position verticale (en pixels, repère Bevy) de la surface du sol.
pub const GROUND_SCREEN_Y: f32 = -230.0;

/// Avance de la caméra sur le joueur, en tuiles : le joueur est affiché
/// à gauche de l'écran et voit venir les obstacles.
pub const CAMERA_LEAD_TILES: f32 = 6.0;

/// Profondeur (axe Z) du décor lointain.
pub const Z_GROUND: f32 = 0.0;
/// Profondeur des blocs, plateformes et pics.
pub const Z_OBSTACLES: f32 = 1.0;
/// Profondeur des portails et checkpoints.
pub const Z_PORTALS: f32 = 2.0;
/// Profondeur du joueur.
pub const Z_PLAYER: f32 = 3.0;
/// Profondeur de l'interface (au-dessus de tout).
pub const Z_UI: f32 = 100.0;
