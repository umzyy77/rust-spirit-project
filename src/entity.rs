use std::collections::HashSet;

use crate::direction::Direction;
use crate::gjk;
use crate::position::Position;
use crate::shapes::Shape;

/// Pixels par seconde
const SPEED: f64 = 250.0;

/// Nombre de raffinements de la dichotomie cherchant le contact exact.
/// 20 itérations divisent le déplacement par ~1 million — largement de
/// quoi descendre sous la précision du pixel.
const CONTACT_SEARCH_ITERATIONS: u32 = 20;

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
    /// Le déplacement est d'abord limité aux bornes de la fenêtre, puis
    /// raccourci par dichotomie jusqu'au contact exact avec la première
    /// entité de `others` qu'il croiserait — `gjk::overlap` ne distingue
    /// jamais le type concret des formes comparées, donc ceci fonctionne
    /// pour n'importe quelle combinaison de formes.
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
        let target = (
            (self.position.x() + dx).clamp(half_width, width - half_width),
            (self.position.y() + dy).clamp(half_height, height - half_height),
        );
        let movement = (target.0 - self.position.x(), target.1 - self.position.y());

        let collides_at = |fraction: f64| -> bool {
            let probe = Position::new(
                self.position.x() + movement.0 * fraction,
                self.position.y() + movement.1 * fraction,
            );
            others.iter().any(|&(other_position, other_shape)| {
                gjk::overlap(self.shape.as_ref(), probe, other_shape, other_position)
            })
        };

        if !collides_at(1.0) {
            return Position::new(target.0, target.1);
        }

        let mut safe_fraction = 0.0;
        let mut blocked_fraction = 1.0;
        for _ in 0..CONTACT_SEARCH_ITERATIONS {
            let mid = (safe_fraction + blocked_fraction) / 2.0;
            if collides_at(mid) {
                blocked_fraction = mid;
            } else {
                safe_fraction = mid;
            }
        }

        Position::new(
            self.position.x() + movement.0 * safe_fraction,
            self.position.y() + movement.1 * safe_fraction,
        )
    }

    /// Applique une position déjà calculée par `resolve_movement`.
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }
}
