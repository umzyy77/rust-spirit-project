use std::collections::HashSet;

use crate::direction::Direction;
use crate::gjk;
use crate::position::Position;
use crate::shapes::Shape;

/// Pixels par seconde
const SPEED: f64 = 250.0;

/// Nombre de passes de correction de chevauchement par frame. Avec peu
/// d'entités, une correction contre un bloqueur ne réintroduit presque
/// jamais de chevauchement avec un autre — quelques passes suffisent
/// largement à stabiliser les cas où ça arrive.
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

    /// Calcule la position de l'entité après `dt` secondes selon les
    /// directions actives, sans modifier `self` : la mutation réelle se fait
    /// via `set_position`, une fois que *toutes* les entités ont calculé
    /// leur mouvement à partir du même instant de départ (sinon l'ordre de
    /// traitement des entités changerait le résultat).
    ///
    /// Le déplacement est d'abord limité aux bornes de la fenêtre, en
    /// bloc, sans égard aux autres entités. Puis, si ça chevauche une
    /// entité de `others`, on recule exactement de la profondeur de
    /// pénétration calculée par `gjk::penetration` (EPA) — plusieurs
    /// passes au cas où corriger un chevauchement en réintroduirait un
    /// autre. `gjk` ne distingue jamais le type concret des formes
    /// comparées, donc ceci fonctionne pour n'importe quelle combinaison.
    pub fn resolve_movement(&self, dt: f64, bounds: (f64, f64), others: &[(Position, &dyn Shape)]) -> Position {
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
                    resolved = Position::new(
                        resolved.x() + hit.axis.0 * hit.depth,
                        resolved.y() + hit.axis.1 * hit.depth,
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
