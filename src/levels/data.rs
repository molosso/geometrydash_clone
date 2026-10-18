//! Format de données d'un niveau, sérialisable en RON.
//!
//! Ce module ne dépend **pas** de Bevy : on peut le tester et l'utiliser
//! dans des outils externes. Toutes les positions sont en *tuiles* entières :
//! une tuile occupe `[x, x+1] × [y, y+1]`, avec `y = 0` juste au-dessus du sol.
//!
//! Exemple de fichier `.ron` :
//!
//! ```ron
//! (
//!     name: "Mon niveau",
//!     length: 100.0,
//!     objects: [
//!         (kind: Block, x: 20, y: 0),
//!         (kind: Spike, x: 30, y: 0),
//!         (kind: Portal(Speed(1.3)), x: 50, y: 0),
//!     ],
//! )
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

/// Les différents portails du jeu.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PortalKind {
    /// Inverse le sens de la gravité à chaque passage.
    GravityFlip,
    /// Force la gravité vers le bas.
    GravityNormal,
    /// Force la gravité vers le haut (« portail inversé »).
    GravityInverted,
    /// Fixe le multiplicateur de vitesse (1.0 = vitesse de base).
    Speed(f32),
    /// Réduit la taille du cube.
    Mini,
    /// Rend au cube sa taille normale.
    Normal,
}

/// Le type d'un objet placé dans le niveau.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ObjectKind {
    /// Bloc plein, solide : on peut atterrir dessus, le toucher de côté tue.
    Block,
    /// Dalle fine posée en haut de la tuile, solide.
    Platform,
    /// Pic dirigé vers le haut, mortel.
    Spike,
    /// Pic dirigé vers le bas (au plafond), mortel.
    SpikeDown,
    /// Point de contrôle : le joueur y réapparaît après une mort.
    Checkpoint,
    /// Portail modifiant l'état du cube.
    Portal(PortalKind),
}

/// Un objet placé sur la grille du niveau.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LevelObject {
    /// Type de l'objet.
    pub kind: ObjectKind,
    /// Colonne de la tuile (0 = point de départ du joueur).
    pub x: i32,
    /// Ligne de la tuile (0 = juste au-dessus du sol).
    pub y: i32,
}

/// Un niveau complet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelData {
    /// Nom affiché dans les menus.
    pub name: String,
    /// Position (en tuiles) de la ligne d'arrivée.
    pub length: f32,
    /// Hauteur du plafond (en tuiles).
    #[serde(default = "default_height")]
    pub height: f32,
    /// Multiplicateur de vitesse initial.
    #[serde(default = "default_start_speed")]
    pub start_speed: f32,
    /// Couleur de fond (sRGB).
    #[serde(default = "default_background")]
    pub background: [f32; 3],
    /// Objets du niveau.
    #[serde(default)]
    pub objects: Vec<LevelObject>,
}

fn default_height() -> f32 {
    10.0
}

fn default_start_speed() -> f32 {
    1.0
}

fn default_background() -> [f32; 3] {
    [0.05, 0.05, 0.14]
}

/// Abscisse minimale d'un objet : laisse une piste d'élan au départ.
pub const MIN_OBJECT_X: i32 = 3;

/// Erreurs possibles lors du chargement d'un niveau.
#[derive(Debug, Clone, PartialEq)]
pub enum LevelError {
    /// Le texte RON est invalide.
    Parse(String),
    /// Le niveau est syntaxiquement correct mais incohérent.
    Invalid(String),
    /// Erreur d'entrée/sortie (lecture ou écriture de fichier).
    Io(String),
}

impl fmt::Display for LevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelError::Parse(msg) => write!(f, "niveau illisible : {msg}"),
            LevelError::Invalid(msg) => write!(f, "niveau invalide : {msg}"),
            LevelError::Io(msg) => write!(f, "erreur de fichier : {msg}"),
        }
    }
}

impl std::error::Error for LevelError {}

impl LevelData {
    /// Lit un niveau depuis du texte RON, puis le valide.
    pub fn from_ron_str(text: &str) -> Result<Self, LevelError> {
        let level: LevelData =
            ron::from_str(text).map_err(|error| LevelError::Parse(error.to_string()))?;
        level.validate()?;
        Ok(level)
    }

    /// Sérialise le niveau en RON lisible (utilisé par l'éditeur de niveaux).
    pub fn to_ron_string(&self) -> Result<String, LevelError> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|error| LevelError::Parse(error.to_string()))
    }

    /// Vérifie la cohérence du niveau.
    pub fn validate(&self) -> Result<(), LevelError> {
        let invalid = |msg: String| Err(LevelError::Invalid(msg));

        if self.name.trim().is_empty() {
            return invalid("le nom est vide".to_string());
        }
        if self.length < 10.0 {
            return invalid(format!("longueur trop courte ({})", self.length));
        }
        if !(4.0..=30.0).contains(&self.height) {
            return invalid(format!("hauteur hors limites 4..30 ({})", self.height));
        }
        if !(0.25..=4.0).contains(&self.start_speed) {
            return invalid(format!(
                "vitesse initiale hors limites ({})",
                self.start_speed
            ));
        }

        for (index, object) in self.objects.iter().enumerate() {
            if object.x < MIN_OBJECT_X {
                return invalid(format!(
                    "objet #{index} : x={} < {MIN_OBJECT_X} (piste d'élan)",
                    object.x
                ));
            }
            if object.y < 0 || object.y as f32 >= self.height {
                return invalid(format!("objet #{index} : y={} hors du niveau", object.y));
            }
            if let ObjectKind::Portal(PortalKind::Speed(speed)) = object.kind {
                if !(0.25..=4.0).contains(&speed) {
                    return invalid(format!("objet #{index} : vitesse {speed} hors limites"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"(
        name: "Test",
        length: 50.0,
        objects: [
            (kind: Block, x: 10, y: 0),
            (kind: Portal(Speed(1.5)), x: 20, y: 0),
            (kind: Portal(GravityFlip), x: 30, y: 0),
        ],
    )"#;

    #[test]
    fn parses_sample_with_defaults() {
        let level = LevelData::from_ron_str(SAMPLE).expect("niveau valide");
        assert_eq!(level.name, "Test");
        assert_eq!(level.height, 10.0);
        assert_eq!(level.start_speed, 1.0);
        assert_eq!(level.objects.len(), 3);
        assert_eq!(
            level.objects[1].kind,
            ObjectKind::Portal(PortalKind::Speed(1.5))
        );
    }

    #[test]
    fn round_trips_through_ron() {
        let level = LevelData::from_ron_str(SAMPLE).unwrap();
        let text = level.to_ron_string().unwrap();
        assert_eq!(LevelData::from_ron_str(&text).unwrap(), level);
    }

    #[test]
    fn rejects_garbage() {
        assert!(matches!(
            LevelData::from_ron_str("pas du ron"),
            Err(LevelError::Parse(_))
        ));
    }

    #[test]
    fn rejects_object_on_runway() {
        let text = r#"(name: "X", length: 50.0, objects: [(kind: Spike, x: 0, y: 0)])"#;
        assert!(matches!(
            LevelData::from_ron_str(text),
            Err(LevelError::Invalid(_))
        ));
    }

    #[test]
    fn rejects_object_above_ceiling() {
        let text = r#"(name: "X", length: 50.0, objects: [(kind: Block, x: 10, y: 99)])"#;
        assert!(matches!(
            LevelData::from_ron_str(text),
            Err(LevelError::Invalid(_))
        ));
    }
}
