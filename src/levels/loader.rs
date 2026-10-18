//! Chargement des niveaux depuis le disque.
//!
//! Ajouter un niveau = déposer un fichier `.ron` dans `assets/levels/`.
//! Les fichiers sont triés par nom : préfixer par `01_`, `02_`… fixe l'ordre.

use std::fs;
use std::path::{Path, PathBuf};

use super::builtin::builtin_levels;
use super::data::{LevelData, LevelError};

/// Résultat d'un chargement : les niveaux valides et les problèmes rencontrés.
#[derive(Debug, Default)]
pub struct LoadReport {
    /// Niveaux lus avec succès, dans l'ordre alphabétique des fichiers.
    pub levels: Vec<LevelData>,
    /// Messages d'erreur (un par fichier refusé).
    pub errors: Vec<String>,
}

/// Dossier des niveaux : `<racine des assets>/assets/levels`.
///
/// La racine est, dans l'ordre : `BEVY_ASSET_ROOT`, `CARGO_MANIFEST_DIR`
/// (défini par `cargo run`), le dossier de l'exécutable, puis le dossier courant.
pub fn levels_dir() -> PathBuf {
    let root = std::env::var_os("BEVY_ASSET_ROOT")
        .or_else(|| std::env::var_os("CARGO_MANIFEST_DIR"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    root.join("assets").join("levels")
}

/// Lit un fichier de niveau.
pub fn load_level_file(path: &Path) -> Result<LevelData, LevelError> {
    let text = fs::read_to_string(path).map_err(|error| LevelError::Io(error.to_string()))?;
    LevelData::from_ron_str(&text)
}

/// Lit tous les fichiers `.ron` d'un dossier.
pub fn load_levels_from_dir(dir: &Path) -> LoadReport {
    let mut report = LoadReport::default();

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            report.errors.push(format!("{} : {error}", dir.display()));
            return report;
        }
    };

    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "ron"))
        .collect();
    paths.sort();

    for path in paths {
        match load_level_file(&path) {
            Ok(level) => report.levels.push(level),
            Err(error) => report.errors.push(format!("{} : {error}", path.display())),
        }
    }
    report
}

/// Charge les niveaux du dossier des assets, avec repli sur les niveaux
/// intégrés si aucun niveau valide n'est trouvé.
pub fn load_all_levels() -> LoadReport {
    let mut report = load_levels_from_dir(&levels_dir());
    if report.levels.is_empty() {
        let (levels, errors) = builtin_levels();
        report.levels = levels;
        report.errors.extend(errors);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_directory_is_reported_not_fatal() {
        let report = load_levels_from_dir(Path::new("/chemin/qui/n/existe/pas"));
        assert!(report.levels.is_empty());
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn project_levels_directory_loads_cleanly() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("levels");
        let report = load_levels_from_dir(&dir);
        assert!(report.errors.is_empty(), "erreurs : {:?}", report.errors);
        assert!(report.levels.len() >= 3);
    }
}
