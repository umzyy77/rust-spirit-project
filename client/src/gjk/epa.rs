use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::{enclosing_simplex, initial_direction, minkowski_support, Vec2};
use common::Position;
use crate::shapes::Shape;

/// Garde-fou : chaque itération d'EPA ajoute un sommet au polytope. Pour un
/// bord courbe (cercle), celui-ci ne fait qu'approcher le cercle par cordes —
/// la profondeur reste une approximation.
const EPA_MAX_ITERATIONS: usize = 20;

/// Sous ce seuil, une itération d'EPA n'apporte plus rien d'utile.
const EPA_EPSILON: f64 = 1e-6;

/// Direction et profondeur minimales pour séparer `shape_a` de `shape_b`
/// (translater `shape_a` de `axis * depth` annule le chevauchement).
pub struct Penetration {
    pub axis: (f64, f64),
    pub depth: f64,
}

/// Si `shape_a` et `shape_b` se chevauchent, calcule la correction minimale
/// pour les séparer : GJK (détection) puis EPA (profondeur). Ne connaît
/// jamais le type concret des formes, seulement `Shape::support`.
pub fn penetration(
    shape_a: &dyn Shape,
    position_a: Position,
    shape_b: &dyn Shape,
    position_b: Position,
) -> Option<Penetration> {
    let support = minkowski_support(shape_a, position_a, shape_b, position_b);
    let simplex = enclosing_simplex(&support, initial_direction(position_a, position_b))?;
    let result = expand(simplex, support);
    Some(Penetration { axis: (result.axis.x, result.axis.y), depth: result.depth })
}

struct RawPenetration {
    axis: Vec2,
    depth: f64,
}

/// Une arête du polytope d'EPA : extrémités, normale sortante (unitaire) et
/// distance à l'origine. `Ord` inversé pour que `BinaryHeap` (tas-max) serve de tas-min.
struct Edge {
    a: Vec2,
    b: Vec2,
    normal: Vec2,
    distance: f64,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for Edge {}
impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.total_cmp(&self.distance)
    }
}

/// Normale sortante et distance à l'origine de l'arête `a -> b`, `None` si
/// longueur nulle. `winding` (calculé une fois par `expand`) donne le côté
/// "sortant" sans ambiguïté, même quand l'origine est sur une arête.
fn edge_for(a: Vec2, b: Vec2, winding: f64) -> Option<Edge> {
    let edge = b - a;
    if edge.dot(edge) < 1e-12 {
        return None;
    }

    let perp = edge.perp();
    let outward = if winding > 0.0 { -perp } else { perp };
    let normal = outward.normalized();
    let distance = normal.dot(a);

    Some(Edge { a, b, normal, distance })
}

/// EPA : étend le triangle de GJK en polygone de plus en plus fidèle au bord
/// de la différence de Minkowski, jusqu'à trouver l'arête la plus proche de
/// l'origine. Tas-min d'arêtes (pas un `Vec` rescanné) : insérer un sommet
/// entre `a` et `b` ne remplace que cette arête, les autres restent valides.
/// Le sens de parcours du triangle est fixé une fois pour éviter toute
/// ambiguïté d'orientation en cas de contact rasant.
fn expand(simplex: [Vec2; 3], support: impl Fn(Vec2) -> Vec2) -> RawPenetration {
    let winding = signed_area(&simplex);

    let mut edges = BinaryHeap::new();
    for i in 0..3 {
        if let Some(edge) = edge_for(simplex[i], simplex[(i + 1) % 3], winding) {
            edges.push(edge);
        }
    }

    for _ in 0..EPA_MAX_ITERATIONS {
        let Some(closest) = edges.pop() else {
            // Cas pathologique (3 arêtes dégénérées) : GJK ne devrait jamais produire ça.
            return RawPenetration { axis: Vec2::new(0.0, 0.0), depth: 0.0 };
        };

        let new_point = support(closest.normal);
        let new_distance = new_point.dot(closest.normal);

        if new_distance - closest.distance < EPA_EPSILON {
            // `normal` pointe de B vers A ; la correction va dans l'autre sens.
            return RawPenetration { axis: -closest.normal, depth: closest.distance };
        }

        if let Some(edge) = edge_for(closest.a, new_point, winding) {
            edges.push(edge);
        }
        if let Some(edge) = edge_for(new_point, closest.b, winding) {
            edges.push(edge);
        }
    }

    match edges.pop() {
        Some(closest) => RawPenetration { axis: -closest.normal, depth: closest.distance },
        None => RawPenetration { axis: Vec2::new(0.0, 0.0), depth: 0.0 },
    }
}

/// Deux fois l'aire signée (formule du lacet) : positive en sens anti-horaire. Seul le signe compte ici.
fn signed_area(polytope: &[Vec2]) -> f64 {
    let len = polytope.len();
    let mut area = 0.0;
    for i in 0..len {
        let a = polytope[i];
        let b = polytope[(i + 1) % len];
        area += a.x * b.y - b.x * a.y;
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::{Circle, Rect};

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
    }

    /// Tolérance plus large pour un bord courbe (cercle), qu'EPA n'approche que par cordes.
    fn assert_close_for_curved_shape(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-2, "{actual} != {expected}");
    }

    #[test]
    fn penetration_depth_between_two_circles() {
        let a = Circle { radius: 10.0 };
        let b = Circle { radius: 10.0 };
        let result = penetration(&a, Position::new(0.0, 0.0), &b, Position::new(15.0, 0.0)).unwrap();

        assert_close_for_curved_shape(result.depth, 5.0);
        assert_close_for_curved_shape(result.axis.0, -1.0);
        assert_close_for_curved_shape(result.axis.1, 0.0);
    }

    #[test]
    fn penetration_depth_between_two_rects_on_the_y_axis() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Rect { width: 20.0, height: 10.0 };
        let result = penetration(&a, Position::new(0.0, 0.0), &b, Position::new(0.0, 12.0)).unwrap();

        assert_close(result.depth, 3.0);
        assert_close(result.axis.0, 0.0);
        assert_close(result.axis.1, -1.0);
    }

    #[test]
    fn penetration_depth_between_circle_and_rect() {
        let circle = Circle { radius: 10.0 };
        let rect = Rect { width: 20.0, height: 20.0 };
        let result = penetration(&circle, Position::new(0.0, 0.0), &rect, Position::new(15.0, 0.0)).unwrap();

        assert_close(result.depth, 5.0);
        assert_close(result.axis.0, -1.0);
        assert_close(result.axis.1, 0.0);
    }

    #[test]
    fn no_penetration_when_separated() {
        let a = Circle { radius: 10.0 };
        let b = Circle { radius: 10.0 };
        assert!(penetration(&a, Position::new(0.0, 0.0), &b, Position::new(25.0, 0.0)).is_none());
    }

    #[test]
    fn penetration_when_rects_touch_exactly_on_one_axis() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Rect { width: 20.0, height: 20.0 };
        let result = penetration(&a, Position::new(0.0, 0.0), &b, Position::new(20.0, 0.0)).unwrap();
        assert_close(result.depth, 0.0);
        assert_close(result.axis.0, -1.0);
        assert_close(result.axis.1, 0.0);
    }

    #[test]
    fn penetration_when_rects_touch_exactly_at_a_corner() {
        let a = Rect { width: 20.0, height: 20.0 };
        let b = Rect { width: 20.0, height: 20.0 };
        let result = penetration(&a, Position::new(0.0, 0.0), &b, Position::new(20.0, 20.0)).unwrap();
        assert_close(result.depth, 0.0);
    }

    #[test]
    fn penetration_when_circles_are_exactly_tangent() {
        let a = Circle { radius: 10.0 };
        let b = Circle { radius: 10.0 };
        let result = penetration(&a, Position::new(0.0, 0.0), &b, Position::new(20.0, 0.0)).unwrap();
        assert_close(result.depth, 0.0);
        assert_close_for_curved_shape(result.axis.0, -1.0);
        assert_close_for_curved_shape(result.axis.1, 0.0);
    }
}
