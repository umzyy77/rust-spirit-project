pub mod circle;
pub mod shape;
pub mod square;

use circle::Circle;
pub use square::Square;

pub use piston_window::graphics::math::Matrix2d;
pub use piston_window::wgpu_graphics::WgpuGraphics;

use std::sync::Arc;

use rapier2d_f64::control::KinematicCharacterController;
use rapier2d_f64::dynamics::{IntegrationParameters, RigidBodySet};
use rapier2d_f64::geometry::{BroadPhaseBvh, ColliderBuilder, ColliderSet, NarrowPhase};
use rapier2d_f64::math::{Pose, Vector};
use rapier2d_f64::parry::shape::{Shape as ParryShape, SharedShape};
use rapier2d_f64::pipeline::QueryFilter;

use crate::entity::{Color, Entity};
use crate::position::Position;

/// Constructeur d'un `Actor` concret, le type de `Square::boxed`/`Circle::boxed`.
pub type Builder = fn(f64, f64, Color, f64) -> Box<dyn Actor>;

/// Table nom → constructeur pour `config::parse_args`. Ajouter une forme :
/// une ligne ici.
pub const SHAPES: &[(&str, Builder)] = &[("square", Square::boxed), ("circle", Circle::boxed)];

/// Pixels par seconde — vitesse de déplacement commune à tout `Actor`.
const SPEED: f64 = 250.0;

/// Déplace `shape` de `desired` depuis `start` en glissant contre `others`,
/// via `KinematicCharacterController` ("move and slide") — glissement,
/// coins et formes convexes quelconques gérés par `rapier2d_f64`.
///
/// `others` forme un monde de collision jetable, reconstruit à chaque appel
/// (pas de `PhysicsPipeline` ni de corps rigides simulés, juste un index
/// spatial interrogé une fois).
fn slide(
    shape: &dyn ParryShape,
    start: Position,
    desired: Vector,
    dt: f64,
    others: &[(Position, Box<dyn ParryShape>)],
) -> Vector {
    let bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();
    let modified: Vec<_> = others
        .iter()
        .map(|(position, other_shape)| {
            colliders.insert(
                ColliderBuilder::new(SharedShape(Arc::from(other_shape.clone_dyn())))
                    .translation(Vector::new(position.x(), position.y()))
                    .build(),
            )
        })
        .collect();

    let mut broad_phase = BroadPhaseBvh::new();
    broad_phase.update(&IntegrationParameters::default(), &colliders, &bodies, &modified, &[], &mut Vec::new());
    let narrow_phase = NarrowPhase::new();
    let query_pipeline =
        broad_phase.as_query_pipeline(narrow_phase.query_dispatcher(), &bodies, &colliders, QueryFilter::default());

    // `snap_to_ground`/les angles de pente sont des réglages de plateformer
    // (sol/plafond relatifs à `up`) sans rapport avec ce jeu vu du dessus.
    let controller = KinematicCharacterController {
        snap_to_ground: None,
        max_slope_climb_angle: 0.0,
        min_slope_slide_angle: 0.0,
        ..Default::default()
    };

    controller
        .move_shape(dt, &query_pipeline, shape, &Pose::translation(start.x(), start.y()), desired, |_| {})
        .translation
}

/// Comportement commun à toute entité mobile. Pas de générique : le trait
/// doit rester "object safe" pour `&dyn Actor`.
pub trait Actor {
    fn get_entity(&self) -> &Entity;
    fn get_entity_mut(&mut self) -> &mut Entity;

    /// Géométrie de la forme — sert au rendu et à la collision ; seule
    /// méthode que `Square`/`Circle` (ou une future forme) doivent fournir.
    fn shape(&self) -> Box<dyn ParryShape>;

    /// Chaque forme sait se dessiner elle-même, pas de `match` ici.
    fn draw(&self, transform: Matrix2d, graphics: &mut WgpuGraphics<'_>);

    /// Avance l'entité de `dt` secondes selon les directions actives, en
    /// glissant contre `others` et en restant dans `bounds`.
    fn update(&mut self, dt: f64, bounds: (f64, f64), others: &[(Position, Box<dyn ParryShape>)]) {
        let distance = SPEED * dt;
        let (width, height) = bounds;

        let desired = self
            .get_entity()
            .moving_directions()
            .iter()
            .fold(Vector::ZERO, |v, direction| {
                let (dx, dy) = direction.delta();
                v + Vector::new(dx, dy) * distance
            });

        let self_shape = self.shape();
        // La sphère englobante mesure jusqu'au coin (trop grande pour des
        // murs droits) ; l'AABB donne la vraie demi-largeur/demi-hauteur.
        let half_extents = self_shape.compute_local_aabb().extents() / 2.0;
        let start = self.get_entity().position();

        let movement = slide(self_shape.as_ref(), start, desired, dt, others);

        let x = (start.x() + movement.x).clamp(half_extents.x, width - half_extents.x);
        let y = (start.y() + movement.y).clamp(half_extents.y, height - half_extents.y);

        self.get_entity_mut().set_position(Position::new(x, y));
    }
}
