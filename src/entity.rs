use std::collections::HashSet;

use crate::direction::Direction;
use crate::position::Position;

/// Rouge, vert, bleu, alpha — chaque composante entre 0.0 et 1.0.
pub type Color = [f32; 4];

/// État commun à toute entité mobile, indépendamment de sa forme — voir
/// `actor::Actor`, qui s'appuie dessus via `get_entity`/`get_entity_mut`.
pub struct Entity {
    position: Position,
    pub color: Color,
    moving: HashSet<Direction>,
}

impl Entity {
    /// Crée une entité immobile à la position `(x, y)` (son centre).
    pub fn new(x: f64, y: f64, color: Color) -> Self {
        Entity {
            position: Position::new(x, y),
            color,
            moving: HashSet::new(),
        }
    }

    /// Position actuelle de l'entité.
    pub fn position(&self) -> Position {
        self.position
    }

    /// Déplace l'entité à `position`.
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    /// Directions actuellement actives.
    pub fn moving_directions(&self) -> &HashSet<Direction> {
        &self.moving
    }

    /// Active ou désactive le déplacement dans `direction`.
    pub fn set_moving(&mut self, direction: Direction, active: bool) {
        if active {
            self.moving.insert(direction);
        } else {
            self.moving.remove(&direction);
        }
    }
}
