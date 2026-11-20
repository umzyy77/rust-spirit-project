use crate::position::Position;
use crate::shapes::Shape;

/// Garde-fou : en 2D, GJK converge toujours en quelques itérations pour un
/// simplexe à 3 sommets maximum. Cette limite n'est jamais censée être
/// atteinte ; elle protège juste contre un cas numérique imprévu.
const MAX_ITERATIONS: usize = 20;

type Vector = (f64, f64);

/// Teste si deux formes convexes (centrées sur `position_a`/`position_b`)
/// se chevauchent, via GJK : on cherche si l'origine appartient à la
/// différence de Minkowski A - B. L'algorithme ne s'appuie que sur
/// `Shape::support` — il ne connaît jamais le type concret de `shape_a`/
/// `shape_b`, ni s'ils sont identiques ou différents.
pub fn overlap(
    shape_a: &dyn Shape,
    position_a: Position,
    shape_b: &dyn Shape,
    position_b: Position,
) -> bool {
    let support = |direction: Vector| -> Vector {
        let point_a = shape_a.support(position_a, direction);
        let point_b = shape_b.support(position_b, negate(direction));
        sub((point_a.x(), point_a.y()), (point_b.x(), point_b.y()))
    };

    let centers_offset = sub(
        (position_a.x(), position_a.y()),
        (position_b.x(), position_b.y()),
    );
    let initial_direction = if centers_offset == (0.0, 0.0) {
        (1.0, 0.0)
    } else {
        centers_offset
    };

    let mut simplex = vec![support(initial_direction)];
    let mut direction = negate(simplex[0]);

    for _ in 0..MAX_ITERATIONS {
        let point = support(direction);
        if dot(point, direction) < 0.0 {
            return false;
        }

        simplex.push(point);

        match simplex.len() {
            2 => {
                let a = simplex[1];
                let b = simplex[0];
                let ab = sub(b, a);
                let ao = negate(a);
                if dot(ab, ao) > 0.0 {
                    direction = perp_toward(ab, ao);
                } else {
                    simplex = vec![a];
                    direction = ao;
                }
            }
            3 => {
                let a = simplex[2];
                let b = simplex[1];
                let c = simplex[0];
                let ab = sub(b, a);
                let ac = sub(c, a);
                let ao = negate(a);

                let ab_perp = perp_toward(ab, negate(ac));
                if dot(ab_perp, ao) > 0.0 {
                    simplex = vec![b, a];
                    direction = ab_perp;
                } else {
                    let ac_perp = perp_toward(ac, negate(ab));
                    if dot(ac_perp, ao) > 0.0 {
                        simplex = vec![c, a];
                        direction = ac_perp;
                    } else {
                        return true;
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    false
}

fn dot(u: Vector, v: Vector) -> f64 {
    u.0 * v.0 + u.1 * v.1
}

fn sub(u: Vector, v: Vector) -> Vector {
    (u.0 - v.0, u.1 - v.1)
}

fn negate(v: Vector) -> Vector {
    (-v.0, -v.1)
}

/// Perpendiculaire à `edge`, réorientée du même côté que `reference`.
fn perp_toward(edge: Vector, reference: Vector) -> Vector {
    let perp = (-edge.1, edge.0);
    if dot(perp, reference) < 0.0 {
        negate(perp)
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
}
