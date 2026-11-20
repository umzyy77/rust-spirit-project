use std::ops::{Add, Neg, Sub};

use crate::position::Position;
use crate::shapes::Shape;

/// Garde-fou : en 2D, GJK converge toujours en quelques itérations pour un
/// simplexe à 3 sommets maximum. Cette limite n'est jamais censée être
/// atteinte ; elle protège juste contre un cas numérique imprévu.
const MAX_ITERATIONS: usize = 20;

/// Même garde-fou pour EPA : chaque itération ajoute un sommet au polytope.
/// Converge vite pour un polygone simple (peu d'arêtes entre le triangle de
/// départ et la bonne arête) ; pour un bord courbe (cercle), le polytope ne
/// fait qu'approcher le vrai cercle par cordes — la profondeur reste une
/// approximation à la résolution de l'écran près, jamais une valeur exacte.
const EPA_MAX_ITERATIONS: usize = 20;

/// Seuil sous lequel on considère qu'une itération d'EPA n'apporte plus
/// rien (le nouveau point sondé n'est quasiment pas plus loin que l'arête
/// qu'on a déjà).
const EPA_EPSILON: f64 = 1e-6;

/// Vecteur 2D interne à cet algorithme — jamais exposé ailleurs dans le
/// projet, pour ne pas imposer ce détail d'implémentation au reste du code.
/// La frontière avec `Shape::support` reste en `(f64, f64)` brut pour la
/// direction passée en entrée ; sa sortie (`Position`) est convertie via
/// `Vec2::from_position`.
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

    /// Rotation de 90° dans un sens fixe (peu importe lequel ici : seule la
    /// cohérence entre deux appels compte, pas le sens en lui-même).
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

/// Direction et profondeur minimales pour séparer `shape_a` de `shape_b` :
/// translater `shape_a` de `axis * depth` (ou `shape_b` de `-axis * depth`)
/// suffit à annuler le chevauchement.
pub struct Penetration {
    pub axis: (f64, f64),
    pub depth: f64,
}

/// Teste si deux formes convexes (centrées sur `position_a`/`position_b`)
/// se chevauchent. Ne lance que GJK, sans l'étape EPA (inutile pour une
/// simple réponse oui/non).
///
/// Non appelée par le jeu (`resolve_movement` a besoin de la profondeur,
/// donc utilise `penetration` directement) — gardée pour isoler, dans les
/// tests, "GJK détecte-t-il correctement le contact" de "EPA calcule-t-il
/// la bonne profondeur".
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

/// Cœur de GJK : cherche un triangle de points de la différence de
/// Minkowski qui contient l'origine. `None` si les formes ne se touchent
/// pas (un point sondé n'avance plus vers l'origine).
///
/// Le simplexe n'a jamais plus de 3 sommets : un tableau sur la pile
/// (`[Vec2; 3]` + `len`) évite une allocation de tas à chaque itération,
/// contrairement à un `Vec` reconstruit en boucle.
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

/// EPA : à partir du triangle de GJK qui contient l'origine, on l'étend en
/// polygone de plus en plus fidèle au vrai bord de la différence de
/// Minkowski, jusqu'à trouver l'arête la plus proche de l'origine. Ce
/// polygone grandit au-delà de 3 sommets (contrairement au simplexe de
/// GJK) : un `Vec` reste nécessaire ici, une seule fois.
///
/// Le sens de parcours (horaire/anti-horaire) du triangle initial est
/// calculé une seule fois et réutilisé pour toutes les arêtes : décider
/// l'orientation sortante arête par arête (via le signe d'un produit
/// scalaire local) devient ambigu exactement au contact rasant, quand
/// l'origine tombe sur une arête — ce qui casserait la convexité du
/// polytope à l'étape suivante.
fn expand(simplex: [Vec2; 3], support: impl Fn(Vec2) -> Vec2) -> RawPenetration {
    let mut polytope = simplex.to_vec();
    let winding = signed_area(&polytope);

    for _ in 0..EPA_MAX_ITERATIONS {
        let (edge_index, normal, distance) = closest_edge(&polytope, winding);

        let new_point = support(normal);
        let new_distance = new_point.dot(normal);

        if new_distance - distance < EPA_EPSILON {
            // `normal` pointe de B vers A ; la correction à appliquer à A
            // va dans l'autre sens.
            return RawPenetration { axis: -normal, depth: distance };
        }

        polytope.insert(edge_index + 1, new_point);
    }

    let (_, normal, distance) = closest_edge(&polytope, winding);
    RawPenetration { axis: -normal, depth: distance }
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

/// Index de départ, normale sortante (unitaire) et distance à l'origine de
/// l'arête du polytope la plus proche de l'origine. `winding` (signe fixe,
/// calculé une fois par `expand`) donne le côté "sortant" sans ambiguïté,
/// y compris quand l'origine est exactement sur une arête.
fn closest_edge(polytope: &[Vec2], winding: f64) -> (usize, Vec2, f64) {
    let len = polytope.len();
    let mut best = (0, Vec2::new(0.0, 0.0), f64::INFINITY);

    for i in 0..len {
        let a = polytope[i];
        let b = polytope[(i + 1) % len];
        let edge = b - a;

        // Sommets confondus (arête de longueur nulle) : rien à en tirer.
        if edge.dot(edge) < 1e-12 {
            continue;
        }

        let perp = edge.perp();
        let outward = if winding > 0.0 { -perp } else { perp };
        let normal = outward.normalized();
        let distance = normal.dot(a);

        if distance < best.2 {
            best = (i, normal, distance);
        }
    }

    best
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
