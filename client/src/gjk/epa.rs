use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::{enclosing_simplex, initial_direction, minkowski_support, Vec2};
use common::Position;
use crate::shapes::Shape;

/// Garde-fou pour EPA : chaque itération ajoute un sommet au polytope.
/// Converge vite pour un polygone simple (peu d'arêtes entre le triangle de
/// départ et la bonne arête) ; pour un bord courbe (cercle), le polytope ne
/// fait qu'approcher le vrai cercle par cordes — la profondeur reste une
/// approximation à la résolution de l'écran près, jamais une valeur exacte.
const EPA_MAX_ITERATIONS: usize = 20;

/// Seuil sous lequel on considère qu'une itération d'EPA n'apporte plus
/// rien (le nouveau point sondé n'est quasiment pas plus loin que l'arête
/// qu'on a déjà).
const EPA_EPSILON: f64 = 1e-6;

/// Direction et profondeur minimales pour séparer `shape_a` de `shape_b` :
/// translater `shape_a` de `axis * depth` (ou `shape_b` de `-axis * depth`)
/// suffit à annuler le chevauchement.
pub struct Penetration {
    pub axis: (f64, f64),
    pub depth: f64,
}

/// Calcule, si `shape_a` et `shape_b` se chevauchent, la correction minimale
/// pour les séparer — via GJK (détection) puis EPA (profondeur, exacte pour
/// un polygone, approchée pour un bord courbe). Comme GJK, l'algorithme ne
/// s'appuie que sur `Shape::support` : il ne connaît jamais le type concret
/// des formes comparées.
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

/// Une arête du polytope d'EPA : ses deux extrémités, sa normale sortante
/// (unitaire) et sa distance à l'origine. Ordonnée par distance croissante
/// pour alimenter un tas-min — `BinaryHeap` est un tas-max en Rust, donc la
/// comparaison est inversée (`other.cmp(self)`).
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

/// Normale sortante (unitaire) et distance à l'origine de l'arête `a -> b`,
/// `None` si ses extrémités sont confondues (arête de longueur nulle, rien
/// à en tirer). `winding` (signe fixe pour tout le polytope, calculé une
/// fois par `expand`) donne le côté "sortant" sans ambiguïté, y compris
/// quand l'origine est exactement sur une arête.
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

/// EPA : à partir du triangle de GJK qui contient l'origine, on l'étend en
/// polygone de plus en plus fidèle au vrai bord de la différence de
/// Minkowski, jusqu'à trouver l'arête la plus proche de l'origine.
///
/// Les arêtes sont gardées dans un tas-min au lieu d'un `Vec` rescanné en
/// entier à chaque itération : insérer un sommet entre `a` et `b` ne fait
/// disparaître QUE l'arête `a -> b` (remplacée par `a -> nouveau` et
/// `nouveau -> b`) — toutes les autres arêtes du polygone restent valides
/// sans jamais être recalculées.
///
/// Le sens de parcours (horaire/anti-horaire) du triangle initial est
/// calculé une seule fois et réutilisé pour toutes les arêtes : décider
/// l'orientation sortante arête par arête (via le signe d'un produit
/// scalaire local) devient ambigu exactement au contact rasant, quand
/// l'origine tombe sur une arête — ce qui casserait la convexité du
/// polytope à l'étape suivante.
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
            // Les 3 arêtes du triangle de départ étaient toutes dégénérées :
            // cas pathologique (GJK ne devrait jamais produire ça), rien de
            // mieux à faire qu'un contact sans profondeur.
            return RawPenetration { axis: Vec2::new(0.0, 0.0), depth: 0.0 };
        };

        let new_point = support(closest.normal);
        let new_distance = new_point.dot(closest.normal);

        if new_distance - closest.distance < EPA_EPSILON {
            // `normal` pointe de B vers A ; la correction à appliquer à A
            // va dans l'autre sens.
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

/// Deux fois l'aire signée du polygone (formule du lacet) : positive si ses
/// sommets tournent dans le sens anti-horaire, négative sinon. Seul le
/// signe compte ici, pas la valeur.
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

    /// Un bord courbe (cercle) n'est jamais atteint exactement par le
    /// polytope d'EPA (qui l'approche par cordes) — seulement une boîte
    /// englobante le serait. Tolérance plus large pour ces cas-là, encore
    /// bien en dessous de ce qui serait visible à l'écran.
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
