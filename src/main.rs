//! Point d'entrée du binaire.
//!
//! Ce fichier est volontairement minuscule : toute la logique vit dans la
//! bibliothèque `cube_dash`, ce qui permet de la tester sans lancer de fenêtre.

use bevy::prelude::AppExit;

/// Construit l'application puis lance la boucle principale de Bevy.
///
/// `App::run` renvoie un [`AppExit`] : retourner cette valeur depuis `main`
/// donne un code de sortie correct au système d'exploitation.
fn main() -> AppExit {
    cube_dash::app::build_app().run()
}
