//! Vérifie que chaque niveau livré avec le jeu est **réalisable**.
//!
//! Un petit « robot » explore toutes les façons de maintenir ou relâcher la
//! touche de saut (une décision toutes les 1/60 s). Si au moins une suite de
//! décisions mène à la ligne d'arrivée, le niveau est faisable.

use std::collections::HashMap;

use cube_dash::collision::LevelGeometry;
use cube_dash::levels::builtin::builtin_levels;
use cube_dash::levels::data::LevelData;
use cube_dash::physics::{step, Gravity, PlayerInput, PlayerState, Rules, SUBSTEP};

/// Nombre de sous-pas par décision (4 × 1/240 s = 1/60 s).
const SUBSTEPS_PER_DECISION: u32 = 4;

/// Clé de déduplication : deux états proches sont considérés équivalents.
type StateKey = (i32, i32, bool, i8, i8, i32, usize);

fn key_of(state: &PlayerState) -> StateKey {
    (
        (state.y * 20.0).round() as i32,
        (state.vel_y * 4.0).round() as i32,
        state.grounded,
        if state.gravity == Gravity::Down { 0 } else { 1 },
        state.size as i8,
        (state.speed_mult * 10.0).round() as i32,
        state.activated.len(),
    )
}

/// Exploration en largeur, une couche par décision.
fn is_beatable(level: &LevelData) -> bool {
    let geometry = LevelGeometry::from_level(level);
    let rules = Rules::default();
    let mut layer = vec![PlayerState::new(level.start_speed)];

    // Borne de sécurité : 120 s de jeu.
    for _tick in 0..(120 * 60) {
        let mut next: HashMap<StateKey, PlayerState> = HashMap::new();
        for state in &layer {
            for held in [true, false] {
                let mut candidate = state.clone();
                let mut dead = false;
                let input = PlayerInput {
                    jump_held: held,
                    jump_pressed: false,
                };
                for _ in 0..SUBSTEPS_PER_DECISION {
                    let events = step(&mut candidate, &input, &geometry, &rules, SUBSTEP);
                    if events.finished {
                        return true;
                    }
                    if events.death.is_some() {
                        dead = true;
                        break;
                    }
                }
                if !dead {
                    next.entry(key_of(&candidate)).or_insert(candidate);
                }
            }
        }
        if next.is_empty() {
            return false;
        }
        layer = next.into_values().collect();
    }
    false
}

#[test]
fn every_builtin_level_is_beatable() {
    let (levels, errors) = builtin_levels();
    assert!(errors.is_empty(), "{errors:?}");
    for level in &levels {
        assert!(
            is_beatable(level),
            "le niveau « {} » est infaisable",
            level.name
        );
    }
}

#[test]
fn solver_rejects_an_impossible_level() {
    // Six pics côte à côte forment un mur de 6 tuiles : aucun saut ne le franchit.
    let text = r#"(
        name: "Impossible",
        length: 60.0,
        objects: [
            (kind: Spike, x: 20, y: 0), (kind: Spike, x: 21, y: 0), (kind: Spike, x: 22, y: 0),
            (kind: Spike, x: 23, y: 0), (kind: Spike, x: 24, y: 0), (kind: Spike, x: 25, y: 0),
        ],
    )"#;
    let level = LevelData::from_ron_str(text).unwrap();
    assert!(!is_beatable(&level));
}
