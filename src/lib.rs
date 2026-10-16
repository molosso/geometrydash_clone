//! # Cube Dash
//!
//! Clone open source d'un jeu de plateforme rythmique, construit avec le moteur
//! [Bevy](https://bevyengine.org).
//!
//! ## Organisation
//!
//! Chaque domaine du jeu est un module indépendant exposant un `Plugin` Bevy.
//! Le module [`app`] les assemble, [`game`] porte la machine à états globale.

pub mod app;
pub mod camera;
pub mod game;
pub mod menu;
pub mod utils;
