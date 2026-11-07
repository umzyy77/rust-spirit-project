use std::collections::HashSet;

use crate::direction::Direction;
use crate::position::Position;

pub const SIZE: f64 = 50.0;
/// Pixels par seconde
const SPEED: f64 = 250.0; 

/// Rouge, vert, bleu, alpha — chaque composante entre 0.0 et 1.0.
pub type Color = [f32; 4];

pub struct Square {
    pub position: Position,
    pub color: Color,
    moving: HashSet<Direction>
}

impl Square {
    /// Crée un carré immobile à la position `(x, y)`.
    pub fn new(x: f64, y: f64, color: Color) -> Self {
        Square {
            position: Position::new(x, y),
            color,
            moving: HashSet::new()
        }
    }

    /// Active ou désactive le déplacement dans `direction`.
    pub fn set_moving(&mut self, direction: Direction, active: bool) {
        if active {
            self.moving.insert(direction);
        } else {
            self.moving.remove(&direction);
        }
    }

    /// Avance le carré de `dt` secondes selon les directions actives, en
    /// restant dans les limites `bounds` (largeur, hauteur).
    pub fn update(&mut self, dt: f64, bounds: (f64, f64)) {
        let distance = SPEED * dt;
        let (width, height) = bounds;

        let mut dx = 0.0;
        let mut dy = 0.0;
        for direction in &self.moving {
            let (ddx, ddy) = direction.delta();
            dx += ddx * distance;
            dy += ddy * distance;
        }

        self.position.translate(dx, dy);
        self.position.clamp(width - SIZE, height - SIZE);
    }
}
