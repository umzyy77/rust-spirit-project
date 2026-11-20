mod epa;

use std::ops::{Add, Neg, Sub};

use common::Position;
use crate::shapes::Shape;

// Nécessaire pour que le type de retour de `penetration` soit public, même sans usage direct du nom.
#[allow(unused_imports)]
pub use epa::{penetration, Penetration};

/// Garde-fou : GJK converge en quelques itérations en 2D ; ne devrait jamais être atteint.
const MAX_ITERATIONS: usize = 20;

/// Vecteur 2D interne à l'algorithme, jamais exposé ailleurs.
#[derive(Clone, Copy, PartialEq)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    fn from_position(position: Position) -> Self {
        Vec2::new(position.x(), position.y())
    }

    fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Rotation de 90° (sens arbitraire, mais cohérent entre deux appels).
    fn perp(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }

    fn normalized(self) -> Vec2 {
        let length = self.dot(self).sqrt();
        if length == 0.0 {
            Vec2::new(0.0, 0.0)
        } else {
            Vec2::new(self.x / length, self.y / length)
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

/// Teste si deux formes convexes se chevauchent (GJK seul, sans EPA). Non
/// appelée par le jeu (`resolve_movement` utilise `penetration` directement) —
/// gardée pour tester GJK indépendamment d'EPA.
#[allow(dead_code)]
pub fn overlap(
    shape_a: &dyn Shape,
    position_a: Position,
    shape_b: &dyn Shape,
    position_b: Position,
) -> bool {
    let support = minkowski_support(shape_a, position_a, shape_b, position_b);
    enclosing_simplex(&support, initial_direction(position_a, position_b)).is_some()
}

/// Point de la différence de Minkowski `shape_a - shape_b` le plus loin
/// dans `direction`.
fn minkowski_support<'a>(
    shape_a: &'a dyn Shape,
    position_a: Position,
    shape_b: &'a dyn Shape,
    position_b: Position,
) -> impl Fn(Vec2) -> Vec2 + 'a {
    move |direction: Vec2| -> Vec2 {
        let point_a = shape_a.support(position_a, (direction.x, direction.y));
        let point_b = shape_b.support(position_b, (-direction.x, -direction.y));
        Vec2::from_position(point_a) - Vec2::from_position(point_b)
    }
}

/// Direction de départ pour chercher le premier point du simplexe : du
/// centre de B vers celui de A, ou un axe arbitraire s'ils coïncident.
fn initial_direction(position_a: Position, position_b: Position) -> Vec2 {
    let offset = Vec2::from_position(position_a) - Vec2::from_position(position_b);
    if offset == Vec2::new(0.0, 0.0) {
        Vec2::new(1.0, 0.0)
    } else {
        offset
    }
}

/// Cœur de GJK : cherche un triangle de la différence de Minkowski qui
/// contient l'origine. `None` si les formes ne se touchent pas. Simplexe sur
/// la pile (`[Vec2; 3]` + `len`) plutôt qu'un `Vec`, pour éviter l'allocation.
fn enclosing_simplex(support: &impl Fn(Vec2) -> Vec2, initial_direction: Vec2) -> Option<[Vec2; 3]> {
    let zero = Vec2::new(0.0, 0.0);
    let mut simplex = [support(initial_direction), zero, zero];
    let mut len = 1;
    let mut direction = -simplex[0];

    for _ in 0..MAX_ITERATIONS {
        let point = support(direction);
        if point.dot(direction) < 0.0 {
            return None;
        }

        simplex[len] = point;
        len += 1;

        match len {
            2 => {
                let a = simplex[1];
                let b = simplex[0];
                let ab = b - a;
                let ao = -a;
                if ab.dot(ao) > 0.0 {
                    direction = perp_toward(ab, ao);
                } else {
                    simplex[0] = a;
                    len = 1;
                    direction = ao;
                }
            }
            3 => {
                let a = simplex[2];
                let b = simplex[1];
                let c = simplex[0];
                let ab = b - a;
                let ac = c - a;
                let ao = -a;

                let ab_perp = perp_toward(ab, -ac);
                if ab_perp.dot(ao) > 0.0 {
                    simplex[0] = b;
                    simplex[1] = a;
                    len = 2;
                    direction = ab_perp;
                } else {
                    let ac_perp = perp_toward(ac, -ab);
                    if ac_perp.dot(ao) > 0.0 {
                        simplex[0] = c;
                        simplex[1] = a;
                        len = 2;
                        direction = ac_perp;
                    } else {
                        return Some(simplex);
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    None
}

/// Perpendiculaire à `edge`, réorientée du même côté que `reference`.
fn perp_toward(edge: Vec2, reference: Vec2) -> Vec2 {
    let perp = edge.perp();
    if perp.dot(reference) < 0.0 {
        -perp
    } else {
        perp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{Circle, Rect};

    #[test]
    fn circles_overlapping() {
        let a = Circle { radius: 10.0 };
        let b = Circle { radius: 10.0 };
        assert!(overlap(&a, Position::new(0.0, 0.0), &b, Position::new(15.0, 0.0)));
    }

    #[test]
    fn circles_separated() {
        let a = Circle { radius: 10.0 };
        let b = Circle { radius: 10.0 };
        assert!(!overlap(&a, Position::new(0.0, 0.0), &b, Position::new(25.0, 0.0)));
    }

    #[test]
    fn rects_overlapping() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Rect { width: 20.0, height: 20.0 };
        assert!(overlap(&a, Position::new(0.0, 0.0), &b, Position::new(15.0, 0.0)));
    }

    #[test]
    fn rects_separated() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Rect { width: 20.0, height: 20.0 };
        assert!(!overlap(&a, Position::new(0.0, 0.0), &b, Position::new(25.0, 0.0)));
    }

    #[test]
    fn circle_and_rect_overlapping() {
        let circle = Circle { radius: 10.0 };
        let rect = Rect { width: 20.0, height: 20.0 };
        assert!(overlap(&circle, Position::new(0.0, 0.0), &rect, Position::new(15.0, 0.0)));
    }

    #[test]
    fn circle_and_rect_separated() {
        let circle = Circle { radius: 10.0 };
        let rect = Rect { width: 20.0, height: 20.0 };
        assert!(!overlap(&circle, Position::new(0.0, 0.0), &rect, Position::new(30.0, 0.0)));
    }

    #[test]
    fn rects_touching_exactly_on_one_axis() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Rect { width: 20.0, height: 20.0 };
        assert!(overlap(&a, Position::new(0.0, 0.0), &b, Position::new(20.0, 0.0)));
    }

    #[test]
    fn rects_touching_exactly_at_a_corner() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Rect { width: 20.0, height: 20.0 };
        assert!(overlap(&a, Position::new(0.0, 0.0), &b, Position::new(20.0, 20.0)));
    }

    #[test]
    fn shapes_with_the_same_center_overlap() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Circle { radius: 5.0 };
        assert!(overlap(&a, Position::new(3.0, 7.0), &b, Position::new(3.0, 7.0)));
    }

    #[test]
    fn shape_fully_contained_in_another() {
        let outer = Rect { width: 40.0, height: 40.0 };
        let inner = Circle { radius: 5.0 };
        assert!(overlap(&outer, Position::new(0.0, 0.0), &inner, Position::new(3.0, 3.0)));
    }

    #[test]
    fn circles_exactly_tangent() {
        let a = Circle { radius: 10.0 };
        let b = Circle { radius: 10.0 };
        assert!(overlap(&a, Position::new(0.0, 0.0), &b, Position::new(20.0, 0.0)));
    }
}
