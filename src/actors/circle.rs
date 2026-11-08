use rapier2d_f64::parry::shape::{Ball, Shape as ParryShape};
use piston_window::graphics::ellipse;

use crate::entity::Color;
use crate::position::Position;

use super::shape::{Geometry, Shape};
use super::{Matrix2d, WgpuGraphics};

pub struct CircleGeometry;

impl Geometry for CircleGeometry {
    fn shape(radius: f64) -> Box<dyn ParryShape> {
        Box::new(Ball::new(radius))
    }

    fn draw(
        radius: f64,
        position: Position,
        color: Color,
        transform: Matrix2d,
        graphics: &mut WgpuGraphics<'_>,
    ) {
        ellipse(
            color,
            [
                position.x() - radius,
                position.y() - radius,
                radius * 2.0,
                radius * 2.0,
            ],
            transform,
            graphics,
        );
    }
}

pub type Circle = Shape<CircleGeometry>;
