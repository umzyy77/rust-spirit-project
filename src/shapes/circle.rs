use super::{RenderKind, Shape};
use crate::position::Position;

pub struct Circle {
    pub radius: f64,
}

impl Shape for Circle {
    fn support(&self, center: Position, direction: (f64, f64)) -> Position {
        let (dx, dy) = direction;
        let length = (dx * dx + dy * dy).sqrt();
        if length == 0.0 {
            return center;
        }
        Position::new(
            center.x() + self.radius * dx / length,
            center.y() + self.radius * dy / length,
        )
    }

    fn half_extents(&self) -> (f64, f64) {
        (self.radius, self.radius)
    }

    fn render_kind(&self) -> RenderKind {
        RenderKind::Ellipse
    }
}
