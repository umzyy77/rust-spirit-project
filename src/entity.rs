use std::collections::HashSet;

use crate::direction::Direction;
use crate::position::Position;
use crate::shapes::Shape;

/// Pixels par seconde
const SPEED: f64 = 250.0;

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

    /// Avance l'entité de `dt` secondes selon les directions actives, en
    /// restant dans les limites `bounds` (largeur, hauteur). Le déplacement
    /// est résolu axe par axe : sur chaque axe, s'il croiserait une autre
    /// entité de `others` (position du centre, demi-largeur, demi-hauteur —
    /// la sienne exclue via `other_position != self.position`), il est
    /// raccourci pour s'arrêter exactement au contact plutôt que d'être
    /// annulé en bloc.
    ///
    /// NB : cette collision reste pour l'instant une boîte englobante
    /// (AABB), même pour un `Circle` — l'étape suivante remplacera ce test
    /// par une vraie formule géométrique basée sur `Shape::support`.
    pub fn update(&mut self, dt: f64, bounds: (f64, f64), others: &[(Position, f64, f64)]) {
        let distance = SPEED * dt;
        let (width, height) = bounds;

        let mut dx = 0.0;
        let mut dy = 0.0;
        for direction in &self.moving {
            let (ddx, ddy) = direction.delta();
            dx += ddx * distance;
            dy += ddy * distance;
        }

        let others: Vec<(Position, f64, f64)> = others
            .iter()
            .copied()
            .filter(|&(other_position, _, _)| other_position != self.position)
            .collect();

        let x = self.resolve_axis(dx, width, &others, |p| p.x(), |p| p.y(), |(w, _)| w, |(_, h)| h);
        self.position = Position::new(x, self.position.y());

        let y = self.resolve_axis(dy, height, &others, |p| p.y(), |p| p.x(), |(_, h)| h, |(w, _)| w);
        self.position = Position::new(self.position.x(), y);
    }

    /// Nouvelle position du centre sur un axe après un déplacement de
    /// `delta`, restant dans `[demi-étendue, bounds_dim - demi-étendue]` et
    /// raccourcie pour s'arrêter au contact d'une entité de `others` (chacune
    /// avec ses propres demi-étendues) dont la bande perpendiculaire
    /// chevauche la sienne. `moving_of`/`perp_of` extraient, pour une
    /// `Position` donnée, la coordonnée sur l'axe qui bouge et celle sur
    /// l'axe perpendiculaire ; `moving_half_of`/`perp_half_of` font la même
    /// chose pour une demi-étendue `(demi-largeur, demi-hauteur)` —
    /// `(|p| p.x(), |p| p.y(), |(w, _)| w, |(_, h)| h)` pour résoudre x, les
    /// quatre inversés pour résoudre y.
    fn resolve_axis(
        &self,
        delta: f64,
        bounds_dim: f64,
        others: &[(Position, f64, f64)],
        moving_of: impl Fn(Position) -> f64,
        perp_of: impl Fn(Position) -> f64,
        moving_half_of: impl Fn((f64, f64)) -> f64,
        perp_half_of: impl Fn((f64, f64)) -> f64,
    ) -> f64 {
        let self_moving = moving_of(self.position);
        let self_perp = perp_of(self.position);
        let self_extents = self.shape.half_extents();
        let self_moving_half = moving_half_of(self_extents);
        let self_perp_half = perp_half_of(self_extents);

        let mut moving = (self_moving + delta).clamp(self_moving_half, bounds_dim - self_moving_half);

        for &(other_position, other_half_width, other_half_height) in others {
            let other_extents = (other_half_width, other_half_height);
            let other_perp = perp_of(other_position);
            let other_perp_half = perp_half_of(other_extents);
            let perp_overlap = self_perp - self_perp_half < other_perp + other_perp_half
                && self_perp + self_perp_half > other_perp - other_perp_half;
            if !perp_overlap {
                continue;
            }

            let other_moving = moving_of(other_position);
            let other_moving_half = moving_half_of(other_extents);
            let gap = self_moving_half + other_moving_half;
            if delta > 0.0 && other_moving >= self_moving {
                moving = moving.min(other_moving - gap);
            } else if delta < 0.0 && other_moving <= self_moving {
                moving = moving.max(other_moving + gap);
            }
        }

        moving
    }
}
