//! # Cube Dash
//!
//! Clone open source d'un jeu de plateforme rythmique, construit avec le moteur
//! [Bevy](https://bevyengine.org).
//!
//! ## Organisation
//!
//! Chaque domaine du jeu est un module indépendant, la plupart exposant un
//! `Plugin` Bevy que [`app`] assemble :
//!
//! | Module        | Rôle                                                        |
//! |---------------|-------------------------------------------------------------|
//! | [`game`]      | machine à états globale et ordre des systèmes               |
//! | [`gameplay`]  | mise en place/nettoyage d'une partie, pause, victoire       |
//! | [`physics`]   | simulation du cube (pure, sans Bevy, très testée)           |
//! | [`collision`] | index spatial des obstacles                                 |
//! | [`levels`]    | format RON, chargement, niveaux intégrés                    |
//! | [`player`]    | entrées, sous-pas de simulation, rendu du cube              |
//! | [`obstacles`] | entités visuelles : sol, blocs, pics, portails             |
//! | [`portals`]   | apparence des portails                                      |
//! | [`camera`]    | caméra à défilement automatique                             |
//! | [`menu`]      | menu principal                                              |
//! | [`ui`]        | ancrage à l'écran et HUD                                    |
//! | [`utils`]     | constantes et conversions de coordonnées                    |

pub mod app;
pub mod camera;
pub mod collision;
pub mod game;
pub mod gameplay;
pub mod levels;
pub mod menu;
pub mod obstacles;
pub mod physics;
pub mod player;
pub mod portals;
pub mod ui;
pub mod utils;
