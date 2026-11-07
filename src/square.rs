use std::collections::HashSet;

use crate::direction::Direction;
use crate::position::Position;

pub const SIZE: f64 = 50.0;
/// Pixels par seconde
const SPEED: f64 = 250.0; 

/// Rouge, vert, bleu, alpha — chaque composante entre 0.0 et 1.0.
pub type Color = [f32; 4];

pub struct Square {
    position: Position,
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

    /// Position actuelle du carré.
    pub fn position(&self) -> Position {
        self.position
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
    /// restant dans les limites `bounds` (largeur, hauteur). Le déplacement
    /// est résolu axe par axe : sur chaque axe, s'il croiserait un autre
    /// carré de `positions` (la sienne exclue, via `other != self.position`),
    /// il est raccourci pour s'arrêter exactement au contact plutôt que
    /// d'être annulé en bloc.
    pub fn update(&mut self, dt: f64, bounds: (f64, f64), positions: &[Position]) {
        let distance = SPEED * dt;
        let (width, height) = bounds;

        let mut dx = 0.0;
        let mut dy = 0.0;
        for direction in &self.moving {
            let (ddx, ddy) = direction.delta();
            dx += ddx * distance;
            dy += ddy * distance;
        }

        let others: Vec<Position> = positions
            .iter()
            .copied()
            .filter(|&other| other != self.position)
            .collect();

        let x = self.resolve_axis(dx, width - SIZE, &others, |p| p.x(), |p| p.y());
        self.position = Position::new(x, self.position.y());

        let y = self.resolve_axis(dy, height - SIZE, &others, |p| p.y(), |p| p.x());
        self.position = Position::new(self.position.x(), y);
    }

    /// Nouvelle position sur un axe après un déplacement de `delta`,
    /// raccourcie pour s'arrêter au contact d'un carré de `others` dont la
    /// bande perpendiculaire chevauche la sienne. `moving_of`/`perp_of`
    /// extraient, pour une `Position` donnée, la coordonnée sur l'axe qui
    /// bouge et celle sur l'axe perpendiculaire — `(|p| p.x(), |p| p.y())`
    /// pour résoudre x, `(|p| p.y(), |p| p.x())` pour résoudre y.
    fn resolve_axis(
        &self,
        delta: f64,
        max: f64,
        others: &[Position],
        moving_of: impl Fn(Position) -> f64,
        perp_of: impl Fn(Position) -> f64,
    ) -> f64 {
        let self_moving = moving_of(self.position);
        let self_perp = perp_of(self.position);

        let mut moving = (self_moving + delta).clamp(0.0, max);

        for other in others {
            let other_perp = perp_of(*other);
            let perp_overlap = self_perp < other_perp + SIZE && self_perp + SIZE > other_perp;
            if !perp_overlap {
                continue;
            }

            let other_moving = moving_of(*other);
            if delta > 0.0 && other_moving >= self_moving {
                moving = moving.min(other_moving - SIZE);
            } else if delta < 0.0 && other_moving <= self_moving {
                moving = moving.max(other_moving + SIZE);
            }
        }

        moving
    }
}
