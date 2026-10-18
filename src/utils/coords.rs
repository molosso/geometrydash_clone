//! Conversion entre le repère « niveau » (en tuiles) et le repère « écran »
//! de Bevy (en pixels).
//!
//! Dans le repère niveau, `y = 0` est la surface du sol et `x = 0` le point
//! de départ du joueur. Ces fonctions sont pures, donc faciles à tester.

use super::constants::{GROUND_SCREEN_Y, TILE_SIZE};

/// Convertit une abscisse en tuiles vers une abscisse en pixels.
pub fn tile_to_world_x(tile_x: f32) -> f32 {
    tile_x * TILE_SIZE
}

/// Convertit une ordonnée en tuiles (0 = surface du sol) vers des pixels.
pub fn tile_to_world_y(tile_y: f32) -> f32 {
    GROUND_SCREEN_Y + tile_y * TILE_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_maps_to_ground_line() {
        assert_eq!(tile_to_world_x(0.0), 0.0);
        assert_eq!(tile_to_world_y(0.0), GROUND_SCREEN_Y);
    }

    #[test]
    fn one_tile_is_tile_size_pixels() {
        assert_eq!(tile_to_world_x(1.0), TILE_SIZE);
        assert_eq!(tile_to_world_y(1.0) - tile_to_world_y(0.0), TILE_SIZE);
    }
}
