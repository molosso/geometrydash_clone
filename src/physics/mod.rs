//! Physique du cube : mouvement, gravité, saut, collisions et portails.
//!
//! Ce module est **pur** (aucune dépendance à Bevy) : la fonction [`step`]
//! fait avancer un [`PlayerState`] d'un petit pas de temps et renvoie les
//! [`StepEvents`] survenus. Bevy se contente de l'appeler (plusieurs fois par
//! image, voir *sous-pas* ci-dessous) puis d'afficher le résultat.
//!
//! ## Anti-traversée (« tunneling »)
//!
//! Le joueur ne doit jamais traverser un obstacle. Deux protections :
//! 1. le jeu découpe chaque image en sous-pas d'au plus [`SUBSTEP`] seconde
//!    (1/240 s) ; à vitesse maximale, le cube avance de moins de 0,2 tuile
//!    par sous-pas, bien moins que la plus fine des dalles ;
//! 2. chaque axe est résolu séparément (X puis Y) : si le cube pénètre un
//!    solide, il est replacé exactement contre sa surface.
//!
//! Unités : tuiles et secondes. `y = 0` est la surface du sol.

pub mod aabb;

pub use aabb::Aabb;

use crate::collision::LevelGeometry;
use crate::levels::data::PortalKind;

/// Vitesse horizontale de base (tuiles par seconde).
pub const BASE_SPEED: f32 = 10.0;
/// Accélération de la gravité (tuiles par seconde²).
pub const GRAVITY: f32 = 100.0;
/// Vitesse verticale initiale d'un saut (tuiles par seconde).
/// Hauteur de saut = v² / 2g ≈ 2,1 tuiles.
pub const JUMP_VELOCITY: f32 = 20.5;
/// Vitesse verticale maximale en valeur absolue.
pub const TERMINAL_VELOCITY: f32 = 40.0;
/// Durée maximale d'un sous-pas de simulation (secondes).
pub const SUBSTEP: f32 = 1.0 / 240.0;
/// Durée maximale simulée par image (évite la « spirale de la mort » si
/// le jeu rame : au-delà, le temps est simplement ralenti).
pub const MAX_FRAME_DT: f32 = 1.0 / 20.0;
/// Abscisse de départ du joueur.
pub const START_X: f32 = 0.0;
/// Les pics sont testés avec un cube réduit à cette fraction : un contact
/// « de justesse » avec la pointe ne tue pas (collisions indulgentes).
pub const HAZARD_SHRINK: f32 = 0.8;
/// Tolérance hors du niveau avant mort de sécurité.
const OUT_OF_BOUNDS_MARGIN: f32 = 3.0;

/// Taille du cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerSize {
    /// Taille normale (1 tuile).
    #[default]
    Normal,
    /// Version miniature (0,6 tuile).
    Mini,
}

impl PlayerSize {
    /// Demi-côté de la boîte de collision.
    pub const fn half_extent(self) -> f32 {
        match self {
            PlayerSize::Normal => 0.5,
            PlayerSize::Mini => 0.3,
        }
    }
}

/// Sens de la gravité.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gravity {
    /// Vers le bas (normal).
    #[default]
    Down,
    /// Vers le haut (inversé).
    Up,
}

impl Gravity {
    /// Signe de l'accélération sur l'axe Y : -1 vers le bas, +1 vers le haut.
    pub const fn sign(self) -> f32 {
        match self {
            Gravity::Down => -1.0,
            Gravity::Up => 1.0,
        }
    }

    /// Gravité opposée.
    pub const fn flipped(self) -> Self {
        match self {
            Gravity::Down => Gravity::Up,
            Gravity::Up => Gravity::Down,
        }
    }
}

/// Cause d'une mort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    /// Choc frontal contre un bloc.
    Block,
    /// Contact avec un pic.
    Spike,
    /// Sorti du niveau (filet de sécurité).
    OutOfBounds,
}

/// Réglages de règles qui influencent la physique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rules {
    /// Autorise un saut supplémentaire en l'air.
    pub double_jump: bool,
}

/// Entrées du joueur pour un sous-pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayerInput {
    /// La touche de saut est maintenue (saut automatique à l'atterrissage).
    pub jump_held: bool,
    /// La touche vient d'être pressée (front montant), utile au double saut.
    pub jump_pressed: bool,
}

/// État complet du cube. `Clone` permet de le sauvegarder pour les checkpoints.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerState {
    /// Abscisse du centre (tuiles).
    pub x: f32,
    /// Ordonnée du centre (tuiles).
    pub y: f32,
    /// Vitesse verticale (tuiles/s).
    pub vel_y: f32,
    /// Multiplicateur de vitesse horizontale.
    pub speed_mult: f32,
    /// Sens de la gravité.
    pub gravity: Gravity,
    /// Taille du cube.
    pub size: PlayerSize,
    /// Vrai si le cube repose sur une surface.
    pub grounded: bool,
    /// Sauts aériens restants.
    pub air_jumps_left: u8,
    /// Indices des portails/checkpoints déjà déclenchés (un seul déclenchement).
    pub activated: Vec<usize>,
}

impl PlayerState {
    /// État initial : posé sur le sol, au départ du niveau.
    pub fn new(speed_mult: f32) -> Self {
        Self {
            x: START_X,
            y: PlayerSize::Normal.half_extent(),
            vel_y: 0.0,
            speed_mult,
            gravity: Gravity::Down,
            size: PlayerSize::Normal,
            grounded: true,
            air_jumps_left: 0,
            activated: Vec::new(),
        }
    }
}

/// Événements survenus pendant un sous-pas.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StepEvents {
    /// Le cube a sauté.
    pub jumped: bool,
    /// Le cube vient de se poser.
    pub landed: bool,
    /// Le cube est mort (et pourquoi).
    pub death: Option<DeathCause>,
    /// Le cube a franchi la ligne d'arrivée.
    pub finished: bool,
    /// Portails traversés pendant ce pas.
    pub portals: Vec<PortalKind>,
    /// Indice d'un checkpoint atteint pendant ce pas.
    pub checkpoint: Option<usize>,
}

/// Applique l'effet d'un portail à l'état du cube.
pub fn apply_portal(state: &mut PlayerState, kind: PortalKind) {
    match kind {
        PortalKind::GravityFlip => {
            state.gravity = state.gravity.flipped();
            state.vel_y *= 0.5;
            state.grounded = false;
        }
        PortalKind::GravityNormal => {
            if state.gravity != Gravity::Down {
                state.gravity = Gravity::Down;
                state.vel_y *= 0.5;
                state.grounded = false;
            }
        }
        PortalKind::GravityInverted => {
            if state.gravity != Gravity::Up {
                state.gravity = Gravity::Up;
                state.vel_y *= 0.5;
                state.grounded = false;
            }
        }
        PortalKind::Speed(multiplier) => state.speed_mult = multiplier,
        PortalKind::Mini => state.size = PlayerSize::Mini,
        PortalKind::Normal => state.size = PlayerSize::Normal,
    }
}

/// Fait avancer le cube de `dt` secondes.
///
/// `dt` doit rester petit (≤ [`SUBSTEP`] idéalement). L'ordre des opérations :
/// saut → déplacement horizontal (X) → déplacement vertical (Y) →
/// pics → portails → checkpoints → arrivée.
pub fn step(
    state: &mut PlayerState,
    input: &PlayerInput,
    geometry: &LevelGeometry,
    rules: &Rules,
    dt: f32,
) -> StepEvents {
    let mut events = StepEvents::default();
    let half = state.size.half_extent();
    let sign = state.gravity.sign();

    // --- 1. Saut -----------------------------------------------------------
    let wants_jump = input.jump_held || input.jump_pressed;
    if state.grounded && wants_jump {
        state.vel_y = -sign * JUMP_VELOCITY;
        state.grounded = false;
        // Le saut aérien est « rechargé » à chaque décollage depuis une surface.
        state.air_jumps_left = u8::from(rules.double_jump);
        events.jumped = true;
    } else if !state.grounded && input.jump_pressed && state.air_jumps_left > 0 {
        state.vel_y = -sign * JUMP_VELOCITY;
        state.air_jumps_left -= 1;
        events.jumped = true;
    }

    // --- 2. Axe X : avance à vitesse constante ----------------------------
    // Toucher un solide par le côté est mortel.
    state.x += BASE_SPEED * state.speed_mult * dt;
    let body = Aabb::new(state.x, state.y, half, half);
    if geometry
        .solids_near(state.x)
        .iter()
        .any(|solid| body.overlaps(solid))
    {
        events.death = Some(DeathCause::Block);
        return events;
    }

    // --- 3. Axe Y : gravité puis résolution des collisions ----------------
    state.vel_y = (state.vel_y + sign * GRAVITY * dt).clamp(-TERMINAL_VELOCITY, TERMINAL_VELOCITY);
    state.y += state.vel_y * dt;

    let moving_down = state.vel_y < 0.0;
    let body = Aabb::new(state.x, state.y, half, half);

    // `resolved_y` = position corrigée du centre, si un contact a eu lieu.
    let mut resolved_y: Option<f32> = None;
    let mut push = |candidate: f32| {
        resolved_y = Some(match resolved_y {
            // En descendant, on garde la surface la plus haute ; en montant, la plus basse.
            Some(current) if moving_down => current.max(candidate),
            Some(current) => current.min(candidate),
            None => candidate,
        });
    };

    for solid in geometry.solids_near(state.x) {
        if body.overlaps(solid) {
            if moving_down {
                push(solid.max_y() + half);
            } else {
                push(solid.min_y() - half);
            }
        }
    }
    // Le sol (y = 0) et le plafond (y = height) sont des plans infinis.
    if moving_down && state.y - half < 0.0 {
        push(half);
    }
    if !moving_down && state.y + half > geometry.height {
        push(geometry.height - half);
    }

    if let Some(corrected) = resolved_y {
        state.y = corrected;
        state.vel_y = 0.0;
        // On est « posé » si le contact est du côté de la gravité.
        let on_gravity_side = (moving_down && sign < 0.0) || (!moving_down && sign > 0.0);
        if on_gravity_side {
            if !state.grounded {
                events.landed = true;
            }
            state.grounded = true;
            state.air_jumps_left = u8::from(rules.double_jump);
        }
    } else {
        state.grounded = false;
    }

    // --- 4. Pics : boîte réduite pour des collisions indulgentes ----------
    let shrunk = half * HAZARD_SHRINK;
    let hazard_body = Aabb::new(state.x, state.y, shrunk, shrunk);
    if geometry
        .hazards_near(state.x)
        .iter()
        .any(|hazard| hazard_body.overlaps(hazard))
    {
        events.death = Some(DeathCause::Spike);
        return events;
    }

    // --- 5. Portails (déclenchés une seule fois chacun) -------------------
    let body = Aabb::new(state.x, state.y, half, half);
    for zone in geometry.portals_near(state.x) {
        if body.overlaps(&zone.aabb) && !state.activated.contains(&zone.index) {
            state.activated.push(zone.index);
            apply_portal(state, zone.kind);
            events.portals.push(zone.kind);
        }
    }

    // --- 6. Checkpoints ----------------------------------------------------
    for zone in geometry.checkpoints_near(state.x) {
        if body.overlaps(&zone.aabb) && !state.activated.contains(&zone.index) {
            state.activated.push(zone.index);
            events.checkpoint = Some(zone.index);
        }
    }

    // --- 7. Filet de sécurité et arrivée ----------------------------------
    if state.y < -OUT_OF_BOUNDS_MARGIN || state.y > geometry.height + OUT_OF_BOUNDS_MARGIN {
        events.death = Some(DeathCause::OutOfBounds);
        return events;
    }
    if state.x >= geometry.finish_x {
        events.finished = true;
    }

    events
}
