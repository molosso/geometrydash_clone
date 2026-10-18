//! Tests d'intégration de la physique : ces tests n'ont besoin ni de Bevy
//! en exécution, ni de fenêtre — la simulation est une fonction pure.

use cube_dash::collision::LevelGeometry;
use cube_dash::levels::data::{LevelData, LevelObject, ObjectKind, PortalKind};
use cube_dash::physics::{
    step, DeathCause, Gravity, PlayerInput, PlayerSize, PlayerState, Rules, SUBSTEP,
};

/// Construit un niveau minimal contenant les objets donnés.
fn level(objects: Vec<(ObjectKind, i32, i32)>) -> LevelData {
    LevelData {
        name: "test".into(),
        length: 200.0,
        height: 10.0,
        start_speed: 1.0,
        background: [0.0; 3],
        objects: objects
            .into_iter()
            .map(|(kind, x, y)| LevelObject { kind, x, y })
            .collect(),
    }
}

/// Fait avancer la simulation de `seconds` secondes. S'arrête à la première mort.
fn run(
    state: &mut PlayerState,
    geometry: &LevelGeometry,
    held: bool,
    seconds: f32,
) -> Option<DeathCause> {
    let input = PlayerInput {
        jump_held: held,
        jump_pressed: false,
    };
    let steps = (seconds / SUBSTEP).round() as u32;
    for _ in 0..steps {
        let events = step(state, &input, geometry, &Rules::default(), SUBSTEP);
        if events.death.is_some() {
            return events.death;
        }
    }
    None
}

#[test]
fn cube_stays_on_ground_when_idle() {
    let geometry = LevelGeometry::from_level(&level(vec![]));
    let mut state = PlayerState::new(1.0);
    assert_eq!(run(&mut state, &geometry, false, 2.0), None);
    assert!(state.grounded);
    assert!((state.y - 0.5).abs() < 1e-3, "y = {}", state.y);
}

#[test]
fn jump_reaches_expected_height_then_lands() {
    let geometry = LevelGeometry::from_level(&level(vec![]));
    let mut state = PlayerState::new(1.0);
    let input = PlayerInput {
        jump_held: true,
        jump_pressed: true,
    };
    step(&mut state, &input, &geometry, &Rules::default(), SUBSTEP);

    let mut apex = state.y;
    let release = PlayerInput::default();
    for _ in 0..(1.0 / SUBSTEP) as u32 {
        step(&mut state, &release, &geometry, &Rules::default(), SUBSTEP);
        apex = apex.max(state.y);
    }
    // Hauteur de saut ≈ 2,1 tuiles au-dessus de la position de repos (0,5).
    assert!((apex - 0.5 - 2.1).abs() < 0.15, "apex = {apex}");
    assert!(state.grounded, "le cube doit être revenu au sol");
}

#[test]
fn hitting_a_block_from_the_side_kills() {
    let geometry = LevelGeometry::from_level(&level(vec![(ObjectKind::Block, 10, 0)]));
    let mut state = PlayerState::new(1.0);
    assert_eq!(
        run(&mut state, &geometry, false, 2.0),
        Some(DeathCause::Block)
    );
}

#[test]
fn touching_a_spike_kills() {
    let geometry = LevelGeometry::from_level(&level(vec![(ObjectKind::Spike, 10, 0)]));
    let mut state = PlayerState::new(1.0);
    assert_eq!(
        run(&mut state, &geometry, false, 2.0),
        Some(DeathCause::Spike)
    );
}

#[test]
fn landing_on_a_block_is_safe() {
    // Une longue rangée de blocs (hauteur 1) : on saute dessus et on y reste posé.
    let blocks: Vec<_> = (14..60).map(|x| (ObjectKind::Block, x, 0)).collect();
    let geometry = LevelGeometry::from_level(&level(blocks));
    let mut state = PlayerState::new(1.0);

    // Court à vide jusqu'à x ≈ 11, puis saute une seule fois.
    assert_eq!(run(&mut state, &geometry, false, 1.1), None);
    let press = PlayerInput {
        jump_held: true,
        jump_pressed: true,
    };
    step(&mut state, &press, &geometry, &Rules::default(), SUBSTEP);

    // Touche relâchée : le cube retombe sur les blocs et y reste.
    assert_eq!(run(&mut state, &geometry, false, 2.0), None);
    assert!(state.grounded);
    assert!((state.y - 1.5).abs() < 1e-3, "y = {}", state.y);
}

#[test]
fn never_tunnels_through_a_thin_platform_at_max_speed() {
    // Dalle très fine (0,3 tuile) et chute à vitesse terminale : le cube doit
    // se poser dessus, jamais la traverser.
    let geometry = LevelGeometry::from_level(&level(vec![(ObjectKind::Platform, 10, 3)]));
    let mut state = PlayerState::new(1.0);
    state.x = 9.8;
    state.y = 8.0;
    state.grounded = false;
    state.vel_y = -40.0;

    let platform_top = 3.0 + 1.0; // sommet de la dalle : y + 1
    let mut was_on_platform = false;
    for _ in 0..2000 {
        let events = step(
            &mut state,
            &PlayerInput::default(),
            &geometry,
            &Rules::default(),
            SUBSTEP,
        );
        assert!(
            events.death.is_none(),
            "mort inattendue : {:?}",
            events.death
        );
        // Tant que le cube est franchement au-dessus de la dalle en x, il reste dessus.
        if state.x > 9.6 && state.x < 11.4 {
            assert!(
                state.y >= platform_top + 0.5 - 0.01,
                "traversée : y = {}",
                state.y
            );
            was_on_platform = true;
        }
    }
    assert!(was_on_platform);
}

#[test]
fn gravity_portal_flips_gravity_and_sticks_to_ceiling() {
    let geometry = LevelGeometry::from_level(&level(vec![(
        ObjectKind::Portal(PortalKind::GravityFlip),
        8,
        0,
    )]));
    let mut state = PlayerState::new(1.0);
    assert_eq!(run(&mut state, &geometry, false, 2.0), None);
    assert_eq!(state.gravity, Gravity::Up);
    assert!(state.grounded);
    assert!((state.y - 9.5).abs() < 1e-3, "y = {}", state.y);
}

#[test]
fn portal_triggers_only_once() {
    let geometry = LevelGeometry::from_level(&level(vec![(
        ObjectKind::Portal(PortalKind::GravityFlip),
        8,
        0,
    )]));
    let mut state = PlayerState::new(1.0);
    run(&mut state, &geometry, false, 2.0);
    assert_eq!(state.activated.len(), 1);
}

#[test]
fn speed_and_size_portals_change_state() {
    let geometry = LevelGeometry::from_level(&level(vec![
        (ObjectKind::Portal(PortalKind::Speed(2.0)), 6, 0),
        (ObjectKind::Portal(PortalKind::Mini), 12, 0),
    ]));
    let mut state = PlayerState::new(1.0);
    run(&mut state, &geometry, false, 1.0);
    assert_eq!(state.speed_mult, 2.0);
    assert_eq!(state.size, PlayerSize::Mini);
}

#[test]
fn checkpoint_is_reported_once() {
    let geometry = LevelGeometry::from_level(&level(vec![(ObjectKind::Checkpoint, 8, 0)]));
    let mut state = PlayerState::new(1.0);
    let mut reached = 0;
    for _ in 0..(2.0 / SUBSTEP) as u32 {
        let events = step(
            &mut state,
            &PlayerInput::default(),
            &geometry,
            &Rules::default(),
            SUBSTEP,
        );
        if events.checkpoint.is_some() {
            reached += 1;
        }
    }
    assert_eq!(reached, 1);
}

#[test]
fn finishing_the_level_is_detected() {
    let mut data = level(vec![]);
    data.length = 20.0;
    let geometry = LevelGeometry::from_level(&data);
    let mut state = PlayerState::new(1.0);
    let mut finished = false;
    for _ in 0..(4.0 / SUBSTEP) as u32 {
        let events = step(
            &mut state,
            &PlayerInput::default(),
            &geometry,
            &Rules::default(),
            SUBSTEP,
        );
        finished |= events.finished;
    }
    assert!(finished);
}

#[test]
fn double_jump_only_when_enabled() {
    let geometry = LevelGeometry::from_level(&level(vec![]));

    let jump_twice = |double_jump: bool| -> f32 {
        let rules = Rules { double_jump };
        let mut state = PlayerState::new(1.0);
        let press = PlayerInput {
            jump_held: false,
            jump_pressed: true,
        };
        step(&mut state, &press, &geometry, &rules, SUBSTEP);
        // Laisse monter puis retombe un peu avant le second appui.
        for _ in 0..80 {
            step(
                &mut state,
                &PlayerInput::default(),
                &geometry,
                &rules,
                SUBSTEP,
            );
        }
        step(&mut state, &press, &geometry, &rules, SUBSTEP);
        let mut apex = state.y;
        for _ in 0..240 {
            step(
                &mut state,
                &PlayerInput::default(),
                &geometry,
                &rules,
                SUBSTEP,
            );
            apex = apex.max(state.y);
        }
        apex
    };

    assert!(jump_twice(true) > jump_twice(false) + 1.0);
}
