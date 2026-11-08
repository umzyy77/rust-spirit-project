use rapier2d_f64::math::Vector;
use rapier2d_f64::parry::shape::{Cuboid, Shape as ParryShape};
use piston_window::graphics::rectangle;

use crate::entity::Color;
use crate::position::Position;

use super::shape::{Geometry, Shape};
use super::{Matrix2d, WgpuGraphics};

pub struct SquareGeometry;

impl Geometry for SquareGeometry {
    fn shape(half_size: f64) -> Box<dyn ParryShape> {
        Box::new(Cuboid::new(Vector::new(half_size, half_size)))
    }

    fn draw(
        half_size: f64,
        position: Position,
        color: Color,
        transform: Matrix2d,
        graphics: &mut WgpuGraphics<'_>,
    ) {
        rectangle(
            color,
            [
                position.x() - half_size,
                position.y() - half_size,
                half_size * 2.0,
                half_size * 2.0,
            ],
            transform,
            graphics,
        );
    }
}

pub type Square = Shape<SquareGeometry>;
