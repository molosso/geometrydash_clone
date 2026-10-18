//! Statistiques d'une partie : tentatives, chronomètre, progression, score.

use bevy::prelude::*;

use crate::levels::ActiveLevel;
use crate::physics::MAX_FRAME_DT;
use crate::player::{PlayerSim, RunPhase};

/// Statistiques de la session de jeu en cours (un niveau).
#[derive(Resource, Debug, Clone)]
pub struct Session {
    /// Indice du niveau dans la bibliothèque.
    pub level_index: usize,
    /// Numéro de la tentative en cours (commence à 1).
    pub attempts: u32,
    /// Durée de la tentative en cours (secondes).
    pub attempt_time: f32,
    /// Durée cumulée de toutes les tentatives (secondes).
    pub total_time: f32,
    /// Progression actuelle, de 0.0 à 1.0.
    pub progress: f32,
    /// Meilleure progression atteinte pendant la session.
    pub best_progress: f32,
    /// Score courant.
    pub score: u32,
    /// Vrai une fois le niveau terminé.
    pub completed: bool,
}

impl Session {
    /// Nouvelle session sur le niveau `level_index`.
    pub fn new(level_index: usize) -> Self {
        Self {
            level_index,
            attempts: 1,
            attempt_time: 0.0,
            total_time: 0.0,
            progress: 0.0,
            best_progress: 0.0,
            score: 0,
            completed: false,
        }
    }

    /// Progression en pourcentage entier (0 à 100).
    pub fn percent(&self) -> u32 {
        (self.progress * 100.0).floor() as u32
    }
}

/// Calcule le score.
///
/// * 1000 points répartis sur la meilleure progression ;
/// * à la victoire, 1000 points de bonus, moins 50 par tentative ratée.
pub fn compute_score(best_progress: f32, completed: bool, attempts: u32) -> u32 {
    let base = (best_progress.clamp(0.0, 1.0) * 1000.0).round() as u32;
    if completed {
        let penalty = attempts.saturating_sub(1).saturating_mul(50);
        base + 1000u32.saturating_sub(penalty)
    } else {
        base
    }
}

/// Met à jour chronomètre, progression et score après la simulation.
pub fn update_session(
    time: Res<Time>,
    sim: Res<PlayerSim>,
    active: Res<ActiveLevel>,
    mut session: ResMut<Session>,
) {
    if sim.phase == RunPhase::Running {
        let dt = time.delta_secs().min(MAX_FRAME_DT);
        session.attempt_time += dt;
        session.total_time += dt;
    }

    session.progress = active.geometry.progress(sim.state.x);
    session.best_progress = session.best_progress.max(session.progress);
    session.score = compute_score(session.best_progress, session.completed, session.attempts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_follows_progress() {
        assert_eq!(compute_score(0.0, false, 1), 0);
        assert_eq!(compute_score(0.5, false, 3), 500);
    }

    #[test]
    fn completion_bonus_shrinks_with_attempts() {
        assert_eq!(compute_score(1.0, true, 1), 2000);
        assert_eq!(compute_score(1.0, true, 5), 1800);
    }

    #[test]
    fn completion_bonus_never_goes_negative() {
        assert_eq!(compute_score(1.0, true, 1000), 1000);
    }

    #[test]
    fn percent_is_floored() {
        let mut session = Session::new(0);
        session.progress = 0.567;
        assert_eq!(session.percent(), 56);
    }
}
