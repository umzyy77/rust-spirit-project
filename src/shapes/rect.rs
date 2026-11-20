use super::{RenderKind, Shape};
use crate::position::Position;

pub struct Rect {
    pub width: f64,
    pub height: f64,
}

impl Shape for Rect {
    fn support(&self, center: Position, direction: (f64, f64)) -> Position {
        let (half_width, half_height) = self.half_extents();
        let x = center.x() + if direction.0 >= 0.0 { half_width } else { -half_width };
        let y = center.y() + if direction.1 >= 0.0 { half_height } else { -half_height };
        Position::new(x, y)
    }

    fn half_extents(&self) -> (f64, f64) {
        (self.width / 2.0, self.height / 2.0)
    }

    fn render_kind(&self) -> RenderKind {
        RenderKind::Rect
    }
}
