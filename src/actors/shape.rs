use std::marker::PhantomData;

use rapier2d_f64::parry::shape::Shape as ParryShape;

use crate::entity::{Color, Entity};
use crate::position::Position;

use super::{Actor, Matrix2d, WgpuGraphics};

/// Ce qui distingue une forme d'une autre : sa géométrie (`shape`) et son
/// dessin (`draw`). Implémenté par un type marqueur sans donnée (ex.
/// `SquareGeometry`) ; le reste est factorisé une fois dans `Shape<G>`.
pub trait Geometry {
    fn shape(param: f64) -> Box<dyn ParryShape>;

    fn draw(
        param: f64,
        position: Position,
        color: Color,
        transform: Matrix2d,
        graphics: &mut WgpuGraphics<'_>,
    );
}

/// Une entité dont la géométrie est déterminée par `G` — zero-cost
/// (`PhantomData`, effacé à la compilation). `param` décrit la forme
/// (demi-largeur pour un carré, rayon pour un cercle).
pub struct Shape<G> {
    entity: Entity,
    param: f64,
    geometry: PhantomData<G>,
}

impl<G> Shape<G> {
    pub fn new(x: f64, y: f64, color: Color, param: f64) -> Self {
        Shape {
            entity: Entity::new(x, y, color),
            param,
            geometry: PhantomData,
        }
    }
}

impl<G: Geometry + 'static> Shape<G> {
    /// Pointeur de fonction rangé dans `actors::SHAPES`.
    pub fn boxed(x: f64, y: f64, color: Color, param: f64) -> Box<dyn Actor> {
        Box::new(Self::new(x, y, color, param))
    }
}

impl<G: Geometry> Actor for Shape<G> {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }

    fn shape(&self) -> Box<dyn ParryShape> {
        G::shape(self.param)
    }

    fn draw(&self, transform: Matrix2d, graphics: &mut WgpuGraphics<'_>) {
        G::draw(
            self.param,
            self.entity.position(),
            self.entity.color,
            transform,
            graphics,
        );
    }
}
