use std::collections::HashSet;

use crate::direction::Direction;
use crate::gjk;
use common::Position;
use crate::shapes::Shape;

/// Pixels par seconde
const SPEED: f64 = 250.0;

/// Passes de correction de chevauchement par frame (une correction peut en réintroduire une autre).
const CONTACT_RESOLUTION_PASSES: u32 = 4;

/// Rouge, vert, bleu, alpha — chaque composante entre 0.0 et 1.0.
pub type Color = [f32; 4];

pub struct Entity {
    position: Position,
    pub color: Color,
    shape: Box<dyn Shape>,
    moving: HashSet<Direction>
}

impl Entity {
    /// Crée une entité immobile centrée en `(x, y)`, avec la forme `shape`.
    pub fn new(x: f64, y: f64, shape: Box<dyn Shape>, color: Color) -> Self {
        Entity {
            position: Position::new(x, y),
            color,
            shape,
            moving: HashSet::new()
        }
    }

    /// Position actuelle du centre de l'entité.
    pub fn position(&self) -> Position {
        self.position
    }

    /// Forme de l'entité (dessinable via `Drawable`, implémenté pour tout `Shape`).
    pub fn shape(&self) -> &dyn Shape {
        self.shape.as_ref()
    }

    /// Active ou désactive le déplacement dans `direction`.
    pub fn set_moving(&mut self, direction: Direction, active: bool) {
        if active {
            self.moving.insert(direction);
        } else {
            self.moving.remove(&direction);
        }
    }

    /// Position après `dt` secondes : clamp aux bornes de la fenêtre, puis
    /// recul de la *moitié* de la pénétration (`gjk::penetration`/EPA) contre
    /// chaque entité de `others` qui chevauche — l'autre joueur fait le même
    /// calcul de son côté, donc les deux moitiés se somment à la séparation
    /// exacte. Ne mute pas `self` : voir `set_position`.
    pub fn resolve_movement(&self, dt: f64, bounds: (f64, f64), others: &[(Position, &dyn Shape)]) -> Position {
        // Immobile : rien à corriger, le serveur ne nous déplace jamais pour un autre joueur.
        if self.moving.is_empty() {
            return self.position;
        }

        let distance = SPEED * dt;
        let (width, height) = bounds;

        let mut dx = 0.0;
        let mut dy = 0.0;
        for direction in &self.moving {
            let (ddx, ddy) = direction.delta();
            dx += ddx * distance;
            dy += ddy * distance;
        }

        let (half_width, half_height) = self.shape.half_extents();
        let mut resolved = Position::new(
            (self.position.x() + dx).clamp(half_width, width - half_width),
            (self.position.y() + dy).clamp(half_height, height - half_height),
        );

        for _ in 0..CONTACT_RESOLUTION_PASSES {
            let mut corrected = false;

            for &(other_position, other_shape) in others {
                if let Some(hit) = gjk::penetration(self.shape.as_ref(), resolved, other_shape, other_position) {
                    let half_depth = hit.depth / 2.0;
                    resolved = Position::new(
                        resolved.x() + hit.axis.0 * half_depth,
                        resolved.y() + hit.axis.1 * half_depth,
                    );
                    corrected = true;
                }
            }

            if !corrected {
                break;
            }
        }

        resolved
    }

    /// Applique une position déjà calculée par `resolve_movement`.
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }
}
