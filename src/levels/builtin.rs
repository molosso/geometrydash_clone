//! Niveaux intégrés à l'exécutable.
//!
//! Ils servent de solution de repli si le dossier `assets/levels` est absent
//! (par exemple si on déplace le binaire seul). Les fichiers sont lus à la
//! compilation grâce à `include_str!`.

use super::data::LevelData;

/// Paires (nom de fichier, contenu RON) des niveaux intégrés.
pub const BUILTIN_LEVELS: [(&str, &str); 3] = [
    (
        "01_premiers_pas.ron",
        include_str!("../../assets/levels/01_premiers_pas.ron"),
    ),
    (
        "02_vitesse.ron",
        include_str!("../../assets/levels/02_vitesse.ron"),
    ),
    (
        "03_gravite.ron",
        include_str!("../../assets/levels/03_gravite.ron"),
    ),
];

/// Charge tous les niveaux intégrés. Un niveau intégré invalide serait un
/// bug du jeu : il est ignoré et signalé dans la liste d'erreurs retournée.
pub fn builtin_levels() -> (Vec<LevelData>, Vec<String>) {
    let mut levels = Vec::new();
    let mut errors = Vec::new();
    for (file, text) in BUILTIN_LEVELS {
        match LevelData::from_ron_str(text) {
            Ok(level) => levels.push(level),
            Err(error) => errors.push(format!("{file} : {error}")),
        }
    }
    (levels, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_builtin_levels_are_valid() {
        let (levels, errors) = builtin_levels();
        assert!(errors.is_empty(), "niveaux intégrés invalides : {errors:?}");
        assert_eq!(levels.len(), BUILTIN_LEVELS.len());
    }
}
