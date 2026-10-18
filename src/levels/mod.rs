//! Niveaux : format de données, chargement et ressources Bevy.
//!
//! * [`data`] : structures sérialisables (RON), sans dépendance à Bevy ;
//! * [`loader`] : lecture du dossier `assets/levels` ;
//! * [`builtin`] : niveaux de secours embarqués dans l'exécutable.

pub mod builtin;
pub mod data;
pub mod loader;

use bevy::prelude::*;

use crate::collision::LevelGeometry;
use data::LevelData;

/// Tous les niveaux disponibles, dans l'ordre d'affichage.
#[derive(Resource, Debug, Default)]
pub struct LevelLibrary {
    /// Niveaux chargés.
    pub levels: Vec<LevelData>,
}

/// Indice du niveau sélectionné dans le menu.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct SelectedLevel(pub usize);

/// Niveau en cours de jeu, avec son index de collisions.
#[derive(Resource, Debug)]
pub struct ActiveLevel {
    /// Données brutes du niveau.
    pub data: LevelData,
    /// Boîtes de collision précalculées.
    pub geometry: LevelGeometry,
}

/// Plugin chargeant les niveaux au démarrage.
pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedLevel>()
            .init_resource::<LevelLibrary>()
            .add_systems(Startup, load_library);
    }
}

/// Charge les niveaux depuis le disque (ou les niveaux intégrés).
fn load_library(mut library: ResMut<LevelLibrary>) {
    let report = loader::load_all_levels();
    for error in &report.errors {
        warn!("niveau ignoré : {error}");
    }
    info!("{} niveau(x) chargé(s)", report.levels.len());
    library.levels = report.levels;
}
